# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**certeza** is a scientific experiment into realistic provability with Rust. This is a research project developing a comprehensive framework for approaching asymptotic test effectiveness in Rust software systems.

### Core Concept
The project explores achieving practical maximum confidence in software testing through tiered verification approaches, acknowledging that complete verification is theoretically impossible (Dijkstra's observation: "testing can only prove the presence of bugs, not their absence").

### Reference Implementation
The framework targets vector-based data structures using the **trueno project** (https://github.com/paiml/trueno) as a reference implementation, with the **paiml-mcp-agent-toolkit (PMAT)** (https://github.com/paiml/paiml-mcp-agent-toolkit) for test orchestration.

## Current Project State

**Status**: Active development with PMAT compliance
- Rust library project scaffolded with full testing framework
- Contains comprehensive testing framework specification (~14K words)
- PMAT-compliant configuration and quality gates implemented
- Example functions with unit tests and property-based tests

## Testing Philosophy: Tiered TDD-X Framework

This project implements a three-tiered testing approach that balances rigor with developer productivity:

### Tier 1: ON-SAVE (Sub-second feedback)
- Unit tests and focused property tests
- Static analysis (`cargo check`, `cargo clippy`)
- Enables rapid iteration in flow state

### Tier 2: ON-COMMIT (1-5 minutes)
- Full property-based test suite with proptest
- Coverage analysis (target: 95%+ line coverage)
- Integration tests
- Pre-commit hook enforcement

### Tier 3: ON-MERGE/NIGHTLY (Hours)
- Comprehensive mutation testing with cargo-mutants (target: >85% mutation score)
- Formal verification for critical paths (using Kani)
- Performance benchmarks
- CI/CD gate for main branch

**Critical Principle**: Different verification techniques operate at different time scales. Fast feedback enables flow; slow feedback causes context switching waste. Never run mutation testing or formal verification in the inner development loop.

## Testing Pyramid Distribution

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

## Risk-Based Verification Strategy

Not all code requires the same verification intensity. Apply rigorous techniques based on risk:

| Risk Level | Components | Verification Approach |
|------------|------------|----------------------|
| **Very High** | `unsafe` blocks, memory allocators, crypto, concurrency primitives | Full framework: Property + Coverage + Mutation (90%) + Formal |
| **High** | Core algorithms, data structure internals, parsers | Property + Coverage + Mutation (85-90%) |
| **Medium** | Business logic, API handlers, utilities | Property + Coverage + Mutation (80%) |
| **Low** | Simple accessors, config, CLI parsing | Unit tests + Coverage (90%) |

**Resource Allocation**: Spend 40% of verification time on the 5-10% highest-risk code.

## Expected Cargo Commands

When Rust code is implemented, the project will use:

### Development
- `cargo check` - Type checking (Tier 1, sub-second)
- `cargo clippy` - Linting (Tier 1, sub-second)
- `cargo test` - Run unit tests (Tier 1, sub-second for focused tests)
- `cargo test --all` - Run full test suite (Tier 2, 1-5 min)

### Coverage Analysis (Tier 2)
- `cargo tarpaulin` or `cargo llvm-cov` - Generate coverage reports
- Target: 95%+ line coverage

### Mutation Testing (Tier 3)
- `cargo mutants` - Run mutation testing
- Target: >85% mutation score
- Analyze surviving mutants for test gaps

### Formal Verification (Tier 3)
- `cargo kani` - Formal verification for critical invariants
- Applied selectively to highest-risk code paths

### Property-Based Testing
Uses **proptest** crate for property-based testing (see specification for detailed examples)

## PMAT Compliance

This project is fully compliant with the **Pragmatic AI Labs Multi-Language Agent Toolkit (PMAT)** standards.

### Makefile Targets (PMAT-Aligned)

The project uses a comprehensive Makefile for all quality operations:

**Tiered Workflow:**
- `make tier1` - Tier 1: ON-SAVE checks (sub-second)
- `make tier2` - Tier 2: ON-COMMIT checks (1-5 minutes)
- `make tier3` - Tier 3: ON-MERGE/NIGHTLY checks (hours)

**Quality Gates:**
- `make quality-gate` - Run all PMAT quality gates
- `make quality-gate-tier2` - Tier 2 quality gates (default for commits)
- `make quality-gate-tier3` - Tier 3 quality gates (pre-merge)

**Testing:**
- `make test` - Run all tests
- `make test-quick` - Run unit tests only (fast)
- `make test-property` - Run property-based tests
- `make coverage` - Generate coverage report (target: 85%+)
- `make mutation` - Run mutation testing (target: 85%+ score)

**Code Quality:**
- `make clippy` - Run clippy linter
- `make clippy-strict` - Run clippy with pedantic/nursery lints
- `make fmt` - Format code
- `make fmt-check` - Check formatting

**Analysis:**
- `make complexity` - Analyze code complexity with PMAT
- `make tdg` - Technical debt grading
- `make security` - Security audit (cargo-audit + cargo-deny)
- `make repo-score` - Calculate repository health score

**Documentation:**
- `make docs` - Generate documentation
- `make validate-docs` - Validate documentation with PMAT

**Setup:**
- `make install-tools` - Install all required tooling
- `make install-hooks` - Install PMAT git hooks

### PMAT Configuration Files

The project includes three PMAT configuration files:

1. **pmat.toml** - Main PMAT configuration
   - Complexity limits: max_cyclomatic=10, max_cognitive=10
   - Coverage requirements: min_coverage=85%
   - SATD: zero tolerance (max_satd=0)
   - Mutation testing: min_mutation_score=85%
   - Documentation: min_rustdoc_coverage=90%

2. **.pmat-gates.toml** - Quality gate enforcement
   - Clippy strict mode enabled
   - Rustfmt checking enabled
   - Coverage threshold: 85%
   - Complexity checking enabled
   - Security audits (cargo-audit, cargo-deny)
   - SATD checking with zero tolerance

3. **pmat-quality.toml** - Detailed quality thresholds
   - Tiered testing configuration aligned with certeza spec
   - Component-level grading thresholds
   - Risk-based verification settings

### PMAT Quality Standards (EXTREME TDD)

The project enforces **EXTREME TDD** standards:

**Coverage Requirements:**
- Line coverage: ≥85% (minimum), 95% (target)
- Branch coverage: ≥80% (minimum), 90% (target)
- Function coverage: ≥90%

**Complexity Limits:**
- Cyclomatic complexity: ≤10 per function
- Cognitive complexity: ≤10 per function
- Nesting depth: ≤5
- Lines per function: ≤50

**Testing Requirements:**
- Minimum 20 unit tests
- Minimum 10 integration tests
- Minimum 5 property-based tests
- Proptest iterations: 256-10,000

**SATD (Self-Admitted Technical Debt):**
- Zero tolerance for TODO, FIXME, HACK comments
- All technical debt must link to GitHub issues
- Fail build on unlinked SATD

**Security:**
- cargo-audit: deny vulnerabilities
- cargo-deny: deny unmaintained/deprecated dependencies
- Unsafe code: max_unsafe_blocks=0 (forbid unsafe)

**Documentation:**
- ≥90% public items documented
- All public functions require examples
- Module and crate documentation required
- Safety documentation for any unsafe code (≥3 lines)

### CI/CD Integration

GitHub Actions workflow (`.github/workflows/ci.yml`) enforces quality gates:

- **Tier 1**: Quick checks on every push (check, clippy, unit tests)
- **Tier 2**: Full test suite + coverage on PR (all tests, coverage ≥85%)
- **Security**: Parallel security audit (cargo-audit, cargo-deny)
- **Tier 3**: Mutation testing on merge to main (≥85% mutation score)

### PMAT Commands

Use PMAT directly for advanced analysis:

```bash
# Generate AI-ready context
pmat context --output context.md --format llm-optimized

# Analyze technical debt
pmat analyze tdg --include-components

# Check complexity
pmat analyze complexity --path src/

# Repository health score (0-110 scale)
pmat repo-score .
pmat repo-score . --deep  # Include git history

# Run mutation testing
pmat mutate --target src/ --threshold 85

# Validate documentation accuracy
pmat validate-readme --targets README.md

# Install pre-commit hooks
pmat hooks install
pmat hooks status

# Run quality gates
pmat quality-gates
```

### Project Scoring (Rust Project Score)

PMAT evaluates the project across 6 dimensions (total: 100 points):

1. **Rust Tooling Compliance** (25 points): Clippy, rustfmt, cargo-deny, cargo-audit
2. **Code Quality** (20 points): Complexity, unsafe code, dead code, SATD
3. **Testing Excellence** (20 points): Unit tests, integration tests, property tests, mutation tests
4. **Documentation** (15 points): Rustdoc coverage, examples, architecture docs
5. **Performance & Security** (10 points): Benchmarks, security analysis
6. **Community & DevOps** (10 points): CI/CD, release process

**Target Grade**: A (90-94) or A+ (95-100)

## Architecture Insights

### Testing Framework Components

1. **Structural Coverage**: Instrumentation-based measurement of code execution
2. **Property-Based Testing**: Specification verification using proptest strategies
3. **Mutation Testing**: Test suite quality assessment (detect test gaps)
4. **Selective Formal Verification**: Mathematical proofs for critical invariants

### Key Design Principles

- **Sustainable Workflows**: Tiered feedback loops prevent burnout and maintain flow state
- **Risk-Based Resource Allocation**: Focus expensive verification on high-risk components
- **Human-Centered Analysis**: Mutation analysis as learning exercise, not just metrics
- **Economic Realism**: Acknowledge costs and diminishing returns of verification techniques

### Theoretical Bounds

The specification acknowledges fundamental limits:
- Coverage ceiling: 100% coverage doesn't guarantee correctness
- Mutation score asymptote: Typically plateaus at 80-95% (equivalent mutants are undecidable)
- Property space incompleteness: Infinite meaningful properties, finite testing
- Formal verification tractability: State explosion limits verification scope

## Documentation Structure

- `docs/specifications/theoretical-max-testing-spec.md` - Main framework specification (v1.1, ~14K words)
- `docs/specifications/IMPROVEMENTS_v1.1.md` - Changelog showing philosophy shift from "theoretical maximum" to "asymptotic effectiveness"

## Development Anti-Patterns

Based on the specification's emphasis on sustainable practices:

1. **Never** run mutation testing on every file save (destroys flow, 10-100x productivity loss)
2. **Never** chase metrics without understanding (Goodhart's Law warning)
3. **Never** apply full verification framework to low-risk code (over-processing waste)
4. **Never** ignore cognitive load limits (use batching, time-boxing, pairing for mutation analysis)

## Quality Standards

When implementing code:
- Strong type safety leveraging Rust's ownership model
- Memory safety violations prevented by language (focus testing on algorithmic correctness)
- Scientific rigor with empirical validation of testing approaches
- Comprehensive documentation with academic-style citations
