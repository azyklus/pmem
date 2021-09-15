use alloc::boxed::Box;
use core::fmt::{self, Debug};

pub trait Access: Debug{}

/// # Error in reading from volatile memory
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Read;

impl Access for Read{}

impl fmt::Display for Read
{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
   {
      f.write_str("a read error occurred")
   }
}

pub type ReadError = Box<Read>;

/// # Error in writing to volatile memory
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Write;

impl Access for Write{}

impl fmt::Display for Write
{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
   {
      f.write_str("a write error occurred")
   }
}

lazy_static! {
   pub static ref READ: Box<Read> = Box::new(Read);
   pub static ref WRITE: Box<Write> = Box::new(Write);
}
