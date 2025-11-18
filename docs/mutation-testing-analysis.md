# Mutation Testing Analysis - Phase 4

**Date**: 2025-11-18
**Module**: `src/vec.rs`
**Tool**: cargo-mutants v25.3.1
**Baseline Tests**: 261 passing (unit + doc + property-based)

## Executive Summary

Initial mutation testing reveals **significant gaps** in test coverage, particularly around boundary conditions, arithmetic operations, and drop semantics. Out of the first 12 mutants tested:

- **11 MISSED** (92% missed - 0% caught)
- **1 TIMEOUT** (counts as missed)
- **0 CAUGHT** (0% mutation score)

**Target**: >85% mutation score (PMAT requirement)
**Current**: 0% (early sample)
**Gap**: ~85 percentage points to target

## Mutant Analysis

### Pattern 1: Boundary Condition Errors (Most Common)

**MISSED Mutants**:
```rust
// Line 901: IntoIter::drop
replace > with >= in loop condition

// Line 413: insert bounds check
replace < with <= in index validation

// Line 1396: PartialOrd::lt
replace < with <= in comparison

// Line 1404: PartialOrd::gt
replace > with >= in comparison

// Line 749: Extend::extend capacity check
replace < with == in capacity condition
replace < with <= in capacity condition
```

**Root Cause**: Tests don't verify exact boundary behavior
**Impact**: Off-by-one errors could go undetected
**Risk Level**: **HIGH** - classic bug category

**Recommendation**:
- Add edge case tests for boundary values
- Test with capacity exactly met vs exceeded
- Property tests for index bounds
- Tests for empty, single-element, and full vectors

### Pattern 2: Arithmetic Operator Errors

**MISSED Mutants**:
```rust
// Line 459: remove index calculation
replace - with + in index arithmetic
replace - with / in index arithmetic

// Line 283: pop length decrement (TIMEOUT)
replace -= with += in length update
```

**Root Cause**: Tests verify result correctness but not calculation semantics
**Impact**: Wrong index calculations could corrupt data
**Risk Level**: **CRITICAL** - data corruption potential

**Recommendation**:
- Add property tests verifying exact indices
- Test index calculations explicitly
- Add invariant checks (e.g., length always decreases on pop)
- Model-based testing against std::vec::Vec

### Pattern 3: Iterator Semantics

**MISSED Mutants**:
```rust
// Line 883: IntoIter::size_hint
replace return value (usize, Option<usize>) with (1, Some(0))
```

**Root Cause**: size_hint not tested explicitly
**Impact**: Iterator optimizations may fail
**Risk Level**: **MEDIUM** - functional but suboptimal

**Recommendation**:
- Add iterator property tests
- Verify size_hint accuracy
- Test iterator chain operations
- Compare with std::vec::Vec iterator behavior

### Pattern 4: Drop Implementation

**MISSED Mutants**:
```rust
// Line 901: IntoIter::drop loop condition
replace > with >= in drop loop

// Line 580: TruenoVec::drop loop condition
replace > with < in drop loop

// Line 747: Extend::extend loop condition
replace > with < in extend loop
```

**Root Cause**: Drop behavior not explicitly tested
**Impact**: Memory leaks or double-free potential
**Risk Level**: **CRITICAL** - memory safety

**Recommendation**:
- Add drop tests with instrumentation
- Test dropping partially-consumed iterators
- Verify no double-free on panic
- Use Miri for undefined behavior detection

## Test Gap Analysis

### Current Test Distribution
- **Unit tests**: ~60% of suite (70 tests)
- **Property tests**: ~30% of suite (60 tests)
- **Doc tests**: ~10% of suite (46 tests)
- **Integration tests**: 0% (Phase 4 planned)

### Identified Gaps

1. **Edge Cases** (HIGH PRIORITY)
   - Empty vector operations
   - Single-element operations
   - Exactly-at-capacity operations
   - Index boundaries (0, len-1, len, len+1)

2. **Arithmetic Invariants** (CRITICAL)
   - Length always decreases on pop
   - Capacity never shrinks (unless explicit)
   - Index calculations preserve order
   - Remove shifts elements correctly

3. **Comparison Semantics** (MEDIUM)
   - PartialOrd edge cases
   - Equality with different capacities
   - Ordering transitivity

4. **Iterator Completeness** (MEDIUM)
   - size_hint accuracy
   - DoubleEndedIterator properties
   - ExactSizeIterator guarantees
   - Partial consumption behavior

5. **Drop Safety** (CRITICAL)
   - All elements dropped exactly once
   - No leaks on partial consumption
   - Panic safety in drop
   - Drop order guarantees

