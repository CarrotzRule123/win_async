use windows_sys::Win32::Storage::FileSystem::{IORING_OP_CODE, IORING_VERSION};

use crate::{IoRingOp, Array};

#[repr(C)]
#[derive(Debug)]
pub enum IoRingCreateReqFlags {
    None = 0,
}

#[repr(C)]
#[derive(Debug)]
pub enum IoRingCreateAdFlags {
    None = 0,
}

#[repr(C)]
#[derive(Debug)]
pub struct IoRingCreateFlags {
    pub required: IoRingCreateReqFlags,
    pub advisory: IoRingCreateAdFlags,
}

#[repr(C)]
#[derive(Debug)]
pub enum SqueueEntryFlags {
    None = 0,
    Drain = 1,
}

#[repr(C)]
#[derive(Debug)]
pub struct SqueueEntry {
    pub opcode: IORING_OP_CODE,
    pub flags: SqueueEntryFlags,
    pub data: u64,
    pub op: IoRingOp,
}

#[repr(C)]
pub struct SqueueHead {
    pub head: u32,
    pub tail: u32,
    pub entries: Array<SqueueEntry>,
}

#[repr(C)]
pub enum SqueueInner {
    Base(SqueueHead),
    Padding(u64),
}

#[repr(C)]
#[derive(Debug)]
pub struct CqueueHead {
    pub head: u64,
    pub tail: u64,
}

#[repr(C)]
#[derive(Debug)]
pub enum CqueueInner {
    Base(CqueueHead),
    Padding(u64),
}

#[repr(C)]
#[derive(Debug)]
pub struct IoRingInner {
    pub rtype: u32,
    pub size: u32,
    pub version: IORING_VERSION,
    pub flags: IoRingCreateFlags,
    pub squeue_size: u32,
    pub squeue_mask: u32,
    pub cqueue_size: u32,
    pub cqueue_mask: u32,
    pub squeue: *mut SqueueInner,
    pub cqueue: *mut CqueueInner,
}
