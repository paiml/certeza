#![allow(unsafe_code)]
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

    /// Clears the vector, removing all values.
    ///
    /// This method removes all elements from the vector but does not change
    /// its capacity. The allocated memory is retained for reuse.
    ///
    /// # Complexity
    ///
    /// O(n) where n is the number of elements
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// let capacity = vec.capacity();
    /// vec.clear();
    ///
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), capacity); // Capacity unchanged
    /// assert!(vec.is_empty());
    /// ```
    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    /// Inserts an element at position `index`, shifting all elements after it
    /// to the right.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) where n is the number of elements after the insertion point
    /// - Space: O(1) amortized (may reallocate if capacity is exceeded)
    ///
    /// # Panics
    ///
    /// Panics if `index > len()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(1);
    /// vec.push(3);
    /// vec.insert(1, 2); // Insert 2 at index 1
    ///
    /// assert_eq!(vec.get(0), Some(&1));
    /// assert_eq!(vec.get(1), Some(&2));
    /// assert_eq!(vec.get(2), Some(&3));
    /// ```
    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len, "insertion index out of bounds");

        if self.len == self.capacity {
            self.grow();
        }

        // SAFETY: len < capacity after grow, index <= len
        unsafe {
            let ptr = self.ptr.as_ptr();
            // Shift elements [index..len) one position to the right
            if index < self.len {
                ptr::copy(ptr.add(index), ptr.add(index + 1), self.len - index);
            }
            // Write the new value at index
            ptr::write(ptr.add(index), value);
        }

        self.len += 1;
    }

    /// Removes and returns the element at position `index`, shifting all
    /// elements after it to the left.
    ///
    /// # Complexity
    ///
    /// O(n) where n is the number of elements after the removal point
    ///
    /// # Panics
    ///
    /// Panics if `index >= len()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// let removed = vec.remove(1);
    /// assert_eq!(removed, 2);
    /// assert_eq!(vec.len(), 2);
    /// assert_eq!(vec.get(0), Some(&1));
    /// assert_eq!(vec.get(1), Some(&3));
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "removal index out of bounds");

        // SAFETY: index < len, so it's a valid initialized element
        unsafe {
            let ptr = self.ptr.as_ptr();
            // Read the value at index
            let value = ptr::read(ptr.add(index));
            // Shift elements [index+1..len) one position to the left
            if index < self.len - 1 {
                ptr::copy(ptr.add(index + 1), ptr.add(index), self.len - index - 1);
            }
            self.len -= 1;
            value
        }
    }

    /// Returns an immutable slice containing all elements of the vector.
    ///
    /// This provides a safe view into the vector's contents without copying.
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
    /// vec.push(3);
    ///
    /// let slice = vec.as_slice();
    /// assert_eq!(slice, &[1, 2, 3]);
    /// assert_eq!(slice.len(), 3);
    /// ```
    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: ptr points to len initialized elements
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Returns a mutable slice containing all elements of the vector.
    ///
    /// This provides a safe mutable view into the vector's contents without copying.
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
    /// vec.push(3);
    ///
    /// let slice = vec.as_slice_mut();
    /// slice[1] = 10;
    /// assert_eq!(vec.get(1), Some(&10));
    /// ```
    #[must_use]
    pub const fn as_slice_mut(&mut self) -> &mut [T] {
        // SAFETY: ptr points to len initialized elements
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
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
// Conversion Trait Implementations
// ============================================================================

impl<T> From<Vec<T>> for TruenoVec<T> {
    /// Converts a `Vec<T>` into a `TruenoVec<T>`.
    ///
    /// This conversion takes ownership of the `Vec` and efficiently reuses
    /// its underlying allocation by transferring elements one by one.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) where n is the number of elements
    /// - Space: O(n) for the new allocation (original Vec allocation is dropped)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let std_vec = vec![1, 2, 3, 4, 5];
    /// let trueno_vec: TruenoVec<i32> = TruenoVec::from(std_vec);
    ///
    /// assert_eq!(trueno_vec.len(), 5);
    /// assert_eq!(trueno_vec.get(0), Some(&1));
    /// assert_eq!(trueno_vec.get(4), Some(&5));
    /// ```
    fn from(vec: Vec<T>) -> Self {
        let mut trueno = Self::with_capacity(vec.len());
        for item in vec {
            trueno.push(item);
        }
        trueno
    }
}

impl<T: Clone> From<&[T]> for TruenoVec<T> {
    /// Converts a slice `&[T]` into a `TruenoVec<T>`.
    ///
    /// This conversion clones all elements from the slice into a new `TruenoVec`.
    /// Requires `T: Clone` since we're creating owned copies.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) where n is the number of elements
    /// - Space: O(n)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let array = [1, 2, 3, 4, 5];
    /// let slice: &[i32] = &array;
    /// let trueno_vec: TruenoVec<i32> = TruenoVec::from(slice);
    ///
    /// assert_eq!(trueno_vec.len(), 5);
    /// assert_eq!(trueno_vec.get(2), Some(&3));
    /// ```
    fn from(slice: &[T]) -> Self {
        let mut trueno = Self::with_capacity(slice.len());
        for item in slice {
            trueno.push(item.clone());
        }
        trueno
    }
}

impl<T> FromIterator<T> for TruenoVec<T> {
    /// Creates a `TruenoVec<T>` from an iterator.
    ///
    /// This enables the `.collect()` method on iterators to produce a `TruenoVec`.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) where n is the number of elements in the iterator
    /// - Space: O(n)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let trueno_vec: TruenoVec<i32> = (0..10).collect();
    ///
    /// assert_eq!(trueno_vec.len(), 10);
    /// assert_eq!(trueno_vec.get(5), Some(&5));
    /// ```
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let numbers = vec![1, 2, 3, 4, 5];
    /// let doubled: TruenoVec<i32> = numbers.iter().map(|x| x * 2).collect();
    ///
    /// assert_eq!(doubled.len(), 5);
    /// assert_eq!(doubled.get(0), Some(&2));
    /// assert_eq!(doubled.get(4), Some(&10));
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower_bound, _) = iter.size_hint();
        let mut trueno = Self::with_capacity(lower_bound);

        for item in iter {
            trueno.push(item);
        }

        trueno
    }
}

impl<T> Extend<T> for TruenoVec<T> {
    /// Extends a `TruenoVec<T>` with the contents of an iterator.
    ///
    /// This method appends all elements from the iterator to the end of the vector.
    ///
    /// # Complexity
    ///
    /// - Time: O(n) where n is the number of elements in the iterator
    /// - Space: O(n) amortized
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(1);
    /// vec.push(2);
    ///
    /// vec.extend(vec![3, 4, 5]);
    ///
    /// assert_eq!(vec.len(), 5);
    /// assert_eq!(vec.get(0), Some(&1));
    /// assert_eq!(vec.get(4), Some(&5));
    /// ```
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.extend(0..10);
    ///
    /// assert_eq!(vec.len(), 10);
    /// ```
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let (lower_bound, _) = iter.size_hint();

        // Reserve space if we know the size hint
        if lower_bound > 0 {
            let needed_capacity = self.len.saturating_add(lower_bound);
            while self.capacity < needed_capacity {
                self.grow();
            }
        }

        for item in iter {
            self.push(item);
        }
    }
}

// ============================================================================
// Iterator Implementations
// ============================================================================

/// Immutable iterator over `TruenoVec<T>`.
///
/// This struct is created by the `iter` method on `TruenoVec`.
pub struct Iter<'a, T> {
    ptr: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        } else {
            // SAFETY: ptr is valid and within bounds
            unsafe {
                let item = &*self.ptr;
                self.ptr = self.ptr.add(1);
                Some(item)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.ptr as usize) / std::mem::size_of::<T>();
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        } else {
            // SAFETY: end-1 is valid and within bounds
            unsafe {
                self.end = self.end.sub(1);
                Some(&*self.end)
            }
        }
    }
}

/// Mutable iterator over `TruenoVec<T>`.
///
/// This struct is created by the `iter_mut` method on `TruenoVec`.
pub struct IterMut<'a, T> {
    ptr: *mut T,
    end: *mut T,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        } else {
            // SAFETY: ptr is valid and within bounds
            unsafe {
                let item = &mut *self.ptr;
                self.ptr = self.ptr.add(1);
                Some(item)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.ptr as usize) / std::mem::size_of::<T>();
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {}

impl<T> DoubleEndedIterator for IterMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        } else {
            // SAFETY: end-1 is valid and within bounds
            unsafe {
                self.end = self.end.sub(1);
                Some(&mut *self.end)
            }
        }
    }
}

/// Consuming iterator over `TruenoVec<T>`.
///
/// This struct is created by the `into_iter` method on `TruenoVec`
/// (provided by the `IntoIterator` trait).
pub struct IntoIter<T> {
    vec: TruenoVec<T>,
    current: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.vec.len {
            let index = self.current;
            self.current += 1;
            // SAFETY: index < len, so it's a valid initialized element
            // We take ownership by reading, so the Drop impl won't try to drop it again
            unsafe { Some(ptr::read(self.vec.ptr.as_ptr().add(index))) }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.vec.len - self.current;
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        // Drop all remaining unconsumed elements
        while self.current < self.vec.len {
            // SAFETY: current is bounded by vec.len, ptr is valid for vec.len elements
            unsafe {
                ptr::drop_in_place(self.vec.ptr.as_ptr().add(self.current));
            }
            self.current += 1;
        }

        // Now deallocate the memory (all elements have been dropped)
        if self.vec.capacity > 0 {
            let layout = Layout::array::<T>(self.vec.capacity).expect("capacity overflow");
            // SAFETY: ptr was allocated with this layout and capacity > 0 guarantees valid alloc
            unsafe {
                alloc::dealloc(self.vec.ptr.as_ptr().cast::<u8>(), layout);
            }
        }

        // Prevent the vec's Drop from running by setting capacity to 0
        // This is safe because we've already handled deallocation
        self.vec.capacity = 0;
        self.vec.len = 0;
    }
}

impl<T> TruenoVec<T> {
    /// Returns an iterator over the vector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// let mut iter = vec.iter();
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), Some(&3));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[must_use]
    pub const fn iter(&self) -> Iter<'_, T> {
        // SAFETY: ptr and ptr+len are both valid
        unsafe {
            Iter {
                ptr: self.ptr.as_ptr(),
                end: self.ptr.as_ptr().add(self.len),
                _marker: PhantomData,
            }
        }
    }

    /// Returns a mutable iterator over the vector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// for elem in vec.iter_mut() {
    ///     *elem *= 2;
    /// }
    ///
    /// assert_eq!(vec.get(0), Some(&2));
    /// assert_eq!(vec.get(1), Some(&4));
    /// assert_eq!(vec.get(2), Some(&6));
    /// ```
    #[must_use]
    pub const fn iter_mut(&mut self) -> IterMut<'_, T> {
        // SAFETY: ptr and ptr+len are both valid
        unsafe {
            IterMut {
                ptr: self.ptr.as_ptr(),
                end: self.ptr.as_ptr().add(self.len),
                _marker: PhantomData,
            }
        }
    }
}

impl<T> IntoIterator for TruenoVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Consumes the vector and returns an iterator over its elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// let sum: i32 = vec.into_iter().sum();
    /// assert_eq!(sum, 6);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            vec: self,
            current: 0,
        }
    }
}

impl<'a, T> IntoIterator for &'a TruenoVec<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut TruenoVec<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

// ============================================================================
// Index Trait Implementations
// ============================================================================

impl<T> std::ops::Index<usize> for TruenoVec<T> {
    type Output = T;

    /// Returns a reference to an element at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// vec.push(30);
    ///
    /// assert_eq!(vec[0], 10);
    /// assert_eq!(vec[1], 20);
    /// assert_eq!(vec[2], 30);
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        let len = self.len;
        self.get(index).unwrap_or_else(|| {
            panic!("index out of bounds: the len is {len} but the index is {index}")
        })
    }
}

impl<T> std::ops::IndexMut<usize> for TruenoVec<T> {
    /// Returns a mutable reference to an element at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// vec.push(30);
    ///
    /// vec[1] = 25;
    /// assert_eq!(vec[1], 25);
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.len;
        self.get_mut(index).unwrap_or_else(|| {
            panic!("index out of bounds: the len is {len} but the index is {index}")
        })
    }
}

// ============================================================================
// Clone Trait Implementation
// ============================================================================

impl<T: Clone> Clone for TruenoVec<T> {
    /// Creates a deep copy of the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    ///
    /// let mut vec1 = TruenoVec::new();
    /// vec1.push(1);
    /// vec1.push(2);
    /// vec1.push(3);
    ///
    /// let mut vec2 = vec1.clone();
    ///
    /// // Modify vec2
    /// vec2.push(4);
    ///
    /// // vec1 is unchanged
    /// assert_eq!(vec1.len(), 3);
    /// assert_eq!(vec2.len(), 4);
    /// ```
    fn clone(&self) -> Self {
        let mut new_vec = Self::with_capacity(self.len);
        for elem in self {
            new_vec.push(elem.clone());
        }
        new_vec
    }
}

// ============================================================================
// PartialEq and Eq Trait Implementations
// ============================================================================

impl<T: PartialEq> PartialEq for TruenoVec<T> {
    /// Compares two vectors for equality.
    ///
    /// Two vectors are equal if they have the same length and all elements
    /// at corresponding indices are equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    ///
    /// let mut vec1 = TruenoVec::new();
    /// vec1.push(1);
    /// vec1.push(2);
    ///
    /// let mut vec2 = TruenoVec::new();
    /// vec2.push(1);
    /// vec2.push(2);
    ///
    /// assert_eq!(vec1, vec2);
    ///
    /// vec2.push(3);
    /// assert_ne!(vec1, vec2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        // First check lengths - fast path
        if self.len != other.len {
            return false;
        }

