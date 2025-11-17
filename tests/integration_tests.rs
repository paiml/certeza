//! Integration Tests for certeza
//!
//! These tests verify the public API of the certeza crate works correctly
//! when used as a library. Integration tests are in the middle tier of the
//! testing pyramid (~10% of tests) and verify system-level properties.

use certeza::TruenoVec;

// ============================================================================
// Basic Integration Tests
// ============================================================================

#[test]
fn test_trueno_vec_basic_operations() {
    let mut vec = TruenoVec::new();

    // Test empty vector
    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);

    // Test push and length
    vec.push(10);
    vec.push(20);
    vec.push(30);
    assert_eq!(vec.len(), 3);
    assert!(!vec.is_empty());

    // Test get
    assert_eq!(vec.get(0), Some(&10));
    assert_eq!(vec.get(1), Some(&20));
    assert_eq!(vec.get(2), Some(&30));
    assert_eq!(vec.get(3), None);

    // Test pop
    assert_eq!(vec.pop(), Some(30));
    assert_eq!(vec.len(), 2);
    assert_eq!(vec.pop(), Some(20));
    assert_eq!(vec.pop(), Some(10));
    assert_eq!(vec.pop(), None);
    assert!(vec.is_empty());
}

#[test]
fn test_trueno_vec_with_capacity() {
    let mut vec = TruenoVec::with_capacity(100);
    assert_eq!(vec.capacity(), 100);
    assert_eq!(vec.len(), 0);

    // Should not reallocate for first 100 elements
    for i in 0..100 {
        vec.push(i);
    }
    assert_eq!(vec.capacity(), 100);
    assert_eq!(vec.len(), 100);

    // Should reallocate after 100 elements
    vec.push(100);
    assert_eq!(vec.capacity(), 200);
    assert_eq!(vec.len(), 101);
}

#[test]
fn test_trueno_vec_stack_semantics() {
    // Verify TruenoVec behaves like a stack (LIFO)
    let mut stack = TruenoVec::new();

    // Push sequence
    for i in 1..=5 {
        stack.push(i);
    }

    // Pop should return in reverse order
    assert_eq!(stack.pop(), Some(5));
    assert_eq!(stack.pop(), Some(4));
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(1));
    assert_eq!(stack.pop(), None);
}

// ============================================================================
// Integration with Standard Rust Types
// ============================================================================

#[test]
fn test_trueno_vec_with_strings() {
    let mut vec = TruenoVec::new();

    vec.push(String::from("hello"));
    vec.push(String::from("world"));
    vec.push(String::from("rust"));

    assert_eq!(vec.len(), 3);
    assert_eq!(vec.get(0).map(String::as_str), Some("hello"));
    assert_eq!(vec.get(1).map(String::as_str), Some("world"));
    assert_eq!(vec.get(2).map(String::as_str), Some("rust"));

    // Test pop with owned strings
    assert_eq!(vec.pop(), Some(String::from("rust")));
    assert_eq!(vec.len(), 2);
}

#[test]
fn test_trueno_vec_with_complex_types() {
    #[derive(Debug, PartialEq, Clone)]
    struct Person {
        name: String,
        age: u32,
    }

    let mut vec = TruenoVec::new();

    vec.push(Person {
        name: String::from("Alice"),
        age: 30,
    });
    vec.push(Person {
        name: String::from("Bob"),
        age: 25,
    });

    assert_eq!(vec.len(), 2);
    assert_eq!(
        vec.get(0),
        Some(&Person {
            name: String::from("Alice"),
            age: 30,
        })
    );

    // Test mutation
    if let Some(person) = vec.get_mut(1) {
        person.age = 26;
    }

    assert_eq!(
        vec.get(1),
        Some(&Person {
            name: String::from("Bob"),
            age: 26,
        })
    );
}

// ============================================================================
// Real-World Usage Scenarios
// ============================================================================

