mod inner;

pub use inner::*;

use std::io;
use std::os::windows::prelude::{AsRawHandle, RawHandle};

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Storage::FileSystem::{
    CreateIoRing, HIORING__, IORING_CREATE_ADVISORY_FLAGS_NONE, IORING_CREATE_FLAGS,
    IORING_CREATE_REQUIRED_FLAGS_NONE, IORING_VERSION_3,
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
}
