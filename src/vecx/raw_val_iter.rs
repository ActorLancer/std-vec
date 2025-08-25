use std::ptr;
use std::mem;
use std::ptr::NonNull;


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
            end: if mem::size_of::<T>() == 0 {
                ((slice.as_ptr() as usize) + slice.len()) as *const T
            } else if slice.len() == 0 {
                slice.as_ptr()
            } else {
                slice.as_ptr().add(slice.len())
            },
        }
    }
}

impl<T> Iterator for RawValIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            // *const T 强转为 usize，加 1，再转回 *const T，
            // 得到了一个完全随机对齐的指针地址，它可能不满足 T 的对齐要求，
            // ZST 的 ptr::read 实际是 no-op，不会真的访问内存
            unsafe {
                if mem::size_of::<T>() == 0 {
                    self.start = (self.start as usize + 1) as *const T;
                    Some(ptr::read(NonNull::<T>::dangling().as_ptr()))
                } else {
                    let old_ptr = self.start;
                    self.start = self.start.offset(1);
                    Some(ptr::read(old_ptr))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let elem_size = mem::size_of::<T>();
        let len = (self.end as usize - self.start as usize)
            / if elem_size == 0 { 1 } else { elem_size };

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
