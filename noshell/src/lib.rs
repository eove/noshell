//! noshell, a `no_std` argument parser and a shell for constrained systems.
#![cfg_attr(not(test), no_std)]
#![allow(async_fn_in_trait)]
#![deny(missing_docs)]

pub use noshell_macros as macros;
pub use noshell_parser as parser;

pub use macros::Parser;
// use noterm::io::blocking::Write;

pub mod lexer;
pub mod line;
pub mod prompt;

/// Defines the possible errors that may occur during usage of the crate.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// An error comes from the parsing of arguments.
    #[error(transparent)]
    Parser(#[from] parser::Error),

    /// Command not found.
    #[error("command not found")]
    CommandNotFound,

    /// Invalid utf8 string.
    #[error("invalid utf8 string")]
    Utf8,

    /// Unknown error, for development only.
    #[error("unknown error")]
    Unknown,
}

/// Unescape special characters in input string.
///
/// This requires allocating an output string to accumulate the resulting string. This is done
/// using `heapless::String`.
pub fn unescape<const SIZE: usize>(input: &str) -> heapless::String<SIZE> {
    let (acc, _) =
        input.chars().fold(
            (heapless::String::new(), false),
            |(mut acc, escaped), c| match escaped {
                // If the character is escaped and is special, consume it as unescaped.
                true if ['$', '"', '\\'].contains(&c) => {
                    let _ = acc.push(c);
                    (acc, false)
                }

                // If the character is a newline, preceded by a backslash, discard both.
                true if '\n' == c => (acc, false),

                // If the character is escaped but not special, consume it as escaped.
                true => {
                    let _ = acc.push('\\');
                    let _ = acc.push(c);
                    (acc, false)
                }

                // If character is not a backslash, then consume it.
                false if c != '\\' => {
                    let _ = acc.push(c);
                    (acc, false)
                }

                // If the character is a backslash, discard it but keep memory of it.
                false => (acc, true),
            },
        );

    acc
}

// /// Command trait.
// pub trait Callback {
//     /// Execute the callback.
//     fn call(&mut self, input: &str);
// }

// /// Command.
// pub struct Command<'a, OutputTy: Write>(pub(crate) TypedCommand<'a, dyn Callback + 'a, OutputTy>);

// pub(crate) struct TypedCommand<'a, CalleeTy: Callback + ?Sized, OutputTy: Write> {
//     callee: &'a CalleeTy,
// }

// /// Command.
// pub struct Command(pub(crate) Call)

// /// Callback inner function type.
// pub struct CallbackImpl<'a, CalleeTy, OutputTy>
// where
//     CalleeTy: FnMut(&str, &mut OutputTy),
//     OutputTy: Write,
// {
//     inner: CalleeTy,
//     output: &'a mut OutputTy,
// }

// impl<'a, CalleeTy, OutputTy> CallbackImpl<'a, CalleeTy, OutputTy>
// where
//     CalleeTy: FnMut(&str, &mut OutputTy),
//     OutputTy: Write,
// {
//     /// Create a new callback.
//     pub fn new(inner: CalleeTy, output: &'a mut OutputTy) -> Self {
//         CallbackImpl { inner, output }
//     }
// }

// impl<CalleeTy, OutputTy> Callback for CallbackImpl<'_, CalleeTy, OutputTy>
// where
//     CalleeTy: FnMut(&str, &mut OutputTy),
//     OutputTy: Write,
// {
//     fn execute(&mut self, input: &str) {
//         (self.inner)(input, self.output)
//     }
// }

// /// Parse top-level commands.
// pub fn lookup_in_static_entries<'a>(name: &str) -> Result<&'a mut Command<'static>, Error> {
//     let entries: &'static mut [Command<'static>] = unsafe {
//         let start = (&NOSHELL_COMMANDS_START as *const u32)
//             .cast::<Command<'static>>()
//             .cast_mut();

//         let end = (&NOSHELL_COMMANDS_END as *const u32)
//             .cast::<Command<'static>>()
//             .cast_mut();

//         let len = (end as usize) - (start as usize);

//         core::slice::from_raw_parts_mut(start, len)
//     };

//     entries
//         .iter_mut()
//         .find(|entry| name == entry.name)
//         .ok_or(Error::CommandNotFound)
// }

