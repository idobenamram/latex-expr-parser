/// A lexer for basic latex expressions.
/// heavely based on typst lexer implementation (https://github.com/typst/typst/blob/main/crates/typst-syntax/src/lexer.rs)
use unscanny::Scanner;

#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum TokenKind {
    /// an identifier, like `A` or `B`
    Identifier,
    /// A left curly brace, `{`
    LeftBrace,
    /// A right curly brace, `}`
    RightBrace,
    /// A left bracket, `[`
    LeftBracket,
    /// A right bracket, `]`
    RightBracket,
    /// A left parenthesis, `(`
    LeftParen,
    /// A right parenthesis, `)`
    RightParen,
    /// whitespace, can contain multiple whitespace characters
    WhiteSpace,
    /// A wedge, `\wedge`
    Wedge,
    /// A dot, `\dot`
    Dot,
    /// A plus, `+`
    Plus,
    /// A minus, `-`
    Minus,
    /// A multiplication, `*`
    Multiply,
    /// A division, `/`
    Divide,
    /// A fraction, `\frac`
    Frac,
    /// the hat operator `\hat`
    Hat,
    /// a underscore, `_` - used for subscript
    Underscore,
    /// a carrot `^` - used for superscript
    Carrot,
    /// an integer
    Int,
    /// a float
    Float,
    /// any other type of number
    Numeric,
    /// End of file
    EOF,
}

impl TokenKind {
    pub fn is_numeric(self) -> bool {
        matches!(self, TokenKind::Int | TokenKind::Float | TokenKind::Numeric)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone)]
pub(crate) struct TokenSubscript {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Self { kind, start, end }
    }
    pub fn single(kind: TokenKind, pos: usize) -> Self {
        Self {
            kind,
            start: pos,
            end: pos,
        }
    }

    pub fn end(pos: usize) -> Self {
        Self {
            kind: TokenKind::EOF,
            start: pos,
            end: pos,
        }
    }
}

pub(crate) struct Lexer<'s> {
    s: Scanner<'s>,
}

impl<'s> Lexer<'s> {
    pub fn new(input: &'s str) -> Self {
        Lexer {
            s: Scanner::new(input),
        }
    }

    fn whitespace(&mut self, start: usize) -> Token {
        self.s.eat_whitespace();
        Token::new(TokenKind::WhiteSpace, start, self.s.cursor() - 1)
    }

    fn latex_command(&mut self, start: usize) -> Token {
        let command_name = self.s.eat_while(|c: char| c != ' ' && c != '{');
        let kind = match command_name {
            "wedge" => TokenKind::Wedge,
            "cdot" => TokenKind::Dot,
            "frac" => TokenKind::Frac,
            "hat" => TokenKind::Hat,
            _ => panic!("not supported latex command: {}", command_name),
        };
        Token::new(kind, start, self.s.cursor() - 1)
    }

    // fn identifier(&mut self, start: usize) -> Token {
    //     // eat while alphabetic characters
    //     self.s.eat_while(|c: char| c.is_alphabetic());
    //     let identifier_end = self.s.cursor() - 1;

    //     // an identifier can have a subscript which will be followed either by a `{` or one more alphanumeric character
    //     if self.s.peek() == Some('_') {
    //         self.s.eat();
    //         let subscript = match self.s.peek() {
    //             Some('{') => {
    //                 // skip the `{`
    //                 let subscript_start = self.s.cursor() + 1;
    //                 self.s.eat_while(|c: char| c != '}');
    //                 let subscript_end = self.s.cursor() - 1;
    //                 // skip the `}`
    //                 self.s.eat();
    //                 Some(TokenSubscript { start: subscript_start, end: subscript_end })
    //             }
    //             Some(c) if c.is_alphanumeric() => {
    //                 let subscript_pos = self.s.cursor();
    //                 self.s.eat();
    //                 Some(TokenSubscript { start: subscript_pos, end: subscript_pos })
    //             }
    //             None => {None}
    //             _ => {None}
    //         };
    //         Token::new(TokenKind::Identifier, start, identifier_end, subscript)
    //     } else {
    //         Token::new(TokenKind::Identifier, start, identifier_end, None)
    //     }
    // }

    fn latex(&mut self, c: char, start: usize) -> Token {
        match c {
            '\\' => self.latex_command(start),
            '0'..='9' => self.number(start, c),
            '+' => Token::single(TokenKind::Plus, start),
            '-' => Token::single(TokenKind::Minus, start),
            '*' => Token::single(TokenKind::Multiply, start),
            '/' => Token::single(TokenKind::Divide, start),
            '{' => Token::single(TokenKind::LeftBrace, start),
            '}' => Token::single(TokenKind::RightBrace, start),
            '[' => Token::single(TokenKind::LeftBracket, start),
            ']' => Token::single(TokenKind::RightBracket, start),
            '(' => Token::single(TokenKind::LeftParen, start),
            ')' => Token::single(TokenKind::RightParen, start),
            '_' => Token::single(TokenKind::Underscore, start),
            '^' => Token::single(TokenKind::Carrot, start),
            c if c.is_alphabetic() => {
                self.s.eat_while(|c: char| c.is_alphabetic());
                Token::new(TokenKind::Identifier, start, self.s.cursor() - 1)
            }

            c => panic!("don't support unicode characters yet: {}", c),
        }
    }

    fn number(&mut self, start: usize, _c: char) -> Token {
        // TODO: support more stuff like in typst number function
        self.s.eat_while(|c: char| c.is_digit(10));
        Token::new(TokenKind::Int, start, self.s.cursor() - 1)
    }

    pub fn next(&mut self) -> Token {
        let start = self.s.cursor();
        match self.s.eat() {
            Some(c) if c.is_whitespace() => self.whitespace(start),
            Some(c) => self.latex(c, start),
            None => Token::end(start),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("input1", "a")]
    #[case("input2", "a + b")]
    #[case("input3", "(a + b)")]
    #[case("input4", "a \\wedge b")]
    #[case("input5", "a \\cdot b")]
    #[case("input6", "a \\wedge b \\wedge c \\wedge d")]
    #[case("input7", "a_{123} + b_s")]
    #[case("input8", "a_sb")]
    #[case("input9", "123")]
    fn test_lexer(#[case] name: &str, #[case] input: &str) {
        let mut lexer = Lexer::new(input);
        let mut tokens = vec![];
        let mut token = lexer.next();
        while token.kind != TokenKind::EOF {
            tokens.push(token);
            token = lexer.next();
        }
        insta::assert_debug_snapshot!(name, tokens);
    }
}
