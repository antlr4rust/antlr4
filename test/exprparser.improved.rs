// Just to see what the improved version might look like for the following grammar:
// grammar Expr;

// program
//     : hello+ EOF
//     ;

// hello
//     | 'Hello, ' world exclam?
//     ;

// world
//     : 'world' exclam?
//     ;

// exclam
//     : '!'

struct ParserContext {

}

trait<'input> Output<T> {
    fn child() -> T;
}

trait<'input> OutputAll<T> {
    fn children() -> Vec<T>
}

// -------- 

impl<'input> OutputAll<Hello> for Program<'input> {

}

// ---

impl<'input> Output<World> for Hello<'input> {
    
}

impl<'input> Output<Option<Exclam> for Hello<'input> {

}

// ---

impl<'input> Output<Option<Exclam> for World<'input> {

}


//------------------- world ----------------
// pub type WorldContextAll<'input> = WorldContext<'input>;

// pub type WorldContext<'input> = BaseParserRuleContext<'input, WorldContextExt<'input>>;

#[derive(Clone)]
pub struct WorldContextExt<'input> {
    ph: PhantomData<&'input str>,
}


impl<'input> ExprParserContext<'input> for World<'input> {

}

impl<'input> CustomRuleContext<'input> for WorldExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = ExprParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_world
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_world }
}

impl<'input> WorldContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn ExprParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<WorldContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            WorldContextExt { ph: PhantomData },
        ))
    }
}

pub trait WorldContextAttrs<'input>:
    ExprParserContext<'input> + BorrowMut<WorldContextExt<'input>>
{
}

impl<'input> WorldContextAttrs<'input> for WorldContext<'input> {}

impl<'input, I> ExprParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn world(&mut self) -> Result<Rc<WorldContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = WorldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 4, RULE_world);
        let mut _localctx: Rc<WorldContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
            recog.base.enter_outer_alt(None, 1)?;
            {
                recog.base.set_state(23);
                recog.base.match_token(Expr_T__2, &mut recog.err_handler)?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => {}
            Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
            Err(ref re) => {
                //_localctx.exception = re;
                recog.err_handler.report_error(&mut recog.base, re);
                recog.err_handler.recover(&mut recog.base, re)?;
            }
        }
        recog.base.exit_rule()?;

        Ok(_localctx)
    }
}
lazy_static! {
    static ref _ATN: Arc<ATN> =
        Arc::new(ATNDeserializer::new(None).deserialize(&mut _serializedATN.iter()));
    static ref _decision_to_DFA: Arc<Vec<antlr4rust::RwLock<DFA>>> = {
        let mut dfa = Vec::new();
        let size = _ATN.decision_to_state.len() as i32;
        for i in 0..size {
            dfa.push(DFA::new(_ATN.clone(), _ATN.get_decision_state(i), i).into())
        }
        Arc::new(dfa)
    };
    static ref _serializedATN: Vec<i32> = vec![
        4, 1, 3, 26, 2, 0, 7, 0, 2, 1, 7, 1, 2, 2, 7, 2, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 4, 0, 12, 8,
        0, 11, 0, 12, 0, 13, 1, 0, 1, 0, 3, 0, 18, 8, 0, 1, 1, 1, 1, 3, 1, 22, 8, 1, 1, 2, 1, 2, 1,
        2, 0, 0, 3, 0, 2, 4, 0, 0, 25, 0, 17, 1, 0, 0, 0, 2, 21, 1, 0, 0, 0, 4, 23, 1, 0, 0, 0, 6,
        7, 3, 2, 1, 0, 7, 8, 5, 0, 0, 1, 8, 18, 1, 0, 0, 0, 9, 11, 3, 2, 1, 0, 10, 12, 3, 4, 2, 0,
        11, 10, 1, 0, 0, 0, 12, 13, 1, 0, 0, 0, 13, 11, 1, 0, 0, 0, 13, 14, 1, 0, 0, 0, 14, 15, 1,
        0, 0, 0, 15, 16, 5, 0, 0, 1, 16, 18, 1, 0, 0, 0, 17, 6, 1, 0, 0, 0, 17, 9, 1, 0, 0, 0, 18,
        1, 1, 0, 0, 0, 19, 22, 5, 1, 0, 0, 20, 22, 5, 2, 0, 0, 21, 19, 1, 0, 0, 0, 21, 20, 1, 0, 0,
        0, 22, 3, 1, 0, 0, 0, 23, 24, 5, 3, 0, 0, 24, 5, 1, 0, 0, 0, 3, 13, 17, 21
    ];
}
