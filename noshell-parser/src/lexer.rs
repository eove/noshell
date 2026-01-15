//! A lexer for generating tokens from a command line.

/// Variant of `Flag` token. Only store the identifier, not the hyphens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Flag<'a> {
    /// A short flag (e.g. -v).
    Short(char),

    /// A long flag (e.g. --verbose).
    Long(&'a str),
}

impl PartialEq<Flag<'_>> for &str {
    fn eq(&self, other: &Flag<'_>) -> bool {
        match other {
            Flag::Short(id) => {
                if let Some(x) = self.chars().next() {
                    *id == x
                } else {
                    false
                }
            }
            Flag::Long(id) => *id == *other,
        }
    }
}

/// Defines a `Token` that has been read from the command line.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Token<'a> {
    /// Flag.
    Flag(Flag<'a>),

    /// Value (i.e. everything that is not a short or long flag).
    Value(&'a str),
}

impl Token<'_> {
    /// Evaluate if the token string is a short flag.
    #[inline(always)]
    pub fn is_short_flag(input: &str) -> bool {
        input.starts_with('-') && input.len() > 1 && !Self::is_number(input)
    }

    /// Evaluate if the token string is a long flag.
    #[inline(always)]
    pub fn is_long_flag(input: &str) -> bool {
        input.starts_with("--") && input.len() > 2
    }

    /// Evaluate if the token string is a flag.
    #[inline(always)]
    pub fn is_flag(input: &str) -> bool {
        Self::is_short_flag(input) || Self::is_long_flag(input)
    }

    /// Evaluate if the token string represents a number.
    pub fn is_number(input: &str) -> bool {
        let mut position_of_e = None;
        let mut have_seen_dot = false;

        // Remove the front sign is any.
        let input = input.trim_start_matches('-');

        for (i, c) in input.as_bytes().iter().enumerate() {
            match c {
                // Digits, OK.
                b'0'..=b'9' => {}

                // Exponential, OK if not the first character.
                b'e' | b'E' if position_of_e.is_none() && i > 0 => {
                    position_of_e = Some(i);
                }

                // Dot is valid if unique, not the first character and before any exponential.
                b'.' if !have_seen_dot && position_of_e.is_none() && i > 0 => {
                    have_seen_dot = true;
                }

                _ => return false,
            }
        }

        if let Some(pos) = position_of_e {
            pos != input.len() - 1
        } else {
            true
        }
    }
}

/// Defines a `Lexer` that is responsible for streaming tokens from the command line input.
///
/// A lexer acts like an forward iterator.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Tokens<'a, IterTy>
where
    IterTy: Iterator<Item = &'a str>,
{
    iter: IterTy,
}

impl<'a, IterTy> Tokens<'a, IterTy>
where
    IterTy: Iterator<Item = &'a str> + Clone,
{
    /// Create a new lexer from the command line input.
    pub fn new(iter: IterTy) -> Self {
        Tokens { iter }
    }

    /// Retreive an iterator to the next value tokens.
    #[inline(always)]
    pub fn values(&self) -> Values<'a, IterTy> {
        Values::new(self.iter.clone())
    }

    /// Retreive an iterator to the next tokens.
    #[inline(always)]
    pub fn tokens(&self) -> Self {
        Tokens::new(self.iter.clone())
    }
}

impl<'a, IterTy> Iterator for Tokens<'a, IterTy>
where
    IterTy: Iterator<Item = &'a str>,
{
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let arg = self.iter.next()?;

        // Long flag.
        if arg.starts_with("--") && arg.len() >= 3 {
            let (_, name) = arg.split_at(2);
            return Some(Token::Flag(Flag::Long(name)));
        }

        // Numbers.
        if arg.starts_with('-') && Token::is_number(arg) {
            return Some(Token::Value(arg));
        }

        // Short flag.
        if arg.starts_with('-') && arg.len() == 2 {
            let (_, name) = arg.split_at(1);
            return Some(Token::Flag(Flag::Short(
                name.chars().nth(0).unwrap_or_default(),
            )));
        }

        Some(Token::Value(arg))
    }
}

/// A iterator over value tokens.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Values<'a, IterTy>
where
    IterTy: Iterator<Item = &'a str>,
{
    iter: IterTy,
    done: bool,
}

impl<'a, IterTy> Values<'a, IterTy>
where
    IterTy: Iterator<Item = &'a str>,
{
    /// Create a value iterator from the given cursor.
    pub fn new(iter: IterTy) -> Self {
        Values { iter, done: false }
    }
}

impl<'a, IterTy> Iterator for Values<'a, IterTy>
where
    IterTy: Iterator<Item = &'a str>,
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let arg = self.iter.next()?;

        if Token::is_flag(arg) {
            self.done = true;
            None
        } else {
            Some(arg)
        }
    }
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;

    use super::*;

    #[test]
    fn it_should_match_short_flag() {
        let mut lexer = Tokens::new(["-f"].into_iter());

        let token = lexer.next();
        assert_that!(token.is_some(), eq(true));
        assert_that!(token.unwrap(), eq(Token::Flag(Flag::Short('f'))));
    }

    #[test]
    fn it_should_match_value_starting_with_dash() {
        let mut lexer = Tokens::new(["-flag"].into_iter());

        let token = lexer.next();
        assert_that!(token.is_some(), eq(true));
        assert_that!(token.unwrap(), eq(Token::Value("-flag")));
    }

    #[test]
    fn it_should_match_long_flag() {
        let mut lexer = Tokens::new(["--flag"].into_iter());

        let token = lexer.next();
        assert_that!(token.is_some(), eq(true));
        assert_that!(token.unwrap(), eq(Token::Flag(Flag::Long("flag"))));
    }

    #[test]
    fn it_should_match_numbers() {
        let lexer =
            Tokens::new(["-2", "2", "-2.", "2.", "-2.e1", "2.e1", "-2e1", "2e1"].into_iter());

        for token in lexer {
            assert_that!(token, matches_pattern!(&Token::Value(_)));
        }
    }
}
