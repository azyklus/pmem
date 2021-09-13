use core::{
   fmt,
};

/// # Error in reading from volatile memory
pub struct ReadError;

impl fmt::Display for ReadError
{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
   {
      f.write_str("a read error occurred")
   }
}

/// # Error in writing to volatile memory
pub struct WriteError;

impl fmt::Display for WriteError
{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
   {
      f.write_str("a write error occurred")
   }
}