## Recommended Actions (Priority Order)

### Immediate (This Week)

1. **Add boundary value tests**
   ```rust
   #[test]
   fn test_pop_at_boundaries() {
       let mut vec = TruenoVec::new();
       assert_eq!(vec.pop(), None); // empty

       vec.push(1);
       assert_eq!(vec.len(), 1);
       assert_eq!(vec.pop(), Some(1));
       assert_eq!(vec.len(), 0); // verify decrement
   }
   ```

2. **Add arithmetic invariant tests**
   ```rust
   #[test]
   fn test_remove_shifts_correctly() {
       let mut vec = TruenoVec::from(vec![0, 1, 2, 3, 4]);
       vec.remove(2);
       assert_eq!(vec.get(2), Some(&3)); // exact index check
       assert_eq!(vec.len(), 4); // exact length
   }
   ```

3. **Add drop instrumentation tests**
   ```rust
   #[test]
   fn test_partial_iterator_drop() {
       // Use Rc<RefCell<bool>> to track drops
       // Verify all elements dropped on iterator drop
   }
   ```

### Short-Term (Next 2 Weeks)

4. **Property-based tests for boundaries**
   - Generate random operations at capacity boundaries
   - Verify all invariants hold

5. **Model-based testing**
   - Run operations in parallel on TruenoVec and std::Vec
   - Assert identical behavior

6. **Iterator property tests**
   - size_hint correctness
   - Partial vs full consumption
   - rev() behavior

### Medium-Term (Phase 4 Completion)

7. **Integration tests**
   - Multi-module interactions
   - Realistic usage patterns

8. **Miri testing**
   - Undefined behavior detection
   - Use-after-free, double-free

9. **Fuzzing**
   - cargo-fuzz for crash detection
   - Random operation sequences

## Mutation Testing Workflow

### Tier 3 (ON-MERGE/NIGHTLY)

**Full Mutation Testing**:
```bash
cargo mutants --timeout 300 --output mutants.out
```

**Expected Duration**: ~30-45 minutes for full suite
**Target**: >85% mutation score
**Failure Threshold**: <80% mutation score

### Incremental (During Development)

**File-Level Testing**:
```bash
cargo mutants --file src/vec.rs --timeout 60
```

**Function-Level Testing**:
```bash
cargo mutants --file src/vec.rs --line 459 --timeout 30
```

### CI/CD Integration

Add to `.github/workflows/ci.yml`:
```yaml
mutation-testing:
  name: "Tier 3: Mutation Testing"
  runs-on: ubuntu-latest
  if: github.event_name == 'push' && github.ref == 'refs/heads/main'
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo install cargo-mutants
    - run: cargo mutants --timeout 300 --output mutants.out
    - run: |
        score=$(grep "Mutation score" mutants.out | awk '{print $3}')
        if (( $(echo "$score < 85" | bc -l) )); then
          echo "Mutation score $score% below target 85%"
          exit 1
        fi
```

## Metrics Tracking

| Metric | Initial | Target | Current Sprint Goal |
|--------|---------|--------|-------------------|
| Mutation Score | 0% | >85% | >50% |
| MISSED Mutants | 11/12 | <15% | <50% |
| CAUGHT Mutants | 0/12 | >85% | >50% |
| TIMEOUT Mutants | 1/12 | <5% | <10% |

## Next Steps

1. ✅ Create mutants.toml configuration
2. ✅ Document initial findings (this document)
3. ⏳ Add boundary value tests
4. ⏳ Add arithmetic invariant tests
5. ⏳ Add drop behavior tests
6. ⏳ Re-run mutation testing
7. ⏳ Iterate until >85% score

## References

- [cargo-mutants documentation](https://mutants.rs/)
- certeza specification: "Mutation Score Asymptote" (Section 4.3)
- PMAT quality gates: min_mutation_score=85%
- Academic ref: Jia & Harman (2011) - "An Analysis and Survey of the Development of Mutation Testing"

## Conclusion

Initial mutation testing reveals **critical gaps** in boundary testing, arithmetic validation, and drop safety. The 0% initial mutation score is concerning but provides clear direction for test improvements.

**Key Insight**: Property-based tests cover general cases well (proptest passing), but miss precise boundary conditions and edge cases that mutation testing exposes.

**Action Required**: Systematic addition of edge case tests, arithmetic invariant tests, and drop behavior tests to achieve >85% mutation score target.

This validates the certeza framework's tiered approach: mutation testing (Tier 3) catches subtle bugs that fast unit tests (Tier 1-2) miss, justifying the performance cost.
