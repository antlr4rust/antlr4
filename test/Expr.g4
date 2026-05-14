grammar Expr;

program
    : hello EOF
    | hello (world)+ EOF
    ;

hello
    : 'Hello!' # BaseHello 
    | 'Hello, ' # ExtHello
    ;

world
    : 'world!'
    ;