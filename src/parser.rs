/// A parser for basic latex expressions.
/// based on matklad's pratt parser blog https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
use crate::{
    lexer::{Lexer, Token, TokenKind},
    token_set::{OPERATORS, PREFIX_BINARY_OPERATORS, PREFIX_UNIARY_COMMANDS_OPERATORS, PREFIX_UNIARY_OPERATORS},
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug)]
pub enum ASTNode {
    Identifier {
        name: String,
        subscript: Option<String>,
    },
    BinaryOpNode {
        op: Token,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    UnaryOpNode {
        op: Token,
        operand: Box<ASTNode>,
    },
}

#[derive(Debug)]
struct TokenStream {
    tokens: Vec<Token>,
}

impl TokenStream {
    fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let mut tokens = vec![];
        let mut token = lexer.next();
        while token.kind != TokenKind::EOF {
            // skip whitespace
            if token.kind != TokenKind::WhiteSpace {
                tokens.push(token);
            }
            token = lexer.next();
        }

        // go from right to left
        tokens.reverse();
        Self { tokens }
    }

    fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token::end(0))
    }

    fn peek(&self) -> Token {
        self.tokens.last().cloned().unwrap_or(Token::end(0))
    }
}

fn prefix_binding_power(op: &Token) -> ((), u8) {
    match op.kind {
        TokenKind::Plus | TokenKind::Minus => ((), 7),
        _ => panic!("bad prefix operator: {:?}", op),
    }
}

fn postfix_binding_power(op: &Token) -> Option<(u8, ())> {
    todo!()
}

fn infix_binding_power(op: &Token) -> Option<(u8, u8)> {
    match op.kind {
        TokenKind::Plus | TokenKind::Minus => Some((3, 4)),
        TokenKind::Multiply | TokenKind::Divide | TokenKind::Wedge | TokenKind::Dot => Some((5, 6)),
        _ => None,
    }
}

pub struct Parser<'s> {
    input: &'s str,
    stream: TokenStream,
}

impl<'s> Parser<'s> {
    pub fn new(input: &'s str) -> Self {
        Parser {
            input,
            stream: TokenStream::new(input),
        }
    }

    pub fn parse(&mut self) -> ASTNode {
        self.parse_expr(0)
    }

    pub fn parse_in_braces(&mut self, min_bp: u8) -> ASTNode {
        let next_token = self.stream.next();
        assert_eq!(next_token.kind, TokenKind::LeftBrace);
        let expr = self.parse_expr(min_bp);
        let next_token = self.stream.next();
        assert_eq!(next_token.kind, TokenKind::RightBrace);
        return expr
    }

    fn parse_expr(&mut self, min_bp: u8) -> ASTNode {
        let token = self.stream.next();
        let mut lhs = match token {
            t if t.kind == TokenKind::Identifier => {
                let name = self.input[t.start..=t.end].to_string();

                let subscript = if let Some(subscript) = t.subscript {
                    let subscript_str = self.input[subscript.start..=subscript.end].to_string();
                    Some(subscript_str)
                } else {
                    None
                };
                ASTNode::Identifier { name, subscript }
            }
            tok if tok.kind == TokenKind::LeftParen => {
                let lhs = self.parse_expr(0);
                let next_token = self.stream.next();
                assert_eq!(next_token.kind, TokenKind::RightParen);
                lhs
            }
            t if PREFIX_UNIARY_OPERATORS.contains(t.kind) => {
                let ((), r_bp) = prefix_binding_power(&t);
                let rhs = self.parse_expr(r_bp);
                ASTNode::UnaryOpNode {
                    op: t,
                    operand: Box::new(rhs),
                }
            }
            t if PREFIX_UNIARY_COMMANDS_OPERATORS.contains(t.kind) => {
                let rhs = self.parse_in_braces(0);
                ASTNode::UnaryOpNode {
                    op: t,
                    operand: Box::new(rhs),
                }
            }
            t if PREFIX_BINARY_OPERATORS.contains(t.kind) => {
                let lhs = self.parse_in_braces(0);
                let rhs = self.parse_in_braces(0);
                ASTNode::BinaryOpNode {
                    op: t,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                }
            }
            t => panic!("bad token: {:?}", t),
        };

        loop {
            let op = match self.stream.peek() {
                t if t.kind == TokenKind::EOF => break,
                t if OPERATORS.contains(t.kind) => t,
                t => panic!("bad token: {:?}", t),
            };

            // if let Some((l_bp, ())) = postfix_binding_power(op) {
            //     if l_bp < min_bp {
            //         break;
            //     }
            //     lexer.next();

            //     lhs = if op == '[' {
            //         let rhs = expr_bp(lexer, 0);
            //         assert_eq!(lexer.next(), Token::Op(']'));
            //         S::Cons(op, vec![lhs, rhs])
            //     } else {
            //         S::Cons(op, vec![lhs])
            //     };
            //     continue;
            // }

            if let Some((l_bp, r_bp)) = infix_binding_power(&op) {
                if l_bp < min_bp {
                    break;
                }
                self.stream.next();

                let rhs = self.parse_expr(r_bp);
                lhs = ASTNode::BinaryOpNode {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            break;
        }

        lhs
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("input1", "a + b + c")]
    #[case("input2", "(a + b) + c")]
    #[case("input3", "-b * (-ca + a \\cdot c) + a \\wedge b")]
    #[case("input4", "\\frac{a_1 + b}{k_{s} \\wedge H}")]
    #[case("input5", "\\hat{a}")]
    fn test_parser(#[case] name: &str, #[case] input: &str) {
        let mut parser = Parser::new(input);
        let ast = parser.parse();
        insta::assert_debug_snapshot!(name, ast);
    }
}
