use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let c = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                c.fetch_add(1, Ordering::Release);
            }
        });

        handles.push(handle);
    }
    // handles.into_iter().map(|handle| handle.join().unwrap());
    handles.into_iter().for_each(|handle| { handle.join().unwrap(); });

    println!("最终计数：{}", counter.load(Ordering::Acquire));
}
