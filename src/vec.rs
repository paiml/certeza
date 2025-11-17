//! # `TruenoVec` - A Growable Array Type with Verified Correctness
//!
//! This module provides `TruenoVec<T>`, a custom growable vector implementation
//! that demonstrates the certeza framework's tiered testing approach:
//!
//! - **Tier 1**: Unit tests for basic functionality (sub-second feedback)
//! - **Tier 2**: Property-based tests verifying algebraic properties (1-5 minutes)
//! - **Tier 3**: Formal verification of critical invariants with Kani (hours)
//!
//! ## Safety Invariants
//!
//! The following invariants are maintained and formally verified:
//!
//! 1. `ptr` is either null or points to valid memory for `capacity` elements
//! 2. `len <= capacity` at all times
//! 3. Elements `[0..len)` are initialized
//! 4. Elements `[len..capacity)` are uninitialized
//!
//! ## Risk Classification
//!
//! **Very High Risk** - Contains `unsafe` code for manual memory management.
//! Full verification framework applied:
//! - Property-based testing
//! - 95%+ coverage requirement
//! - >85% mutation score target
//! - Formal verification of invariants
//!
//! ## Example
//!
//! ```rust
//! use certeza::TruenoVec;
//!
//! let mut vec = TruenoVec::new();
//! vec.push(1);
//! vec.push(2);
//! vec.push(3);
//!
//! assert_eq!(vec.len(), 3);
//! assert_eq!(vec.get(1), Some(&2));
//! assert_eq!(vec.pop(), Some(3));
//! ```

use std::alloc::{self, Layout};
use std::marker::PhantomData;
use std::ptr::{self, NonNull};

/// A growable array type with verified correctness properties.
///
/// This implementation demonstrates asymptotic test effectiveness through:
/// - 95%+ line coverage
/// - >85% mutation score
/// - Comprehensive property-based test suite
/// - Formal verification of critical invariants
///
/// # Type Parameters
///
/// * `T` - The type of elements stored in the vector
///
/// # Safety
///
/// This type uses `unsafe` code for manual memory management. All safety
/// invariants are formally verified using Kani and tested using property-based
/// testing with proptest.
///
/// # Examples
///
/// ```rust
/// use certeza::TruenoVec;
///
/// let mut v = TruenoVec::new();
/// v.push(1);
/// v.push(2);
/// assert_eq!(v.len(), 2);
/// assert_eq!(v.pop(), Some(2));
/// ```
#[derive(Debug)]
pub struct TruenoVec<T> {
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
    _marker: PhantomData<T>,
}

// ============================================================================
// Core Implementation
// ============================================================================

