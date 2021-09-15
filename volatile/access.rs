use core::result;

use super::error;

pub type Result<T> = result::Result<T, error::Access>;

pub trait Read{}

pub trait Write{}

pub trait ReadWrite: Read + Write{}

pub struct ReadImpl;
pub struct WriteImpl;
pub struct ReadWriteImpl;

impl Read for ReadImpl{}
impl Write for WriteImpl{}

impl Read for ReadWriteImpl{}
impl Write for ReadWriteImpl{}
impl ReadWrite for ReadWriteImpl{}
