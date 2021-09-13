use core::result;
use super::error;

pub type Result<T> = result::Result<T, AccessError>;

pub trait Read
{
   fn read<T>(&self) -> Result<T>
   {
      unimplemented!("please implement function")
   }
}

pub trait Write
{
   fn write<T>(&mut self, src: *mut T) -> Result<()>
   {
      unimplemented!("please implement function")
   }
}

pub trait ReadWrite: Read + Write{}

pub struct ReadImpl;
pub struct WriteImpl;
pub struct ReadWriteImpl;

impl Read for ReadImpl{}
impl Write for WriteImpl{}
impl ReadWrite for ReadWriteImpl{}
