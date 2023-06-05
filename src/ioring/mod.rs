mod inner;
mod op;
mod util;

pub use inner::*;
pub use op::*;
pub use util::*;

use std::io;
use std::os::windows::prelude::{AsRawHandle, RawHandle};

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Storage::FileSystem::{
    CreateIoRing, SubmitIoRing, HIORING__, IORING_CREATE_ADVISORY_FLAGS_NONE, IORING_CREATE_FLAGS,
    IORING_CREATE_REQUIRED_FLAGS_NONE, IORING_OP_READ, IORING_OP_REGISTER_BUFFERS,
    IORING_VERSION_3,
};

pub struct IoRing {
    pub handle: HANDLE,
}

impl AsRawHandle for IoRing {
    fn as_raw_handle(&self) -> RawHandle {
        self.handle as RawHandle
    }
}

impl IoRing {
    pub fn new(entries: u32) -> io::Result<Self> {
        let mut handle = 0;
        unsafe {
            CreateIoRing(
                IORING_VERSION_3,
                IORING_CREATE_FLAGS {
                    Required: IORING_CREATE_REQUIRED_FLAGS_NONE,
                    Advisory: IORING_CREATE_ADVISORY_FLAGS_NONE,
                },
                entries,
                entries * 2,
                &mut handle as *mut isize as *mut *mut HIORING__,
            )
        };
        if handle == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { handle })
    }

    pub fn inner<'a>(&self) -> &'a IoRingInner {
        unsafe { &*(self.handle as *mut IoRingInner) }
    }

    pub fn push_entry(&self, entry: SqueueEntry) {
        let ring = self.inner();
        let squeue = unsafe { &mut *ring.squeue };

        if let SqueueInner::Base(squeue) = squeue {
            let tail = (squeue.tail & ring.squeue_mask) as isize;
            squeue.entries.set_offset(tail, entry);
        }
    }

    pub fn push_entry_read(&self, file: HANDLE, buffer: HANDLE, offset: u64, length: u32) {
        self.push_entry(SqueueEntry {
            opcode: IORING_OP_READ,
            flags: SqueueEntryFlags::None,
            data: 140,
            op: IoRingOp::Read(OpRead {
                flags: IoRingOpFlags::None,
                padding: 0,
                file,
                buffer,
                offset,
                length,
                key: 0,
            }),
        })
    }

    pub fn push_entry_register_buffers(&self, buffer: HANDLE, count: u32, length: u32) {
        self.push_entry(SqueueEntry {
            opcode: IORING_OP_REGISTER_BUFFERS,
            flags: SqueueEntryFlags::None,
            data: 140,
            op: IoRingOp::RegisterBuffers(OpRegisterBuffers {
                flags: IoRingOpFlags::None,
                count,
                info: &BufferInfo {
                    address: buffer,
                    length,
                } as *const BufferInfo,
            }),
        })
    }

    pub fn submit(&self, wait: u32, millis: u32, mut entries: u32) -> io::Result<()> {
        let res = unsafe {
            SubmitIoRing(
                self.handle as *const HIORING__,
                wait,
                millis,
                &mut entries as *mut u32,
            )
        };
        if res != 0 {
            return Err(io::Error::from_raw_os_error(res as i32));
        }
        Ok(())
    }
}
