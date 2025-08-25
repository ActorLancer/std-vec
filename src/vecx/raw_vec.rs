use std::isize;
use std::ptr::NonNull;
use std::alloc::{self, Layout};
use std::mem::{self};

pub struct RawVec<T> {
    pub ptr: NonNull<T>,
    pub cap: usize,
}

unsafe impl<T:Send> Send for RawVec<T> {}
unsafe impl<T:Sync> Sync for RawVec<T> {}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "TODO: implement ZST support");

        RawVec {
            ptr: NonNull::dangling(),
            cap: 0,
        }
    }

    pub fn grow(&mut self) {
        // 保证新申请的内存没有超过 `isize::MAX` 字节
        let new_cap = if self.cap == 0 { 1 } else { 2 * self.cap };

        // `Layout::array` 会检查申请的内存空间是否小于 usize::MAX
        // 但是因为 old_layout.size() <= isize::MAX，
        // 所以这里的 unwrap 永远不可能失败
        let new_layout = Layout::array::<T>(new_cap).unwrap();

        // 保证新申请的内存没有超过 `size::MAX` 字节
        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        // 如果分配失败，`new_ptr` 就会成为空指针，我们需要对应 abort 的操作
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };

        self.cap = new_cap;
    }
}
