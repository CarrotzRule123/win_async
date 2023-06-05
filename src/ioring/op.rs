use windows_sys::Win32::Foundation::HANDLE;

#[repr(C)]
#[derive(Debug)]
pub enum IoRingOpFlags {
    None = 0,
    RegisteredFile = 1,
    RegisteredBuffer = 2,
}

#[repr(C)]
#[derive(Debug)]
pub enum RegBufferReqFlags {
    None = 0,
}

#[repr(C)]
#[derive(Debug)]
pub enum IoRingOp {
    RegisterBuffers(OpRegisterBuffers),
    Read(OpRead),
    Align([u8; 34]),
}

#[repr(C)]
#[derive(Debug)]
pub struct OpRead {
    pub flags: IoRingOpFlags,
    pub padding: u32,
    pub file: HANDLE,
    pub buffer: HANDLE,
    pub offset: u64,
    pub length: u32,
    pub key: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct BufferInfo {
    pub address: HANDLE,
    pub length: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct OpRegisterBuffers {
    pub flags: IoRingOpFlags,
    pub count: u32,
    pub info: *const BufferInfo,
}
