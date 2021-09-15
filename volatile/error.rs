use core::fmt::{self, Debug};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Access
{
  Read,
  Write,
}

impl fmt::Display for Access
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
  {
    f.write_str("an error occurred while accessing the requested memory")
  }
}
