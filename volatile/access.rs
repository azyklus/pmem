pub trait Read{}
pub trait Write{}

pub trait ReadWrite: Read + Write {}

pub struct ReadImpl;
pub struct WriteImpl;
pub struct ReadWriteImpl;

impl Read for ReadImpl{}
impl Write for WriteImpl{}
impl ReadWrite for ReadWriteImpl{}
