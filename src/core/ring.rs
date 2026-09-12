//! Fixed-capacity ring buffer. Preallocated in the constructor; `push` performs
//! no heap allocation once the buffer reaches capacity (Tier A requirement).

#[derive(Clone, Debug)]
pub struct RingBuffer<T: Clone> {
    data: Vec<T>,
    start: usize,
    len: usize,
}

impl<T: Clone> RingBuffer<T> {
    /// Create a ring buffer with the given capacity, pre-filled with `init`.
    ///
    /// # Panics
    /// Panics if `capacity == 0`.
    pub fn new(capacity: usize, init: T) -> Self {
        assert!(capacity > 0, "RingBuffer capacity must be > 0");
        Self { data: vec![init; capacity], start: 0, len: 0 }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.len()
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == self.data.len()
    }

    /// Push a value, returning the evicted oldest value once full.
    #[inline]
    pub fn push(&mut self, value: T) -> Option<T> {
        let cap = self.data.len();
        if self.len == cap {
            let evicted = std::mem::replace(&mut self.data[self.start], value);
            self.start = if self.start + 1 == cap { 0 } else { self.start + 1 };
            Some(evicted)
        } else {
            let idx = self.start + self.len;
            let idx = if idx >= cap { idx - cap } else { idx };
            self.data[idx] = value;
            self.len += 1;
            None
        }
    }

    /// Logical index `i` where `0` is the oldest retained value.
    #[inline]
    pub fn get(&self, i: usize) -> &T {
        debug_assert!(i < self.len);
        let cap = self.data.len();
        let mut idx = self.start + i;
        if idx >= cap {
            idx -= cap;
        }
        &self.data[idx]
    }

    /// Most recently pushed value.
    #[inline]
    pub fn last(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            Some(self.get(self.len - 1))
        }
    }

    /// Copy all retained values, oldest first, into `out` (which is resized).
    pub fn fill_vec(&self, out: &mut Vec<T>) {
        out.clear();
        out.reserve(self.len);
        for i in 0..self.len {
            out.push(self.get(i).clone());
        }
    }

    pub fn clear(&mut self, init: T) {
        for slot in self.data.iter_mut() {
            *slot = init.clone();
        }
        self.start = 0;
        self.len = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_evicts_oldest() {
        let mut rb: RingBuffer<f64> = RingBuffer::new(3, 0.0);
        assert_eq!(rb.push(1.0), None);
        assert_eq!(rb.push(2.0), None);
        assert_eq!(rb.push(3.0), None);
        assert!(rb.is_full());
        assert_eq!(rb.push(4.0), Some(1.0));
        let mut v = Vec::new();
        rb.fill_vec(&mut v);
        assert_eq!(v, vec![2.0, 3.0, 4.0]);
        assert_eq!(*rb.get(0), 2.0);
        assert_eq!(*rb.last().unwrap(), 4.0);
    }
}
