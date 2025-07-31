use core::{clone, default, fmt, iter, ops};
use std::mem::MaybeUninit;
use thiserror::Error;

/// Errors that can occur when pushing to a queue.
#[derive(Debug, Error)]
pub(crate) enum QueuePushError {
    #[error("The queue (size={size}) is full.")]
    QueueIsFull { size: usize },
}

/// A queue that acts FIFO.
pub(crate) struct Queue<const N: usize, T> {
    /// The data.
    data: [MaybeUninit<T>; N],
    /// The index to the first filled slot.
    head: usize,
    /// The index to the first free slot.
    tail: usize,
    /// The number of the elements in the queue currently.
    len: usize,
}

/// A queue that acts FIFO.
pub(crate) struct QueueIterator<'a, const N: usize, T> {
    /// The queue to iterate over.
    queue: &'a Queue<N, T>,
    /// The current queue index.
    index: usize,
}

impl<'a, const N: usize, T> iter::Iterator for QueueIterator<'a, N, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let queue = self.queue;
        // Queue is empty.
        if queue.len == 0 {
            return None;
        }

        // The queue is wrapped so `head > tail` or queue is full `head = tail`.
        if queue.head >= queue.tail {
            if !(queue.head..queue.tail).contains(&self.index) {
                return None;
            }
            #[expect(
                unsafe_code,
                reason = "Elements from head to tail are guaranteed to be initialised by push."
            )]
            let value = unsafe { queue.data[self.index].assume_init_ref() };
            self.index = self.index.wrapping_add(1) % N;
            Some(value)
        }
        // The queue is not wrapped head < head
        else {
            if !(0..queue.tail).contains(&self.index) && !(queue.head..N).contains(&self.index) {
                return None;
            }
            #[expect(
                unsafe_code,
                reason = "Elements from head to tail are guaranteed to be initialised by push."
            )]
            let value = unsafe { queue.data[self.index].assume_init_ref() };
            self.index = self.index.wrapping_add(1) % N;
            Some(value)
        }
    }
}

impl<const N: usize, T: Clone> clone::Clone for Queue<N, T> {
    fn clone(&self) -> Self {
        let mut new_queue = Queue::<N, T>::default();

        // Queue is empty.
        if self.len == 0 {
            return new_queue;
        }

        // The queue is wrapped so `head > tail` or queue is full `head = tail`.
        if self.head >= self.tail {
            // Clone from head to end
            for i in self.head..N {
                #[expect(
                    unsafe_code,
                    reason = "Elements from head to end are guaranteed to be initialised by push."
                )]
                let value = unsafe { self.data[i].assume_init_ref() }.clone();
                new_queue.push(value).expect(
                    "The number of elements being pushed in is strictly less than the size.",
                );
            }

            // Clone from start to tail
            for i in 0..self.tail {
                #[expect(
                    unsafe_code,
                    reason = "Elements from start to tail are guaranteed to be initialised by push."
                )]
                let value = unsafe { self.data[i].assume_init_ref() }.clone();
                new_queue.push(value).expect(
                    "The number of elements being pushed in is strictly less than the size.",
                );
            }
        }
        // The queue is not wrapped head < head
        else {
            for i in self.head..self.tail {
                #[expect(
                    unsafe_code,
                    reason = "Elements from head to tail are guaranteed to be initialised by push."
                )]
                let value = unsafe { self.data[i].assume_init_ref() }.clone();
                new_queue.push(value).expect(
                    "The number of elements being pushed in is strictly less than the size.",
                );
            }
        }

        new_queue
    }
}

impl<const N: usize, T> default::Default for Queue<N, T> {
    fn default() -> Self {
        Self {
            #[expect(
                unsafe_code,
                reason = "Array will only be indexed when memory is valid."
            )]
            data: unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() },
            head: 0,
            tail: 0,
            len: 0,
        }
    }
}

impl<const N: usize, T> ops::Drop for Queue<N, T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<const N: usize, T> Queue<N, T> {
    /// Push a `value` onto the end of the queue if there is space.
    pub(crate) fn push(&mut self, value: T) -> Result<(), QueuePushError> {
        if self.len >= N {
            return Err(QueuePushError::QueueIsFull { size: N });
        }

        self.data[self.tail].write(value);
        self.tail = self.tail.wrapping_add(1) % N;
        self.len = self.len.wrapping_add(1);

        Ok(())
    }

    /// Pop a `value` from the front of the queue if it is not empty.
    pub(crate) const fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        #[expect(
            unsafe_code,
            reason = "Head should always point to valid values. Popping means we take ownership so we can copy it."
        )]
        let value = unsafe { self.data[self.head].assume_init_read() };
        self.head = self.head.wrapping_add(1) % N;
        self.len = self.len.wrapping_sub(1);

        Some(value)
    }

    /// Clear the queue of items.
    pub(crate) fn clear(&mut self) {
        while let Some(value) = self.pop() {
            drop(value);
        }
    }

    // Return an iterator over the items.
    pub(crate) const fn iter(&self) -> QueueIterator<'_, N, T> {
        QueueIterator {
            queue: self,
            index: 0,
        }
    }
}

impl<const N: usize, T: fmt::Debug> fmt::Debug for Queue<N, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Queue")
            .field("capacity", &N)
            .field("head", &self.head)
            .field("tail", &self.tail)
            .field("len", &self.len)
            .field("items", &DebugRing(self))
            .finish_non_exhaustive()
    }
}

// Intermediate struct to help in implementing debug for queue.
struct DebugRing<'a, const N: usize, T>(&'a Queue<N, T>);

impl<const N: usize, T: fmt::Debug> fmt::Debug for DebugRing<'_, N, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();

        for value in self.0.iter() {
            list.entry(value);
        }

        list.finish()
    }
}
