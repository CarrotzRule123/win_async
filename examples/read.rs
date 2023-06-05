// use std::fs::File;
// use std::os::windows::prelude::AsRawHandle;

use win_async::IoRing;

fn main() {
    let ring = IoRing::new(16).unwrap();
    let mut buffer = [0u8; 10];
    ring.push_entry_register_buffers(&mut buffer as *mut [u8; 10] as isize, 1, 10);
    ring.submit(1, u32::MAX, 1).unwrap();
}
