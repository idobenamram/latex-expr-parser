/// A parser for basic latex expressions.
/// based on matklad's pratt parser blog https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
use crate::{
    lexer::{Lexer, Token, TokenKind},
    token_set::{
        OPERATORS, PREFIX_BINARY_OPERATORS, PREFIX_UNIARY_COMMANDS_OPERATORS,
        PREFIX_UNIARY_OPERATORS, SUB_SUP_OPERATORS,
    },
};

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum ASTNode {
    Identifier {
        name: String,
    },
    Int {
        value: i64,
    },
    BinaryOpNode {
        op: TokenKind,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    UnaryOpNode {
        op: TokenKind,
        operand: Box<ASTNode>,
    },
}

impl ASTNode {
    fn binary(op: TokenKind, lhs: ASTNode, rhs: ASTNode) -> ASTNode {
        ASTNode::BinaryOpNode {
            op,
            left: Box::new(lhs),
            right: Box::new(rhs),
        }
    }
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

fn infix_binding_power(op: &TokenKind) -> Option<(u8, u8)> {
    match op {
        TokenKind::Plus | TokenKind::Minus => Some((3, 4)),
        TokenKind::Multiply | TokenKind::Divide => Some((5, 6)),
        TokenKind::Wedge | TokenKind::Dot => Some((6, 7)),
        // exponentiation
        TokenKind::Carrot | TokenKind::Underscore => Some((7, 8)),
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
        return expr;
    }

    pub fn parse_sub_sup(&mut self) -> ASTNode {
        match self.stream.peek() {
            t if t.kind == TokenKind::LeftBrace => self.parse_in_braces(0),
            t if t.kind.is_numeric() => {
                self.stream.next();
                let name = self.input[t.start..=t.end].to_string();
                ASTNode::Identifier { name }
            }
            t => panic!("bad token: {:?}", t),
        }
    }

    fn parse_expr(&mut self, min_bp: u8) -> ASTNode {
        let token = self.stream.next();
        let mut lhs = match token {
            t if t.kind.is_identifier() => {
                let name = self.input[t.start..=t.end].to_string();
                ASTNode::Identifier { name }
            }
            t if t.kind.is_numeric() => {
                // TODO: handle floats and numeric
                let value = self.input[t.start..=t.end]
                    .parse::<i64>()
                    .expect("should be a valid int");
                ASTNode::Int { value }
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
                    op: t.kind,
                    operand: Box::new(rhs),
                }
            }
            t if PREFIX_UNIARY_COMMANDS_OPERATORS.contains(t.kind) => {
                let rhs = self.parse_in_braces(0);
                ASTNode::UnaryOpNode {
                    op: t.kind,
                    operand: Box::new(rhs),
                }
            }
            t if PREFIX_BINARY_OPERATORS.contains(t.kind) => {
                let lhs = self.parse_in_braces(0);
                let rhs = self.parse_in_braces(0);
                ASTNode::binary(t.kind, lhs, rhs)
            }
            t => panic!("bad token: {:?}", t),
        };

        loop {
            let op = match self.stream.peek() {
                t if t.kind == TokenKind::EOF => break,
                t if OPERATORS.contains(t.kind) => t,
                t if t.kind.ident_or_numeric() => t,
                t => panic!("bad token: {:?}", t),
            };

            // TODO: should probably be handled with binding power
            if SUB_SUP_OPERATORS.contains(op.kind) {
                self.stream.next();
                let rhs = self.parse_sub_sup();
                lhs = ASTNode::binary(op.kind, lhs, rhs);
                continue;
            }

            // in the case of no operator, we should assume multiplication
            if op.kind == TokenKind::LeftParen || op.kind.ident_or_numeric() {
                let (l_bp, r_bp) = infix_binding_power(&TokenKind::Multiply)
                    .expect("multiplication is an infix operator");
                if l_bp < min_bp {
                    break;
                }

                let rhs = self.parse_expr(r_bp);
                lhs = ASTNode::binary(TokenKind::Multiply, lhs, rhs);
                continue;
            }

            if let Some((l_bp, r_bp)) = infix_binding_power(&op.kind) {
                if l_bp < min_bp {
                    break;
                }
                self.stream.next();

                let rhs = self.parse_expr(r_bp);
                lhs = ASTNode::binary(op.kind, lhs, rhs);
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
    #[case("input6", "a_1^2")]
    #[case("input7", "c + 2(a+b)")]
    // #[case("input8", "c \\wedge 2(a+b)")] // TODO: doesn't really work.
    #[case("input9", "2(a \\wedge b)^3")]
    #[case("input10", "-ia \\wedge b")]
    #[case("input11", "(1 + 2)(a + b)")]
    #[case("input12", "2a + a2")]
    fn test_parser(#[case] name: &str, #[case] input: &str) {
        let mut parser = Parser::new(input);
        let ast = parser.parse();
        insta::assert_debug_snapshot!(name, ast);
    }
}
