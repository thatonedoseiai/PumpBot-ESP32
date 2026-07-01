#![no_std]
#![no_main]

extern crate alloc;

use embassy_sync::blocking_mutex::{Mutex, CriticalSectionMutex};
use esp_hal::time::Instant;
use alloc::vec::Vec;
use core::ops::Drop;
use core::option::Option::Some;
use core::cell::RefCell;
use esp_println::println;

pub(crate) static PROFILER: CriticalSectionMutex<RefCell<Profiler>> = Mutex::new(RefCell::new(Profiler::new()));

pub struct Span {
    pub name: &'static str,
    pub start: u64,
    pub end: u64,
    pub depth: usize,
}

pub struct Profiler {
    spans: Vec<Span>,
    stack: Vec<(&'static str, u64, usize)>,
    depth: usize,
}

impl Profiler {
    pub const fn new() -> Self {
        Self { spans: Vec::new(), stack: Vec::new(), depth: 0 }
    }

    pub fn enter(&mut self, name: &'static str) {
        // let now = unsafe { esp_idf_sys::esp_timer_get_time() };
        let now = Instant::now().duration_since_epoch();
        self.stack.push((name, now.as_micros(), self.depth));
        self.depth += 1;
    }

    pub fn exit(&mut self) {
        // let now = unsafe { esp_idf_sys::esp_timer_get_time() };
        let now = Instant::now().duration_since_epoch();
        if let Some((name, start, depth)) = self.stack.pop() {
            self.depth = depth;
            self.spans.push(Span { name, start, end: now.as_micros(), depth });
        }
    }

    pub fn dump(&self) {
        // Print as folded stack format for inferno, or as a simple table
        println!("=== PROFILE RESULTS ===");
        for span in &self.spans {
            let indent = "  ".repeat(span.depth);
            println!("{}[{}] {}µs", indent, span.name, span.end - span.start);
        }
    }
}

pub struct SpanGuard;

impl SpanGuard {
    pub fn new(name: &'static str) -> Self {
        PROFILER.lock(|f| f.borrow_mut().enter(name));
        SpanGuard
    }

    pub fn dump() {
        PROFILER.lock(|f| f.borrow().dump());
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        PROFILER.lock(|f| f.borrow_mut().exit());
    }
}

#[macro_export]
macro_rules! timed {
    ($label:expr, $block:expr) => {{
        let _guard = SpanGuard::new($label);
        $block
    }};
}