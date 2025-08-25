// use std::ptr::{self, NonNull};
// use std::mem::{self, ManuallyDrop};
// use std::alloc::{self, Layout};
use std::{ptr, mem};

use super::raw_val_iter::RawValIter;
use super::raw_vec::RawVec;
use super::Vecx;

// pub struct IntoIterx<T> {
//     buf: NonNull<T>,    // 底层缓冲区
//     cap: usize,         // 容量
//     start: *const T,    // 下一个要返回的元素（前向）
//     end: *const T,      // 下一个要返回的元素（后向）
// }

// pub struct IntoIterx<T> {
//     _buf: RawVec<T>,    // 实际上并不关心这个，只需要保证他们分配的内存不被释放
//     start: *const T,
//     end: *const T,
// }

pub struct IntoIterx<T> {
    _buf: RawVec<T>,
    iter: RawValIter<T>,
}

// next 和 next_back 保持不变，因为它们并没有用到 buf

impl<T> IntoIterator for Vecx<T> {
    type Item = T;
    type IntoIter = IntoIterx<T>;
    fn into_iter(self) -> Self::IntoIter {
        // // 确保 Vecx 不会被 drop
        // // 将原来 Vecx 持有数据的内存释放工作交给 IntoIterx
        // let vec = ManuallyDrop::new(self);

        // // 因为 Vecx 实现了 Drop，所以不能销毁它
        // let ptr = vec.ptr;
        // let cap = vec.cap;
        // let len = vec.len;

        // IntoIterx {
        //     buf: ptr,
        //     cap,
        //     start: ptr.as_ptr(),
        //     end: if cap == 0 {
        //         // 不能通过这个指针获取偏移，因为没有分配内存
        //         ptr.as_ptr()
        //     } else {
        //         // 指向“最后一个元素的下一个位置”
        //         unsafe { ptr.as_ptr().add(len) }
        //     }
        // }

        // // 需要使用 ptr::read 非安全地把 buf 移出，因为它没有实现 Copy
        // // 而且 Vecx 实现了 Drop Trait（因此我们不能销毁它）
        // // 把 `RawVec<T>` 这个结构体的内容按位复制出来（move 语义），不会调用其 `Drop` 实现
        // let buf = unsafe { ptr::read(&self.buf) };
        // let len = self.len;
        // mem::forget(self);

        // IntoIterx {
        //     start: buf.ptr.as_ptr(),
        //     end: if buf.cap == 0 {
        //         buf.ptr.as_ptr()
        //     } else {
        //         unsafe { buf.ptr.as_ptr().add(len) }
        //     },
        //     _buf: buf,
        // }

        unsafe {
            let iter = RawValIter::new(&self);

            let buf = ptr::read(&self.buf);
            mem::forget(self);

            IntoIterx {
                iter,
                _buf: buf,
            }
        }
    }
}

// 向前迭代
impl<T> Iterator for IntoIterx<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // if self.start == self.end {
        //     None
        // } else {
        //     unsafe {
        //         let result = ptr::read(self.start);
        //         self.start = self.start.offset(1);
        //         Some(result)
        //     }
        // }
        self.iter.next()
    }

    // 当前迭代器剩余元素数量
    fn size_hint(&self) -> (usize, Option<usize>) {
        // let len = (self.end as usize - self.start as usize)
        //     / mem::size_of::<T>();

        // (len, Some(len))
        self.iter.size_hint()
    }
}

// 向后迭代
impl<T> DoubleEndedIterator for IntoIterx<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // if self.start == self.end {
        //     None
        // } else {
        //     unsafe {
        //         self.end = self.end.offset(-1);
        //         Some(ptr::read(self.end))
        //     }
        // }
        self.iter.next_back()
    }
}

// 因为 IntoIterx 拥有其分配的所有权，需要实现 Drop 来释放它
impl<T> Drop for IntoIterx<T> {
    fn drop(&mut self) {
        // if self.cap != 0 {
        //     // 将剩下的元素 drop
        //     for _ in &mut *self {}

        //     let layout = Layout::array::<T>(self.cap).unwrap();
        //     unsafe {
        //         alloc::dealloc(self.buf.as_ptr() as *mut u8, layout);
        //     }
        // }

        // 只需要确保 Vecx 中所有元素都被读取
        // 在这之后这些元素会被自动清理
        for _ in &mut *self {}
    }
}
