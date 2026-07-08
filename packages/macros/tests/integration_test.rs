use std::sync::atomic::{AtomicU64, Ordering};

struct Counter {
    _revision_bump_counter: AtomicU64,
    value: u64,
    items: Vec<String>,
}

#[arona_macros::auto_bump]
impl Counter {
    fn new() -> Self {
        Self {
            _revision_bump_counter: AtomicU64::new(0),
            value: 0,
            items: Vec::new(),
        }
    }

    #[bump]
    fn increment(&mut self) {
        self.value += 1;
    }

    #[bump]
    fn add(&mut self, n: u64) {
        self.value += n;
    }

    #[bump]
    fn with_early_return(&mut self, x: u64) -> bool {
        if x == 0 {
            return false;
        }
        self.value += x;
        true
    }

    #[bump]
    fn conditional_mutation(&mut self, idx: usize, new_val: &str) -> bool {
        if let Some(item) = self.items.get_mut(idx) {
            *item = new_val.to_string();
            return true;
        }
        false
    }

    #[bump]
    fn push_item(&mut self, val: &str) {
        self.items.push(val.to_string());
    }

    #[bump]
    fn calls_other_method(&mut self) {
        self.push_item("nested");
    }

    fn get(&self) -> u64 {
        self.value
    }

    fn revision(&self) -> u64 {
        self._revision_bump_counter.load(Ordering::Relaxed)
    }
}

#[test]
fn test_auto_bump_increment() {
    let mut c = Counter::new();
    assert_eq!(c.revision(), 0);
    c.increment();
    assert_eq!(c.get(), 1);
    assert_eq!(c.revision(), 1);
}

#[test]
fn test_auto_bump_add() {
    let mut c = Counter::new();
    c.add(5);
    assert_eq!(c.get(), 5);
    assert_eq!(c.revision(), 1);
    c.add(3);
    assert_eq!(c.get(), 8);
    assert_eq!(c.revision(), 2);
}

#[test]
fn test_auto_bump_early_return() {
    let mut c = Counter::new();
    let result = c.with_early_return(0);
    assert!(!result);
    assert_eq!(c.get(), 0);
    assert_eq!(c.revision(), 1);

    let result = c.with_early_return(10);
    assert!(result);
    assert_eq!(c.get(), 10);
    assert_eq!(c.revision(), 2);
}

#[test]
fn test_no_bump_on_read() {
    let mut c = Counter::new();
    c.increment();
    assert_eq!(c.revision(), 1);
    let _ = c.get();
    assert_eq!(c.revision(), 1);
}

#[test]
fn test_conditional_mutation_found() {
    let mut c = Counter::new();
    c.push_item("hello");
    assert_eq!(c.revision(), 1);
    let found = c.conditional_mutation(0, "world");
    assert!(found);
    assert_eq!(c.items[0], "world");
    assert_eq!(c.revision(), 2);
}

#[test]
fn test_conditional_mutation_not_found() {
    let mut c = Counter::new();
    let found = c.conditional_mutation(99, "nope");
    assert!(!found);
    assert_eq!(c.revision(), 1);
}

#[test]
fn test_nested_bump_method_calls_other() {
    let mut c = Counter::new();
    c.calls_other_method();
    assert_eq!(c.items.len(), 1);
    assert_eq!(c.items[0], "nested");
    assert_eq!(c.revision(), 2);
}

#[test]
fn test_revision_monotonic_across_mixed_ops() {
    let mut c = Counter::new();
    assert_eq!(c.revision(), 0);
    c.increment(); // +1 (outer only)
    let _ = c.get(); // +0 (no #[bump])
    c.push_item("a"); // +1
    c.conditional_mutation(0, "b"); // +1
    let _ = c.conditional_mutation(99, "x"); // +1 (early return still bumps)
    c.calls_other_method(); // +2 (outer + inner push_item)
    assert_eq!(c.revision(), 6);
}
