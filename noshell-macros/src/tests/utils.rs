#![allow(dead_code)]

use std::{io::Write, process::Stdio};

use proc_macro2::TokenStream;

pub fn format_rust_string(input: &str) -> String {
    let mut child = std::process::Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("should spawn rustfmt process");

    let stdin = child
        .stdin
        .as_mut()
        .expect("should get child process stdin");

    stdin
        .write_all(input.as_bytes())
        .expect("should write to child process stdin");

    let output = child
        .wait_with_output()
        .expect("should get output of child process");

    String::from_utf8(output.stdout).expect("output should only contains utf8 characters")
}

pub fn format_rust_token_stream(input: TokenStream) -> String {
    format_rust_string(&input.to_string())
}
