// //! https://nomicon.purewhite.io/exception-safety.html

// struct Hole<'a, T: 'a> {
//     data: &'a mut [T],
//     /// `elt` 从始至终都是 Some
//     elt: Option<T>,
//     pos: usize,
// }

// impl<'a, T> Hole<'a, T> {
//     fn new(data: &'a mut [T], pos: usize) -> Self {
//         unsafe {
//             let elt = ptr::read(&data[pos]);
//             Hole {
//                 data: data,
//                 elt: Some(elt),
//                 pos: pos,
//             }
//         }
//     }

//     fn pos(&self) -> usize { self.pos }

//     fn removed(&self) -> &T { self.elt.as_ref().unwrap() }

//     fn get(&self, index: usize) -> &T { &self.data[index] }

//     unsafe fn move_to(&mut self, index: usize) {
//         let index_ptr: *const _ = &self.data[index];
//         let hole_ptr = &mut self.data[self.pos];
//         ptr::copy_nonoverlapping(index_ptr, hole_ptr, 1);
//         self.pos = index;
//     }
// }

// impl<'a, T> Drop for Hole<'a, T> {
//     fn drop(&mut self) {
//         // fill the hole again
//         unsafe {
//             let pos = self.pos;
//             ptr::write(&mut self.data[pos], self.elt.take().unwrap());
//         }
//     }
// }

// impl<T: Ord> BinaryHeap<T> {
//     fn sift_up(&mut self, pos: usize) {
//         unsafe {
//             // 取出 `pos` 的值，然后创建一个 hole
//             let mut hole = Hole::new(&mut self.data, pos);

//             while hole.pos() != 0 {
//                 let parent = parent(hole.pos());
//                 if hole.removed() <= hole.get(parent) { break }
//                 hole.move_to(parent);
//             }
//             // 无论是否 panic，这里的 hole 都会被无条件填充
//         }
//     }
// }
