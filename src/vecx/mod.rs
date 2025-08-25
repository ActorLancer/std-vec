pub mod into_iter;
pub mod raw_vec;
pub mod drain;
pub mod raw_val_iter;

use std::marker::PhantomData;
// use std::ptr::NonNull;  // 保证指针非空，在 T 上是协变的
// use std::{isize, mem};
// use std::alloc::{self, Layout};
use std::ptr;
use std::ops::{Drop, Deref, DerefMut};

use raw_val_iter::RawValIter;
use raw_vec::RawVec;
use drain::Drain;

pub struct Vecx<T> {
    // ptr: NonNull<T>,    // 指向堆内存的指针
    // cap: usize,         // 分配的容量（capacity）
    // len: usize,         // 已初始化的元素个数（length）
    buf: RawVec<T>,
    len: usize,
}

impl<T> Vecx<T> {
    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    fn cap(&self) -> usize {
        self.buf.cap
    }

    pub fn new() -> Self {
        Vecx {
            buf: RawVec::new(),
            len: 0,
        }
    }

    pub fn push(&mut self, elem: T) {
        if self.len == self.cap() { self.buf.grow(); }

        unsafe {
            ptr::write(self.ptr().add(self.len), elem);
        }

        // No OOM
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(ptr::read(self.ptr().add(self.len)))
            }
        }
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "index out bounds");
        if self.len == self.cap() { self.buf.grow(); }

        unsafe {
            ptr::copy(
                self.ptr().add(index),
                self.ptr().add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr().add(index), elem);
        }

        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");

        unsafe {
            self.len -= 1;
            let result = ptr::read(self.ptr().add(index));
            ptr::copy(
                self.ptr().add(index + 1),
                self.ptr().add(index),
                self.len - index
            );
            result
        }
    }

    pub fn drain(&mut self) -> Drain<T> {
        let iter = unsafe { RawValIter::new(&self) };
        // 这里事关 mem::forget 的安全
        // 如果 Drain 被 forget,会导致整个 Vecx 的内存泄漏，
        // 因此在这里完成
        self.len = 0;

        Drain {
            iter,
            vec: PhantomData,
        }
    }
}

// // NonNull<T> 本身不会自动传递 Send / Sync 语义
// unsafe impl<T: Send> Send for Vecx<T> {}
// unsafe impl<T: Sync> Sync for Vecx<T> {}

// impl<T> Vecx<T> {
//     pub fn new() -> Self {
//         // 零大小类型需要特殊处理
//         assert!(mem::size_of::<T>() != 0, "We're not already to handle ZSTs");
//         Vecx {
//             ptr: NonNull::dangling(),
//             len: 0,
//             cap: 0,
//         }
//     }

//     // 1. OOM（内存不足）：标准库提供 alloc::handle_alloc_error(layout)，会直接abort程序，而不是panic，避免
//     // unwind 期间再次分配（例如创建回溯信息）。超大申请“触发 OOM”，采用 对齐标准库行为：一旦分配失败就 abort
//     // 2. 将指针封装成 NonNull<T>，真正空的 Vec 不会真的分配（cap = 0），只要 cap == 0 就不会解引用指针
//     // 3. GEP inbounds 表示：指针偏移在同一“已分配实体”范围内，ptr::offset 使用 isize 作为元素偏移，因此无论元素
//     // 大小，总分配字节数必须 <= isize::MAX
//     // 4. x86_64 实际只暴露 ~48 位虚拟地址：在这类平台上，一般先地址空间或无路空间耗尽，早就失败了；但在某些 32 位目
//     // 标，理论可以申请超过 isize::MAX 字节，所以仍要做保守检查（不做平台特化）
//     fn grow(&mut self) {
//         let (new_cap, new_layout) = if self.cap == 0 {
//             (1, Layout::array::<T>(1).unwrap())
//         } else {
//             // 因为 self.cap <= isize::MAX，所以不会溢出
//             let new_cap = 2 * self.cap;

