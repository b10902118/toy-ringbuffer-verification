use std::collections::VecDeque;
use verified_ringbuffer::{RingBuffer, CAPACITY};

#[test]
fn full_empty_and_wraparound() {
    let mut q = RingBuffer::new();
    assert_eq!(q.pop(), None);
    for i in 0..CAPACITY {
        assert!(q.push(i as u64));
    }
    assert!(q.is_full());
    assert!(!q.push(999));
    for i in 0..4 {
        assert_eq!(q.pop(), Some(i));
    }
    for i in 8..12 {
        assert!(q.push(i));
    }
    for i in 4..12 {
        assert_eq!(q.pop(), Some(i));
    }
    assert!(q.is_empty());
    assert_eq!(q.pop(), None);
    assert!(q.push(u64::MAX));
    assert_eq!(q.pop(), Some(u64::MAX));
}

#[test]
fn compare_with_standard_queue() {
    let mut q = RingBuffer::default();
    let mut model = VecDeque::new();
    let mut seed = 42_u64;
    for step in 0..100_000 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        if (seed >> 32) & 1 == 0 {
            let accepted = model.len() < CAPACITY;
            assert_eq!(q.push(seed), accepted, "push at step {step}");
            if accepted {
                model.push_back(seed);
            }
        } else {
            assert_eq!(q.pop(), model.pop_front(), "pop at step {step}");
        }
        assert_eq!(q.len(), model.len());
        assert_eq!(q.is_empty(), model.is_empty());
        assert_eq!(q.is_full(), model.len() == CAPACITY);
    }
    while let Some(value) = model.pop_front() {
        assert_eq!(q.pop(), Some(value));
    }
    assert_eq!(q.pop(), None);
}
