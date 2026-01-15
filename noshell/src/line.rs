//! Line parsing.

use core::fmt;

use futures::{Stream, StreamExt, pin_mut};
use heapless::String;
use noterm::cursor::{MoveRight, MoveToNextLine};
use noterm::events::{Event, KeyCode, KeyEvent};
use noterm::io::blocking::Write;
use noterm::style::Print;
use noterm::{Executable, Queuable};

use crate::prompt::Prompt;
use crate::unescape;

#[cfg(test)]
mod tests;

#[cfg(test)]
extern crate std;

/// Error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Input/ouput error.
    #[error(transparent)]
    Io(#[from] noterm::io::Error),

    /// Unknown error.
    #[error("unknown error")]
    Unknown,
}

/// Re-export result type.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Read a line.
pub async fn readline<OutputTy, EventsTy, ContentTy, const SIZE: usize>(
    output: &mut OutputTy,
    events: EventsTy,
    prompt: Prompt<ContentTy>,
) -> Result<String<SIZE>>
where
    OutputTy: Write,
    EventsTy: Stream<Item = noterm::io::Result<Event>>,
    ContentTy: Iterator + Clone,
    <ContentTy as Iterator>::Item: fmt::Display,
{
    // Prepare the output of the line.
    let mut line: String<SIZE> = String::new();

    // Write the prompt, then read for input events.
    prompt.reset(output)?;

    // Pin the events, so that it stays on the stack while calling async/await.
    pin_mut!(events);

    // Create the escaped state.
    let mut escaped = false;

    loop {
        match events.next().await {
            Some(Ok(event)) => {
                #[cfg(test)]
                println!("event: {:?}", event);

                match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: _,
                        kind: _,
                    }) if !escaped => break,

                    Event::Key(KeyEvent {
                        code,
                        modifiers: _,
                        kind: _,
                    }) => match code {
                        KeyCode::Enter if escaped => {
                            let _ = line.push('\n');
                            output.queue(MoveToNextLine(1))?;
                            output.queue(MoveRight(4))?;
                            output.flush()?;
                            escaped = false;
                        }

                        KeyCode::Char(c) => {
                            let _ = line.push(c);
                            output.execute(Print(c))?;
                            escaped = c == '\\';
                        }

                        _ => {}
                    },

                    _ => {}
                }
            }

            Some(Err(err)) => return Err(Error::from(err)),
            None => break,
        }
    }

    Ok(unescape::<SIZE>(&line))
}
