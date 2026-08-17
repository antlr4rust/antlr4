#![allow(non_snake_case)]
//! Regression test for the `ListenerId` ABA type confusion.
//!
//! `ListenerId` identifies a listener by its heap address, but an address is only unique
//! among values that are alive at the same time. `remove_parse_listeners` drops every
//! listener box *without* consuming the corresponding `ListenerId`s, so a later
//! `add_parse_listener` can be handed the freed address. A stale id then matches a listener
//! of a completely different type, and `remove_parse_listener` reinterprets it — reachable
//! without any `unsafe` on the caller's part.
//!
//! The expected behaviour is that a stale id matches nothing and the lookup panics. Until
//! listener identity stops being an address, this test is red: no panic happens and a `B` is
//! silently returned as a `Box<A>`.

mod gen {
    use antlr4rust::common_token_stream::CommonTokenStream;
    use antlr4rust::token_factory::ArenaCommonFactory;
    use antlr4rust::tree::ParseTreeListener;
    use antlr4rust::InputStream;
    use csvlexer::*;
    use csvparser::*;

    #[path = "csvlexer.rs"]
    pub mod csvlexer;
    #[path = "csvlistener.rs"]
    pub mod csvlistener;
    #[path = "csvparser.rs"]
    pub mod csvparser;
    #[path = "csvvisitor.rs"]
    pub mod csvvisitor;

    /// The two listener types are deliberately given the same size and alignment so that the
    /// allocator is likely to hand `B` the block just freed from `A`. That is what arms the
    /// ABA; the test asserts below that it actually happened.
    struct A {
        tag: [u64; 4],
    }
    // Read only through the type confusion, which the compiler cannot see.
    #[allow(dead_code)]
    struct B {
        text: String,
        pad: u64,
    }

    impl<'i> ParseTreeListener<'i, CSVParserContextType> for A {}
    impl<'i> csvlistener::CSVListener<'i> for A {}
    impl<'i> ParseTreeListener<'i, CSVParserContextType> for B {}
    impl<'i> csvlistener::CSVListener<'i> for B {}

    const _: () = {
        assert!(std::mem::size_of::<A>() == std::mem::size_of::<B>());
        assert!(std::mem::align_of::<A>() == std::mem::align_of::<B>());
    };

    #[test]
    #[should_panic(expected = "listener not found")]
    fn stale_listener_id_does_not_resurrect_a_different_listener() {
        let tf = ArenaCommonFactory::default();
        let lexer = CSVLexer::new_with_token_factory(InputStream::new("a\n"), &tf);
        let mut parser = CSVParser::new(CommonTokenStream::new(lexer));

        let a = Box::new(A {
            tag: [0xAAAA_AAAA_AAAA_AAAA; 4],
        });
        let addr_a = &*a as *const A as usize;
        let id_a = parser.add_parse_listener(a);

        // Drops A's box without consuming `id_a`, which is now stale.
        parser.remove_parse_listeners();

        let b = Box::new(B {
            text: String::from("hello world"),
            pad: 0,
        });
        let addr_b = &*b as *const B as usize;
        parser.add_parse_listener(b);

        // If the allocator did not reuse the block there is no ABA to observe and the test
        // would pass for the wrong reason, so fail loudly and distinguishably instead.
        assert_eq!(
            addr_a, addr_b,
            "allocator did not reuse the freed block, so the ABA was never set up - \
             this run is inconclusive rather than a pass"
        );

        // Must panic: `id_a` refers to a listener that no longer exists. Today it instead
        // matches `b` by address and hands back a `B` reinterpreted as an `A`.
        let confused = parser.remove_parse_listener(id_a);

        // Only reached while the bug is live. Leak rather than free `B`'s allocation through
        // `A`'s layout, so a red run does not compound the type confusion with a bad dealloc.
        println!(
            "TYPE CONFUSION: read B as A, tag[0] = {:#x} (= {} = \"hello world\".len())",
            confused.tag[0], confused.tag[0]
        );
        std::mem::forget(confused);
    }

    /// A `ListenerId` minted by one parser must not match a listener held by another.
    ///
    /// This type checks whenever both parsers are for the same grammar, so it is reachable,
    /// and it is the reason listener ids come from a process-global counter rather than a
    /// per-parser one. With a per-parser counter both listeners below would hold id `0` and
    /// this would hand back a `B` reinterpreted as an `A`.
    #[test]
    #[should_panic(expected = "listener not found")]
    fn listener_id_from_another_parser_does_not_match() {
        let tf1 = ArenaCommonFactory::default();
        let lexer1 = CSVLexer::new_with_token_factory(InputStream::new("a\n"), &tf1);
        let mut parser1 = CSVParser::new(CommonTokenStream::new(lexer1));

        let tf2 = ArenaCommonFactory::default();
        let lexer2 = CSVLexer::new_with_token_factory(InputStream::new("a\n"), &tf2);
        let mut parser2 = CSVParser::new(CommonTokenStream::new(lexer2));

        let id_a = parser1.add_parse_listener(Box::new(A {
            tag: [0xAAAA_AAAA_AAAA_AAAA; 4],
        }));
        parser2.add_parse_listener(Box::new(B {
            text: String::from("hello world"),
            pad: 0,
        }));

        // `id_a` belongs to `parser1`, so `parser2` must not resolve it.
        let confused = parser2.remove_parse_listener(id_a);
        std::mem::forget(confused);
    }
}
