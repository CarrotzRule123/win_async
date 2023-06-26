mod ioring;
mod iocp;

pub use ioring::*;
pub use iocp::*;

pub struct Event {
    pub key: u32,
    pub readable: bool,
    pub writeable: bool,
}