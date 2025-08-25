use std::marker::PhantomData;

use super::Vecx;
use super::raw_val_iter::RawValIter;

pub struct Drain<'a, T:'a> {
    // 这里需要限制生命周期，因此使用了 `&'a mut Vec<T>`
    // 也就是语义上包含的内容
    // 只会调用 `pop()` 和 `remove(0)` 两个方法
    pub vec: PhantomData<&'a mut Vecx<T>>,
    pub iter: RawValIter<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        for _ in &mut * self {}
    }
}
