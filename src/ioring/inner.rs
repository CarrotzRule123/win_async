use windows_sys::Win32::Storage::FileSystem::{IORING_CREATE_FLAGS, IORING_VERSION};

#[repr(C)]
pub struct SqueueHeadInner {
    pub head: u64,
    pub tail: u64,
    pub padding: u64,
}

#[repr(C)]
pub enum SqueueInner {
    Base(SqueueHeadInner),
    Padding(u64),
}

#[repr(C)]
pub struct CqueueHeadInner {
    pub head: u64,
    pub tail: u64,
}

#[repr(C)]
pub enum CqueueInner {
    Base(CqueueHeadInner),
    Padding(u64),
}

#[repr(C)]
pub struct IoRingInner {
    pub rtype: u32,
    pub size: u32,
    pub version: IORING_VERSION,
    pub flags: IORING_CREATE_FLAGS,
    pub squeue_size: u32,
    pub squeue_mask: u32,
    pub cqueue_size: u32,
    pub cqueue_mask: u32,
    pub squeue: SqueueInner,
    pub cqueue: CqueueInner,
}