// unsafe extern "C" {
//     static NOSHELL_COMMANDS_START: u32;
//     static NOSHELL_COMMANDS_END: u32;
// }

// /// Character write trait.
// pub trait Write {
//     /// Error type.
//     type Error;

//     /// Write the given data to the underlying byte stream.
//     async fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error>;
// }

// /// Character read trait.
// pub trait Read {
//     /// Error type;
//     type Error;

//     /// Read some data from the underlying byte stream.
//     async fn read(&self, data: &mut [u8]) -> Result<usize, Self::Error>;
// }

// /// Run the shell.
// pub async fn run<IO: Read + Write>(mut io: IO) -> Result<(), Error> {
//     let mut input = [0u8; 1024];
//     let mut output = [0u8; 1024];

//     let mut cursor = 0;

//     loop {
//         'restart: {
//             let cmdline = loop {
//                 match io.read(&mut input[cursor..]).await {
//                     Ok(len) => {
//                         if let Some(eol) = input[cursor..cursor + len]
//                             .iter()
//                             .position(|&x| x as char == '\n')
//                         {
//                             let end = cursor + eol;
//                             cursor = 0;

//                             let cmdline = str::from_utf8(&input[..end]).map_err(|_| Error::Utf8)?;

//                             break cmdline;
//                         } else {
//                             cursor += len;

//                             if cursor >= input.len() {
//                                 cursor = 0;
//                                 break 'restart;
//                             }
//                         }
//                     }

//                     Err(_) => {
//                         cursor = 0;
//                         break 'restart;
//                     }
//                 }
//             };

//             let Some(name) = cmdline.split(" ").next() else {
//                 break 'restart;
//             };

//             let Ok(cmd) = lookup_in_static_entries(name) else {
//                 break 'restart;
//             };

