// Generated from Expr.g4 by ANTLR 4.13.2
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(nonstandard_style)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_braces)]
#![allow(unused_parens)]
use super::exprlistener::*;
use antlr4rust::PredictionContextCache;
use antlr4rust::TokenSource;
use antlr4rust::atn::{ATN, INVALID_ALT};
use antlr4rust::atn_deserializer::ATNDeserializer;
use antlr4rust::dfa::DFA;
use antlr4rust::error_strategy::{DefaultErrorStrategy, ErrorStrategy};
use antlr4rust::errors::*;
use antlr4rust::int_stream::EOF;
use antlr4rust::lazy_static;
use antlr4rust::parser::{BaseParser, Parser, ParserNodeType, ParserRecog};
use antlr4rust::parser_atn_simulator::ParserATNSimulator;
// HEIORHWEIORHIOWE
// HERE BaseParserRuleContext is a weirdo thing that has child_of_type somewhere. find it.
use antlr4rust::parser_rule_context::{BaseParserRuleContext, ParserRuleContext, cast, cast_mut};
use antlr4rust::recognizer::{Actions, Recognizer};
use antlr4rust::rule_context::{BaseRuleContext, CustomRuleContext, RuleContext};
use antlr4rust::token::{OwningToken, TOKEN_EOF, Token};
use antlr4rust::token_factory::{CommonTokenFactory, TokenAware, TokenFactory};
use antlr4rust::token_stream::TokenStream;
use antlr4rust::tree::*;
use antlr4rust::vocabulary::{Vocabulary, VocabularyImpl};
use antlr4rust::{TidAble, TidExt};

use std::any::{Any, TypeId};
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

const _: () = {
    assert!(
        antlr4rust::const_check::version("0", "6"),
        "Incompatible version: either switch to v0.6 or regenerate with this version"
    );
};

pub const Expr_T__0: i32 = 1;
pub const Expr_T__1: i32 = 2;
pub const Expr_T__2: i32 = 3;
pub const Expr_EOF: i32 = EOF;
pub const RULE_program: usize = 0;
pub const RULE_hello: usize = 1;
pub const RULE_world: usize = 2;
pub const ruleNames: [&'static str; 3] = ["program", "hello", "world"];

pub const _LITERAL_NAMES: [Option<&'static str>; 4] =
    [None, Some("'Hello!'"), Some("'Hello, '"), Some("'world!'")];
pub const _SYMBOLIC_NAMES: [Option<&'static str>; 0] = [];
lazy_static! {
    static ref _shared_context_cache: Arc<PredictionContextCache> =
        Arc::new(PredictionContextCache::new());
    static ref VOCABULARY: Box<dyn Vocabulary> = Box::new(VocabularyImpl::new(
        _LITERAL_NAMES.iter(),
        _SYMBOLIC_NAMES.iter(),
        None
    ));
}

type BaseParserType<'input, I> = BaseParser<
    'input,
    ExprParserExt<'input>,
    I,
    ExprParserContextType,
    dyn ExprListener<'input> + 'input,
>;

type TokenType<'input> = <LocalTokenFactory<'input> as TokenFactory<'input>>::Tok;
pub type LocalTokenFactory<'input> = CommonTokenFactory;

pub type ExprTreeWalker<'input, 'a> =
    ParseTreeWalker<'input, 'a, ExprParserContextType, dyn ExprListener<'input> + 'a>;

/// Parser for Expr grammar
pub struct ExprParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    base: BaseParserType<'input, I>,
    interpreter: Rc<ParserATNSimulator>,
    _shared_context_cache: Box<PredictionContextCache>,
    pub err_handler: Box<dyn ErrorStrategy<'input, BaseParserType<'input, I>>>,
}

impl<'input, I> ExprParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn set_error_strategy(
        &mut self,
        strategy: Box<dyn ErrorStrategy<'input, BaseParserType<'input, I>>>,
    ) {
        self.err_handler = strategy
    }

    pub fn with_strategy(
        input: I,
        strategy: Box<dyn ErrorStrategy<'input, BaseParserType<'input, I>>>,
    ) -> Self {
        let interpreter = Rc::new(ParserATNSimulator::new(
            _ATN.clone(),
            _decision_to_DFA.clone(),
            _shared_context_cache.clone(),
        ));
        Self {
            base: BaseParser::new_base_parser(
                input,
                Rc::clone(&interpreter),
                ExprParserExt {
                    _pd: Default::default(),
                },
            ),
            interpreter,
            _shared_context_cache: Box::new(PredictionContextCache::new()),
            err_handler: strategy,
        }
    }
}

type DynStrategy<'input, I> = Box<dyn ErrorStrategy<'input, BaseParserType<'input, I>> + 'input>;

