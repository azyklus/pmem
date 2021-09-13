use core::fmt;

pub trait Access{}

/// # Error in reading from volatile memory
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ReadError;

impl Access for ReadError{}

impl fmt::Display for ReadError
{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
   {
      f.write_str("a read error occurred")
   }
}

/// # Error in writing to volatile memory
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WriteError;

impl Access for WriteError{}

impl fmt::Display for WriteError
{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
   {
      f.write_str("a write error occurred")
   }
}