//             // Layout::array 会检查申请的空间是否小于等于 usize::MAX
//             // 但是因为 old_layout.size() <= isize::MAX，
//             // 所以这里的 unwrap 永远不会失败
//             let new_layout = Layout::array::<T>(new_cap).unwrap();
//             (new_cap, new_layout)
//         };

//         // 保证新申请的内存没有超出 isize::MAX 字节的大小
//         assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

//         let new_ptr = if self.cap == 0 {
//             unsafe { alloc::alloc(new_layout) }
//         } else {
//             let old_layout = Layout::array::<T>(self.cap).unwrap();
//             let old_ptr = self.ptr.as_ptr() as *mut u8;
//             unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
//         };

//         // 如果分配失败，new_ptr 就会成为空指针，我们需要对应 abort 的操作
//         self.ptr = match NonNull::new(new_ptr as *mut T) {
//             Some(p) => p,
//             // Panic 会触发栈展开（unwinding），在展开过程中可能需要分配内存（如生成错误信息、backtrace）
//             // 如果系统内存不足，这可能导致二次分配失败
//             None => alloc::handle_alloc_error(new_layout),
//         };
//         self.cap = new_cap;
//     }

//     pub fn push(&mut self, elem: T) {
//         if self.len == self.cap { self.grow(); }

//         unsafe {
//             ptr::write(self.ptr.as_ptr().add(self.len), elem);
//         }

//         // 不可能出错，因为出错之前一定会 OOM（out of memory）
//         self.len += 1;
//     }

//     pub fn pop(&mut self) -> Option<T> {
//         if self.len == 0 {
//             None
//         } else {
//             self.len -= 1;
//             unsafe {
//                 Some(ptr::read(self.ptr.as_ptr().add(self.len)))
//             }
//         }
//     }

//     pub fn insert(&mut self, index: usize, elem: T) {
//         // 注意：`<=` 是因为我们可以把值插入到任何索引范围（[0, length - 1]）内的位置之后
//         // 这种情况等同于 push
//         assert!(index <= self.len, "index out of bounds");
//         if self.len == self.cap { self.grow(); }

//         unsafe {
//             // ptr::copy(src, dest, len) 的含义：“从 src 复制连续的 len 元素到 dst”
//             ptr::copy(
//                 self.ptr.as_ptr().add(index),
//                 self.ptr.as_ptr().add(index + 1),
//                 self.len - index
//             );
//             ptr::write(self.ptr.as_ptr().add(index), elem);
//         }

//         self.len += 1;
//     }

//     pub fn remove(&mut self, index: usize) -> T {
//         // 注意：使用 `<` 是因为 index 不能删除超出元素下标的范围
//         assert!(index < self.len, "index out of bounds");

//         unsafe {
//             self.len -= 1;
//             let result = ptr::read(self.ptr.as_ptr().add(index));
//             ptr::copy(
//                 self.ptr.as_ptr().add(index + 1),
//                 self.ptr.as_ptr().add(index),
//                 self.len - index
//             );
//             result
//         }
//     }
// }

// impl<T> Drop for Vecx<T> {
//     fn drop(&mut self) {
//         if self.cap != 0 {
//             // 如果 T 实现了 Drop trait，则逐个调用元素的析构函数，释放 T 自身的外部资源（文件、socket、堆内存等）
//             if std::mem::needs_drop::<T>() {
//                 while let Some(_) = self.pop() {}
//             }
//             let layout = Layout::array::<T>(self.cap).unwrap();
//             unsafe {
//                 alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
//             }
//         }
//     }
// }

impl<T> Drop for Vecx<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
        // 剩余清理工作由 RawVec 自动完成
    }
}


/// 实现了 Deref 和 DerefMut 这两个 trait 就可以有 len、first、last、索引、切片、排序、iter、iter_mut 以及
/// slice 提供的其他各种功能
impl<T> Deref for Vecx<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe {
            // std::slice::from_raw_parts(self.ptr.as_ptr(), self.len )
            std::slice::from_raw_parts(self.ptr(), self.len)
        }
    }
}

impl<T> DerefMut for Vecx<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
            std::slice::from_raw_parts_mut(self.ptr(), self.len)
        }
    }
}