impl<'input, I> ExprParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn with_dyn_strategy(input: I) -> Self {
        Self::with_strategy(input, Box::new(DefaultErrorStrategy::new()))
    }
}

impl<'input, I> ExprParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn new(input: I) -> Self {
        Self::with_strategy(input, Box::new(DefaultErrorStrategy::new()))
    }
}

/// Trait for monomorphized trait object that corresponds to the nodes of parse tree generated for ExprParser
pub trait ExprParserContext<'input>:
    for<'x> Listenable<dyn ExprListener<'input> + 'x>
    + ParserRuleContext<'input, TF = LocalTokenFactory<'input>, Ctx = ExprParserContextType>
{
}

antlr4rust::coerce_from! { 'input : ExprParserContext<'input> }

impl<'input> ExprParserContext<'input> for TerminalNode<'input, ExprParserContextType> {}
impl<'input> ExprParserContext<'input> for ErrorNode<'input, ExprParserContextType> {}

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn ExprParserContext<'input> + 'input }

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn ExprListener<'input> + 'input }

pub struct ExprParserContextType;
antlr4rust::tid! {ExprParserContextType}

impl<'input> ParserNodeType<'input> for ExprParserContextType {
    type TF = LocalTokenFactory<'input>;
    type Type = dyn ExprParserContext<'input> + 'input;
}

impl<'input, I> Deref for ExprParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    type Target = BaseParserType<'input, I>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'input, I> DerefMut for ExprParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub struct ExprParserExt<'input> {
    _pd: PhantomData<&'input str>,
}

impl<'input> ExprParserExt<'input> {}
antlr4rust::tid! { ExprParserExt<'a> }

impl<'input> TokenAware<'input> for ExprParserExt<'input> {
    type TF = LocalTokenFactory<'input>;
}

impl<'input, I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>>
    ParserRecog<'input, BaseParserType<'input, I>> for ExprParserExt<'input>
{
}

impl<'input, I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>>
    Actions<'input, BaseParserType<'input, I>> for ExprParserExt<'input>
{
    fn get_grammar_file_name(&self) -> &str {
        "Expr.g4"
    }

    fn get_rule_names(&self) -> &[&str] {
        &ruleNames
    }

    fn get_vocabulary(&self) -> &dyn Vocabulary {
        &**VOCABULARY
    }
}
//------------------- program ----------------
pub type ProgramContextAll<'input> = ProgramContext<'input>;

pub type ProgramContext<'input> = BaseParserRuleContext<'input, ProgramContextExt<'input>>;

#[derive(Clone)]
pub struct ProgramContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> ExprParserContext<'input> for ProgramContext<'input> {}