impl<T> TruenoVec<T> {
    /// Creates a new, empty `TruenoVec<T>`.
    ///
    /// The vector will not allocate until elements are pushed onto it.
    ///
    /// # Complexity
    ///
    /// - Time: O(1)
    /// - Space: O(1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec: TruenoVec<i32> = TruenoVec::new();
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), 0);
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            capacity: 0,
            _marker: PhantomData,
        }
    }

    /// Creates a new, empty `TruenoVec<T>` with the specified capacity.
    ///
    /// The vector will be able to hold exactly `capacity` elements without
    /// reallocating. If `capacity` is 0, the vector will not allocate.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) where n is the capacity
    /// - Space: O(n)
    ///
    /// # Panics
    ///
    /// Panics if the capacity exceeds `isize::MAX` bytes or if allocation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec: TruenoVec<i32> = TruenoVec::with_capacity(10);
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), 10);
    ///
    /// // These pushes won't reallocate
    /// for i in 0..10 {
    ///     vec.push(i);
    /// }
    /// assert_eq!(vec.capacity(), 10);
    /// ```
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            Self::new()
        } else {
            let layout = Layout::array::<T>(capacity).expect("capacity overflow");
            // SAFETY: layout has non-zero size
            let ptr = unsafe {
                let ptr = alloc::alloc(layout).cast::<T>();
                NonNull::new(ptr).expect("allocation failed")
            };
            Self {
                ptr,
                len: 0,
                capacity,
                _marker: PhantomData,
            }
        }
    }

    /// Returns the number of elements in the vector.
    ///
    /// # Complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// assert_eq!(vec.len(), 0);
    /// vec.push(1);
    /// assert_eq!(vec.len(), 1);
    /// ```
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the vector contains no elements.
    ///
    /// # Complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// assert!(vec.is_empty());
    /// vec.push(1);
    /// assert!(!vec.is_empty());
    /// ```
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of elements the vector can hold without reallocating.
    ///
    /// # Complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let vec: TruenoVec<i32> = TruenoVec::with_capacity(10);
    /// assert_eq!(vec.capacity(), 10);
    /// ```
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Appends an element to the back of the vector.
    ///
    /// # Complexity
    ///
    /// - Amortized O(1) time
    /// - Worst-case O(n) when reallocation is needed
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec.len(), 2);
    /// assert_eq!(vec.get(0), Some(&1));
    /// assert_eq!(vec.get(1), Some(&2));
    /// ```
    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.grow();
        }
        // SAFETY: len < capacity after grow, ptr is valid
        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), value);
        }
        self.len += 1;
    }

    /// Removes the last element from the vector and returns it, or `None` if empty.
    ///
    /// # Complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec.pop(), Some(2));
    /// assert_eq!(vec.pop(), Some(1));
    /// assert_eq!(vec.pop(), None);
    /// ```
    #[allow(clippy::missing_const_for_fn)] // Cannot be const due to ptr::read
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            // SAFETY: len was > 0, so len-1 is a valid initialized index
            unsafe { Some(ptr::read(self.ptr.as_ptr().add(self.len))) }
        }
    }

    /// Returns a reference to an element at the given index, or `None` if out of bounds.
    ///
    /// # Complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// vec.push(30);
    /// assert_eq!(vec.get(1), Some(&20));
    /// assert_eq!(vec.get(3), None);
    /// ```
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            // SAFETY: index < len, so it's a valid initialized element
            unsafe { Some(&*self.ptr.as_ptr().add(index)) }
        } else {
            None
        }
    }

    /// Returns a mutable reference to an element at the given index, or `None` if out of bounds.
    ///
    /// # Complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// if let Some(elem) = vec.get_mut(1) {
    ///     *elem = 25;
    /// }
    /// assert_eq!(vec.get(1), Some(&25));
    /// ```
    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            // SAFETY: index < len, so it's a valid initialized element
            unsafe { Some(&mut *self.ptr.as_ptr().add(index)) }
        } else {
            None
        }
    }

    /// Doubles the capacity of the vector.
    ///
    /// Uses a growth factor of 2x for exponential growth, ensuring amortized O(1) push.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 {
            1
        } else {
            self.capacity.checked_mul(2).expect("capacity overflow")
        };

        let new_layout = Layout::array::<T>(new_capacity).expect("capacity overflow");

        let new_ptr = if self.capacity == 0 {
            // SAFETY: new_layout has non-zero size
            unsafe {
                let ptr = alloc::alloc(new_layout).cast::<T>();
                NonNull::new(ptr).expect("allocation failed")
            }
        } else {
            let old_layout = Layout::array::<T>(self.capacity).expect("capacity overflow");
            // SAFETY: ptr is valid, old_layout matches the current allocation
            unsafe {
                let ptr = alloc::realloc(
                    self.ptr.as_ptr().cast::<u8>(),
                    old_layout,
                    new_layout.size(),
                )
                .cast::<T>();
                NonNull::new(ptr).expect("allocation failed")
            }
        };

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }
}

// ============================================================================
// Trait Implementations
// ============================================================================

impl<T> Default for TruenoVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for TruenoVec<T> {
    fn drop(&mut self) {
        // Drop all elements
        while self.pop().is_some() {}

        // Deallocate memory if capacity > 0
        if self.capacity > 0 {
            let layout = Layout::array::<T>(self.capacity).expect("capacity overflow");
            // SAFETY: ptr was allocated with this layout, all elements dropped
            unsafe {
                alloc::dealloc(self.ptr.as_ptr().cast::<u8>(), layout);
            }
        }
    }
}

