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
        // assert!(mem::size_of::<T>() != 0, "TODO: implement ZST support");
        let cap = if mem::size_of::<T>() == 0 { usize::MAX } else { 0 };

        // `NonNull::dangling()` 有双重含义：
        // `未分配内存（unallocated）`，`零大小类型（zero-sized allocation）`
        RawVec {
            ptr: NonNull::dangling(),
            cap,
        }
    }

    pub fn grow(&mut self) {
        // 当 T 的 size 为 0 时，设置cap 为 usize::MAX
        assert!(mem::size_of::<T>() != 0, "capacity overflow");

        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            // 保证新申请的内存没有超过 `isize::MAX` 字节
            let new_cap = 2 * self.cap;

            // `Layout::array` 会检查申请的空间 是否小于等于 usize::MAX，
            // 但是因为 old_layout.size() <= isize::MAX，
            // 所以这里的 unwrap 永远不会失败
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        // 保证新申请的内存没有超过 `isize::MAX` 字节
        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        // 如果分配失败，`new_ptr` 就会成为空指针，需要处理这个意外情况
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;

        // // 保证新申请的内存没有超过 `isize::MAX` 字节
        // let new_cap = if self.cap == 0 { 1 } else { 2 * self.cap };

        // // `Layout::array` 会检查申请的内存空间是否小于 usize::MAX
        // // 但是因为 old_layout.size() <= isize::MAX，
        // // 所以这里的 unwrap 永远不可能失败
        // let new_layout = Layout::array::<T>(new_cap).unwrap();

        // // 保证新申请的内存没有超过 `size::MAX` 字节
        // assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        // let new_ptr = if self.cap == 0 {
        //     unsafe { alloc::alloc(new_layout) }
        // } else {
        //     let old_layout = Layout::array::<T>(self.cap).unwrap();
        //     let old_ptr = self.ptr.as_ptr() as *mut u8;
        //     unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        // };

        // // 如果分配失败，`new_ptr` 就会成为空指针，我们需要对应 abort 的操作
        // self.ptr = match NonNull::new(new_ptr as *mut T) {
        //     Some(p) => p,
        //     None => alloc::handle_alloc_error(new_layout),
        // };

        // self.cap = new_cap;
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        let elem_size = mem::size_of::<T>();

        if self.cap != 0 && elem_size != 0 {
            unsafe {
                alloc::dealloc(
                    self.ptr.as_ptr() as *mut u8,
                    Layout::array::<T>(self.cap).unwrap(),
                );
            }
        }
    }
}
