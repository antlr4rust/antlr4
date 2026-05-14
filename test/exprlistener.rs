#![allow(nonstandard_style)]
// Generated from Expr.g4 by ANTLR 4.13.2
use antlr4rust::tree::ParseTreeListener;
use super::exprparser::*;

pub trait ExprListener<'input> : ParseTreeListener<'input,ExprParserContextType>{
/**
 * Enter a parse tree produced by {@link ExprParser#program}.
 * @param ctx the parse tree
 */
fn enter_program(&mut self, _ctx: &ProgramContext<'input>) { }
/**
 * Exit a parse tree produced by {@link ExprParser#program}.
 * @param ctx the parse tree
 */
fn exit_program(&mut self, _ctx: &ProgramContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code BaseHello}
 * labeled alternative in {@link ExprParser#hello}.
 * @param ctx the parse tree
 */
fn enter_BaseHello(&mut self, _ctx: &BaseHelloContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code BaseHello}
 * labeled alternative in {@link ExprParser#hello}.
 * @param ctx the parse tree
 */
fn exit_BaseHello(&mut self, _ctx: &BaseHelloContext<'input>) { }
/**
 * Enter a parse tree produced by the {@code ExtHello}
 * labeled alternative in {@link ExprParser#hello}.
 * @param ctx the parse tree
 */
fn enter_ExtHello(&mut self, _ctx: &ExtHelloContext<'input>) { }
/**
 * Exit a parse tree produced by the {@code ExtHello}
 * labeled alternative in {@link ExprParser#hello}.
 * @param ctx the parse tree
 */
fn exit_ExtHello(&mut self, _ctx: &ExtHelloContext<'input>) { }
/**
 * Enter a parse tree produced by {@link ExprParser#world}.
 * @param ctx the parse tree
 */
fn enter_world(&mut self, _ctx: &WorldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link ExprParser#world}.
 * @param ctx the parse tree
 */
fn exit_world(&mut self, _ctx: &WorldContext<'input>) { }

}

antlr4rust::coerce_from!{ 'input : ExprListener<'input> }


