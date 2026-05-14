// Generated from Expr.g4 by ANTLR 4.13.2

use super::exprparser::*;
use antlr4rust::tree::ParseTreeListener;

// A complete Visitor for a parse tree produced by ExprParser.

pub trait ExprBaseListener<'input>:
    ParseTreeListener<'input, ExprParserContextType> {

    /**
     * Enter a parse tree produced by \{@link ExprBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_program(&mut self, _ctx: &ProgramContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  ExprBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_program(&mut self, _ctx: &ProgramContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link ExprBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_basehello(&mut self, _ctx: &BaseHelloContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  ExprBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_basehello(&mut self, _ctx: &BaseHelloContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link ExprBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_exthello(&mut self, _ctx: &ExtHelloContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  ExprBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_exthello(&mut self, _ctx: &ExtHelloContext<'input>) {}


    /**
     * Enter a parse tree produced by \{@link ExprBaseParser#s}.
     * @param ctx the parse tree
     */
    fn enter_world(&mut self, _ctx: &WorldContext<'input>) {}
    /**
     * Exit a parse tree produced by \{@link  ExprBaseParser#s}.
     * @param ctx the parse tree
     */
    fn exit_world(&mut self, _ctx: &WorldContext<'input>) {}


}