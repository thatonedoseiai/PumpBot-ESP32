use std::sync::Mutex;
use std::collections::HashMap;

pub(crate) static PROFILER: Mutex<Profiler> = Mutex::new(Profiler::new());

pub struct Span {
    pub name: &'static str,
    pub start: i64,
    pub end: i64,
    pub depth: usize,
}

pub struct Profiler {
    spans: Vec<Span>,
    stack: Vec<(&'static str, i64, usize)>,
    depth: usize,
}

impl Profiler {
    pub const fn new() -> Self {
        Self { spans: Vec::new(), stack: Vec::new(), depth: 0 }
    }

    pub fn enter(&mut self, name: &'static str) {
        let now = unsafe { esp_idf_sys::esp_timer_get_time() };
        self.stack.push((name, now, self.depth));
        self.depth += 1;
    }

    pub fn exit(&mut self) {
        let now = unsafe { esp_idf_sys::esp_timer_get_time() };
        if let Some((name, start, depth)) = self.stack.pop() {
            self.depth = depth;
            self.spans.push(Span { name, start, end: now, depth });
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
        PROFILER.lock().unwrap().enter(name);
        SpanGuard
    }

    pub fn dump() {
        PROFILER.lock().unwrap().dump();
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        PROFILER.lock().unwrap().exit();
    }
}

#[macro_export]
macro_rules! timed {
    ($label:expr, $block:expr) => {{
        let _guard = SpanGuard::new($label);
        $block
    }};
}