impl<'input, 'a> Listenable<dyn ExprListener<'input> + 'a> for ProgramContext<'input> {
    fn enter(&self, listener: &mut (dyn ExprListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_program(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn ExprListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_program(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input> CustomRuleContext<'input> for ProgramContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = ExprParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_program
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_program }
}
antlr4rust::tid! {ProgramContextExt<'a>}

impl<'input> ProgramContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn ExprParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<ProgramContextAll<'input>> {
        Rc::new(BaseParserRuleContext::new_parser_ctx(
            parent,
            invoking_state,
            ProgramContextExt { ph: PhantomData },
        ))
    }
}

pub trait ProgramContextAttrs<'input>:
    ExprParserContext<'input> + BorrowMut<ProgramContextExt<'input>>
{
    fn hello(&self) -> Option<Rc<HelloContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(0)
    }
    /// Retrieves first TerminalNode corresponding to token EOF
    /// Returns `None` if there is no child corresponding to token EOF
    fn EOF(&self) -> Option<Rc<TerminalNode<'input, ExprParserContextType>>>
    where
        Self: Sized,
    {
        self.get_token(Expr_EOF, 0)
    }
    fn world_all(&self) -> Vec<Rc<WorldContextAll<'input>>>
    where
        Self: Sized,
    {
        self.children_of_type()
    }
    fn world(&self, i: usize) -> Option<Rc<WorldContextAll<'input>>>
    where
        Self: Sized,
    {
        self.child_of_type(i)
    }
}

impl<'input> ProgramContextAttrs<'input> for ProgramContext<'input> {}

impl<'input, I> ExprParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn program(&mut self) -> Result<Rc<ProgramContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = ProgramContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 0, RULE_program);
        let mut _localctx: Rc<ProgramContextAll> = _localctx;
        let mut _la: i32 = -1;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(17);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.interpreter.adaptive_predict(1, &mut recog.base)? {
                1 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 1)?;
                    recog.base.enter_outer_alt(None, 1)?;
                    {
                        /*InvokeRule hello*/
                        recog.base.set_state(6);
                        recog.hello()?;

                        recog.base.set_state(7);
                        recog.base.match_token(Expr_EOF, &mut recog.err_handler)?;
                    }
                }
                2 => {
                    //recog.base.enter_outer_alt(_localctx.clone(), 2)?;
                    recog.base.enter_outer_alt(None, 2)?;
                    {
                        /*InvokeRule hello*/
                        recog.base.set_state(9);
                        recog.hello()?;

                        recog.base.set_state(11);
                        recog.err_handler.sync(&mut recog.base)?;
                        _la = recog.base.input.la(1);
                        loop {
                            {
                                {
                                    /*InvokeRule world*/
                                    recog.base.set_state(10);
                                    recog.world()?;
                                }
                            }
                            recog.base.set_state(13);
                            recog.err_handler.sync(&mut recog.base)?;
                            _la = recog.base.input.la(1);
                            if !(_la == Expr_T__2) {
                                break;
                            }
                        }
                        recog.base.set_state(15);
                        recog.base.match_token(Expr_EOF, &mut recog.err_handler)?;
                    }
                }

                _ => {}
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
//------------------- hello ----------------
#[derive(Debug)]
pub enum HelloContextAll<'input> {
    BaseHelloContext(BaseHelloContext<'input>),
    ExtHelloContext(ExtHelloContext<'input>),
    Error(HelloContext<'input>),
}
antlr4rust::tid! {HelloContextAll<'a>}

impl<'input> antlr4rust::parser_rule_context::DerefSeal for HelloContextAll<'input> {}

impl<'input> ExprParserContext<'input> for HelloContextAll<'input> {}

impl<'input> Deref for HelloContextAll<'input> {
    type Target = dyn HelloContextAttrs<'input> + 'input;
    fn deref(&self) -> &Self::Target {
        use HelloContextAll::*;
        match self {
            BaseHelloContext(inner) => inner,
            ExtHelloContext(inner) => inner,
            Error(inner) => inner,
        }
    }
}
impl<'input, 'a> Listenable<dyn ExprListener<'input> + 'a> for HelloContextAll<'input> {
    fn enter(&self, listener: &mut (dyn ExprListener<'input> + 'a)) -> Result<(), ANTLRError> {
        self.deref().enter(listener)
    }
    fn exit(&self, listener: &mut (dyn ExprListener<'input> + 'a)) -> Result<(), ANTLRError> {
        self.deref().exit(listener)
    }
}

pub type HelloContext<'input> = BaseParserRuleContext<'input, HelloContextExt<'input>>;

#[derive(Clone)]
pub struct HelloContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> ExprParserContext<'input> for HelloContext<'input> {}

impl<'input, 'a> Listenable<dyn ExprListener<'input> + 'a> for HelloContext<'input> {}

impl<'input> CustomRuleContext<'input> for HelloContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = ExprParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_hello
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_hello }
}
antlr4rust::tid! {HelloContextExt<'a>}

impl<'input> HelloContextExt<'input> {
    fn new(
        parent: Option<Rc<dyn ExprParserContext<'input> + 'input>>,
        invoking_state: i32,
    ) -> Rc<HelloContextAll<'input>> {
        Rc::new(HelloContextAll::Error(
            BaseParserRuleContext::new_parser_ctx(
                parent,
                invoking_state,
                HelloContextExt { ph: PhantomData },
            ),
        ))
    }
}

pub trait HelloContextAttrs<'input>:
    ExprParserContext<'input> + BorrowMut<HelloContextExt<'input>>
{
}

impl<'input> HelloContextAttrs<'input> for HelloContext<'input> {}

pub type BaseHelloContext<'input> = BaseParserRuleContext<'input, BaseHelloContextExt<'input>>;

pub trait BaseHelloContextAttrs<'input>: ExprParserContext<'input> {}

impl<'input> BaseHelloContextAttrs<'input> for BaseHelloContext<'input> {}

pub struct BaseHelloContextExt<'input> {
    base: HelloContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {BaseHelloContextExt<'a>}

impl<'input> ExprParserContext<'input> for BaseHelloContext<'input> {}

impl<'input, 'a> Listenable<dyn ExprListener<'input> + 'a> for BaseHelloContext<'input> {
    fn enter(&self, listener: &mut (dyn ExprListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_BaseHello(self);
        Ok(())
    }
}

impl<'input> CustomRuleContext<'input> for BaseHelloContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = ExprParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_hello
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_hello }
}

impl<'input> Borrow<HelloContextExt<'input>> for BaseHelloContext<'input> {
    fn borrow(&self) -> &HelloContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<HelloContextExt<'input>> for BaseHelloContext<'input> {
    fn borrow_mut(&mut self) -> &mut HelloContextExt<'input> {
        &mut self.base
    }
}

impl<'input> HelloContextAttrs<'input> for BaseHelloContext<'input> {}

