# antlr4rust
[![Crate](https://img.shields.io/crates/v/antlr4rust)](https://crates.io/crates/antlr4rust)
[![docs](https://img.shields.io/docsrs/antlr4rust/latest)](https://docs.rs/antlr4rust/latest/antlr4rust/)


> [!note]
> You don't have to build this ANTLR jar yourself, you can use the 
> [latest release jar from here instead](https://github.com/antlr4rust/antlr4/releases)!

## [ANTLR4](https://github.com/antlr/antlr4) runtime for Rust programming language.

For examples you can see [grammars](grammars), [tests/gen](tests/gen) for corresponding generated code 
and [tests/general_tests.rs](tests/general_tests.rs) and [tests/visitors_tests.rs](tests/visitors_tests.rs) for actual usage examples

## ANTLR4 Tool(parser generator)

Can be built using maven, or downloaded from [github.com/antlr4rust/antlr4](https://github.com/antlr4rust/antlr4/releases/)

### Implementation status

For now development is going on in this repository, it remains unclear how to best make this widely available.

Previous versions are not maintained any more,
so if you hit a problem, first migrate to the latest version.

### Usage

You should use the ANTLR4 "tool" to generate a parser, that will use the ANTLR 
runtime located here. You can run it with the following command, to have all generated rust sources output in directory/"module" `gen`:
```bash
java -jar <path to ANTLR4 tool> -Dlanguage=Rust <g4 location> -o gen
```
For a full list of antlr4 tool options, please visit the 
[tool documentation page](https://github.com/antlr/antlr4/blob/master/doc/tool-options.md).

You can also see [build.rs](build.rs) as an example of `build.rs` configuration 
to rebuild parser automatically if grammar file was changed.

Then add following to `Cargo.toml` of the crate from which generated parser 
is going to be used:
```toml 
[dependencies]
antlr4rust = "0.6"
```
 
### Parse Tree structure

It is possible to generate idiomatic Rust syntax trees. For this you would need to use labels feature of ANTLR tool.
You can see [Labels](grammars/Labels.g4) grammar for example.
Consider following rule :
```text
e   : a=e op='*' b=e   # mult
    | left=e '+' b=e   # add
		 
```
For such rule ANTLR will generate enum `EContextAll` with a `MultContext` and an `AddContext` variant
(plus an `Error` variant), so you will be able to match on them in your code.
Also corresponding struct for each alternative will contain fields you labeled. 
I.e. `MultContext` will contain `a` and `b` fields holding the child subtrees (`Option<Rc<EContextAll>>`) and
an `op` field holding the matched token (`Option<TokenType>`).
It also is possible to disable generic parse tree creation to keep only selected children via
`parser.build_parse_trees = false`, but unfortunately currently it will prevent visitors from working. 
  
### Differences with Java
Although Rust runtime API has been made as close as possible to Java, 
there are quite some differences because Rust is not an OOP language and is much more explicit. 

 - If you are using labeled alternatives, 
 struct generated for the rule is an enum with variant for each alternative
 - Parser needs to have ownership for listeners, but it is possible to get listener back via `ListenerId`
 otherwise `ParseTreeWalker` should be used.
 - In embedded actions to access parser you should use `recog` variable instead of `self`/`this`. 
 This is because predicates have to be inserted into two syntactically different places in generated parser 
 and in one of them it is impossible to have parser as `self`.
 - str based `InputStream` have different index behavior when there are unicode characters. 
 If you need exactly the same behavior, use `[u32]` based `InputStream`, or implement custom `CharStream`.
 - In actions you have to escape `'` in rust lifetimes with `\ ` because ANTLR considers them as strings, e.g. `Struct<\'lifetime>`
 - To make custom tokens you should use `@tokenfactory` custom action, instead of usual `TokenLabelType` parser option.
 ANTLR parser options can accept only single identifiers while Rust target needs know about lifetime as well. 
 Also in Rust target `TokenFactory` is the way to specify token type. As example you can see [CSV](grammars/CSV.g4) test grammar.
 - All rule context variables (rule argument or rule return) should implement `Default + Clone`.
 
### Unsafe
`unsafe` is used in a few places; see the `# Safety` sections and `// SAFETY:` comments in the code for the details.

 - Downcasting. Parse tree nodes, tokens and token factories borrow from the input, so `std::any::Any`,
 which requires `'static`, can't downcast them. The [`tid`](src/tid.rs) module, adapted from the `better_any` crate,
 provides the equivalent for types with a single lifetime: the unsafe traits `Tid` and `TidAble`, and the `TidExt` downcasts.
 Implement `TidAble` for your own types through the `tid!` macro rather than by hand.
 `Transition::cast` also downcasts, after checking the `Any` type id.
 - Parse listeners. `remove_parse_listener` hands a listener back at the type it was added with.
 That relies on listener ids never being reused, and on the unsafe `CoerceFrom`/`CoerceTo` traits,
 which the `coerce_from!` macro implements for the generated listener and context traits.
 - Filling in contexts. Generated parsers write label and attribute values into the context of the rule being parsed,
 which the parser already shares through `Rc`. They use `cast_mut`, an `unsafe fn` equivalent to the standard library's
 unstable `Rc::get_mut_unchecked`, in `unsafe` blocks whose borrow only covers a single field write.

What this means for your code:

 - Parsers generated from grammars that use labels (`x=…`, `xs+=…`) or assign rule attributes (`$v = …`) contain `unsafe` blocks,
 so a crate with `#![forbid(unsafe_code)]` can't include them. Put them in a crate of their own,
 or use `deny`, which a module-level `#[allow(unsafe_code)]` can override.
 The `unsafe impl`s that `tid!` and `coerce_from!` expand to don't trigger the `unsafe_code` lint.
 - In embedded actions, don't hold a reference into the current rule's context, for instance one obtained through `recog.ctx`,
 across a label or `$attr` assignment: the generated write would alias it.

### Versioning
In addition to usual Rust semantic versioning, 
patch version changes of the crate should not require updating of generator part.

Current MSRV for the crate is `1.85`.
  
## Licence

BSD 3-clause. 
Unless you explicitly state otherwise, 
any contribution intentionally submitted for inclusion in this project by you
shall be licensed as above, without any additional terms or conditions.

> [!NOTE]
> This is an effort to pick things up where [Konstantin aka @rrevenantt](https://github.com/rrevenantt) left them a few years back.
> This is work in progress and I'm mostly trying to solve my problem as of now, but I'd be real happy to either contribute this back
> to the original repo and/or work towards making this a contribution to the antlr4 project.
>
> Help, feedback et al are really appreciated!