//             let len = cmd.run(cmdline, &mut output);
//             io.write(&output[..len]).await.ok();
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use speculoos::prelude::*;

    use crate as noshell;

    use super::*;

    #[rstest]
    #[case(r#""#, "")]
    #[case(r#"'"#, "'")]
    #[case(r#"''"#, "''")]
    #[case(r#"word"#, "word")]
    #[case(r#"\$word"#, "$word")]
    #[case(r#"\\word"#, "\\word")]
    #[case(r#"\"word"#, "\"word")]
    #[case(r#"\x33word"#, "\\x33word")]
    fn it_should_unescape_string(#[case] input: &str, #[case] expected: &str) {
        assert_that!(unescape::<256>(input).as_str()).is_equal_to(expected);
    }

    #[test]
    fn it_should_parse_args_with_simple_type() {
        #[derive(Debug, noshell::Parser)]
        struct MyArgs {
            value: u32,
        }

        let argv = ["--value", "233"].into_iter();
        let res = MyArgs::try_parse_from(argv);

        assert_that!(res).is_ok();

        let args = res.unwrap();
        assert_that!(args.value).is_equal_to(233);
    }

    #[test]
    fn it_should_parse_args_with_option_type() {
        #[derive(Debug, noshell::Parser)]
        struct MyArgs {
            value: Option<u32>,
        }

        let argv = [].into_iter();
        let res = MyArgs::try_parse_from(argv);

        assert_that!(res).is_ok();

        let args = res.unwrap();
        assert_that!(args.value).is_none();

        let argv = ["--value", "233"].into_iter();
        let res = MyArgs::try_parse_from(argv);

        assert_that!(res).is_ok();

        let args = res.unwrap();
        assert_that!(args.value).is_some().is_equal_to(233);
    }

    #[test]
    fn it_should_parse_args_with_option_option_type() {
        #[derive(Debug, noshell::Parser)]
        struct MyArgs {
            value: Option<Option<u32>>,
        }

        let argv = [].into_iter();
        let res = MyArgs::try_parse_from(argv);

        assert_that!(res).is_ok();

        let args = res.unwrap();
        assert_that!(args.value).is_none();

        let argv = ["--value"].into_iter();
        let res = MyArgs::try_parse_from(argv);

        assert_that!(res).is_ok();

        let args = res.unwrap();
        assert_that!(args.value).is_some().is_none();
    }

    #[test]
    fn it_should_parse_args_with_option_vec_type() {
        use heapless::Vec;

        #[derive(Debug, noshell::Parser)]
        struct MyArgs {
            value: Option<Vec<u32, 8>>,
        }

        // No argument.
        let argv = [].into_iter();
        let res = MyArgs::try_parse_from(argv);

        assert_that!(res).is_ok();

        let args = res.unwrap();
        assert_that!(args.value).is_none();

        // Argument without value.
        let argv = ["--value"].into_iter();
        let res = MyArgs::try_parse_from(argv);

        assert_that!(res).is_err();

        // Argument with single value.
        let argv = ["--value", "23"].into_iter();
        let res = MyArgs::try_parse_from(argv);

        assert_that!(res).is_ok();
        let args = res.unwrap();

        assert_that!(args.value).is_some();
        let vals = args.value.unwrap();

        assert_that!(vals.len()).is_greater_than(0);
        assert_that!(vals.first()).is_some().is_equal_to(&23);

        // Argument with multiple values.
        let argv = ["--value", "23", "42", "72"].into_iter();
        let res = MyArgs::try_parse_from(argv);

        assert_that!(res).is_ok();
        let args = res.unwrap();

        assert_that!(args.value).is_some();
        let vals = args.value.unwrap();

        assert_that!(vals.len()).is_greater_than(0);
        let mut iter = vals.iter();

        assert_that!(iter.next()).is_some().is_equal_to(&23);
        assert_that!(iter.next()).is_some().is_equal_to(&42);
        assert_that!(iter.next()).is_some().is_equal_to(&72);
        assert_that!(iter.next()).is_none();
        assert_that!(iter.next()).is_none();
    }

    #[test]
    #[should_panic]
    fn it_should_panic_at_parsing_args_with_option_vec_type() {
        use heapless::Vec;

        #[derive(Debug, noshell::Parser)]
        struct MyArgs {
            #[allow(unused)]
            value: Option<Vec<u32, 4>>,
        }

        // Argument with too much values.
        let argv = ["--value", "1", "2", "3", "4", "5"].into_iter();
        let _ = MyArgs::try_parse_from(argv);
    }

    #[test]
    fn it_should_parse_args_with_vec_type() {
        use heapless::Vec;

        #[derive(Debug, noshell::Parser)]
        struct MyArgs {
            value: Vec<u32, 8>,
        }

        // No argument.
        let argv = [].into_iter();
        let res = MyArgs::try_parse_from(argv);

        assert_that!(res).is_err();

        // Argument without value.
        let argv = ["--value"].into_iter();
        let res = MyArgs::try_parse_from(argv);

        assert_that!(res).is_err();

        // Argument with single value.
        let argv = ["--value", "23"].into_iter();
        let res = MyArgs::try_parse_from(argv);

        assert_that!(res).is_ok();
        let args = res.unwrap();

        assert_that!(args.value.len()).is_greater_than(0);
        assert_that!(args.value.first()).is_some().is_equal_to(&23);

        // Argument with multiple values.
        let argv = ["--value", "23", "42", "72"].into_iter();
        let res = MyArgs::try_parse_from(argv);

        assert_that!(res).is_ok();
        let args = res.unwrap();

        assert_that!(args.value.len()).is_greater_than(0);
        let mut iter = args.value.iter();

        assert_that!(iter.next()).is_some().is_equal_to(&23);
        assert_that!(iter.next()).is_some().is_equal_to(&42);
        assert_that!(iter.next()).is_some().is_equal_to(&72);
        assert_that!(iter.next()).is_none();
        assert_that!(iter.next()).is_none();
    }

    #[test]
    #[should_panic]
    fn it_should_panic_at_parsing_args_with_vec_type() {
        use heapless::Vec;

        #[derive(Debug, noshell::Parser)]
        struct MyArgs {
            #[allow(unused)]
            value: Vec<u32, 4>,
        }

        // Argument with too much values.
        let argv = ["--value", "1", "2", "3", "4", "5"].into_iter();
        let _ = MyArgs::try_parse_from(argv);
    }

    // #[derive(noshell::Parser)]
    // struct ShellArgs {
    //     #[arg(long, default_value = "false")]
    //     debug: bool,
    // }

    // static SHELL_COMMAND: Command<'_> = Command::new("shell", |input: &str, output: impl Write| {
    //     let words = Shlex::new(input);
    //     let args = ShellArgs::parse
    // });
}
