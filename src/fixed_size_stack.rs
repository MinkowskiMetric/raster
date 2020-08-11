fn assign_swap<T>(target: &mut T, value: T) -> T {
    let mut swap_value = value;
    std::mem::swap(target, &mut swap_value);
    swap_value
}

pub struct FixedSizeStack<'a, T: Sized> {
    data: &'a mut [Option<T>],
    top: usize,
}

impl<'a, T: Sized> FixedSizeStack<'a, T> {
    pub fn new(data: &'a mut [Option<T>]) -> Self {
        Self { data, top: 0 }
    }

    pub fn push(&mut self, value: T) {
        debug_assert!(self.top < self.data.len());
        debug_assert!(self.data[self.top].is_none());

        self.data[self.top] = Some(value);
        self.top += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.top > 0 {
            self.top -= 1;
            debug_assert!(self.data[self.top].is_some());
            assign_swap(&mut self.data[self.top], None)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.top
    }

    pub fn is_full(&self) -> bool {
        self.top >= self.data.len()
    }
}
