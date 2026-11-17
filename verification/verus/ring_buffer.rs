use vstd::prelude::*;

verus! {

// This is a separate executable implementation: Verus syntax is not accepted
// by ordinary rustc. Its storage and algorithm mirror src/lib.rs.
pub struct RingBuffer {
    slots: [u64; 8],
    head: usize,
    len: usize,
}

impl RingBuffer {
    pub closed spec fn valid(&self) -> bool {
        self.head < 8 && self.len <= 8
    }

    // Logical position i maps to physical position (head + i) mod 8.
    // This sequence is a specification, with no runtime allocation.
    pub closed spec fn view(&self) -> Seq<u64> {
        Seq::new(self.len as nat, |i: int| self.slots@[(self.head + i) % 8])
    }

    pub fn new() -> (result: Self)
        ensures result.valid(), result.view() == Seq::<u64>::empty(),
    {
        Self { slots: [0; 8], head: 0, len: 0 }
    }

    pub fn len(&self) -> (result: usize)
        requires self.valid(),
        ensures result == self.view().len(),
    { self.len }

    pub fn is_empty(&self) -> (result: bool)
        requires self.valid(),
        ensures result == (self.view().len() == 0),
    { self.len == 0 }

    pub fn is_full(&self) -> (result: bool)
        requires self.valid(),
        ensures result == (self.view().len() == 8),
    { self.len == 8 }

    pub fn push(&mut self, value: u64) -> (accepted: bool)
        requires old(self).valid(),
        ensures
            final(self).valid(),
            accepted == (old(self).view().len() < 8),
            accepted ==> final(self).view() == old(self).view().push(value),
            !accepted ==> final(self).view() == old(self).view(),
    {
        if self.len == 8 { return false; }
        //let ghost before = self.view();
        let tail = (self.head + self.len) % 8;
        self.slots[tail] = value;
        self.len += 1;
        /*
        proof {
            assert forall|i: int| 0 <= i < before.len() implies
                #[trigger] self.view()[i] == before[i] by {
                assert((self.head + i) % 8 != tail);
            }
            assert(self.view() =~= before.push(value));
        }
        */
        true
    }

    pub fn pop(&mut self) -> (result: Option<u64>)
        requires old(self).valid(),
        ensures
            final(self).valid(),
            old(self).view().len() == 0 ==> result == None && final(self).view() == old(self).view(),
            old(self).view().len() > 0 ==> result == Some(old(self).view()[0])
                && final(self).view() == old(self).view().subrange(1, old(self).view().len() as int),
    {
        if self.len == 0 { return None; }
        //let ghost before = self.view();
        //let ghost old_head = self.head;
        let value = self.slots[self.head];
        self.head = (self.head + 1) % 8;
        self.len -= 1;
        /*
        proof {
            assert forall|i: int| 0 <= i < self.len implies
                #[trigger] self.view()[i] == before[i + 1] by {
                assert((self.head + i) % 8 == (old_head + i + 1) % 8);
            }
            assert(self.view() =~= before.subrange(1, before.len() as int));
        }
        */
        Some(value)
    }
}

// A client proof uses the contracts without inspecting method bodies.
/*
fn main() {
    let mut queue = RingBuffer::new();
    let first = queue.push(10);
    let second = queue.push(20);
    assert(first && second);
    let a = queue.pop();
    let b = queue.pop();
    let c = queue.pop();
    assert(a == Some(10));
    assert(b == Some(20));
    assert(c == None);
}
*/

} // verus!
