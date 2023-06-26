use std::ffi::c_void;
use std::os::windows::prelude::RawSocket;

use windows_sys::Win32::Foundation::{
    RtlNtStatusToDosError, HANDLE, NTSTATUS, STATUS_NOT_FOUND, STATUS_PENDING, STATUS_SUCCESS,
    UNICODE_STRING,
};
use windows_sys::Win32::Storage::FileSystem::{
    NtCreateFile, FILE_OPEN, FILE_SHARE_READ, FILE_SHARE_WRITE, SYNCHRONIZE,
};
use windows_sys::Win32::System::WindowsProgramming::{
    NtDeviceIoControlFile, IO_STATUS_BLOCK, IO_STATUS_BLOCK_0, OBJECT_ATTRIBUTES,
};

#[link(name = "ntdll")]
extern "system" {
    /// See <https://processhacker.sourceforge.io/doc/ntioapi_8h.html#a0d4d550cad4d62d75b76961e25f6550c>
    ///
    /// This is an undocumented API and as such not part of <https://github.com/microsoft/win32metadata>
    /// from which `windows-sys` is generated, and also unlikely to be added, so
    /// we manually declare it here
    fn NtCancelIoFileEx(
        FileHandle: HANDLE,
        IoRequestToCancel: *mut IO_STATUS_BLOCK,
        IoStatusBlock: *mut IO_STATUS_BLOCK,
    ) -> NTSTATUS;
}

macro_rules! s {
    ($($id:expr)+) => {
        &[$($id as u16),+]
    }
}

const AFD_NAME: &[u16] = s!['\\' 'D' 'e' 'v' 'i' 'c' 'e' '\\' 'A' 'f' 'd' '\\' 'I' 'o'];

pub type AfdPollEvent = u32;

pub enum AfdPollEvents {
    Receive = 0x001,
    ReceiveExpedited = 0x002,
    Send = 0x004,
    Disconnect = 0x008,
    Abort = 0x010,
    LocalClose = 0x020,
    Accept = 0x080,
    ConnectFail = 0x100,
}

#[repr(C)]
pub struct AfdPollHandleInfo {
    pub handle: HANDLE,
    pub events: u32,
    pub status: NTSTATUS,
}

#[repr(C)]
pub struct AfdPollInfo {
    pub timeout: i64,
    pub handle_count: u32,
    pub exclusive: u32,
    pub handles: [AfdPollHandleInfo; 1],
}

pub struct Afd {
    handle: HANDLE,
}

impl Afd {
    pub(super) fn new() -> std::io::Result<Self> {
        let mut device_name = UNICODE_STRING {
            Length: std::mem::size_of_val(AFD_NAME) as u16,
            MaximumLength: std::mem::size_of_val(AFD_NAME) as u16,
            Buffer: AFD_NAME.as_ptr() as *mut u16,
        };
        let mut device_attributes = OBJECT_ATTRIBUTES {
            Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: 0,
            ObjectName: &mut device_name,
            Attributes: 0,
            SecurityDescriptor: std::ptr::null_mut(),
            SecurityQualityOfService: std::ptr::null_mut(),
        };
        let mut handle = 0 as HANDLE;
        let mut iosb = unsafe { std::mem::zeroed::<IO_STATUS_BLOCK>() };
        let result = unsafe {
            NtCreateFile(
                &mut handle,
                SYNCHRONIZE,
                &mut device_attributes,
                &mut iosb,
                std::ptr::null_mut(),
                0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                FILE_OPEN,
                0,
                std::ptr::null_mut(),
                0,
            )
        };

        if result != STATUS_SUCCESS {
            let error = unsafe { RtlNtStatusToDosError(result) };
            return Err(std::io::Error::from_raw_os_error(error as i32));
        }
        Ok(Self { handle })
    }

    pub fn poll(
        &self,
        info: *mut AfdPollInfo,
        iosb: *mut IO_STATUS_BLOCK,
        base_socket: RawSocket,
    ) -> std::io::Result<()> {
        const IOCTL_AFD_POLL: u32 = 0x00012024;
        unsafe { (*iosb).Anonymous.Status = STATUS_PENDING };
        let result = unsafe {
            NtDeviceIoControlFile(
                self.handle,
                0,
                None,
                iosb as *mut c_void,
                iosb,
                IOCTL_AFD_POLL,
                info as *mut c_void,
                std::mem::size_of::<AfdPollInfo>() as u32,
                info as *mut c_void,
                std::mem::size_of::<AfdPollInfo>() as u32,
            )
        };

        match result {
            STATUS_SUCCESS => Ok(()),
            STATUS_PENDING => Err(std::io::ErrorKind::WouldBlock.into()),
            status => {
                let error = unsafe { RtlNtStatusToDosError(status) };
                Err(std::io::Error::from_raw_os_error(error as i32))
            }
        }
    }

    pub unsafe fn cancel(&self, iosb: *mut IO_STATUS_BLOCK) -> std::io::Result<()> {
        if (*iosb).Anonymous.Status != STATUS_PENDING {
            return Ok(());
        }
        let mut cancel_iosb = IO_STATUS_BLOCK {
            Anonymous: IO_STATUS_BLOCK_0 { Status: 0 },
            Information: 0,
        };
        let status = NtCancelIoFileEx(self.handle as HANDLE, iosb, &mut cancel_iosb);

        if status == STATUS_SUCCESS || status == STATUS_NOT_FOUND {
            Ok(())
        } else {
            let error = RtlNtStatusToDosError(status);
            Err(std::io::Error::from_raw_os_error(error as i32))
        }
    }
}