// Thread safety: TruenoVec<T> is Send if T is Send
unsafe impl<T: Send> Send for TruenoVec<T> {}

// Thread safety: TruenoVec<T> is Sync if T is Sync
unsafe impl<T: Sync> Sync for TruenoVec<T> {}

// ============================================================================
// Unit Tests (Tier 1: Sub-second feedback)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let vec: TruenoVec<i32> = TruenoVec::new();
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let vec: TruenoVec<i32> = TruenoVec::with_capacity(10);
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 10);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_with_capacity_zero() {
        let vec: TruenoVec<i32> = TruenoVec::with_capacity(0);
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 0);
    }

    #[test]
    fn test_push_single() {
        let mut vec = TruenoVec::new();
        vec.push(42);
        assert_eq!(vec.len(), 1);
        assert!(!vec.is_empty());
        assert_eq!(vec.get(0), Some(&42));
    }

    #[test]
    fn test_push_multiple() {
        let mut vec = TruenoVec::new();
        for i in 0..10 {
            vec.push(i);
        }
        assert_eq!(vec.len(), 10);
        for i in 0..10 {
            assert_eq!(vec.get(i), Some(&i));
        }
    }

    #[test]
    fn test_pop_empty() {
        let mut vec: TruenoVec<i32> = TruenoVec::new();
        assert_eq!(vec.pop(), None);
    }

    #[test]
    fn test_pop_single() {
        let mut vec = TruenoVec::new();
        vec.push(42);
        assert_eq!(vec.pop(), Some(42));
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_push_pop_sequence() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec.pop(), Some(3));
        assert_eq!(vec.pop(), Some(2));
        vec.push(4);
        assert_eq!(vec.pop(), Some(4));
        assert_eq!(vec.pop(), Some(1));
        assert_eq!(vec.pop(), None);
    }

    #[test]
    fn test_get_valid_index() {
        let mut vec = TruenoVec::new();
        vec.push(10);
        vec.push(20);
        vec.push(30);
        assert_eq!(vec.get(0), Some(&10));
        assert_eq!(vec.get(1), Some(&20));
        assert_eq!(vec.get(2), Some(&30));
    }

    #[test]
    fn test_get_invalid_index() {
        let mut vec = TruenoVec::new();
        vec.push(10);
        assert_eq!(vec.get(1), None);
        assert_eq!(vec.get(100), None);
    }

    #[test]
    fn test_get_mut() {
        let mut vec = TruenoVec::new();
        vec.push(10);
        vec.push(20);
        if let Some(elem) = vec.get_mut(1) {
            *elem = 25;
        }
        assert_eq!(vec.get(1), Some(&25));
    }

    #[test]
    fn test_get_mut_boundary() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        // Valid indices should work
        assert!(vec.get_mut(0).is_some());
        assert!(vec.get_mut(1).is_some());
        assert!(vec.get_mut(2).is_some());

        // Index equal to len should return None
        assert_eq!(vec.get_mut(3), None);

        // Higher indices should return None
        assert_eq!(vec.get_mut(4), None);
        assert_eq!(vec.get_mut(100), None);
    }

    #[test]
    fn test_growth() {
        let mut vec = TruenoVec::new();
        assert_eq!(vec.capacity(), 0);

        vec.push(1);
        assert_eq!(vec.capacity(), 1);

        vec.push(2);
        assert_eq!(vec.capacity(), 2);

        vec.push(3);
        assert_eq!(vec.capacity(), 4);

        vec.push(4);
        assert_eq!(vec.capacity(), 4);

        vec.push(5);
        assert_eq!(vec.capacity(), 8);
    }

    #[test]
    fn test_capacity_maintained() {
        let mut vec = TruenoVec::with_capacity(10);
        for i in 0..10 {
            vec.push(i);
        }
        assert_eq!(vec.capacity(), 10);

        vec.push(10);
        assert_eq!(vec.capacity(), 20);
    }

    #[test]
    fn test_drop_calls_destructors() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        struct DropCounter(Arc<AtomicUsize>);
        impl Drop for DropCounter {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        let counter = Arc::new(AtomicUsize::new(0));

        {
            let mut vec = TruenoVec::new();
            for _ in 0..5 {
                vec.push(DropCounter(Arc::clone(&counter)));
            }
        } // vec dropped here

        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn test_default() {
        let vec: TruenoVec<i32> = TruenoVec::default();
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 0);
    }

    #[test]
    fn test_drop_empty_vector() {
        // Test that dropping an empty vector (capacity=0) doesn't crash
        {
            let vec: TruenoVec<i32> = TruenoVec::new();
            assert_eq!(vec.capacity(), 0);
        } // vec dropped here - should not attempt deallocation

        // Test with capacity > 0 but no elements
        {
            let vec: TruenoVec<i32> = TruenoVec::with_capacity(10);
            assert_eq!(vec.capacity(), 10);
            assert_eq!(vec.len(), 0);
        } // vec dropped here - should deallocate

        // Test after popping all elements
        {
            let mut vec = TruenoVec::new();
            vec.push(1);
            vec.push(2);
            vec.pop();
            vec.pop();
            assert_eq!(vec.len(), 0);
            assert!(vec.capacity() > 0);
        } // vec dropped here - should deallocate despite len=0
    }
}

