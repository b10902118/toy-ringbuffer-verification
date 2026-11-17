use verified_ringbuffer::RingBuffer;

fn main() {
    let mut queue = RingBuffer::new();
    assert!(queue.push(10));
    assert!(queue.push(20));
    println!("{:?}, {:?}, {:?}", queue.pop(), queue.pop(), queue.pop());
}