impl<'input> BaseHelloContextExt<'input> {
    fn new(ctx: &dyn HelloContextAttrs<'input>) -> Rc<HelloContextAll<'input>> {
        Rc::new(HelloContextAll::BaseHelloContext(
            BaseParserRuleContext::copy_from(
                ctx,
                BaseHelloContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

pub type ExtHelloContext<'input> = BaseParserRuleContext<'input, ExtHelloContextExt<'input>>;

pub trait ExtHelloContextAttrs<'input>: ExprParserContext<'input> {}

impl<'input> ExtHelloContextAttrs<'input> for ExtHelloContext<'input> {}

pub struct ExtHelloContextExt<'input> {
    base: HelloContextExt<'input>,
    ph: PhantomData<&'input str>,
}

antlr4rust::tid! {ExtHelloContextExt<'a>}

impl<'input> ExprParserContext<'input> for ExtHelloContext<'input> {}

impl<'input, 'a> Listenable<dyn ExprListener<'input> + 'a> for ExtHelloContext<'input> {
    fn enter(&self, listener: &mut (dyn ExprListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_ExtHello(self);
        Ok(())
    }
}

impl<'input> CustomRuleContext<'input> for ExtHelloContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = ExprParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_hello
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_hello }
}

impl<'input> Borrow<HelloContextExt<'input>> for ExtHelloContext<'input> {
    fn borrow(&self) -> &HelloContextExt<'input> {
        &self.base
    }
}
impl<'input> BorrowMut<HelloContextExt<'input>> for ExtHelloContext<'input> {
    fn borrow_mut(&mut self) -> &mut HelloContextExt<'input> {
        &mut self.base
    }
}

impl<'input> HelloContextAttrs<'input> for ExtHelloContext<'input> {}

impl<'input> ExtHelloContextExt<'input> {
    fn new(ctx: &dyn HelloContextAttrs<'input>) -> Rc<HelloContextAll<'input>> {
        Rc::new(HelloContextAll::ExtHelloContext(
            BaseParserRuleContext::copy_from(
                ctx,
                ExtHelloContextExt {
                    base: ctx.borrow().clone(),
                    ph: PhantomData,
                },
            ),
        ))
    }
}

impl<'input, I> ExprParser<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input>> + TidAble<'input>,
{
    pub fn hello(&mut self) -> Result<Rc<HelloContextAll<'input>>, ANTLRError> {
        let mut recog = self;
        let _parentctx = recog.ctx.take();
        let mut _localctx = HelloContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 2, RULE_hello);
        let mut _localctx: Rc<HelloContextAll> = _localctx;
        let result: Result<(), ANTLRError> = (|| {
            recog.base.set_state(21);
            recog.err_handler.sync(&mut recog.base)?;
            match recog.base.input.la(1) {
                Expr_T__0 => {
                    let tmp = BaseHelloContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 1)?;
                    _localctx = tmp;
                    {
                        recog.base.set_state(19);
                        recog.base.match_token(Expr_T__0, &mut recog.err_handler)?;
                    }
                }

                Expr_T__1 => {
                    let tmp = ExtHelloContextExt::new(&**_localctx);
                    recog.base.enter_outer_alt(Some(tmp.clone()), 2)?;
                    _localctx = tmp;
                    {
                        recog.base.set_state(20);
                        recog.base.match_token(Expr_T__1, &mut recog.err_handler)?;
                    }
                }

                _ => Err(ANTLRError::NoAltError(NoViableAltError::new(
                    &mut recog.base,
                )))?,
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
//------------------- world ----------------
pub type WorldContextAll<'input> = WorldContext<'input>;

pub type WorldContext<'input> = BaseParserRuleContext<'input, WorldContextExt<'input>>;

#[derive(Clone)]
pub struct WorldContextExt<'input> {
    ph: PhantomData<&'input str>,
}

impl<'input> ExprParserContext<'input> for WorldContext<'input> {}

impl<'input, 'a> Listenable<dyn ExprListener<'input> + 'a> for WorldContext<'input> {
    fn enter(&self, listener: &mut (dyn ExprListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.enter_every_rule(self)?;
        listener.enter_world(self);
        Ok(())
    }
    fn exit(&self, listener: &mut (dyn ExprListener<'input> + 'a)) -> Result<(), ANTLRError> {
        listener.exit_world(self);
        listener.exit_every_rule(self)?;
        Ok(())
    }
}

impl<'input> CustomRuleContext<'input> for WorldContextExt<'input> {
    type TF = LocalTokenFactory<'input>;
    type Ctx = ExprParserContextType;
    fn get_rule_index(&self) -> usize {
        RULE_world
    }
    //fn type_rule_index() -> usize where Self: Sized { RULE_world }
}
antlr4rust::tid! {WorldContextExt<'a>}

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
