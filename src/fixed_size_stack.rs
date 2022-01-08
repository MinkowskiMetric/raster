use std::mem::MaybeUninit;

pub struct FixedSizeStack<'a, T: Sized> {
    data: &'a mut [MaybeUninit<T>],
    top: usize,
}

impl<'a, T: Sized> FixedSizeStack<'a, T> {
    pub fn new(data: &'a mut [MaybeUninit<T>]) -> Self {
        Self { data, top: 0 }
    }

    pub fn try_push(&mut self, value: T) -> bool {
        if self.len() < self.capacity() {
            self.data[self.top].write(value);
            self.top += 1;
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.top > 0 {
            self.top -= 1;
            Some(unsafe { self.data[self.top].assume_init_read() })
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.top
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }
}

impl<'a, T: Sized> Drop for FixedSizeStack<'a, T> {
    fn drop(&mut self) {
        for idx in 0..self.top {
            unsafe { self.data[idx].assume_init_drop() };
        }
    }
}
