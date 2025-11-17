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

```rust
use certeza::{add, multiply};

fn main() {
    let sum = add(2, 3);
    println!("2 + 3 = {}", sum);

    let product = multiply(4, 5);
    println!("4 * 5 = {}", product);
}
```

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

### Unit Tests

9 unit tests covering basic functionality:

```bash
cargo test --lib
```

### Property-Based Tests

10 property-based tests using proptest, verifying mathematical properties:

- Commutativity: `add(a, b) == add(b, a)`
- Associativity: `add(add(a, b), c) == add(a, add(b, c))`
- Identity: `add(a, 0) == a`
- Distributivity: `a * (b + c) == (a * b) + (a * c)`

```bash
cargo test property_
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
