#![allow(unused)]

use require_lifetimes::require_lifetimes;

/// This function prints the given input.
/// You will need to annotate its lifetimes.
/// (2 marks)
#[require_lifetimes]
pub fn print<'a>(a: &'a i32) {
    println!("{a}");
}

/// This function returns the first parameter it is given.
/// You will need to annotate its lifetimes.
/// (3 marks)
#[require_lifetimes]
pub fn first<'a>(a: &'a i32, b: &'a i32) -> &'a i32 {
    a
}

/// A struct to hold the data of a string being split.
/// You will need to annotate its lifetimes.
/// (2 marks)
pub struct StringSplitter<'a> {
    pub text: &'a str,
    pub pattern: &'a str,
}

/// This function creates a string splitter with given data.
/// You will need to annotate its lifetimes.
/// (3 marks)
#[require_lifetimes]
pub fn split<'a>(text: &'a str, pattern: &'a str) -> StringSplitter<'a> {
    StringSplitter { text, pattern }
}