#[test]
fn test_trueno_vec_as_buffer() {
    // Simulate using TruenoVec as a buffer for collecting data
    let mut buffer = TruenoVec::with_capacity(256);

    // Collect some data (0..256 to avoid u8 overflow)
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    for i in 0..256 {
        buffer.push(i as u8);
    }

    assert_eq!(buffer.len(), 256);
    assert_eq!(buffer.capacity(), 256);

    // Process and remove data
    let mut sum = 0u64;
    while let Some(value) = buffer.pop() {
        sum += u64::from(value);
    }

    // Sum of 0..256 = (n * (n-1)) / 2 = (256 * 255) / 2
    assert_eq!(sum, (256 * 255) / 2);
    assert!(buffer.is_empty());
}

#[test]
fn test_trueno_vec_interleaved_operations() {
    // Test realistic pattern of interleaved push/pop operations
    let mut vec = TruenoVec::new();

    // Build up
    for i in 0..10 {
        vec.push(i);
    }
    assert_eq!(vec.len(), 10);

    // Remove some
    for _ in 0..5 {
        vec.pop();
    }
    assert_eq!(vec.len(), 5);

    // Add more
    for i in 10..15 {
        vec.push(i);
    }
    assert_eq!(vec.len(), 10);

    // Verify contents
    assert_eq!(vec.get(0), Some(&0));
    assert_eq!(vec.get(4), Some(&4));
    assert_eq!(vec.get(5), Some(&10));
    assert_eq!(vec.get(9), Some(&14));
}

// ============================================================================
// Error Cases and Edge Conditions
// ============================================================================

#[test]
fn test_trueno_vec_out_of_bounds_access() {
    let mut vec = TruenoVec::new();
    vec.push(1);
    vec.push(2);

    // Out of bounds should return None
    assert_eq!(vec.get(2), None);
    assert_eq!(vec.get(100), None);
    assert_eq!(vec.get(usize::MAX), None);
}

#[test]
fn test_trueno_vec_pop_until_empty() {
    let mut vec = TruenoVec::new();
    vec.push(1);
    vec.push(2);

    assert_eq!(vec.pop(), Some(2));
    assert_eq!(vec.pop(), Some(1));
    assert_eq!(vec.pop(), None);
    assert_eq!(vec.pop(), None); // Multiple pops on empty
    assert_eq!(vec.pop(), None);
}

#[test]
fn test_trueno_vec_zero_capacity_initialization() {
    let vec: TruenoVec<i32> = TruenoVec::with_capacity(0);
    assert_eq!(vec.capacity(), 0);
    assert_eq!(vec.len(), 0);
    assert!(vec.is_empty());
}

// ============================================================================
// Library API Tests
// ============================================================================

#[test]
fn test_certeza_math_functions() {
    // Test the example math functions from lib.rs
    use certeza::{add, multiply};

    assert_eq!(add(2, 3), 5);
    assert_eq!(add(0, 0), 0);
    assert_eq!(add(100, 200), 300);

    assert_eq!(multiply(2, 3), 6);
    assert_eq!(multiply(0, 100), 0);
    assert_eq!(multiply(10, 10), 100);
}

#[test]
fn test_default_trait_integration() {
    let vec: TruenoVec<i32> = TruenoVec::default();
    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 0);
}

// ============================================================================
// Thread Safety Integration Tests
// ============================================================================

#[test]
fn test_trueno_vec_send_across_threads() {
    // Verify TruenoVec can be sent across threads
    let mut vec = TruenoVec::new();
    for i in 0..100 {
        vec.push(i);
    }

    let handle = std::thread::spawn(move || {
        assert_eq!(vec.len(), 100);
        vec.pop()
    });

    assert_eq!(handle.join().unwrap(), Some(99));
}

#[test]
fn test_trueno_vec_with_arc() {
    use std::sync::Arc;

    // Test that TruenoVec works with Arc<T> where T: Sync
    let mut vec = TruenoVec::new();
    let data = Arc::new(42);

    vec.push(Arc::clone(&data));
    vec.push(Arc::clone(&data));
    vec.push(Arc::clone(&data));

    assert_eq!(Arc::strong_count(&data), 4); // 1 original + 3 in vec

    vec.pop();
    assert_eq!(Arc::strong_count(&data), 3);
}