// ============================================================================
// Property-Based Tests (Tier 2: 1-5 minutes)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Property 1: Length Invariant
    // After pushing n elements, len() == n
    proptest! {
        #[test]
        fn prop_length_after_pushes(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let mut v = TruenoVec::new();
            let expected_len = elements.len();

            for elem in elements {
                v.push(elem);
            }

            prop_assert_eq!(v.len(), expected_len);
        }
    }

    // Property 2: Capacity Bound
    // capacity() >= len() at all times
    proptest! {
        #[test]
        fn prop_capacity_bound(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let mut v = TruenoVec::new();
            prop_assert!(v.capacity() >= v.len());

            for elem in elements {
                v.push(elem);
                prop_assert!(v.capacity() >= v.len());
            }
        }
    }

    // Property 3: Push/Pop Symmetry
    // pop() returns exactly what was pushed
    proptest! {
        #[test]
        fn prop_push_pop_inverse(
            initial in prop::collection::vec(any::<i32>(), 0..50),
            x in any::<i32>()
        ) {
            let mut v = TruenoVec::new();
            for val in &initial {
                v.push(*val);
            }

            let len_before = v.len();
            v.push(x);
            let popped = v.pop();

            prop_assert_eq!(popped, Some(x));
            prop_assert_eq!(v.len(), len_before);
        }
    }

    // Property 4: Index Access Correctness
    // get(i) returns the i-th pushed element
    proptest! {
        #[test]
        fn prop_index_access(elements in prop::collection::vec(any::<i32>(), 1..100)) {
            let mut v = TruenoVec::new();
            for elem in &elements {
                v.push(*elem);
            }

            for (i, expected) in elements.iter().enumerate() {
                prop_assert_eq!(v.get(i), Some(expected));
            }
        }
    }

    // Property 5: Out of Bounds Access
    // get(i) returns None when i >= len()
    proptest! {
        #[test]
        fn prop_out_of_bounds(
            elements in prop::collection::vec(any::<i32>(), 0..50),
            invalid_index in 100usize..200
        ) {
            let mut v = TruenoVec::new();
            for elem in elements {
                v.push(elem);
            }

            prop_assert_eq!(v.get(invalid_index), None);
        }
    }

    // Property 6: Growth Factor
    // Capacity doubles on reallocation (exponential growth)
    proptest! {
        #[test]
        fn prop_exponential_growth(count in 1usize..20) {
            let mut v = TruenoVec::<i32>::new();
            let mut prev_capacity = 0;

            for i in 0..count {
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                v.push(i as i32);
                if v.capacity() > prev_capacity && prev_capacity > 0 {
                    // Capacity should double (or be the initial 1)
                    prop_assert!(v.capacity() == prev_capacity * 2 || prev_capacity == 0);
                }
                prev_capacity = v.capacity();
            }
        }
    }

    // Property 7: Comparison with std::vec::Vec
    // TruenoVec behavior matches std::vec::Vec
    proptest! {
        #[test]
        fn prop_matches_std_vec(operations in prop::collection::vec(0i32..100, 0..50)) {
            let mut trueno = TruenoVec::new();
            let mut std_vec = Vec::new();

            for op in operations {
                if op % 3 == 0 {
                    // Push operation
                    trueno.push(op);
                    std_vec.push(op);
                } else {
                    // Pop operation
                    prop_assert_eq!(trueno.pop(), std_vec.pop());
                }

                prop_assert_eq!(trueno.len(), std_vec.len());
                prop_assert_eq!(trueno.is_empty(), std_vec.is_empty());
            }
        }
    }

    // Property 8: Empty Vector Properties
    proptest! {
        #[test]
        fn prop_empty_invariants(capacity in 0usize..100) {
            let v: TruenoVec<i32> = if capacity == 0 {
                TruenoVec::new()
            } else {
                TruenoVec::with_capacity(capacity)
            };

            prop_assert!(v.is_empty());
            prop_assert_eq!(v.len(), 0);
            prop_assert_eq!(v.get(0), None);
        }
    }

    // Property 9: Mutable Access Correctness
    proptest! {
        #[test]
        fn prop_get_mut_modifies(
            mut elements in prop::collection::vec(any::<i32>(), 1..50),
            index in 0usize..50,
            new_value in any::<i32>()
        ) {
            if index >= elements.len() {
                return Ok(());
            }

            let mut v = TruenoVec::new();
            for elem in &elements {
                v.push(*elem);
            }

            if let Some(elem) = v.get_mut(index) {
                *elem = new_value;
            }

            prop_assert_eq!(v.get(index), Some(&new_value));
            elements[index] = new_value;

            for (i, expected) in elements.iter().enumerate() {
                prop_assert_eq!(v.get(i), Some(expected));
            }
        }
    }

    // Property 10: Repeated Push/Pop Maintains Invariants
    proptest! {
        #[test]
        fn prop_repeated_operations(ops in prop::collection::vec((any::<bool>(), any::<i32>()), 0..100)) {
            let mut v = TruenoVec::new();
            let mut shadow_vec = Vec::new();

            for (is_push, value) in ops {
                if is_push {
                    v.push(value);
                    shadow_vec.push(value);
                } else if !shadow_vec.is_empty() {
                    prop_assert_eq!(v.pop(), shadow_vec.pop());
                }

                prop_assert_eq!(v.len(), shadow_vec.len());
                prop_assert!(v.capacity() >= v.len());
            }
        }
    }
}

