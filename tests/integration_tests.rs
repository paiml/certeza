//! Integration tests for certeza framework
//!
//! These tests demonstrate TruenoVec in realistic usage scenarios,
//! verifying system-level properties and cross-module interactions.

use certeza::TruenoVec;

/// Test scenario: Building a dynamic list of user records
#[test]
fn test_user_records_scenario() {
    #[derive(Debug, Clone, PartialEq)]
    struct UserRecord {
        id: u64,
        name: String,
        score: i32,
    }

    let mut users = TruenoVec::new();

    // Add users dynamically
    users.push(UserRecord {
        id: 1,
        name: "Alice".to_string(),
        score: 100,
    });
    users.push(UserRecord {
        id: 2,
        name: "Bob".to_string(),
        score: 85,
    });
    users.push(UserRecord {
        id: 3,
        name: "Charlie".to_string(),
        score: 92,
    });

    assert_eq!(users.len(), 3);

    // Access and verify
    if let Some(user) = users.get(1) {
        assert_eq!(user.name, "Bob");
        assert_eq!(user.score, 85);
    }

    // Update a record
    if let Some(user) = users.get_mut(2) {
        user.score = 95;
    }

    assert_eq!(users.get(2).unwrap().score, 95);

    // Remove last user
    let removed = users.pop();
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().name, "Charlie");
    assert_eq!(users.len(), 2);
}

/// Test scenario: Stack-based expression evaluation
#[test]
fn test_stack_evaluation_scenario() {
    let mut stack = TruenoVec::new();

    // Simulate evaluating: (2 + 3) * 4
    // In postfix: 2 3 + 4 *
    stack.push(2);
    stack.push(3);

    let b = stack.pop().unwrap();
    let a = stack.pop().unwrap();
    stack.push(a + b); // 5

    stack.push(4);

    let b = stack.pop().unwrap();
    let a = stack.pop().unwrap();
    stack.push(a * b); // 20

    assert_eq!(stack.pop(), Some(20));
    assert!(stack.is_empty());
}

/// Test scenario: Batch processing with pre-allocated capacity
#[test]
fn test_batch_processing_scenario() {
    const BATCH_SIZE: usize = 1000;

    // Pre-allocate for efficiency
    let mut batch = TruenoVec::with_capacity(BATCH_SIZE);

    // Verify no allocation during batch fill
    let initial_capacity = batch.capacity();

    for i in 0..BATCH_SIZE {
        batch.push(i);
    }

    assert_eq!(batch.len(), BATCH_SIZE);
    assert_eq!(batch.capacity(), initial_capacity);

    // Verify data integrity
    for i in 0..BATCH_SIZE {
        assert_eq!(batch.get(i), Some(&i));
    }
}

/// Test scenario: Undo/Redo system
#[test]
fn test_undo_redo_scenario() {
    #[derive(Debug, Clone, PartialEq)]
    #[allow(dead_code)]
    enum Action {
        Insert(char),
        Delete,
    }

    let mut undo_stack = TruenoVec::new();
    let mut redo_stack = TruenoVec::new();

    // Perform actions
    undo_stack.push(Action::Insert('H'));
    undo_stack.push(Action::Insert('e'));
    undo_stack.push(Action::Insert('l'));
    undo_stack.push(Action::Insert('l'));
    undo_stack.push(Action::Insert('o'));

    assert_eq!(undo_stack.len(), 5);

    // Undo last two actions
    if let Some(action) = undo_stack.pop() {
        redo_stack.push(action);
    }
    if let Some(action) = undo_stack.pop() {
        redo_stack.push(action);
    }

    assert_eq!(undo_stack.len(), 3);
    assert_eq!(redo_stack.len(), 2);

    // Redo one action
    if let Some(action) = redo_stack.pop() {
        undo_stack.push(action);
    }

    assert_eq!(undo_stack.len(), 4);
    assert_eq!(redo_stack.len(), 1);
}

/// Test scenario: Dynamic graph adjacency list
#[test]
fn test_graph_adjacency_scenario() {
    type NodeId = usize;

    struct Graph {
        adjacency: Vec<TruenoVec<NodeId>>,
    }

    impl Graph {
        fn new(num_nodes: usize) -> Self {
            let mut adjacency = Vec::new();
            for _ in 0..num_nodes {
                adjacency.push(TruenoVec::new());
            }
            Self { adjacency }
        }

        fn add_edge(&mut self, from: NodeId, to: NodeId) {
            if from < self.adjacency.len() {
                self.adjacency[from].push(to);
            }
        }

        fn neighbors(&self, node: NodeId) -> Option<&TruenoVec<NodeId>> {
            self.adjacency.get(node)
        }
    }

    let mut graph = Graph::new(5);

    // Build a small graph: 0 -> 1, 0 -> 2, 1 -> 3, 2 -> 3, 3 -> 4
    graph.add_edge(0, 1);
    graph.add_edge(0, 2);
    graph.add_edge(1, 3);
    graph.add_edge(2, 3);
    graph.add_edge(3, 4);

    // Verify adjacency lists
    let node0_neighbors = graph.neighbors(0).unwrap();
    assert_eq!(node0_neighbors.len(), 2);
    assert_eq!(node0_neighbors.get(0), Some(&1));
    assert_eq!(node0_neighbors.get(1), Some(&2));

    let node3_neighbors = graph.neighbors(3).unwrap();
    assert_eq!(node3_neighbors.len(), 1);
    assert_eq!(node3_neighbors.get(0), Some(&4));
}

