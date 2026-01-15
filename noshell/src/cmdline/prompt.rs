//! Prompt.

use core::fmt;

use noterm::cursor::{MoveRight, MoveToNextLine};
use noterm::style::Print;
use noterm::{Queuable, io};

/// A Prompt is composed of several styled string parts.
pub struct Prompt<ContentTy> {
    parts: ContentTy,
}

impl<ContentTy> Prompt<ContentTy> {
    /// Create a new prompt from contents.
    pub fn new(parts: ContentTy) -> Self {
        Prompt { parts }
    }
}

impl<ContentTy> Prompt<ContentTy>
where
    ContentTy: Iterator + Clone,
    <ContentTy as Iterator>::Item: fmt::Display,
{
    /// Reset the prompt and print it to the output.
    pub fn reset<OutputTy>(&self, output: &mut OutputTy) -> io::Result<()>
    where
        OutputTy: io::blocking::Write,
    {
        let parts = self.parts.clone();
        output.queue(MoveToNextLine(1))?;

        for part in parts {
            output.queue(Print(part))?;
        }

        output.queue(MoveRight(1))?;
        output.flush()?;
        Ok(())
    }
}