        // Compare elements
        for i in 0..self.len {
            if self.get(i) != other.get(i) {
                return false;
            }
        }

        true
    }
}

impl<T: Eq> Eq for TruenoVec<T> {}

// ============================================================================
// Debug Trait Implementation
// ============================================================================

impl<T: std::fmt::Debug> std::fmt::Debug for TruenoVec<T> {
    /// Formats the vector for debugging output.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// assert_eq!(format!("{:?}", vec), "[1, 2, 3]");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

// ============================================================================
// Phase 3.1 Ergonomic Trait Implementations (Deref/AsRef)
// ============================================================================

impl<T> std::ops::Deref for TruenoVec<T> {
    type Target = [T];

    /// Dereferences the `TruenoVec<T>` to a slice `&[T]`.
    ///
    /// This allows `TruenoVec` to be used seamlessly with any function
    /// that accepts a slice, enabling automatic coercion and access to
    /// all slice methods.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// // Deref coercion allows slice methods
    /// assert_eq!(vec.len(), 3);
    /// assert_eq!(vec.first(), Some(&1));
    /// assert_eq!(vec.last(), Some(&3));
    ///
    /// // Works with functions expecting slices
    /// fn sum_slice(s: &[i32]) -> i32 {
    ///     s.iter().sum()
    /// }
    /// assert_eq!(sum_slice(&vec), 6);
    /// ```
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> std::ops::DerefMut for TruenoVec<T> {
    /// Mutably dereferences the `TruenoVec<T>` to a mutable slice `&mut [T]`.
    ///
    /// This allows mutable access to slice methods through deref coercion.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(3);
    /// vec.push(1);
    /// vec.push(2);
    ///
    /// // DerefMut coercion allows mutable slice methods
    /// vec.sort();
    /// assert_eq!(vec.as_slice(), &[1, 2, 3]);
    ///
    /// vec.reverse();
    /// assert_eq!(vec.as_slice(), &[3, 2, 1]);
    /// ```
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<T> AsRef<[T]> for TruenoVec<T> {
    /// Returns a reference to the vector's contents as a slice.
    ///
    /// This enables `TruenoVec` to be used with generic functions that
    /// accept `AsRef<[T]>` bounds, a common pattern in the Rust standard library.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// // Works with generic functions using AsRef<[T]>
    /// fn process<T: AsRef<[i32]>>(data: T) -> i32 {
    ///     data.as_ref().iter().sum()
    /// }
    ///
    /// assert_eq!(process(&vec), 6);
    /// assert_eq!(process(vec![1, 2, 3]), 6);
    /// ```
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsMut<[T]> for TruenoVec<T> {
    /// Returns a mutable reference to the vector's contents as a slice.
    ///
    /// This enables `TruenoVec` to be used with generic functions that
    /// accept `AsMut<[T]>` bounds, enabling mutable slice operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// // Works with generic functions using AsMut<[T]>
    /// fn double_in_place<T: AsMut<[i32]>>(mut data: T) {
    ///     for x in data.as_mut() {
    ///         *x *= 2;
    ///     }
    /// }
    ///
    /// double_in_place(&mut vec);
    /// assert_eq!(vec.as_slice(), &[2, 4, 6]);
    /// ```
    fn as_mut(&mut self) -> &mut [T] {
        self.as_slice_mut()
    }
}

impl<T> AsRef<Self> for TruenoVec<T> {
    /// Returns a reference to self.
    ///
    /// This implementation allows `TruenoVec` to work with APIs that
    /// require `AsRef<TruenoVec<T>>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    ///
    /// let vec = TruenoVec::from(vec![1, 2, 3]);
    /// let vec_ref: &TruenoVec<i32> = vec.as_ref();
    /// assert_eq!(vec_ref.len(), 3);
    /// ```
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T> AsMut<Self> for TruenoVec<T> {
    /// Returns a mutable reference to self.
    ///
    /// This implementation allows `TruenoVec` to work with APIs that
    /// require `AsMut<TruenoVec<T>>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    ///
    /// let mut vec = TruenoVec::from(vec![1, 2, 3]);
    /// let vec_mut: &mut TruenoVec<i32> = vec.as_mut();
    /// vec_mut.push(4);
    /// assert_eq!(vec_mut.len(), 4);
    /// ```
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

// ============================================================================
// Phase 3.2 Comparison and Hash Trait Implementations
// ============================================================================

impl<T: PartialOrd> PartialOrd for TruenoVec<T> {
    /// Compares two vectors lexicographically.
    ///
    /// Vectors are compared element-by-element from left to right.
    /// The first mismatching element determines the ordering.
    /// If all elements match but one vector is shorter, the shorter vector is less.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    ///
    /// let mut vec1 = TruenoVec::new();
    /// vec1.push(1);
    /// vec1.push(2);
    ///
    /// let mut vec2 = TruenoVec::new();
    /// vec2.push(1);
    /// vec2.push(3);
    ///
    /// assert!(vec1 < vec2);
    /// assert!(vec2 > vec1);
    ///
    /// let mut vec3 = TruenoVec::new();
    /// vec3.push(1);
    /// assert!(vec3 < vec1); // Shorter vector is less
    /// ```
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }

    fn lt(&self, other: &Self) -> bool {
        self.as_slice() < other.as_slice()
    }

    fn le(&self, other: &Self) -> bool {
        self.as_slice() <= other.as_slice()
    }

    fn gt(&self, other: &Self) -> bool {
        self.as_slice() > other.as_slice()
    }

    fn ge(&self, other: &Self) -> bool {
        self.as_slice() >= other.as_slice()
    }
}

impl<T: Ord> Ord for TruenoVec<T> {
    /// Returns the total ordering between two vectors.
    ///
    /// This provides a total ordering for vectors where `T: Ord`,
    /// enabling use in `BTreeMap` and `BTreeSet` as keys, and
    /// allowing sorting of vectors.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    /// use std::cmp::Ordering;
    ///
    /// let mut vec1 = TruenoVec::new();
    /// vec1.push(1);
    /// vec1.push(2);
    ///
    /// let mut vec2 = TruenoVec::new();
    /// vec2.push(1);
    /// vec2.push(3);
    ///
    /// assert_eq!(vec1.cmp(&vec2), Ordering::Less);
    ///
    /// // Can be used for sorting
    /// let mut vecs = vec![vec2.clone(), vec1.clone()];
    /// vecs.sort();
    /// assert_eq!(vecs[0], vec1);
    /// assert_eq!(vecs[1], vec2);
    /// ```
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<T: std::hash::Hash> std::hash::Hash for TruenoVec<T> {
    /// Hashes the vector by hashing all its elements in order.
    ///
    /// This enables `TruenoVec` to be used as keys in `HashMap` and
    /// values in `HashSet`, provided `T: Hash`.
    ///
    /// # Examples
    ///
    /// ```
    /// use certeza::TruenoVec;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    ///
    /// let mut key = TruenoVec::new();
    /// key.push(1);
    /// key.push(2);
    /// key.push(3);
    ///
    /// map.insert(key.clone(), "value");
    /// assert_eq!(map.get(&key), Some(&"value"));
    ///
    /// // Different vectors have different hashes
    /// let mut key2 = TruenoVec::new();
    /// key2.push(1);
    /// key2.push(2);
    /// assert_eq!(map.get(&key2), None);
    /// ```
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the length first (same as std::Vec)
        self.len.hash(state);
        // Hash each element
        for elem in self {
            elem.hash(state);
        }
    }
}

// ============================================================================
// Phase 3.3: Display and Borrow Traits (Polish & Advanced Use Cases)
// ============================================================================

/// Display trait for user-friendly formatting
///
/// Formats the vector as `[elem1, elem2, ...]` similar to `std::Vec`.
///
/// # Examples
///
/// ```
/// use certeza::TruenoVec;
///
/// let vec = TruenoVec::from(vec![1, 2, 3]);
/// assert_eq!(format!("{}", vec), "[1, 2, 3]");
/// ```
impl<T: std::fmt::Display> std::fmt::Display for TruenoVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, elem) in self.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{elem}")?;
        }
        write!(f, "]")
    }
}

/// Borrow<[T]> trait for advanced borrowing patterns
///
/// Enables using `TruenoVec` with APIs that accept `Borrow<[T]>` bounds,
/// particularly useful for `HashMap` and `BTreeMap` lookups.
///
/// # Examples
///
/// ```
/// use certeza::TruenoVec;
/// use std::borrow::Borrow;
///
/// let vec = TruenoVec::from(vec![1, 2, 3]);
/// let slice: &[i32] = vec.borrow();
/// assert_eq!(slice, &[1, 2, 3]);
/// ```
impl<T> std::borrow::Borrow<[T]> for TruenoVec<T> {
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

/// `BorrowMut<[T]>` trait for advanced mutable borrowing patterns
///
/// Enables using `TruenoVec` with APIs that accept `BorrowMut<[T]>` bounds.
///
/// # Examples
///
/// ```
/// use certeza::TruenoVec;
/// use std::borrow::BorrowMut;
///
/// let mut vec = TruenoVec::from(vec![1, 2, 3]);
/// let slice: &mut [i32] = vec.borrow_mut();
/// slice[0] = 10;
/// assert_eq!(vec.get(0), Some(&10));
/// ```
impl<T> std::borrow::BorrowMut<[T]> for TruenoVec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_slice_mut()
    }
}

