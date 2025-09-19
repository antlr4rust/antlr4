grammar ReferenceToATN;

@tokenfactory{
pub type LocalTokenFactory<'input> = antlr4rust::token_factory::OwningTokenFactory; // need single quote here '
}

a : (ID|ATN)* ATN? {println!("{}",$text);};
ID : 'a'..'z'+ ;
ATN : '0'..'9'+;
WS : (' '|'\n') -> skip ;
