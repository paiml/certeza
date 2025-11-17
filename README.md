# certeza

[![CI](https://github.com/paiml/certeza/actions/workflows/ci.yml/badge.svg)](https://github.com/paiml/certeza/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**certeza** - A scientific experiment into realistic provability with Rust.

## Overview

certeza is a comprehensive framework for approaching **asymptotic test effectiveness** in Rust software systems through the pragmatic integration of:

- **Property-based testing** (using proptest)
- **Mutation testing** (using cargo-mutants)
- **Structural coverage analysis** (using cargo-llvm-cov)
- **Selective formal verification** (using Kani)

### Philosophy

While complete verification remains theoretically impossible (Dijkstra: "testing can only prove the presence of bugs, not their absence"), this framework provides a reproducible methodology for achieving practical maximum confidence in critical systems.

## Tiered TDD-X Workflow

The framework implements three-tiered verification that balances rigor with developer productivity:

### Tier 1: ON-SAVE (Sub-second feedback)
- Unit tests and focused property tests
- Static analysis (`cargo check`, `cargo clippy`)
- Enables rapid iteration in flow state

**Command**: `make tier1`

### Tier 2: ON-COMMIT (1-5 minutes)
- Full property-based test suite
- Coverage analysis (target: 95%+ line coverage)
- Integration tests
- Pre-commit hook enforcement

**Command**: `make tier2`

### Tier 3: ON-MERGE/NIGHTLY (Hours)
- Comprehensive mutation testing (target: >85% mutation score)
- Formal verification for critical paths
- Performance benchmarks
- CI/CD gate for main branch

**Command**: `make tier3`

**Critical Principle**: Different verification techniques operate at different time scales. Fast feedback enables flow; slow feedback causes context switching waste. Never run mutation testing or formal verification in the inner development loop.

## Quick Start

```bash
# Clone the repository
git clone https://github.com/paiml/certeza.git
cd certeza

# Run quick checks (Tier 1)
make tier1

# Run full test suite with coverage (Tier 2)
make tier2

# Build the project
cargo build

# Run all tests
cargo test

# Generate documentation
cargo doc --open
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
certeza = "0.1.0"
```

## Usage Example

### TruenoVec - Custom Growable Vector

The primary demonstration of the certeza framework is `TruenoVec<T>`, a custom growable vector implementation that showcases the complete three-tiered testing approach.

```rust
use certeza::TruenoVec;

fn main() {
    // Create a new vector
    let mut vec = TruenoVec::new();

    // Push elements
    vec.push(1);
    vec.push(2);
    vec.push(3);

    // Access elements
    assert_eq!(vec.len(), 3);
    assert_eq!(vec.get(1), Some(&2));

    // Pop elements
    assert_eq!(vec.pop(), Some(3));
    assert_eq!(vec.len(), 2);

    // Pre-allocate capacity for performance
    let mut vec2 = TruenoVec::with_capacity(1000);
    for i in 0..1000 {
        vec2.push(i);  // No reallocation
    }
}
```

### Key Features

- **Manual Memory Management**: Uses `NonNull<T>` for safe manual allocation
- **2x Growth Factor**: Exponential growth for amortized O(1) push operations
- **RAII Semantics**: Proper `Drop` implementation ensures no memory leaks
- **Thread Safety**: `Send + Sync` where `T` is `Send/Sync`
- **Zero-Cost Abstraction**: 24-byte overhead (ptr + len + capacity)
- **Full Iterator Support**: Bidirectional iterators (`iter`, `iter_mut`, `into_iter`)
- **Advanced Operations**: `insert`, `remove`, `clear` with optimal performance
- **Comprehensive Testing**: 127 tests with 97.7% mutation score

### Comprehensive Test Coverage

**Total Tests: 127 tests across all tiers** ✅ **97.7% mutation score**

#### Tier 1: Unit Tests (80 tests)
- Basic operations: `new`, `with_capacity`, `push`, `pop`, `clear`
- Index access: `get`, `get_mut` with bounds checking
- Advanced operations: `insert`, `remove` at various positions
- Iterator support: `iter`, `iter_mut`, `into_iter` with bidirectional iteration
- Growth behavior and capacity management
- Drop trait verification with destructor counting
- Memory deallocation and leak prevention tests
- Edge case handling (empty vectors, single elements, boundary conditions)
- Mutation-resistant test cases for critical operations
- Sub-second execution

#### Tier 2: Property-Based Tests (15 properties)
- Length invariant after push operations
- Capacity bound (capacity >= len) maintained
- Push/pop symmetry (inverse operations)
- Index access correctness
- Out-of-bounds safety
- Exponential growth factor verification
- Behavioral equivalence with `std::Vec`
- Empty vector invariants
- Mutable access correctness
- Repeated operations integrity
- Clear operation preserves capacity
- Insert/remove maintain order
- Insert-then-remove inverse operations
- Clear-then-reuse functionality

#### Tier 2: Integration Tests (26 scenarios)
- **Basic Integration Tests**:
  - Basic vector operations workflow
  - Capacity management and preallocation
  - Stack semantics (LIFO behavior)
  - Integration with standard Rust types (String, complex structs)
  - Real-world buffer usage patterns
  - Interleaved push/pop operations
  - Out-of-bounds access safety
  - Thread safety (Send + Sync)
  - Arc integration for shared ownership
- **Advanced Scenario-Based Tests**:
  - User records management system
  - Stack-based expression evaluation
  - Batch processing with pre-allocation
  - Undo/Redo system implementation
  - Graph adjacency list data structures
  - Ring buffer patterns
  - Large dataset processing (10,000 elements)
  - Type safety with various element types
  - Thread safety compilation checks
  - Memory efficiency validation (24-byte overhead)
  - Edge case handling
  - std::Vec behavioral equivalence

#### Tier 3: Formal Verification (3 Kani proofs)
- Capacity invariant proof (capacity >= len for all paths)
- Push/pop correctness proof (mathematical verification)
- Bounds checking verification (no buffer overflows)

#### Documentation Tests (21 tests)
- All public API examples verified
- 100% rustdoc coverage for public items
- Comprehensive examples in module documentation

### Performance Characteristics

Benchmarks demonstrate competitive performance with `std::Vec`:

- **Push (sequential)**: ~10 ns/operation average
- **Pop**: O(1) constant time
- **Get (random access)**: O(1) with cache efficiency
- **Growth pattern**: ~14 reallocations for 10,000 elements (log₂ n)
- **Memory efficiency**: 24-byte overhead per vector

## PMAT Compliance

This project is fully compliant with the **Pragmatic AI Labs Multi-Language Agent Toolkit (PMAT)** standards, enforcing **EXTREME TDD**:

### Quality Standards

- **Coverage**: ≥85% line coverage (minimum), 95% (target)
- **Complexity**: ≤10 cyclomatic complexity per function
- **SATD**: Zero tolerance for TODO/FIXME/HACK comments
- **Mutation Testing**: ≥85% mutation score
- **Documentation**: ≥90% rustdoc coverage

### Makefile Commands

```bash
# Development workflow
make tier1              # Quick checks (sub-second)
make tier2              # Full test suite (1-5 min)
make tier3              # Mutation testing (hours)

# Testing
make test               # Run all tests
make test-quick         # Run unit tests only
make test-property      # Run property-based tests
make coverage           # Generate coverage report

# Code quality
make clippy             # Run clippy linter
make clippy-strict      # Run strict clippy
make fmt                # Format code
make fmt-check          # Check formatting

# Analysis
make complexity         # Analyze complexity with PMAT
make tdg                # Technical debt grading
make security           # Security audit

# Documentation
make docs               # Generate documentation
make validate-docs      # Validate docs with PMAT

# Setup
make install-tools      # Install required tooling
make install-hooks      # Install PMAT git hooks
```

## Testing Framework

### Running Tests

```bash
# Tier 1: Quick checks (unit tests)
make tier1
cargo test --lib

# Tier 2: Full test suite (unit + property + integration + doc tests)
make tier2
cargo test --all

# Integration tests
cargo test --test integration_tests

# Benchmarks
cargo test --benches

# Property-based tests with custom case count
PROPTEST_CASES=1000 cargo test property_
```

### Mutation Testing

Run mutation testing to verify test suite quality:

```bash
make mutation
# or
cargo mutants --no-times
```

**Target**: >85% mutation score

### Coverage Analysis

Generate coverage report:

```bash
make coverage
# or
cargo llvm-cov --all-features --workspace
```

**Target**: 95%+ line coverage, 90%+ branch coverage

## Risk-Based Verification

Not all code requires the same verification intensity:

| Risk Level | Components | Verification Approach |
|------------|------------|----------------------|
| **Very High** | `unsafe` blocks, memory allocators, crypto | Full framework: Property + Coverage + Mutation (90%) + Formal |
| **High** | Core algorithms, data structures, parsers | Property + Coverage + Mutation (85-90%) |
| **Medium** | Business logic, API handlers | Property + Coverage + Mutation (80%) |
| **Low** | Simple accessors, config | Unit tests + Coverage (90%) |

**Resource Allocation**: Spend 40% of verification time on the 5-10% highest-risk code.

## Architecture

### Testing Pyramid

```
┌─────────────────┐
│  Formal (Kani)  │  ~1-5% code (invariant proofs)
├─────────────────┤
│   Integration   │  ~10% tests (system properties)
├─────────────────┤
│  Property-Based │  ~30% tests (algorithmic correctness)
├─────────────────┤
│   Unit Tests    │  ~60% tests (basic functionality)
└─────────────────┘
```

### Key Design Principles

- **Sustainable Workflows**: Tiered feedback loops prevent burnout
- **Risk-Based Resource Allocation**: Focus on high-risk components
- **Human-Centered Analysis**: Mutation analysis as learning exercise
- **Economic Realism**: Acknowledge costs and diminishing returns

## Documentation

- [Main Specification](docs/specifications/theoretical-max-testing-spec.md) - Complete framework specification (v1.1, ~14K words)
- [Improvements v1.1](docs/specifications/IMPROVEMENTS_v1.1.md) - Philosophy shift from "theoretical maximum" to "asymptotic effectiveness"
- [CLAUDE.md](CLAUDE.md) - Guidance for Claude Code development

## CI/CD

GitHub Actions workflow enforces quality gates:

- **Tier 1**: Quick checks on every push
- **Tier 2**: Full test suite + coverage on PR (≥85%)
- **Security**: Parallel security audit (cargo-audit, cargo-deny)
- **Tier 3**: Mutation testing on merge to main (≥85% mutation score)

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `make tier2`
2. Coverage ≥85%: `make coverage`
3. Clippy passes: `make clippy-strict`
4. Code is formatted: `cargo fmt`
5. No SATD comments (TODO/FIXME/HACK)

## License

MIT License - see [LICENSE](LICENSE) file for details.

## References

This project implements research findings from the certeza specification, incorporating:

- Property-based testing methodologies (QuickCheck, Hypothesis, proptest)
- Mutation testing research (Jia & Harman, 2011)
- Toyota Production System principles (Kaizen, Muda elimination)
- Continuous integration best practices (Fowler, Google)
- Software economics (Boehm, McConnell)
- Safety-critical systems standards (DO-178C, IEC 61508)

## Acknowledgments

- **trueno project** (https://github.com/paiml/trueno) - Reference implementation
- **PMAT** (https://github.com/paiml/paiml-mcp-agent-toolkit) - Test orchestration toolkit
- Pragmatic AI Labs Research Division
