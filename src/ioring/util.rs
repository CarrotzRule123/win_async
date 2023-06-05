pub struct Array<T> {
    _inner: T
}

impl <T: Sized>Array<T> {
    pub fn get_offset<'a>(&self, idx: isize) -> &'a T {
        let size = std::mem::size_of::<T>();
        let ptr = self as *const Self;
        unsafe {
            let idx = ptr.offset(size as isize * idx);
            &*(idx as *const T)
        }
    }
    
    pub fn set_offset<'a>(&self, idx: isize, value: T) {
        let size = std::mem::size_of::<T>();
        let ptr = self as *const Self;
        unsafe {
            let idx = ptr.offset(size as isize * idx);
            *(idx as *mut T) = value
        }
    }
}