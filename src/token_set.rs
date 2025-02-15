/// pretty much copied from https://github.com/typst/typst/blob/main/crates/typst-syntax/src/set.rs
use crate::lexer::TokenKind;

pub(crate) struct TokenKindSet(u128);

impl TokenKindSet {
    pub(crate) const EMPTY: Self = Self(0);

    pub const fn new(kinds: &[TokenKind]) -> Self {
        let mut set = 0;
        let mut i = 0;
        while i < kinds.len() {
            let kind = kinds[i] as u8;
            assert!(kind < 128, "token kind out of range");
            set |= 1 << kind;
            i += 1;
        }

        Self(set)
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, kind: TokenKind) -> bool {
        (kind as u8) < 128 && (self.0 & bit(kind)) != 0
    }
}

const fn bit(kind: TokenKind) -> u128 {
    1 << (kind as usize)
}

pub(crate) const INFIX_OPERATORS: TokenKindSet = TokenKindSet::new(&[
    TokenKind::Plus,
    TokenKind::Minus,
    TokenKind::Wedge,
    TokenKind::Dot,
]);

pub(crate) const PREFIX_OPERATORS: TokenKindSet = TokenKindSet::new(&[
    TokenKind::Plus,
    TokenKind::Minus,
]);

pub(crate) const PARENTHESIS: TokenKindSet = TokenKindSet::new(&[
    TokenKind::LeftParen,
    TokenKind::RightParen,
]);

pub(crate) const OPERATORS: TokenKindSet = INFIX_OPERATORS
    .union(PREFIX_OPERATORS)
    .union(PARENTHESIS);
