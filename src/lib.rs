pub(crate) mod lexer;
mod parser;
pub(crate) mod token_set;

// Re-export commonly used items
pub use parser::{Parser, ASTNode};
pub use lexer::TokenKind;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "serde")]
use serde_json;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn parse_latex(input: &str) -> String {
    let mut parser = Parser::new(input);
    let ast = parser.parse();
    serde_json::to_string(&ast).unwrap()
}