// ============================================================================
// Unit Tests (Tier 1: Sub-second feedback)
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::disallowed_methods)]
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

    #[test]
    fn test_clear_empty() {
        let mut vec: TruenoVec<i32> = TruenoVec::new();
        vec.clear();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_clear_with_elements() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let capacity = vec.capacity();
        vec.clear();

        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
        assert_eq!(vec.capacity(), capacity); // Capacity should be unchanged
    }

    #[test]
    fn test_clear_with_drop() {
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
            vec.clear();
            assert_eq!(counter.load(Ordering::SeqCst), 5); // All elements dropped
            assert_eq!(vec.len(), 0);
        }
    }

    #[test]
    fn test_pop_decrements_length() {
        // Explicitly test that pop decrements length correctly
        // This catches mutations where -= is replaced with += or other operators
        let mut vec = TruenoVec::new();
        vec.push(10);
        vec.push(20);
        vec.push(30);

        let initial_len = vec.len();
        assert_eq!(initial_len, 3);

        // Each pop must decrement by exactly 1
        let val1 = vec.pop();
        assert_eq!(val1, Some(30));
        assert_eq!(vec.len(), initial_len - 1);
        assert_eq!(vec.len(), 2); // Must be exactly 2, not 4 (if += used)

        let val2 = vec.pop();
        assert_eq!(val2, Some(20));
        assert_eq!(vec.len(), 1); // Must be exactly 1, not 5

        let val3 = vec.pop();
        assert_eq!(val3, Some(10));
        assert_eq!(vec.len(), 0); // Must be exactly 0, not 6

        // Further pops should return None and keep length at 0
        assert_eq!(vec.pop(), None);
        assert_eq!(vec.len(), 0); // Must stay 0, not become 1
    }

    #[test]
    fn test_drop_with_capacity_deallocates() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        // Test that drop properly deallocates memory for vector with capacity > 0
        // We use DropCounter to verify all cleanup happens correctly
        struct DropCounter(Arc<AtomicUsize>);
        impl Drop for DropCounter {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        let counter = Arc::new(AtomicUsize::new(0));

        {
            // Create vector with capacity and add elements
            let mut vec = TruenoVec::with_capacity(5);
            let capacity = vec.capacity();
            assert!(capacity > 0); // Capacity must be > 0 for this test
            assert_eq!(capacity, 5);

            vec.push(DropCounter(Arc::clone(&counter)));
            vec.push(DropCounter(Arc::clone(&counter)));
            vec.push(DropCounter(Arc::clone(&counter)));
            assert_eq!(vec.len(), 3);
            assert_eq!(vec.capacity(), 5); // Capacity unchanged
        } // vec dropped here - must deallocate because capacity > 0

        // All 3 elements must be dropped
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_drop_deallocates_empty_with_capacity() {
        // Specifically test the capacity > 0 check in drop
        // This catches mutations like: if self.capacity > 0 -> if self.capacity < 0
        let vec: TruenoVec<i32> = TruenoVec::with_capacity(100);

        // Verify we have capacity but no elements
        assert_eq!(vec.capacity(), 100);
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());

        // When vec is dropped, it MUST deallocate because capacity > 0
        // If the condition is mutated to capacity < 0, this would leak memory
        // Since usize can't be < 0, the deallocation would never happen
        drop(vec);

        // We can't directly test for memory leaks in safe Rust, but we can
        // ensure the drop path is exercised for vectors with capacity > 0
        // The mutation test will catch if the deallocation block is skipped
    }

    #[test]
    fn test_insert_at_start() {
        let mut vec = TruenoVec::new();
        vec.push(2);
        vec.push(3);
        vec.insert(0, 1);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(0), Some(&1));
        assert_eq!(vec.get(1), Some(&2));
        assert_eq!(vec.get(2), Some(&3));
    }

    #[test]
    fn test_insert_at_middle() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(3);
        vec.insert(1, 2);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(0), Some(&1));
        assert_eq!(vec.get(1), Some(&2));
        assert_eq!(vec.get(2), Some(&3));
    }

    #[test]
    fn test_insert_at_end() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.insert(2, 3); // Insert at end (equivalent to push)

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(2), Some(&3));
    }

    #[test]
    fn test_insert_empty_vector() {
        let mut vec: TruenoVec<i32> = TruenoVec::new();
        vec.insert(0, 42);

        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(0), Some(&42));
    }

    #[test]
    #[should_panic(expected = "insertion index out of bounds")]
    fn test_insert_out_of_bounds() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.insert(5, 2); // index > len
    }

    #[test]
    fn test_remove_at_start() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let removed = vec.remove(0);
        assert_eq!(removed, 1);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(0), Some(&2));
        assert_eq!(vec.get(1), Some(&3));
    }

    #[test]
    fn test_remove_at_middle() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let removed = vec.remove(1);
        assert_eq!(removed, 2);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(0), Some(&1));
        assert_eq!(vec.get(1), Some(&3));
    }

    #[test]
    fn test_remove_at_end() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let removed = vec.remove(2);
        assert_eq!(removed, 3);
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_remove_all_elements() {
        let mut vec = TruenoVec::new();
        vec.push(1);

        let removed = vec.remove(0);
        assert_eq!(removed, 1);
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    #[should_panic(expected = "removal index out of bounds")]
    fn test_remove_empty_vector() {
        let mut vec: TruenoVec<i32> = TruenoVec::new();
        vec.remove(0);
    }

    #[test]
    #[should_panic(expected = "removal index out of bounds")]
    fn test_remove_out_of_bounds() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.remove(5); // index >= len
    }

    #[test]
    fn test_iter() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let mut iter = vec.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_empty() {
        let vec: TruenoVec<i32> = TruenoVec::new();
        let mut iter = vec.iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_collect() {
        let mut vec = TruenoVec::new();
        for i in 0..10 {
            vec.push(i);
        }

        let collected: Vec<_> = vec.iter().copied().collect();
        assert_eq!(collected, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn test_iter_rev() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let collected: Vec<_> = vec.iter().copied().rev().collect();
        assert_eq!(collected, vec![3, 2, 1]);
    }

    #[test]
    fn test_iter_mut() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        for elem in &mut vec {
            *elem *= 2;
        }

        assert_eq!(vec.get(0), Some(&2));
        assert_eq!(vec.get(1), Some(&4));
        assert_eq!(vec.get(2), Some(&6));
    }

    #[test]
    fn test_iter_mut_empty() {
        let mut vec: TruenoVec<i32> = TruenoVec::new();
        let mut iter = vec.iter_mut();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_into_iter() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let mut iter = vec.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_into_iter_sum() {
        let mut vec = TruenoVec::new();
        for i in 1..=10 {
            vec.push(i);
        }

        let sum: i32 = vec.into_iter().sum();
        assert_eq!(sum, 55); // 1+2+3+...+10 = 55
    }

    #[test]
    fn test_for_loop_ref() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let mut sum = 0;
        for &elem in &vec {
            sum += elem;
        }
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_for_loop_mut() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        for elem in &mut vec {
            *elem += 10;
        }

        assert_eq!(vec.get(0), Some(&11));
        assert_eq!(vec.get(1), Some(&12));
        assert_eq!(vec.get(2), Some(&13));
    }

    #[test]
    fn test_exact_size_iterator() {
        let mut vec = TruenoVec::new();
        for i in 0..10 {
            vec.push(i);
        }

        let iter = vec.iter();
        assert_eq!(iter.len(), 10);

        let iter_mut = vec.iter_mut();
        assert_eq!(iter_mut.len(), 10);
    }

    #[test]
    fn test_into_iter_drop() {
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

            // Consume only 2 elements
            let mut iter = vec.into_iter();
            iter.next();
            iter.next();
            // iter dropped here, should drop remaining 3 + consumed 2 = 5 total
        }

        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    // ========================================================================
    // Index Trait Tests
    // ========================================================================

    #[test]
    fn test_index_read() {
        let mut vec = TruenoVec::new();
        vec.push(10);
        vec.push(20);
        vec.push(30);

        assert_eq!(vec[0], 10);
        assert_eq!(vec[1], 20);
        assert_eq!(vec[2], 30);
    }

    #[test]
    fn test_index_mut_write() {
        let mut vec = TruenoVec::new();
        vec.push(10);
        vec.push(20);
        vec.push(30);

        vec[1] = 25;
        assert_eq!(vec[1], 25);
        assert_eq!(vec.get(1), Some(&25));
    }

    #[test]
    fn test_index_mut_compound_assignment() {
        let mut vec = TruenoVec::new();
        vec.push(10);
        vec.push(20);
        vec.push(30);

        vec[0] += 5;
        vec[1] *= 2;
        vec[2] -= 10;

        assert_eq!(vec[0], 15);
        assert_eq!(vec[1], 40);
        assert_eq!(vec[2], 20);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_index_out_of_bounds() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        let _ = vec[5]; // Should panic
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_index_mut_out_of_bounds() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec[5] = 10; // Should panic
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_index_empty_vector() {
        let vec: TruenoVec<i32> = TruenoVec::new();
        let _ = vec[0]; // Should panic
    }

    // ========================================================================
    // Clone Trait Tests
    // ========================================================================

    #[test]
    #[allow(clippy::redundant_clone)]
    fn test_clone_empty() {
        let vec: TruenoVec<i32> = TruenoVec::new();
        let cloned = vec.clone();

        // Both should be empty
        assert_eq!(vec.len(), 0);
        assert_eq!(cloned.len(), 0);
        assert_eq!(vec.capacity(), cloned.capacity());
        assert!(vec.is_empty());
        assert!(cloned.is_empty());
    }

    #[test]
    fn test_clone_with_elements() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let cloned = vec.clone();

        assert_eq!(vec.len(), cloned.len());
        for i in 0..vec.len() {
            assert_eq!(vec.get(i), cloned.get(i));
        }
    }

    #[test]
    fn test_clone_independence() {
        let mut vec1 = TruenoVec::new();
        vec1.push(1);
        vec1.push(2);
        vec1.push(3);

        let mut vec2 = vec1.clone();

        // Modify vec2
        vec2.push(4);
        vec2[0] = 10;

        // vec1 should be unchanged
        assert_eq!(vec1.len(), 3);
        assert_eq!(vec1[0], 1);
        assert_eq!(vec1[1], 2);
        assert_eq!(vec1[2], 3);

        // vec2 has the changes
        assert_eq!(vec2.len(), 4);
        assert_eq!(vec2[0], 10);
        assert_eq!(vec2[1], 2);
        assert_eq!(vec2[2], 3);
        assert_eq!(vec2[3], 4);
    }

    #[test]
    fn test_clone_with_strings() {
        let mut vec1 = TruenoVec::new();
        vec1.push(String::from("hello"));
        vec1.push(String::from("world"));

        let vec2 = vec1.clone();

        assert_eq!(vec1.len(), vec2.len());
        assert_eq!(vec1[0], "hello");
        assert_eq!(vec2[0], "hello");
        assert_eq!(vec1[1], "world");
        assert_eq!(vec2[1], "world");
    }

    #[test]
    fn test_clone_capacity() {
        let mut vec = TruenoVec::with_capacity(100);
        vec.push(1);
        vec.push(2);

        let cloned = vec.clone();

        // Clone should have capacity equal to length (optimized allocation)
        assert_eq!(cloned.capacity(), 2);
        assert_eq!(cloned.len(), 2);
    }

    #[test]
    fn test_clone_drop_independence() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        struct DropCounter(Arc<AtomicUsize>);
        impl Drop for DropCounter {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        impl Clone for DropCounter {
            fn clone(&self) -> Self {
                Self(Arc::clone(&self.0))
            }
        }

        let counter = Arc::new(AtomicUsize::new(0));

        {
            let mut vec1 = TruenoVec::new();
            vec1.push(DropCounter(Arc::clone(&counter)));
            vec1.push(DropCounter(Arc::clone(&counter)));

            {
                let vec2 = vec1.clone();
                assert_eq!(vec2.len(), 2);
            } // vec2 dropped here - should drop 2 elements

            assert_eq!(counter.load(Ordering::SeqCst), 2);
        } // vec1 dropped here - should drop 2 more elements

        assert_eq!(counter.load(Ordering::SeqCst), 4);
    }

    // ========================================================================
    // PartialEq and Eq Trait Tests
    // ========================================================================

    #[test]
    fn test_equality_same_elements() {
        let mut vec1 = TruenoVec::new();
        vec1.push(1);
        vec1.push(2);
        vec1.push(3);

        let mut vec2 = TruenoVec::new();
        vec2.push(1);
        vec2.push(2);
        vec2.push(3);

        assert_eq!(vec1, vec2);
    }

    #[test]
    fn test_equality_empty_vectors() {
        let vec1: TruenoVec<i32> = TruenoVec::new();
        let vec2: TruenoVec<i32> = TruenoVec::new();

        assert_eq!(vec1, vec2);
    }

    #[test]
    fn test_inequality_different_length() {
        let mut vec1 = TruenoVec::new();
        vec1.push(1);
        vec1.push(2);

        let mut vec2 = TruenoVec::new();
        vec2.push(1);
        vec2.push(2);
        vec2.push(3);

        assert_ne!(vec1, vec2);
    }

    #[test]
    fn test_inequality_different_elements() {
        let mut vec1 = TruenoVec::new();
        vec1.push(1);
        vec1.push(2);
        vec1.push(3);

        let mut vec2 = TruenoVec::new();
        vec2.push(1);
        vec2.push(5);
        vec2.push(3);

        assert_ne!(vec1, vec2);
    }

    #[test]
    fn test_equality_with_strings() {
        let mut vec1 = TruenoVec::new();
        vec1.push(String::from("hello"));
        vec1.push(String::from("world"));

        let mut vec2 = TruenoVec::new();
        vec2.push(String::from("hello"));
        vec2.push(String::from("world"));

        assert_eq!(vec1, vec2);
    }

    #[test]
    fn test_equality_reflexive() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);

        assert_eq!(vec, vec);
    }

    #[test]
    fn test_equality_symmetric() {
        let mut vec1 = TruenoVec::new();
        vec1.push(1);
        vec1.push(2);

        let mut vec2 = TruenoVec::new();
        vec2.push(1);
        vec2.push(2);

        assert_eq!(vec1, vec2);
        assert_eq!(vec2, vec1);
    }

    #[test]
    fn test_equality_transitive() {
        let mut vec1 = TruenoVec::new();
        vec1.push(1);
        vec1.push(2);

        let mut vec2 = TruenoVec::new();
        vec2.push(1);
        vec2.push(2);

        let mut vec3 = TruenoVec::new();
        vec3.push(1);
        vec3.push(2);

        assert_eq!(vec1, vec2);
        assert_eq!(vec2, vec3);
        assert_eq!(vec1, vec3);
    }

    // ========================================================================
    // Debug Trait Tests
    // ========================================================================

    #[test]
    fn test_debug_empty() {
        let vec: TruenoVec<i32> = TruenoVec::new();
        assert_eq!(format!("{vec:?}"), "[]");
    }

    #[test]
    fn test_debug_single_element() {
        let mut vec = TruenoVec::new();
        vec.push(42);
        assert_eq!(format!("{vec:?}"), "[42]");
    }

    #[test]
    fn test_debug_multiple_elements() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(format!("{vec:?}"), "[1, 2, 3]");
    }

    #[test]
    fn test_debug_with_strings() {
        let mut vec = TruenoVec::new();
        vec.push(String::from("hello"));
        vec.push(String::from("world"));
        assert_eq!(format!("{vec:?}"), "[\"hello\", \"world\"]");
    }

    #[test]
    fn test_debug_nested() {
        let mut inner = TruenoVec::new();
        inner.push(1);
        inner.push(2);

        let mut outer = TruenoVec::new();
        outer.push(inner.clone());
        outer.push(inner);

        assert_eq!(format!("{outer:?}"), "[[1, 2], [1, 2]]");
    }

    // ========================================================================
    // Conversion Trait Tests (From, FromIterator, Extend)
    // ========================================================================

    #[test]
    fn test_from_vec_basic() {
        let std_vec = vec![1, 2, 3, 4, 5];
        let trueno_vec: TruenoVec<i32> = TruenoVec::from(std_vec);

        assert_eq!(trueno_vec.len(), 5);
        assert_eq!(trueno_vec.get(0), Some(&1));
        assert_eq!(trueno_vec.get(2), Some(&3));
        assert_eq!(trueno_vec.get(4), Some(&5));
    }

    #[test]
    fn test_from_vec_empty() {
        let std_vec: Vec<i32> = vec![];
        let trueno_vec: TruenoVec<i32> = TruenoVec::from(std_vec);

        assert_eq!(trueno_vec.len(), 0);
        assert!(trueno_vec.is_empty());
    }

    #[test]
    fn test_from_vec_large() {
        let std_vec: Vec<i32> = (0..100).collect();
        let trueno_vec: TruenoVec<i32> = TruenoVec::from(std_vec);

        assert_eq!(trueno_vec.len(), 100);
        for i in 0..100 {
            assert_eq!(trueno_vec.get(i), Some(&i32::try_from(i).unwrap()));
        }
    }

    #[test]
    fn test_from_vec_drops_properly() {
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
            let mut std_vec = Vec::new();
            for _ in 0..5 {
                std_vec.push(DropCounter(Arc::clone(&counter)));
            }

            let trueno_vec = TruenoVec::from(std_vec);
            assert_eq!(trueno_vec.len(), 5);
        } // trueno_vec dropped here

        // All 5 elements should be dropped exactly once
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn test_from_slice_basic() {
        let array = [1, 2, 3, 4, 5];
        let slice: &[i32] = &array;
        let trueno_vec: TruenoVec<i32> = TruenoVec::from(slice);

        assert_eq!(trueno_vec.len(), 5);
        assert_eq!(trueno_vec.get(0), Some(&1));
        assert_eq!(trueno_vec.get(2), Some(&3));
        assert_eq!(trueno_vec.get(4), Some(&5));
    }

    #[test]
    fn test_from_slice_empty() {
        let slice: &[i32] = &[];
        let trueno_vec: TruenoVec<i32> = TruenoVec::from(slice);

        assert_eq!(trueno_vec.len(), 0);
        assert!(trueno_vec.is_empty());
    }

    #[test]
    fn test_from_slice_clones_elements() {
        // Test that elements are cloned, not moved
        let original = vec![String::from("hello"), String::from("world")];
        let slice = original.as_slice();

        let trueno_vec: TruenoVec<String> = TruenoVec::from(slice);

        // Original should still be valid
        assert_eq!(original.len(), 2);
        assert_eq!(original[0], "hello");
        assert_eq!(original[1], "world");

        // TruenoVec should have cloned values
        assert_eq!(trueno_vec.len(), 2);
        assert_eq!(trueno_vec.get(0), Some(&String::from("hello")));
        assert_eq!(trueno_vec.get(1), Some(&String::from("world")));
    }

    #[test]
    fn test_from_slice_large() {
        let slice: Vec<i32> = (0..100).collect();
        let trueno_vec: TruenoVec<i32> = TruenoVec::from(slice.as_slice());

        assert_eq!(trueno_vec.len(), 100);
        for i in 0..100 {
            assert_eq!(trueno_vec.get(i), Some(&i32::try_from(i).unwrap()));
        }
    }

    #[test]
    fn test_from_iterator_range() {
        let trueno_vec: TruenoVec<i32> = (0..10).collect();

        assert_eq!(trueno_vec.len(), 10);
        for i in 0..10 {
            assert_eq!(trueno_vec.get(i), Some(&i32::try_from(i).unwrap()));
        }
    }

    #[test]
    fn test_from_iterator_vec() {
        let std_vec = vec![1, 2, 3, 4, 5];
        let trueno_vec: TruenoVec<i32> = std_vec.into_iter().collect();

        assert_eq!(trueno_vec.len(), 5);
        assert_eq!(trueno_vec.get(0), Some(&1));
        assert_eq!(trueno_vec.get(4), Some(&5));
    }

    #[test]
    fn test_from_iterator_empty() {
        let empty: Vec<i32> = vec![];
        let trueno_vec: TruenoVec<i32> = empty.into_iter().collect();

        assert_eq!(trueno_vec.len(), 0);
        assert!(trueno_vec.is_empty());
    }

    #[test]
    fn test_from_iterator_map() {
        let numbers = [1, 2, 3, 4, 5];
        let doubled: TruenoVec<i32> = numbers.iter().map(|x| x * 2).collect();

        assert_eq!(doubled.len(), 5);
        assert_eq!(doubled.get(0), Some(&2));
        assert_eq!(doubled.get(2), Some(&6));
        assert_eq!(doubled.get(4), Some(&10));
    }

    #[test]
    fn test_from_iterator_filter() {
        let trueno_vec: TruenoVec<i32> = (0..20).filter(|x| x % 2 == 0).collect();

        assert_eq!(trueno_vec.len(), 10); // 0, 2, 4, 6, 8, 10, 12, 14, 16, 18
        assert_eq!(trueno_vec.get(0), Some(&0));
        assert_eq!(trueno_vec.get(5), Some(&10));
        assert_eq!(trueno_vec.get(9), Some(&18));
    }

    #[test]
    fn test_from_iterator_large() {
        let trueno_vec: TruenoVec<i32> = (0..1000).collect();

        assert_eq!(trueno_vec.len(), 1000);
        assert_eq!(trueno_vec.get(0), Some(&0));
        assert_eq!(trueno_vec.get(500), Some(&500));
        assert_eq!(trueno_vec.get(999), Some(&999));
    }

    #[test]
    fn test_extend_empty_vec() {
        let mut trueno_vec: TruenoVec<i32> = TruenoVec::new();
        trueno_vec.extend(vec![1, 2, 3, 4, 5]);

        assert_eq!(trueno_vec.len(), 5);
        assert_eq!(trueno_vec.get(0), Some(&1));
        assert_eq!(trueno_vec.get(4), Some(&5));
    }

    #[test]
    fn test_extend_existing_vec() {
        let mut trueno_vec = TruenoVec::new();
        trueno_vec.push(1);
        trueno_vec.push(2);

        trueno_vec.extend(vec![3, 4, 5]);

        assert_eq!(trueno_vec.len(), 5);
        assert_eq!(trueno_vec.get(0), Some(&1));
        assert_eq!(trueno_vec.get(2), Some(&3));
        assert_eq!(trueno_vec.get(4), Some(&5));
    }

    #[test]
    fn test_extend_multiple_times() {
        let mut trueno_vec = TruenoVec::new();

        trueno_vec.extend(vec![1, 2]);
        assert_eq!(trueno_vec.len(), 2);

        trueno_vec.extend(vec![3, 4]);
        assert_eq!(trueno_vec.len(), 4);

        trueno_vec.extend(vec![5, 6]);
        assert_eq!(trueno_vec.len(), 6);

        for i in 0..6 {
            assert_eq!(trueno_vec.get(i), Some(&i32::try_from(i + 1).unwrap()));
        }
    }

    #[test]
    fn test_extend_from_range() {
        let mut trueno_vec = TruenoVec::new();
        trueno_vec.extend(0..10);

        assert_eq!(trueno_vec.len(), 10);
        for i in 0..10 {
            assert_eq!(trueno_vec.get(i), Some(&i32::try_from(i).unwrap()));
        }
    }

    #[test]
    fn test_extend_from_iterator() {
        let mut trueno_vec = TruenoVec::new();
        trueno_vec.push(1);

        let numbers = vec![2, 3, 4];
        trueno_vec.extend(numbers.into_iter().map(|x| x * 2));

        assert_eq!(trueno_vec.len(), 4);
        assert_eq!(trueno_vec.get(0), Some(&1));
        assert_eq!(trueno_vec.get(1), Some(&4));
        assert_eq!(trueno_vec.get(2), Some(&6));
        assert_eq!(trueno_vec.get(3), Some(&8));
    }

    #[test]
    fn test_extend_empty_iterator() {
        let mut trueno_vec = TruenoVec::new();
        trueno_vec.push(1);
        trueno_vec.push(2);

        let empty: Vec<i32> = vec![];
        trueno_vec.extend(empty);

        assert_eq!(trueno_vec.len(), 2);
        assert_eq!(trueno_vec.get(0), Some(&1));
        assert_eq!(trueno_vec.get(1), Some(&2));
    }

    #[test]
    fn test_extend_large() {
        let mut trueno_vec = TruenoVec::new();
        trueno_vec.extend(0..500);

        assert_eq!(trueno_vec.len(), 500);

        trueno_vec.extend(500..1000);

        assert_eq!(trueno_vec.len(), 1000);
        for i in 0..1000 {
            assert_eq!(trueno_vec.get(i), Some(&i32::try_from(i).unwrap()));
        }
    }

    #[test]
    fn test_extend_preserves_capacity() {
        let mut trueno_vec = TruenoVec::with_capacity(100);
        trueno_vec.extend(0..50);

        assert_eq!(trueno_vec.len(), 50);
        assert!(trueno_vec.capacity() >= 100);
    }

    #[test]
    fn test_conversion_chain() {
        // Test that conversions can be chained
        let std_vec = vec![1, 2, 3, 4, 5];
        let trueno_vec: TruenoVec<i32> = TruenoVec::from(std_vec);

        // Extend it
        let mut extended = trueno_vec;
        extended.extend(6..10);

        assert_eq!(extended.len(), 9);
        for i in 0..9 {
            assert_eq!(extended.get(i), Some(&i32::try_from(i + 1).unwrap()));
        }
    }

    #[test]
    fn test_collect_then_extend() {
        let trueno_vec: TruenoVec<i32> = (0..5).collect();
        let mut extended = trueno_vec;
        extended.extend(5..10);

        assert_eq!(extended.len(), 10);
        for i in 0..10 {
            assert_eq!(extended.get(i), Some(&i32::try_from(i).unwrap()));
        }
    }

    // ========================================================================
    // Phase 3.1: Deref/DerefMut Trait Tests
    // ========================================================================

    #[test]
    fn test_deref_to_slice() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        // Deref coercion to slice
        let slice: &[i32] = &vec;
        assert_eq!(slice.len(), 3);
        assert_eq!(slice[0], 1);
        assert_eq!(slice[1], 2);
        assert_eq!(slice[2], 3);
    }

    #[test]
    fn test_deref_slice_methods() {
        let mut vec = TruenoVec::new();
        vec.push(10);
        vec.push(20);
        vec.push(30);

        // Use slice methods through deref
        assert_eq!(vec.first(), Some(&10));
        assert_eq!(vec.last(), Some(&30));
        assert!(!vec.is_empty());
        assert_eq!(vec.len(), 3);
    }

    #[test]
    fn test_deref_empty_vec() {
        let vec: TruenoVec<i32> = TruenoVec::new();
        let slice: &[i32] = &vec;
        assert_eq!(slice.len(), 0);
        assert!(slice.is_empty());
        assert_eq!(vec.first(), None);
        assert_eq!(vec.last(), None);
    }

    #[test]
    fn test_deref_with_functions() {
        fn sum_slice(s: &[i32]) -> i32 {
            s.iter().sum()
        }

        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);

        // Deref coercion allows passing TruenoVec to functions expecting &[T]
        assert_eq!(sum_slice(&vec), 10);
    }

    #[test]
    fn test_deref_iter() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        // Use slice's iter method through deref
        let sum: i32 = vec.iter().sum();
        assert_eq!(sum, 6);

        let doubled: Vec<i32> = vec.iter().map(|x| x * 2).collect();
        assert_eq!(doubled, vec![2, 4, 6]);
    }

    #[test]
    fn test_deref_contains() {
        let mut vec = TruenoVec::new();
        vec.push(5);
        vec.push(10);
        vec.push(15);

        // Use slice's contains method through deref
        assert!(vec.contains(&10));
        assert!(!vec.contains(&20));
    }

    #[test]
    fn test_deref_starts_with_ends_with() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);

        // Use slice methods through deref
        assert!(vec.starts_with(&[1, 2]));
        assert!(vec.ends_with(&[3, 4]));
        assert!(!vec.starts_with(&[2, 3]));
    }

    #[test]
    fn test_deref_mut_to_slice() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        // DerefMut coercion to mutable slice
        let slice: &mut [i32] = &mut vec;
        slice[0] = 10;
        slice[1] = 20;
        slice[2] = 30;

        assert_eq!(vec.get(0), Some(&10));
        assert_eq!(vec.get(1), Some(&20));
        assert_eq!(vec.get(2), Some(&30));
    }

    #[test]
    fn test_deref_mut_sort() {
        let mut vec = TruenoVec::new();
        vec.push(3);
        vec.push(1);
        vec.push(4);
        vec.push(2);

        // Use mutable slice methods through DerefMut
        vec.sort_unstable();
        assert_eq!(vec.as_slice(), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_deref_mut_reverse() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        // Use reverse method through DerefMut
        vec.reverse();
        assert_eq!(vec.as_slice(), &[3, 2, 1]);
    }

    #[test]
    fn test_deref_mut_fill() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        // Use fill method through DerefMut
        vec.fill(42);
        assert_eq!(vec.as_slice(), &[42, 42, 42]);
    }

    #[test]
    fn test_deref_mut_swap() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        // Use swap method through DerefMut
        vec.swap(0, 2);
        assert_eq!(vec.as_slice(), &[3, 2, 1]);
    }

    #[test]
    fn test_deref_mut_iter_mut() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        // Use iter_mut through DerefMut
        for elem in &mut vec {
            *elem *= 2;
        }

        assert_eq!(vec.as_slice(), &[2, 4, 6]);
    }

    #[test]
    fn test_deref_mut_split_at_mut() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);

        // Use split_at_mut through DerefMut
        let (left, right) = vec.split_at_mut(2);
        left[0] = 10;
        right[0] = 30;

        assert_eq!(vec.as_slice(), &[10, 2, 30, 4]);
    }

    // ========================================================================
    // Phase 3.1: AsRef Trait Tests
    // ========================================================================

    #[test]
    fn test_as_ref_slice() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let slice: &[i32] = vec.as_ref();
        assert_eq!(slice, &[1, 2, 3]);
    }

    #[test]
    fn test_as_ref_empty() {
        let vec: TruenoVec<i32> = TruenoVec::new();
        let slice: &[i32] = vec.as_ref();
        assert_eq!(slice, &[] as &[i32]);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_as_ref_generic_function() {
        fn process<T: AsRef<[i32]>>(data: T) -> i32 {
            data.as_ref().iter().sum()
        }

        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert_eq!(process(&vec), 6);
        assert_eq!(process(vec![1, 2, 3]), 6);
        assert_eq!(process([1, 2, 3]), 6);
    }

    #[test]
    fn test_as_ref_with_strings() {
        let mut vec = TruenoVec::new();
        vec.push(String::from("hello"));
        vec.push(String::from("world"));

        let slice: &[String] = vec.as_ref();
        assert_eq!(slice.len(), 2);
        assert_eq!(slice[0], "hello");
        assert_eq!(slice[1], "world");
    }

    #[test]
    fn test_as_ref_trueno_vec() {
        let vec = TruenoVec::from(vec![1, 2, 3]);
        let vec_ref: &TruenoVec<i32> = vec.as_ref();
        assert_eq!(vec_ref.len(), 3);
        assert_eq!(vec_ref.get(0), Some(&1));
    }

    #[test]
    fn test_as_ref_multiple_calls() {
        let mut vec = TruenoVec::new();
        vec.push(42);

        let ref1: &[i32] = vec.as_ref();
        let ref2: &[i32] = vec.as_ref();

        // Multiple AsRef calls should return equivalent slices
        assert_eq!(ref1, ref2);
        assert_eq!(ref1.as_ptr(), ref2.as_ptr());
    }

    // ========================================================================
    // Phase 3.1: AsMut Trait Tests
    // ========================================================================

    #[test]
    fn test_as_mut_slice() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let slice: &mut [i32] = vec.as_mut();
        slice[0] = 10;
        slice[2] = 30;

        assert_eq!(vec.as_slice(), &[10, 2, 30]);
    }

    #[test]
    fn test_as_mut_empty() {
        let mut vec: TruenoVec<i32> = TruenoVec::new();
        let slice: &mut [i32] = vec.as_mut();
        assert_eq!(slice, &mut [] as &mut [i32]);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_as_mut_generic_function() {
        fn double_in_place<T: AsMut<[i32]>>(mut data: T) {
            for x in data.as_mut() {
                *x *= 2;
            }
        }

        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        double_in_place(&mut vec);
        assert_eq!(vec.as_slice(), &[2, 4, 6]);

        let mut std_vec = vec![1, 2, 3];
        double_in_place(&mut std_vec);
        assert_eq!(std_vec, vec![2, 4, 6]);
    }

    #[test]
    fn test_as_mut_fill_via_generic() {
        fn fill_with_value<T: AsMut<[i32]>>(mut data: T, value: i32) {
            for x in data.as_mut() {
                *x = value;
            }
        }

        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        fill_with_value(&mut vec, 99);
        assert_eq!(vec.as_slice(), &[99, 99, 99]);
    }

    #[test]
    fn test_as_mut_trueno_vec() {
        let mut vec = TruenoVec::from(vec![1, 2, 3]);
        let vec_mut: &mut TruenoVec<i32> = vec.as_mut();
        vec_mut.push(4);
        vec_mut.push(5);

        assert_eq!(vec.len(), 5);
        assert_eq!(vec.as_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_as_mut_sorting() {
        fn sort_slice<T: AsMut<[i32]>>(mut data: T) {
            data.as_mut().sort_unstable();
        }

        let mut vec = TruenoVec::new();
        vec.push(3);
        vec.push(1);
        vec.push(4);
        vec.push(2);

        sort_slice(&mut vec);
        assert_eq!(vec.as_slice(), &[1, 2, 3, 4]);
    }

    // ========================================================================
    // Phase 3.1: as_slice and as_slice_mut Tests
    // ========================================================================

    #[test]
    fn test_as_slice_basic() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let slice = vec.as_slice();
        assert_eq!(slice, &[1, 2, 3]);
        assert_eq!(slice.len(), 3);
    }

    #[test]
    fn test_as_slice_empty() {
        let vec: TruenoVec<i32> = TruenoVec::new();
        let slice = vec.as_slice();
        assert_eq!(slice, &[] as &[i32]);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_as_slice_after_push() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        assert_eq!(vec.as_slice(), &[1]);

        vec.push(2);
        assert_eq!(vec.as_slice(), &[1, 2]);

        vec.push(3);
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_as_slice_mut_basic() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let slice = vec.as_slice_mut();
        slice[1] = 20;

        assert_eq!(vec.get(1), Some(&20));
        assert_eq!(vec.as_slice(), &[1, 20, 3]);
    }

    #[test]
    fn test_as_slice_mut_empty() {
        let mut vec: TruenoVec<i32> = TruenoVec::new();
        let slice = vec.as_slice_mut();
        assert_eq!(slice, &mut [] as &mut [i32]);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_as_slice_mut_modify_all() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        for elem in vec.as_slice_mut() {
            *elem *= 10;
        }

        assert_eq!(vec.as_slice(), &[10, 20, 30]);
    }

    // ========================================================================
    // Phase 3.1: Integration Tests
    // ========================================================================

    #[test]
    fn test_deref_and_as_ref_consistency() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let via_deref: &[i32] = &vec;
        let via_as_ref: &[i32] = vec.as_ref();
        let via_as_slice = vec.as_slice();

        assert_eq!(via_deref, via_as_ref);
        assert_eq!(via_deref, via_as_slice);
        assert_eq!(via_deref.as_ptr(), via_as_ref.as_ptr());
        assert_eq!(via_deref.as_ptr(), via_as_slice.as_ptr());
    }

    #[test]
    fn test_deref_mut_and_as_mut_consistency() {
        let mut vec1 = TruenoVec::from(vec![1, 2, 3]);
        let mut vec2 = TruenoVec::from(vec![1, 2, 3]);

        let slice1: &mut [i32] = &mut vec1;
        let slice2: &mut [i32] = vec2.as_mut();

        slice1[0] = 10;
        slice2[0] = 10;

        assert_eq!(vec1.as_slice(), vec2.as_slice());
    }

    #[test]
    fn test_slice_methods_comprehensive() {
        let mut vec = TruenoVec::new();
        for i in 0..10 {
            vec.push(i);
        }

        // Test various slice methods through Deref
        assert_eq!(vec.first(), Some(&0));
        assert_eq!(vec.last(), Some(&9));
        assert!(vec.contains(&5));
        assert!(!vec.contains(&20));
        let slice = vec.as_slice();
        assert_eq!(&slice[2..5], &[2, 3, 4]);
        assert_eq!(slice.get(5..8), Some(&[5, 6, 7][..]));
    }

    #[test]
    fn test_mutable_slice_methods_comprehensive() {
        let mut vec = TruenoVec::from(vec![5, 2, 8, 1, 9, 3]);

        // Sort through DerefMut
        vec.sort_unstable();
        assert_eq!(vec.as_slice(), &[1, 2, 3, 5, 8, 9]);

        // Reverse
        vec.reverse();
        assert_eq!(vec.as_slice(), &[9, 8, 5, 3, 2, 1]);

        // Rotate
        vec.rotate_left(2);
        assert_eq!(vec.as_slice(), &[5, 3, 2, 1, 9, 8]);
    }

    #[test]
    fn test_generic_function_interoperability() {
        fn count_evens<T: AsRef<[i32]>>(data: T) -> usize {
            data.as_ref().iter().filter(|&&x| x % 2 == 0).count()
        }

        let trueno_vec = TruenoVec::from(vec![1, 2, 3, 4, 5, 6]);
        let std_vec = vec![1, 2, 3, 4, 5, 6];

        assert_eq!(count_evens(&trueno_vec), count_evens(&std_vec));
        assert_eq!(count_evens(&trueno_vec), 3);
    }

    // ========================================================================
    // Phase 3.2: PartialOrd Trait Tests
    // ========================================================================

    #[test]
    fn test_partial_ord_less_than() {
        let mut vec1 = TruenoVec::new();
        vec1.push(1);
        vec1.push(2);

        let mut vec2 = TruenoVec::new();
        vec2.push(1);
        vec2.push(3);

        assert!(vec1 < vec2);
        assert!((vec2 >= vec1));
    }

    #[test]
    fn test_partial_ord_greater_than() {
        let mut vec1 = TruenoVec::new();
        vec1.push(2);
        vec1.push(3);

        let mut vec2 = TruenoVec::new();
        vec2.push(1);
        vec2.push(4);

        assert!(vec1 > vec2);
        assert!((vec2 <= vec1));
    }

    #[test]
    fn test_partial_ord_less_equal() {
        let mut vec1 = TruenoVec::new();
        vec1.push(1);
        vec1.push(2);

        let mut vec2 = TruenoVec::new();
        vec2.push(1);
        vec2.push(2);

        let mut vec3 = TruenoVec::new();
        vec3.push(1);
        vec3.push(3);

        assert!(vec1 <= vec2); // Equal
        assert!(vec1 <= vec3); // Less
        assert!((vec3 > vec1));
    }

    #[test]
    fn test_partial_ord_greater_equal() {
        let mut vec1 = TruenoVec::new();
        vec1.push(1);
        vec1.push(2);

        let mut vec2 = TruenoVec::new();
        vec2.push(1);
        vec2.push(2);

        let mut vec3 = TruenoVec::new();
        vec3.push(1);
        vec3.push(1);

        assert!(vec1 >= vec2); // Equal
        assert!(vec1 >= vec3); // Greater
        assert!((vec3 < vec1));
    }

    #[test]
    fn test_partial_ord_different_lengths() {
        let mut vec1 = TruenoVec::new();
        vec1.push(1);
        vec1.push(2);
        vec1.push(3);

        let mut vec2 = TruenoVec::new();
        vec2.push(1);
        vec2.push(2);

        // Shorter vector is less
        assert!(vec2 < vec1);
        assert!(vec1 > vec2);
        assert!((vec1 >= vec2));
    }

    #[test]
    fn test_partial_ord_empty_vectors() {
        let vec1: TruenoVec<i32> = TruenoVec::new();
        let vec2: TruenoVec<i32> = TruenoVec::new();

        assert!((vec1 >= vec2));
        assert!((vec1 <= vec2));
        assert!(vec1 <= vec2);
        assert!(vec1 >= vec2);
    }

    #[test]
    fn test_partial_ord_empty_vs_non_empty() {
        let vec1: TruenoVec<i32> = TruenoVec::new();
        let mut vec2 = TruenoVec::new();
        vec2.push(1);

        assert!(vec1 < vec2);
        assert!((vec2 >= vec1));
    }

    #[test]
    fn test_partial_ord_lexicographic() {
        let mut vec1 = TruenoVec::new();
        vec1.push(1);
        vec1.push(2);
        vec1.push(3);

        let mut vec2 = TruenoVec::new();
        vec2.push(1);
        vec2.push(2);
        vec2.push(4);

        assert!(vec1 < vec2);
    }

    #[test]
    fn test_partial_ord_with_floats() {
        let mut vec1 = TruenoVec::new();
        vec1.push(1.5);
        vec1.push(2.5);

        let mut vec2 = TruenoVec::new();
        vec2.push(1.5);
        vec2.push(3.5);

        assert!(vec1 < vec2);
    }

    // ========================================================================
    // Phase 3.2: Ord Trait Tests
    // ========================================================================

    #[test]
    fn test_ord_cmp() {
        use std::cmp::Ordering;

        let mut vec1 = TruenoVec::new();
        vec1.push(1);
        vec1.push(2);

        let mut vec2 = TruenoVec::new();
        vec2.push(1);
        vec2.push(3);

        assert_eq!(vec1.cmp(&vec2), Ordering::Less);
        assert_eq!(vec2.cmp(&vec1), Ordering::Greater);
        assert_eq!(vec1.cmp(&vec1), Ordering::Equal);
    }

    #[test]
    fn test_ord_sorting() {
        let vec1 = TruenoVec::from(vec![3, 2, 1]);
        let vec2 = TruenoVec::from(vec![1, 2, 3]);
        let vec3 = TruenoVec::from(vec![2, 1, 3]);

        let mut vectors = [vec1.clone(), vec2.clone(), vec3.clone()];
        vectors.sort();

        assert_eq!(vectors[0], vec2);
        assert_eq!(vectors[1], vec3);
        assert_eq!(vectors[2], vec1);
    }

    #[test]
    fn test_ord_btree_map() {
        use std::collections::BTreeMap;

        let mut map = BTreeMap::new();

        let mut key1 = TruenoVec::new();
        key1.push(1);
        key1.push(2);

        let mut key2 = TruenoVec::new();
        key2.push(1);
        key2.push(3);

        map.insert(key1.clone(), "first");
        map.insert(key2.clone(), "second");

        assert_eq!(map.get(&key1), Some(&"first"));
        assert_eq!(map.get(&key2), Some(&"second"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_ord_btree_set() {
        use std::collections::BTreeSet;

        let mut set = BTreeSet::new();

        let vec1 = TruenoVec::from(vec![1, 2]);
        let vec2 = TruenoVec::from(vec![1, 3]);
        let vec3 = TruenoVec::from(vec![1, 2]); // Duplicate

        set.insert(vec1.clone());
        set.insert(vec2.clone());
        set.insert(vec3);

        assert_eq!(set.len(), 2); // vec3 is duplicate, not inserted
        assert!(set.contains(&vec1));
        assert!(set.contains(&vec2));
    }

    #[test]
    fn test_ord_min_max() {
        let vec1 = TruenoVec::from(vec![1, 2, 3]);
        let vec2 = TruenoVec::from(vec![1, 2, 4]);
        let vec3 = TruenoVec::from(vec![1, 2, 2]);

        assert_eq!(vec1.clone().min(vec2.clone()), vec1);
        assert_eq!(vec1.clone().max(vec3), vec1);
        assert_eq!(vec2.clone().max(vec1), vec2);
    }

    #[test]
    fn test_ord_different_lengths() {
        use std::cmp::Ordering;

        let vec1 = TruenoVec::from(vec![1, 2]);
        let vec2 = TruenoVec::from(vec![1, 2, 3]);

        assert_eq!(vec1.cmp(&vec2), Ordering::Less);
        assert_eq!(vec2.cmp(&vec1), Ordering::Greater);
    }

    // ========================================================================
    // Phase 3.2: Hash Trait Tests
    // ========================================================================

    #[test]
    fn test_hash_basic() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let vec1 = TruenoVec::from(vec![1, 2, 3]);
        let vec2 = TruenoVec::from(vec![1, 2, 3]);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        vec1.hash(&mut hasher1);
        vec2.hash(&mut hasher2);

        // Same content should have same hash
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_hash_different_content() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let vec1 = TruenoVec::from(vec![1, 2, 3]);
        let vec2 = TruenoVec::from(vec![1, 2, 4]);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        vec1.hash(&mut hasher1);
        vec2.hash(&mut hasher2);

        // Different content should (likely) have different hash
        assert_ne!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_hash_different_lengths() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let vec1 = TruenoVec::from(vec![1, 2]);
        let vec2 = TruenoVec::from(vec![1, 2, 3]);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        vec1.hash(&mut hasher1);
        vec2.hash(&mut hasher2);

        // Different lengths should have different hash
        assert_ne!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_hash_hashmap_insert() {
        use std::collections::HashMap;

        let mut map = HashMap::new();

        let key = TruenoVec::from(vec![1, 2, 3]);
        map.insert(key.clone(), "value");

        assert_eq!(map.get(&key), Some(&"value"));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_hash_hashmap_multiple_keys() {
        use std::collections::HashMap;

        let mut map = HashMap::new();

        let key1 = TruenoVec::from(vec![1, 2, 3]);
        let key2 = TruenoVec::from(vec![4, 5, 6]);
        let key3 = TruenoVec::from(vec![1, 2]); // Different from key1

        map.insert(key1.clone(), "first");
        map.insert(key2.clone(), "second");
        map.insert(key3.clone(), "third");

        assert_eq!(map.get(&key1), Some(&"first"));
        assert_eq!(map.get(&key2), Some(&"second"));
        assert_eq!(map.get(&key3), Some(&"third"));
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_hash_hashmap_update() {
        use std::collections::HashMap;

        let mut map = HashMap::new();

        let key = TruenoVec::from(vec![1, 2, 3]);
        map.insert(key.clone(), "first");
        map.insert(key.clone(), "second"); // Update

        assert_eq!(map.get(&key), Some(&"second"));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_hash_hashset() {
        use std::collections::HashSet;

        let mut set = HashSet::new();

        let vec1 = TruenoVec::from(vec![1, 2, 3]);
        let vec2 = TruenoVec::from(vec![4, 5, 6]);
        let vec3 = TruenoVec::from(vec![1, 2, 3]); // Duplicate

        set.insert(vec1.clone());
        set.insert(vec2.clone());
        set.insert(vec3);

        assert_eq!(set.len(), 2); // vec3 is duplicate
        assert!(set.contains(&vec1));
        assert!(set.contains(&vec2));
    }

    #[test]
    fn test_hash_empty_vector() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let vec1: TruenoVec<i32> = TruenoVec::new();
        let vec2: TruenoVec<i32> = TruenoVec::new();

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        vec1.hash(&mut hasher1);
        vec2.hash(&mut hasher2);

        // Empty vectors should have same hash
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_hash_with_strings() {
        use std::collections::HashMap;

        let mut map = HashMap::new();

        let mut key = TruenoVec::new();
        key.push(String::from("hello"));
        key.push(String::from("world"));

        map.insert(key.clone(), 42);

        assert_eq!(map.get(&key), Some(&42));
    }

    // ========================================================================
    // Phase 3.2: Integration Tests
    // ========================================================================

    #[test]
    fn test_comparison_and_hash_together() {
        use std::collections::{BTreeMap, HashMap};

        let vec1 = TruenoVec::from(vec![1, 2, 3]);
        let vec2 = TruenoVec::from(vec![1, 2, 4]);

        // Can use in HashMap
        let mut hash_map = HashMap::new();
        hash_map.insert(vec1.clone(), "hash");
        assert_eq!(hash_map.get(&vec1), Some(&"hash"));

        // Can use in BTreeMap
        let mut btree_map = BTreeMap::new();
        btree_map.insert(vec1.clone(), "btree");
        assert_eq!(btree_map.get(&vec1), Some(&"btree"));

        // Can compare
        assert!(vec1 < vec2);
    }

    #[test]
    fn test_sorting_collection_of_vectors() {
        let mut vecs = [
            TruenoVec::from(vec![3, 2, 1]),
            TruenoVec::from(vec![1, 2, 3]),
            TruenoVec::from(vec![2, 1, 3]),
            TruenoVec::from(vec![1]),
        ];

        vecs.sort_unstable();

        assert_eq!(vecs[0].as_slice(), &[1]);
        assert_eq!(vecs[1].as_slice(), &[1, 2, 3]);
        assert_eq!(vecs[2].as_slice(), &[2, 1, 3]);
        assert_eq!(vecs[3].as_slice(), &[3, 2, 1]);
    }

    #[test]
    fn test_comparison_operators_comprehensive() {
        let vec1 = TruenoVec::from(vec![1, 2, 3]);
        let vec2 = TruenoVec::from(vec![1, 2, 3]);
        let vec3 = TruenoVec::from(vec![1, 2, 4]);
        let vec4 = TruenoVec::from(vec![1, 2]);

        // Equality
        assert!(vec1 == vec2);
        assert!(vec1 != vec3);

        // Less than
        assert!(vec1 < vec3);
        assert!(vec4 < vec1);

        // Greater than
        assert!(vec3 > vec1);
        assert!(vec1 > vec4);

        // Less than or equal
        assert!(vec1 <= vec2);
        assert!(vec1 <= vec3);

        // Greater than or equal
        assert!(vec1 >= vec2);
        assert!(vec3 >= vec1);
    }

    // Phase 3.3: Display Trait Tests

    #[test]
    fn test_display_empty() {
        let vec: TruenoVec<i32> = TruenoVec::new();
        assert_eq!(format!("{vec}"), "[]");
    }

    #[test]
    fn test_display_single_element() {
        let vec = TruenoVec::from(vec![42]);
        assert_eq!(format!("{vec}"), "[42]");
    }

    #[test]
    fn test_display_multiple_elements() {
        let vec = TruenoVec::from(vec![1, 2, 3, 4, 5]);
        assert_eq!(format!("{vec}"), "[1, 2, 3, 4, 5]");
    }

    #[test]
    fn test_display_strings() {
        let vec = TruenoVec::from(vec!["hello", "world"]);
        assert_eq!(format!("{vec}"), "[hello, world]");
    }

    #[test]
    fn test_display_negative_numbers() {
        let vec = TruenoVec::from(vec![-1, -2, -3]);
        assert_eq!(format!("{vec}"), "[-1, -2, -3]");
    }

    #[test]
    fn test_display_chars() {
        let vec = TruenoVec::from(vec!['a', 'b', 'c']);
        assert_eq!(format!("{vec}"), "[a, b, c]");
    }

    #[test]
    fn test_display_after_push() {
        let mut vec = TruenoVec::new();
        vec.push(1);
        vec.push(2);
        assert_eq!(format!("{vec}"), "[1, 2]");
    }

    #[test]
    fn test_display_after_pop() {
        let mut vec = TruenoVec::from(vec![1, 2, 3]);
        vec.pop();
        assert_eq!(format!("{vec}"), "[1, 2]");
    }

    #[test]
    fn test_display_floating_point() {
        let vec = TruenoVec::from(vec![1.5, 2.7, 3.9]);
        assert_eq!(format!("{vec}"), "[1.5, 2.7, 3.9]");
    }

    #[test]
    fn test_display_with_println() {
        // Verify Display works with println! and format! macros
        let vec = TruenoVec::from(vec![1, 2, 3]);
        let display_output = format!("{vec}");
        let debug_output = format!("{vec:?}");

        // Display should produce clean output
        assert_eq!(display_output, "[1, 2, 3]");
        // Both Display and Debug should be available
        assert!(!debug_output.is_empty());
    }

    #[test]
    fn test_display_long_vector() {
        let vec: TruenoVec<i32> = (1..=20).collect();
        assert_eq!(
            format!("{vec}"),
            "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]"
        );
    }

    #[test]
    fn test_display_consistency_with_std_vec() {
        let trueno_vec = TruenoVec::from(vec![10, 20, 30]);
        let std_vec = vec![10, 20, 30];

        // Both should format the same way
        assert_eq!(format!("{trueno_vec}"), format!("{:?}", std_vec));
    }

    // Phase 3.3: Borrow Trait Tests

    #[test]
    fn test_borrow_to_slice() {
        use std::borrow::Borrow;

        let vec = TruenoVec::from(vec![1, 2, 3]);
        let slice: &[i32] = vec.borrow();
        assert_eq!(slice, &[1, 2, 3]);
    }

    #[test]
    fn test_borrow_empty() {
        use std::borrow::Borrow;

        let vec: TruenoVec<i32> = TruenoVec::new();
        let slice: &[i32] = vec.borrow();
        assert_eq!(slice, &[] as &[i32]);
    }

    #[test]
    fn test_borrow_mut_to_slice() {
        use std::borrow::BorrowMut;

        let mut vec = TruenoVec::from(vec![1, 2, 3]);
        let slice: &mut [i32] = vec.borrow_mut();
        slice[0] = 10;
        assert_eq!(slice, &[10, 2, 3]);
    }

    #[test]
    fn test_borrow_mut_modify() {
        use std::borrow::BorrowMut;

        let mut vec = TruenoVec::from(vec![5, 10, 15]);
        {
            let slice: &mut [i32] = vec.borrow_mut();
            for elem in slice.iter_mut() {
                *elem *= 2;
            }
        }
        assert_eq!(vec.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn test_borrow_with_generic_function() {
        use std::borrow::Borrow;

        fn sum_slice<T: Borrow<[i32]>>(data: &T) -> i32 {
            data.borrow().iter().sum()
        }

        let vec = TruenoVec::from(vec![1, 2, 3, 4]);
        assert_eq!(sum_slice(&vec), 10);
    }

    #[test]
    fn test_borrow_mut_with_generic_function() {
        use std::borrow::BorrowMut;

        fn double_all<T: BorrowMut<[i32]>>(data: &mut T) {
            for elem in data.borrow_mut().iter_mut() {
                *elem *= 2;
            }
        }

        let mut vec = TruenoVec::from(vec![1, 2, 3]);
        double_all(&mut vec);
        assert_eq!(vec.as_slice(), &[2, 4, 6]);
    }

    #[test]
    fn test_borrow_consistency_with_as_ref() {
        use std::borrow::Borrow;

        let vec = TruenoVec::from(vec![7, 8, 9]);
        let borrowed: &[i32] = vec.borrow();
        let as_ref: &[i32] = vec.as_ref();

        assert_eq!(borrowed, as_ref);
        assert_eq!(borrowed.as_ptr(), as_ref.as_ptr());
    }

    #[test]
    fn test_borrow_mut_consistency_with_as_mut() {
        use std::borrow::BorrowMut;

        let mut vec1 = TruenoVec::from(vec![1, 2, 3]);
        let mut vec2 = TruenoVec::from(vec![1, 2, 3]);

        let borrowed: &mut [i32] = vec1.borrow_mut();
        let as_mut: &mut [i32] = vec2.as_mut();

        borrowed[0] = 99;
        as_mut[0] = 99;

        assert_eq!(vec1.as_slice(), vec2.as_slice());
    }

    #[test]
    fn test_borrow_hashmap_lookup() {
        use std::borrow::Borrow;
        use std::collections::HashMap;

        let mut map: HashMap<Vec<i32>, &str> = HashMap::new();
        map.insert(vec![1, 2, 3], "found");

        let key = TruenoVec::from(vec![1, 2, 3]);
        // This works because TruenoVec implements Borrow<[T]>
        // and Vec's Borrow<[T]> matches
        let borrowed: &[i32] = key.borrow();

        // Verify the map has the value and borrow works
        assert_eq!(map.get(&vec![1, 2, 3]), Some(&"found"));
        assert_eq!(borrowed, &[1, 2, 3]);
    }

    #[test]
    fn test_borrow_slice_methods() {
        use std::borrow::Borrow;

        let vec = TruenoVec::from(vec![1, 2, 3, 4, 5]);
        let slice: &[i32] = vec.borrow();

        assert_eq!(slice.len(), 5);
        assert_eq!(slice.first(), Some(&1));
        assert_eq!(slice.last(), Some(&5));
        assert_eq!(&slice[1..3], &[2, 3]);
    }
}

// ============================================================================
// Property-Based Tests (Tier 2: 1-5 minutes)
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::disallowed_methods)]
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

    // Property 11: Clear Empties Vector
    proptest! {
        #[test]
        fn prop_clear_empties(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let mut v = TruenoVec::new();
            for elem in elements {
                v.push(elem);
            }

            let capacity_before = v.capacity();
            v.clear();

            prop_assert!(v.is_empty());
            prop_assert_eq!(v.len(), 0);
            prop_assert_eq!(v.capacity(), capacity_before); // Capacity unchanged
        }
    }

    // Property 12: Insert Maintains Order
    proptest! {
        #[test]
        fn prop_insert_maintains_order(
            mut elements in prop::collection::vec(any::<i32>(), 1..50),
            index in 0usize..50,
            value in any::<i32>()
        ) {
            let len = elements.len();
            if index > len {
                return Ok(());
            }

            let mut v = TruenoVec::new();
            for elem in &elements {
                v.push(*elem);
            }

            v.insert(index, value);
            elements.insert(index, value);

            prop_assert_eq!(v.len(), elements.len());
            for (i, expected) in elements.iter().enumerate() {
                prop_assert_eq!(v.get(i), Some(expected));
            }
        }
    }

    // Property 13: Remove Maintains Order
    proptest! {
        #[test]
        fn prop_remove_maintains_order(
            mut elements in prop::collection::vec(any::<i32>(), 1..50),
            index in 0usize..50
        ) {
            if elements.is_empty() || index >= elements.len() {
                return Ok(());
            }

            let mut v = TruenoVec::new();
            for elem in &elements {
                v.push(*elem);
            }

            let removed_trueno = v.remove(index);
            let removed_std = elements.remove(index);

            prop_assert_eq!(removed_trueno, removed_std);
            prop_assert_eq!(v.len(), elements.len());

            for (i, expected) in elements.iter().enumerate() {
                prop_assert_eq!(v.get(i), Some(expected));
            }
        }
    }

    // Property 14: Insert Then Remove Inverse
    proptest! {
        #[test]
        fn prop_insert_remove_inverse(
            elements in prop::collection::vec(any::<i32>(), 0..50),
            index in 0usize..50,
            value in any::<i32>()
        ) {
            if index > elements.len() {
                return Ok(());
            }

            let mut v = TruenoVec::new();
            for elem in &elements {
                v.push(*elem);
            }

            let len_before = v.len();
            v.insert(index, value);
            let removed = v.remove(index);

            prop_assert_eq!(removed, value);
            prop_assert_eq!(v.len(), len_before);

            // Verify original elements are intact
            for (i, expected) in elements.iter().enumerate() {
                prop_assert_eq!(v.get(i), Some(expected));
            }
        }
    }

    // Property 15: Clear Then Reuse
    proptest! {
        #[test]
        fn prop_clear_then_reuse(
            first_batch in prop::collection::vec(any::<i32>(), 0..50),
            second_batch in prop::collection::vec(any::<i32>(), 0..50)
        ) {
            let mut v = TruenoVec::new();

            // Fill with first batch
            for elem in &first_batch {
                v.push(*elem);
            }

            // Clear
            v.clear();
            prop_assert_eq!(v.len(), 0);

            // Fill with second batch
            for elem in &second_batch {
                v.push(*elem);
            }

            // Verify second batch is correct
            prop_assert_eq!(v.len(), second_batch.len());
            for (i, expected) in second_batch.iter().enumerate() {
                prop_assert_eq!(v.get(i), Some(expected));
            }
        }
    }

    // ========================================================================
    // Property-Based Tests for Conversion Traits
    // ========================================================================

    // Property 16: From<Vec<T>> Preserves Elements
    proptest! {
        #[test]
        fn prop_from_vec_preserves_elements(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let expected = elements.clone();
            let trueno_vec: TruenoVec<i32> = TruenoVec::from(elements);

            prop_assert_eq!(trueno_vec.len(), expected.len());
            for (i, &expected_val) in expected.iter().enumerate() {
                prop_assert_eq!(trueno_vec.get(i), Some(&expected_val));
            }
        }
    }

    // Property 17: From<&[T]> Preserves Elements
    proptest! {
        #[test]
        fn prop_from_slice_preserves_elements(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let slice = elements.as_slice();
            let trueno_vec: TruenoVec<i32> = TruenoVec::from(slice);

            prop_assert_eq!(trueno_vec.len(), elements.len());
            for (i, &expected_val) in elements.iter().enumerate() {
                prop_assert_eq!(trueno_vec.get(i), Some(&expected_val));
            }
        }
    }

    // Property 18: FromIterator Matches Vec::from_iter
    proptest! {
        #[test]
        fn prop_from_iterator_matches_vec(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let trueno_vec: TruenoVec<i32> = elements.iter().copied().collect();
            let std_vec: Vec<i32> = elements;

            prop_assert_eq!(trueno_vec.len(), std_vec.len());
            for (i, &expected_val) in std_vec.iter().enumerate() {
                prop_assert_eq!(trueno_vec.get(i), Some(&expected_val));
            }
        }
    }

    // Property 19: Extend Matches Vec::extend
    proptest! {
        #[test]
        fn prop_extend_matches_vec(
            initial in prop::collection::vec(any::<i32>(), 0..50),
            extension in prop::collection::vec(any::<i32>(), 0..50)
        ) {
            let mut trueno_vec = TruenoVec::new();
            let mut std_vec = Vec::new();

            for &elem in &initial {
                trueno_vec.push(elem);
                std_vec.push(elem);
            }

            trueno_vec.extend(extension.iter().copied());
            std_vec.extend(extension.iter().copied());

            prop_assert_eq!(trueno_vec.len(), std_vec.len());
            for (i, &expected_val) in std_vec.iter().enumerate() {
                prop_assert_eq!(trueno_vec.get(i), Some(&expected_val));
            }
        }
    }

    // Property 20: Extend Maintains Length Invariant
    proptest! {
        #[test]
        fn prop_extend_length_invariant(
            initial in prop::collection::vec(any::<i32>(), 0..50),
            extension in prop::collection::vec(any::<i32>(), 0..50)
        ) {
            let mut trueno_vec = TruenoVec::new();

            for elem in &initial {
                trueno_vec.push(*elem);
            }

            let len_before = trueno_vec.len();
            trueno_vec.extend(extension.iter().copied());

            prop_assert_eq!(trueno_vec.len(), len_before + extension.len());
        }
    }

    // Property 21: Multiple Extends Preserve Order
    proptest! {
        #[test]
        fn prop_multiple_extends_preserve_order(
            batch1 in prop::collection::vec(any::<i32>(), 0..30),
            batch2 in prop::collection::vec(any::<i32>(), 0..30),
            batch3 in prop::collection::vec(any::<i32>(), 0..30)
        ) {
            let mut trueno_vec = TruenoVec::new();
            trueno_vec.extend(batch1.iter().copied());
            trueno_vec.extend(batch2.iter().copied());
            trueno_vec.extend(batch3.iter().copied());

            let mut expected = Vec::new();
            expected.extend(&batch1);
            expected.extend(&batch2);
            expected.extend(&batch3);

            prop_assert_eq!(trueno_vec.len(), expected.len());
            for (i, &expected_val) in expected.iter().enumerate() {
                prop_assert_eq!(trueno_vec.get(i), Some(&expected_val));
            }
        }
    }

    // Property 22: FromIterator with Range
    proptest! {
        #[test]
        fn prop_from_iterator_range(start in 0i32..100, count in 0usize..100) {
            let end = start.saturating_add(i32::try_from(count).unwrap_or(i32::MAX));
            let trueno_vec: TruenoVec<i32> = (start..end).collect();

            prop_assert_eq!(trueno_vec.len(), count);
            for (i, expected_val) in (start..end).enumerate() {
                prop_assert_eq!(trueno_vec.get(i), Some(&expected_val));
            }
        }
    }

    // Property 23: Conversion Roundtrip Vec -> TruenoVec -> Iter -> Vec
    proptest! {
        #[test]
        fn prop_conversion_roundtrip(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let original = elements.clone();
            let trueno_vec: TruenoVec<i32> = TruenoVec::from(elements);
            let back_to_vec: Vec<i32> = trueno_vec.iter().copied().collect();

            prop_assert_eq!(original, back_to_vec);
        }
    }

    // Property 24: Extend with Empty Iterator is No-op
    proptest! {
        #[test]
        fn prop_extend_empty_noop(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let mut trueno_vec = TruenoVec::new();
            for elem in &elements {
                trueno_vec.push(*elem);
            }

            let len_before = trueno_vec.len();
            let empty: Vec<i32> = vec![];
            trueno_vec.extend(empty);

            prop_assert_eq!(trueno_vec.len(), len_before);
            for (i, &expected_val) in elements.iter().enumerate() {
                prop_assert_eq!(trueno_vec.get(i), Some(&expected_val));
            }
        }
    }

    // Property 25: FromIterator Filter and Map Composition
    proptest! {
        #[test]
        fn prop_from_iterator_filter_map(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let trueno_vec: TruenoVec<i32> = elements
                .iter()
                .filter(|&&x| x % 2 == 0)
                .map(|&x| x.wrapping_mul(2))
                .collect();

            let expected: Vec<i32> = elements
                .iter()
                .filter(|&&x| x % 2 == 0)
                .map(|&x| x.wrapping_mul(2))
                .collect();

            prop_assert_eq!(trueno_vec.len(), expected.len());
            for (i, &expected_val) in expected.iter().enumerate() {
                prop_assert_eq!(trueno_vec.get(i), Some(&expected_val));
            }
        }
    }

    // Property 26: Extend Maintains Capacity Bound
    proptest! {
        #[test]
        fn prop_extend_capacity_bound(
            initial in prop::collection::vec(any::<i32>(), 0..50),
            extension in prop::collection::vec(any::<i32>(), 0..50)
        ) {
            let mut trueno_vec = TruenoVec::new();
            for elem in &initial {
                trueno_vec.push(*elem);
            }

            trueno_vec.extend(extension.iter().copied());

            // Capacity must always be >= len
            prop_assert!(trueno_vec.capacity() >= trueno_vec.len());
        }
    }

    // Property 27: From<Vec<T>> and From<&[T]> Produce Same Result
    proptest! {
        #[test]
        fn prop_from_vec_and_slice_equivalent(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let from_vec: TruenoVec<i32> = TruenoVec::from(elements.clone());
            let from_slice: TruenoVec<i32> = TruenoVec::from(elements.as_slice());

            prop_assert_eq!(from_vec.len(), from_slice.len());
            for i in 0..from_vec.len() {
                prop_assert_eq!(from_vec.get(i), from_slice.get(i));
            }
        }
    }

    // Property 28: Collect and Extend are Equivalent for Same Data
    proptest! {
        #[test]
        fn prop_collect_extend_equivalent(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let collected: TruenoVec<i32> = elements.iter().copied().collect();

            let mut extended = TruenoVec::new();
            extended.extend(elements.iter().copied());

            prop_assert_eq!(collected.len(), extended.len());
            for i in 0..collected.len() {
                prop_assert_eq!(collected.get(i), extended.get(i));
            }
        }
    }

    // ========================================================================
    // Phase 3.1: Property-Based Tests for Deref/AsRef Traits
    // ========================================================================

    // Property 29: Deref Produces Same Slice as as_slice
    proptest! {
        #[test]
        fn prop_deref_equals_as_slice(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let vec = TruenoVec::from(elements);
            let via_deref: &[i32] = &vec;
            let via_as_slice = vec.as_slice();

            prop_assert_eq!(via_deref, via_as_slice);
            prop_assert_eq!(via_deref.len(), via_as_slice.len());
            prop_assert_eq!(via_deref.as_ptr(), via_as_slice.as_ptr());
        }
    }

    // Property 30: AsRef<[T]> Produces Same Slice as as_slice
    proptest! {
        #[test]
        fn prop_as_ref_equals_as_slice(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let vec = TruenoVec::from(elements);
            let via_as_ref: &[i32] = vec.as_ref();
            let via_as_slice = vec.as_slice();

            prop_assert_eq!(via_as_ref, via_as_slice);
            prop_assert_eq!(via_as_ref.len(), vec.len());
        }
    }

    // Property 31: Deref Enables Slice Methods
    proptest! {
        #[test]
        fn prop_deref_slice_methods(elements in prop::collection::vec(any::<i32>(), 1..100)) {
            let expected_first = elements.first().copied();
            let expected_last = elements.last().copied();
            let expected_len = elements.len();
            let expected_empty = elements.is_empty();
            let vec = TruenoVec::from(elements);

            // Test various slice methods through Deref
            prop_assert_eq!(vec.first().copied(), expected_first);
            prop_assert_eq!(vec.last().copied(), expected_last);
            prop_assert_eq!(vec.len(), expected_len);
            prop_assert_eq!(vec.is_empty(), expected_empty);

            for elem in &vec {
                prop_assert!(vec.contains(elem));
            }
        }
    }

    // Property 32: DerefMut Enables Mutable Slice Methods
    proptest! {
        #[test]
        fn prop_deref_mut_sorting(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let mut vec = TruenoVec::from(elements.clone());
            let mut std_vec = elements;

            // Sort using DerefMut
            vec.sort_unstable();
            std_vec.sort_unstable();

            prop_assert_eq!(vec.len(), std_vec.len());
            for i in 0..vec.len() {
                prop_assert_eq!(vec.get(i), std_vec.get(i));
            }
        }
    }

    // Property 33: DerefMut Reverse Correctness
    proptest! {
        #[test]
        fn prop_deref_mut_reverse(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let mut vec = TruenoVec::from(elements.clone());
            let mut std_vec = elements;

            vec.reverse();
            std_vec.reverse();

            prop_assert_eq!(vec.len(), std_vec.len());
            for i in 0..vec.len() {
                prop_assert_eq!(vec.get(i), std_vec.get(i));
            }
        }
    }

    // Property 34: AsRef Generic Function Interoperability
    proptest! {
        #[test]
        fn prop_as_ref_generic_interop(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            fn sum_via_as_ref<T: AsRef<[i32]>>(data: T) -> i32 {
                data.as_ref().iter().fold(0i32, |acc, &x| acc.wrapping_add(x))
            }

            let vec = TruenoVec::from(elements.clone());
            let sum_trueno = sum_via_as_ref(&vec);
            let sum_std = sum_via_as_ref(&elements);

            prop_assert_eq!(sum_trueno, sum_std);
        }
    }

    // Property 35: AsMut Generic Function Interoperability
    proptest! {
        #[test]
        fn prop_as_mut_generic_interop(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            fn double_via_as_mut<T: AsMut<[i32]>>(mut data: T) {
                for x in data.as_mut() {
                    *x = x.wrapping_mul(2);
                }
            }

            let mut vec = TruenoVec::from(elements.clone());
            let mut std_vec = elements;

            double_via_as_mut(&mut vec);
            double_via_as_mut(&mut std_vec);

            prop_assert_eq!(vec.len(), std_vec.len());
            for i in 0..vec.len() {
                prop_assert_eq!(vec.get(i), std_vec.get(i));
            }
        }
    }

    // Property 36: Deref Coercion with Function Parameters
    proptest! {
        #[test]
        fn prop_deref_function_param(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            fn process_slice(s: &[i32]) -> i32 {
                s.iter().map(|x| x.wrapping_mul(2)).fold(0i32, i32::wrapping_add)
            }

            let vec = TruenoVec::from(elements.clone());
            let result_trueno = process_slice(&vec);
            let result_std = process_slice(&elements);

            prop_assert_eq!(result_trueno, result_std);
        }
    }

    // Property 37: as_slice Length Invariant
    proptest! {
        #[test]
        fn prop_as_slice_length_invariant(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let vec = TruenoVec::from(elements.clone());
            let slice = vec.as_slice();

            prop_assert_eq!(slice.len(), vec.len());
            prop_assert_eq!(slice.len(), elements.len());
        }
    }

    // Property 38: Slice Methods Equivalence with std::Vec
    proptest! {
        #[test]
        fn prop_slice_methods_std_vec_equivalence(elements in prop::collection::vec(any::<i32>(), 1..100)) {
            let vec = TruenoVec::from(elements.clone());

            prop_assert_eq!(vec.first(), elements.first());
            prop_assert_eq!(vec.last(), elements.last());
            prop_assert_eq!(vec.is_empty(), elements.is_empty());

            for (i, elem) in elements.iter().enumerate() {
                prop_assert_eq!(vec.get(i), Some(elem));
            }
        }
    }

    // Property 39: DerefMut Fill Correctness
    proptest! {
        #[test]
        fn prop_deref_mut_fill(elements in prop::collection::vec(any::<i32>(), 0..100), fill_value in any::<i32>()) {
            let mut vec = TruenoVec::from(elements.clone());
            vec.fill(fill_value);

            prop_assert_eq!(vec.len(), elements.len());
            for i in 0..vec.len() {
                prop_assert_eq!(vec.get(i), Some(&fill_value));
            }
        }
    }

    // Property 40: Multiple Deref Calls Return Same Pointer
    proptest! {
        #[test]
        fn prop_multiple_deref_same_pointer(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let vec = TruenoVec::from(elements);
            let slice1: &[i32] = &vec;
            let slice2: &[i32] = &vec;

            prop_assert_eq!(slice1.as_ptr(), slice2.as_ptr());
            prop_assert_eq!(slice1.len(), slice2.len());
        }
    }

    // Property 41: AsRef and AsRef<TruenoVec> Consistency
    proptest! {
        #[test]
        fn prop_as_ref_self_consistency(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            let vec = TruenoVec::from(elements);
            let vec_ref: &TruenoVec<i32> = vec.as_ref();

            prop_assert_eq!(vec.len(), vec_ref.len());
            prop_assert_eq!(vec.capacity(), vec_ref.capacity());
            prop_assert_eq!(vec.is_empty(), vec_ref.is_empty());
        }
    }

    // Property 42: Deref and Index Consistency
    proptest! {
        #[test]
        fn prop_deref_index_consistency(elements in prop::collection::vec(any::<i32>(), 1..100)) {
            let vec = TruenoVec::from(elements.clone());

            for (i, elem) in elements.iter().enumerate() {
                prop_assert_eq!(&vec[i], elem);
                prop_assert_eq!(vec.get(i), Some(elem));
            }
        }
    }

    // Property 43: Slice Range Operations via Deref
    proptest! {
        #[test]
        fn prop_slice_range_operations(elements in prop::collection::vec(any::<i32>(), 5..100)) {
            let vec = TruenoVec::from(elements.clone());
            let len = vec.len();

            // Test various range operations
            if len > 2 {
                let range = 1..(len - 1);
                let vec_slice = vec.as_slice();
                prop_assert_eq!(&vec_slice[range.clone()], &elements[range]);
            }

            // Test starts_with
            if len >= 3 {
                prop_assert!(vec.starts_with(&elements[0..3]));
            }

            // Test ends_with
            if len >= 3 {
                prop_assert!(vec.ends_with(&elements[(len - 3)..len]));
            }
        }
    }

    // ========================================================================
    // Phase 3.2: Property-Based Tests for PartialOrd/Ord/Hash
    // ========================================================================

    // Property 44: PartialOrd Matches std::Vec
    proptest! {
        #[test]
        fn prop_partial_ord_matches_vec(
            elements1 in prop::collection::vec(any::<i32>(), 0..50),
            elements2 in prop::collection::vec(any::<i32>(), 0..50)
        ) {
            let trueno1 = TruenoVec::from(elements1.clone());
            let trueno2 = TruenoVec::from(elements2.clone());

            // Compare TruenoVec with same comparison as std::Vec
            prop_assert_eq!(trueno1 < trueno2, elements1 < elements2);
            prop_assert_eq!(trueno1 > trueno2, elements1 > elements2);
            prop_assert_eq!(trueno1 <= trueno2, elements1 <= elements2);
            prop_assert_eq!(trueno1 >= trueno2, elements1 >= elements2);
        }
    }

    // Property 45: Ord Transitivity
    proptest! {
        #[test]
        fn prop_ord_transitivity(
            elements1 in prop::collection::vec(any::<i32>(), 0..20),
            elements2 in prop::collection::vec(any::<i32>(), 0..20),
            elements3 in prop::collection::vec(any::<i32>(), 0..20)
        ) {
            let vec1 = TruenoVec::from(elements1);
            let vec2 = TruenoVec::from(elements2);
            let vec3 = TruenoVec::from(elements3);

            // If a < b and b < c, then a < c
            if vec1 < vec2 && vec2 < vec3 {
                prop_assert!(vec1 < vec3);
            }
        }
    }

    // Property 46: Ord Antisymmetry
    proptest! {
        #[test]
        fn prop_ord_antisymmetry(
            elements1 in prop::collection::vec(any::<i32>(), 0..50),
            elements2 in prop::collection::vec(any::<i32>(), 0..50)
        ) {
            let vec1 = TruenoVec::from(elements1);
            let vec2 = TruenoVec::from(elements2);

            // If a <= b and b <= a, then a == b
            if vec1 <= vec2 && vec2 <= vec1 {
                prop_assert_eq!(vec1, vec2);
            }
        }
    }

    // Property 47: Ord cmp Consistency with Operators
    proptest! {
        #[test]
        fn prop_ord_cmp_consistency(
            elements1 in prop::collection::vec(any::<i32>(), 0..50),
            elements2 in prop::collection::vec(any::<i32>(), 0..50)
        ) {
            use std::cmp::Ordering;

            let vec1 = TruenoVec::from(elements1);
            let vec2 = TruenoVec::from(elements2);

            let ordering = vec1.cmp(&vec2);

            match ordering {
                Ordering::Less => prop_assert!(vec1 < vec2),
                Ordering::Greater => prop_assert!(vec1 > vec2),
                Ordering::Equal => prop_assert_eq!(vec1, vec2),
            }
        }
    }

    // Property 48: Sorting Preserves Elements
    proptest! {
        #[test]
        fn prop_sorting_preserves_elements(vecs in prop::collection::vec(
            prop::collection::vec(any::<i32>(), 0..10),
            0..20
        )) {
            let mut trueno_vecs: Vec<TruenoVec<i32>> = vecs.iter()
                .map(|v| TruenoVec::from(v.clone()))
                .collect();

            let original_count = trueno_vecs.len();
            trueno_vecs.sort();

            // Sorting preserves number of elements
            prop_assert_eq!(trueno_vecs.len(), original_count);

            // Sorting produces sorted order
            for i in 0..trueno_vecs.len().saturating_sub(1) {
                prop_assert!(trueno_vecs[i] <= trueno_vecs[i + 1]);
            }
        }
    }

    // Property 49: Hash Equality Implies Same Hash
    proptest! {
        #[test]
        fn prop_hash_equality_same_hash(elements in prop::collection::vec(any::<i32>(), 0..100)) {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let vec1 = TruenoVec::from(elements.clone());
            let vec2 = TruenoVec::from(elements);

            // Equal vectors must have equal hashes
            prop_assert_eq!(&vec1, &vec2);

            let mut hasher1 = DefaultHasher::new();
            let mut hasher2 = DefaultHasher::new();

            vec1.hash(&mut hasher1);
            vec2.hash(&mut hasher2);

            prop_assert_eq!(hasher1.finish(), hasher2.finish());
        }
    }

    // Property 50: Hash Consistent with Eq
    proptest! {
        #[test]
        fn prop_hash_consistent_with_eq(
            elements1 in prop::collection::vec(any::<i32>(), 0..50),
            elements2 in prop::collection::vec(any::<i32>(), 0..50)
        ) {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let vec1 = TruenoVec::from(elements1);
            let vec2 = TruenoVec::from(elements2);

            if vec1 == vec2 {
                let mut hasher1 = DefaultHasher::new();
                let mut hasher2 = DefaultHasher::new();

                vec1.hash(&mut hasher1);
                vec2.hash(&mut hasher2);

                // If equal, hashes must be equal
                prop_assert_eq!(hasher1.finish(), hasher2.finish());
            }
        }
    }

    // Property 51: HashMap Operations Correctness
    proptest! {
        #[test]
        fn prop_hashmap_operations(
            keys in prop::collection::vec(prop::collection::vec(any::<i32>(), 0..10), 0..20)
        ) {
            use std::collections::HashMap;

            let mut map = HashMap::new();

            // Insert all keys (later duplicates will overwrite earlier ones)
            for (i, key_vec) in keys.iter().enumerate() {
                let key = TruenoVec::from(key_vec.clone());
                map.insert(key, i);
            }

            // Verify that we can retrieve values
            // For duplicates, the last inserted value should be present
            for key_vec in &keys {
                let key = TruenoVec::from(key_vec.clone());

                // Find the last occurrence of this key in the original list
                let last_index = keys.iter()
                    .enumerate()
                    .rev()
                    .find(|(_, k)| k == &key_vec)
                    .map(|(idx, _)| idx)
                    .unwrap();

                if let Some(&value) = map.get(&key) {
                    // Value should be the last inserted one for this key
                    prop_assert_eq!(value, last_index);
                }
            }
        }
    }

    // Property 52: BTreeMap Ordering
    proptest! {
        #[test]
        fn prop_btreemap_ordering(
            keys in prop::collection::vec(prop::collection::vec(any::<i32>(), 0..10), 0..20)
        ) {
            use std::collections::BTreeMap;

            let mut map = BTreeMap::new();

            for (i, key_vec) in keys.iter().enumerate() {
                let key = TruenoVec::from(key_vec.clone());
                map.insert(key, i);
            }

            // Keys should be in sorted order
            let keys_in_order: Vec<_> = map.keys().collect();
            for i in 0..keys_in_order.len().saturating_sub(1) {
                prop_assert!(keys_in_order[i] <= keys_in_order[i + 1]);
            }
        }
    }

    // Property 53: Lexicographic Ordering Matches std::Vec
    proptest! {
        #[test]
        fn prop_lexicographic_ordering(
            elements1 in prop::collection::vec(any::<i32>(), 0..50),
            elements2 in prop::collection::vec(any::<i32>(), 0..50)
        ) {


            let trueno1 = TruenoVec::from(elements1.clone());
            let trueno2 = TruenoVec::from(elements2.clone());

            // Ordering should match std::Vec
            prop_assert_eq!(trueno1.cmp(&trueno2), elements1.cmp(&elements2));
        }
    }

    // ========================================================================
    // Phase 3.3: Property-Based Tests for Display and Borrow
    // ========================================================================

    // Property 54: Display Format Consistency
    proptest! {
        #[test]
        fn prop_display_format_consistency(elements in prop::collection::vec(any::<i32>(), 0..20)) {
            let vec = TruenoVec::from(elements.clone());
            let display_output = format!("{vec}");
            let expected_output = format!("{elements:?}");

            // Display should match std::Vec's Debug output
            prop_assert_eq!(display_output, expected_output);
        }
    }

    // Property 55: Display Empty Vector
    proptest! {
        #[test]
        fn prop_display_empty(_n in 0..100usize) {
            let vec: TruenoVec<i32> = TruenoVec::new();
            prop_assert_eq!(format!("{}", vec), "[]");
        }
    }

    // Property 56: Display After Operations
    proptest! {
        #[test]
        fn prop_display_after_push(
            initial in prop::collection::vec(any::<i32>(), 0..10),
            x in any::<i32>()
        ) {
            let mut vec = TruenoVec::from(initial.clone());
            vec.push(x);

            let mut expected = initial;
            expected.push(x);

            prop_assert_eq!(format!("{}", vec), format!("{:?}", expected));
        }
    }

    // Property 57: Borrow Equivalence with AsRef
    proptest! {
        #[test]
        fn prop_borrow_equals_as_ref(elements in prop::collection::vec(any::<i32>(), 0..50)) {
            use std::borrow::Borrow;

            let vec = TruenoVec::from(elements);
            let borrowed: &[i32] = vec.borrow();
            let as_ref: &[i32] = vec.as_ref();

            prop_assert_eq!(borrowed, as_ref);
            prop_assert_eq!(borrowed.as_ptr(), as_ref.as_ptr());
        }
    }

    // Property 58: BorrowMut Equivalence with AsMut
    proptest! {
        #[test]
        fn prop_borrow_mut_equals_as_mut(elements in prop::collection::vec(any::<i32>(), 0..50)) {
            use std::borrow::BorrowMut;

            let mut vec1 = TruenoVec::from(elements.clone());
            let mut vec2 = TruenoVec::from(elements);

            let borrowed_slice: &mut [i32] = vec1.borrow_mut();
            let as_mut_slice: &mut [i32] = vec2.as_mut();

            let borrowed_ptr = borrowed_slice.as_ptr();
            let as_mut_ptr = as_mut_slice.as_ptr();

            // Both should return the same underlying pointer
            prop_assert_eq!(borrowed_ptr, vec1.as_slice().as_ptr());
            prop_assert_eq!(as_mut_ptr, vec2.as_slice().as_ptr());
        }
    }

    // Property 59: Borrow Slice Operations
    proptest! {
        #[test]
        fn prop_borrow_slice_operations(elements in prop::collection::vec(any::<i32>(), 1..50)) {
            use std::borrow::Borrow;

            let vec = TruenoVec::from(elements.clone());
            let slice: &[i32] = vec.borrow();

            // All slice operations should work correctly
            prop_assert_eq!(slice.len(), elements.len());
            prop_assert_eq!(slice.first(), elements.first());
            prop_assert_eq!(slice.last(), elements.last());
            prop_assert_eq!(slice, elements.as_slice());
        }
    }

    // Property 60: Display Respects Element Display Impl
    proptest! {
        #[test]
        fn prop_display_respects_element_display(elements in prop::collection::vec(any::<u32>(), 0..15)) {
            let vec = TruenoVec::from(elements.clone());
            let display_str = format!("{vec}");

            // Display should contain each element's string representation
            for elem in &elements {
                let elem_str = format!("{elem}");
                prop_assert!(display_str.contains(&elem_str));
            }
        }
    }
}

// ============================================================================
// Formal Verification (Tier 3: ON-MERGE/NIGHTLY)
// ============================================================================

#[cfg(kani)]
#[cfg_attr(test, mutants::skip)] // Skip mutation testing for Kani verification code
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
