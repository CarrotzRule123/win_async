use std::io;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Storage::FileSystem::{
    CreateIoRing, HIORING__, IORING_CREATE_ADVISORY_FLAGS_NONE, IORING_CREATE_FLAGS,
    IORING_CREATE_REQUIRED_FLAGS_NONE, IORING_INFO, IORING_VERSION_3,
};

pub struct IoRing {
    pub info: IORING_INFO,
    pub handle: HANDLE,
}

impl IoRing {
    pub fn new(entries: u32) -> io::Result<Self> {
        let handle = 0;
        let info = IORING_INFO {
            IoRingVersion: IORING_VERSION_3,
            Flags: IORING_CREATE_FLAGS {
                Required: IORING_CREATE_REQUIRED_FLAGS_NONE,
                Advisory: IORING_CREATE_ADVISORY_FLAGS_NONE,
            },
            SubmissionQueueSize: entries,
            CompletionQueueSize: entries * 2,
        };
        let result = unsafe {
            CreateIoRing(
                info.IoRingVersion,
                info.Flags,
                info.SubmissionQueueSize,
                info.CompletionQueueSize,
                &mut (handle as *mut HIORING__) as *mut *mut HIORING__,
            )
        };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { handle, info })
    }
}