/// Test scenario: Ring buffer implementation
#[test]
fn test_ring_buffer_scenario() {
    struct RingBuffer<T> {
        buffer: TruenoVec<T>,
        #[allow(dead_code)]
        head: usize,
        tail: usize,
        capacity: usize,
    }

    impl<T: Clone> RingBuffer<T> {
        fn new(capacity: usize) -> Self {
            Self {
                buffer: TruenoVec::with_capacity(capacity),
                head: 0,
                tail: 0,
                capacity,
            }
        }

        fn push(&mut self, item: T) {
            if self.buffer.len() < self.capacity {
                self.buffer.push(item);
                self.tail = self.buffer.len();
            }
        }

        fn len(&self) -> usize {
            self.buffer.len()
        }
    }

    let mut ring = RingBuffer::new(5);
    ring.push(1);
    ring.push(2);
    ring.push(3);

    assert_eq!(ring.len(), 3);
}

/// Test scenario: Large dataset processing
#[test]
fn test_large_dataset_scenario() {
    const LARGE_SIZE: usize = 10_000;

    let mut data = TruenoVec::new();

    // Fill with sequential data
    for i in 0..LARGE_SIZE {
        data.push(i * 2);
    }

    assert_eq!(data.len(), LARGE_SIZE);

    // Verify exponential growth kept reallocations minimal
    // With 2x growth, we expect approximately log2(10000) ≈ 14 reallocations
    // Final capacity should be >= 16384 (2^14)
    assert!(data.capacity() >= 16384);

    // Verify data integrity
    for i in 0..LARGE_SIZE {
        assert_eq!(data.get(i), Some(&(i * 2)));
    }

    // Test pop operations
    for _ in 0..1000 {
        data.pop();
    }

    assert_eq!(data.len(), LARGE_SIZE - 1000);
}

/// Test scenario: Type safety with different element types
#[test]
fn test_type_safety_scenario() {
    // String vector
    let mut strings = TruenoVec::new();
    strings.push("hello".to_string());
    strings.push("world".to_string());
    assert_eq!(strings.get(0).map(String::as_str), Some("hello"));

    // Option vector
    let mut options: TruenoVec<Option<i32>> = TruenoVec::new();
    options.push(Some(42));
    options.push(None);
    options.push(Some(100));
    assert_eq!(options.get(1), Some(&None));

    // Nested vector
    let mut nested = TruenoVec::new();
    let mut inner1 = TruenoVec::new();
    inner1.push(1);
    inner1.push(2);
    nested.push(inner1);
    assert_eq!(nested.len(), 1);
}

/// Test scenario: Concurrent safety verification (compile-time)
/// This test verifies that TruenoVec is Send + Sync where appropriate
#[test]
fn test_thread_safety_compile_time() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    // TruenoVec<i32> should be Send and Sync
    assert_send::<TruenoVec<i32>>();
    assert_sync::<TruenoVec<i32>>();

    // TruenoVec<String> should be Send and Sync
    assert_send::<TruenoVec<String>>();
    assert_sync::<TruenoVec<String>>();
}

/// Test scenario: Memory efficiency verification
#[test]
fn test_memory_efficiency_scenario() {
    use std::mem;

    let vec = TruenoVec::<i32>::new();

    // TruenoVec should have minimal overhead
    // Expected: pointer (8 bytes) + len (8 bytes) + capacity (8 bytes) + PhantomData (0 bytes)
    let size = mem::size_of_val(&vec);
    assert_eq!(size, 24); // 3 * 8 bytes (no padding needed)

    // Verify zero-cost abstraction for empty vec
    let empty = TruenoVec::<u8>::new();
    assert_eq!(empty.capacity(), 0);
    assert_eq!(empty.len(), 0);
}

/// Test scenario: Error handling and edge cases
#[test]
fn test_edge_cases_scenario() {
    let mut vec = TruenoVec::new();

    // Pop from empty
    assert_eq!(vec.pop(), None);

    // Get out of bounds
    assert_eq!(vec.get(0), None);
    assert_eq!(vec.get(100), None);

    // Single element operations
    vec.push(42);
    assert_eq!(vec.len(), 1);
    assert_eq!(vec.get(0), Some(&42));
    assert_eq!(vec.pop(), Some(42));
    assert!(vec.is_empty());

    // Capacity preservation after pops
    let mut vec2 = TruenoVec::with_capacity(100);
    for i in 0..50 {
        vec2.push(i);
    }
    let capacity_before = vec2.capacity();
    for _ in 0..25 {
        vec2.pop();
    }
    assert_eq!(vec2.capacity(), capacity_before); // Capacity doesn't shrink
}

/// Test scenario: Comparison with standard Vec behavior
#[test]
fn test_std_vec_equivalence() {
    let mut trueno = TruenoVec::new();
    let mut std_vec = Vec::new();

    // Parallel operations
    for i in 0..100 {
        trueno.push(i);
        std_vec.push(i);
    }

    assert_eq!(trueno.len(), std_vec.len());

    for i in 0..100 {
        assert_eq!(trueno.get(i), std_vec.get(i));
    }

    for _ in 0..50 {
        assert_eq!(trueno.pop(), std_vec.pop());
    }

    assert_eq!(trueno.len(), std_vec.len());
}
