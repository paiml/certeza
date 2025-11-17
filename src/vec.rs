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
            unsafe {
                ptr::drop_in_place(self.vec.ptr.as_ptr().add(self.current));
            }
            self.current += 1;
        }

        // Now deallocate the memory (all elements have been dropped)
        if self.vec.capacity > 0 {
            let layout = Layout::array::<T>(self.vec.capacity).expect("capacity overflow");
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
    fn test_clone_empty() {
        let vec: TruenoVec<i32> = TruenoVec::new();
        let cloned = vec.clone();

        assert_eq!(vec.len(), cloned.len());
        assert_eq!(vec.capacity(), cloned.capacity());
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
        let numbers = vec![1, 2, 3, 4, 5];
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
            assert_eq!(
                trueno_vec.get(i),
                Some(&i32::try_from(i + 1).unwrap())
            );
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
            let std_vec: Vec<i32> = elements.iter().copied().collect();

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
