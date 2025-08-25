use std::ptr;
use std::mem;


pub struct RawValIter<T> {
    pub start: *const T,
    pub end: *const T,
}

impl<T> RawValIter<T> {
    // 构建 RawVaIter 是不安全的，因为它没有关联的生命周期，
    // 将 RawValIter 存储在与它实际分配相同的结构体中是非常有必要的，
    // 但这里是具体的实现细节，不用对外公开
    pub unsafe fn new(slice: &[T]) -> Self {
        RawValIter {
            start: slice.as_ptr(),
            end: if slice.len() == 0 {
                slice.as_ptr()
            } else {
                // 未分配内存，需要避免使用 offset，
                // 因为那样会给 LLVM 的 GEP 传递错误的信息
                slice.as_ptr().add(slice.len())
            }
        }
    }
}

impl<T> Iterator for RawValIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let result = ptr::read(self.start);
                self.start = self.start.add(1);
                Some(result)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize)
            / mem::size_of::<T>();

        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for RawValIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = self.end.offset(-1);
                Some(self.end.read())
            }
        }
    }
}