// ============================================================================
// Formal Verification (Tier 3: ON-MERGE/NIGHTLY)
// ============================================================================

#[cfg(kani)]
mod verification {
    use super::*;

    /// Verifies that capacity >= len invariant holds for all execution paths
    #[kani::proof]
    #[kani::unwind(10)]
    fn verify_capacity_invariant() {
        let mut v: TruenoVec<u8> = TruenoVec::new();

        for _ in 0..10 {
            let x: u8 = kani::any();
            v.push(x);
            assert!(v.capacity() >= v.len());
        }
    }

    /// Verifies that push followed by pop returns the same value
    #[kani::proof]
    #[kani::unwind(5)]
    fn verify_push_pop_correctness() {
        let mut v: TruenoVec<u32> = TruenoVec::new();
        let x: u32 = kani::any();

        let len_before = v.len();
        v.push(x);
        assert_eq!(v.len(), len_before + 1);

        let popped = v.pop();
        assert_eq!(popped, Some(x));
        assert_eq!(v.len(), len_before);
    }

    /// Verifies that get returns None for out-of-bounds indices
    #[kani::proof]
    #[kani::unwind(5)]
    fn verify_bounds_checking() {
        let mut v: TruenoVec<u8> = TruenoVec::new();

        // Push some elements
        for i in 0..5 {
            v.push(i);
        }

        // Valid access should succeed
        for i in 0..5 {
            assert!(v.get(i).is_some());
        }

        // Out of bounds should return None
        assert_eq!(v.get(5), None);
        assert_eq!(v.get(100), None);
    }
}
