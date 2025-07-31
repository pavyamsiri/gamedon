/// A latch of flags.
#[derive(Debug, Clone, Copy, Default)]
pub struct FlagLatch {
    /// The previous latch value.
    prev: bool,
    /// The current latch value.
    curr: bool,
}

impl FlagLatch {
    /// Update the latch.
    #[inline]
    pub fn tick(&mut self) {
        self.prev = self.curr.clone();
    }

    /// Whether the latch sees a rising edge.
    #[inline]
    pub const fn rising(&self) -> bool {
        !self.prev && self.curr
    }

    /// Logical or a `value` into the latch.
    #[inline]
    pub const fn or(&mut self, value: bool) {
        self.curr |= value;
    }
}
