use std::vec::Vec;
use std::cell::RefCell;

pub struct Queue<T> {
    c: RefCell<Vec<T>>
}

pub struct EspError {}
type TickType_t = u32;

impl<T> Queue<T> where T: Copy {
    pub fn new(_: usize) -> Self {
        Queue {
            c: RefCell::new(Vec::<T>::new())
        }
    }

    // pub unsafe fn new_borrowed(ptr: QueueHandle_t) -> Self { }
    // pub fn as_raw(&self) -> QueueHandle_t { }

    pub fn send_back(&self, item: T, timeout: TickType_t) -> Result<bool, EspError> {
        self.c.borrow_mut().insert(0, item);
        Ok(true)
    }

    pub fn send_front(&self, item: T, timeout: TickType_t) -> Result<bool, EspError> {
        self.c.borrow_mut().push(item);
        Ok(true)
    }

    pub fn recv_front(&self, timeout: TickType_t) -> Option<(T, bool)> {
        Some((self.c.borrow_mut().pop()?, true))
    }

    pub fn peek_front(&self, timeout: TickType_t) -> Option<T> {
        self.c.borrow().last().copied()
    }
}