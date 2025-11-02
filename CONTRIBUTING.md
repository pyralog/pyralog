# Contributing to Pyralog

Thank you for your interest in contributing to Pyralog! This document provides guidelines and instructions for contributing.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Setup](#development-setup)
4. [Making Changes](#making-changes)
5. [Testing](#testing)
6. [Code Style](#code-style)
7. [Submitting Changes](#submitting-changes)
8. [Review Process](#review-process)

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of:
- Experience level
- Gender identity and expression
- Sexual orientation
- Disability
- Personal appearance
- Body size
- Race
- Ethnicity
- Age
- Religion
- Nationality

### Expected Behavior

- Be respectful and inclusive
- Accept constructive criticism gracefully
- Focus on what's best for the community
- Show empathy towards other community members

### Unacceptable Behavior

- Harassment of any kind
- Trolling or insulting comments
- Publishing others' private information
- Other conduct that would be inappropriate in a professional setting

## Getting Started

### Areas for Contribution

We welcome contributions in:

1. **Code**: Features, bug fixes, optimizations
2. **Documentation**: Guides, examples, API docs
3. **Testing**: Unit tests, integration tests, benchmarks
4. **Bug Reports**: Detailed issue reports
5. **Feature Requests**: Well-thought-out proposals
6. **Code Reviews**: Reviewing pull requests

### Good First Issues

Look for issues labeled:
- `good-first-issue`: Good for newcomers
- `help-wanted`: We need help on these
- `documentation`: Documentation improvements
- `bug`: Bug fixes needed

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Git
- Linux, macOS, or Windows
- 8GB RAM recommended
- SSD for fast compilation

### Clone and Build

```bash
# Fork the repository on GitHub first

# Clone your fork
git clone https://github.com/YOUR_USERNAME/dlog.git
cd dlog

# Add upstream remote
git remote add upstream https://github.com/original/dlog.git

# Build the project
cargo build

# Run tests
cargo test

# Run with optimizations
cargo build --release
```

### IDE Setup

#### Visual Studio Code

Install extensions:
- rust-analyzer
- CodeLLDB (for debugging)
- Even Better TOML
- Error Lens

#### IntelliJ IDEA / CLion

Install:
- Rust plugin
- TOML plugin

### Development Dependencies

```bash
# Install clippy (linter)
rustup component add clippy

# Install rustfmt (formatter)
rustup component add rustfmt

# Install cargo-watch (auto-rebuild)
cargo install cargo-watch

# Install cargo-edit (dependency management)
cargo install cargo-edit
```

## Making Changes

### Workflow

1. **Create a branch**
   ```bash
   git checkout -b feature/my-feature
   # or
   git checkout -b fix/bug-description
   ```

2. **Make your changes**
   - Write clean, readable code
   - Follow Rust best practices
   - Add tests for new functionality
   - Update documentation

3. **Commit your changes**
   ```bash
   git add .
   git commit -m "feat: add awesome feature"
   ```

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(storage): add compression support for segments

fix(consensus): prevent split-brain during network partition

docs(readme): update installation instructions

perf(storage): optimize index lookup with binary search
```

### Branch Naming

- `feature/feature-name`: New features
- `fix/bug-description`: Bug fixes
- `docs/topic`: Documentation
- `refactor/component`: Refactoring
- `perf/optimization`: Performance improvements

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests for a specific module
cargo test --package dlog-storage

# Run with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored
```

### Writing Tests

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        let result = my_function();
        assert_eq!(result, expected_value);
    }
}
```

#### Integration Tests

Create files in `tests/` directory:

```rust
// tests/integration_test.rs
use dlog::prelude::*;

#[tokio::test]
async fn test_end_to_end() {
    // Test complete workflow
}
```

#### Benchmarks

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            my_function(black_box(input));
        });
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## Code Style

### Formatting

```bash
# Format all code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting

```bash
# Run clippy
cargo clippy

# Run clippy with all features
cargo clippy --all-features

# Fail on warnings
cargo clippy -- -D warnings
```

### Code Guidelines

1. **Naming**
   - Use `snake_case` for functions and variables
   - Use `PascalCase` for types and traits
   - Use `SCREAMING_SNAKE_CASE` for constants

2. **Documentation**
   - Document all public APIs
   - Include examples in doc comments
   - Use `//!` for module-level docs
   - Use `///` for item-level docs

3. **Error Handling**
   - Use `Result<T>` for fallible operations
   - Create specific error types
   - Provide context in error messages

4. **Performance**
   - Avoid unnecessary allocations
   - Use zero-copy when possible
   - Profile before optimizing

5. **Safety**
   - Minimize `unsafe` code
   - Document why `unsafe` is needed
   - Provide safety invariants

### Example: Well-Documented Code

```rust
/// Appends a record to the log.
///
/// This operation is asynchronous and returns the offset where
/// the record was written. The record will be assigned the current
/// epoch of the partition's sequencer.
///
/// # Arguments
///
/// * `record` - The record to append
///
/// # Returns
///
/// Returns the offset where the record was written.
///
/// # Errors
///
/// Returns an error if:
/// - The partition is not accepting writes (sealed epoch)
/// - The storage is full
/// - There's an I/O error
///
/// # Example
///
/// ```
/// use dlog::prelude::*;
/// use bytes::Bytes;
///
/// # async fn example() -> Result<()> {
/// let storage = LogStorage::create(path, config).await?;
/// let record = Record::new(None, Bytes::from("data"));
/// let offset = storage.append(record).await?;
/// println!("Written at offset: {}", offset);
/// # Ok(())
/// # }
/// ```
pub async fn append(&self, record: Record) -> Result<LogOffset> {
    // Implementation
}
```

## Submitting Changes

### Before Submitting

Checklist:
- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Added tests for new functionality
- [ ] Updated documentation
- [ ] Ran `cargo fmt`
- [ ] Ran `cargo clippy`
- [ ] Updated CHANGELOG.md (if applicable)

### Pull Request Process

1. **Push your branch**
   ```bash
   git push origin feature/my-feature
   ```

2. **Create Pull Request**
   - Go to GitHub and create a PR
   - Fill out the PR template
   - Link related issues

3. **PR Title Format**
   ```
   feat(storage): add segment compression
   ```

4. **PR Description Template**
   ```markdown
   ## Description
   Brief description of changes

   ## Motivation
   Why is this change needed?

   ## Changes
   - Change 1
   - Change 2

   ## Testing
   How was this tested?

   ## Checklist
   - [ ] Tests added
   - [ ] Documentation updated
   - [ ] Follows code style
   ```

### CI Checks

Your PR will be checked for:
- ✅ Compilation
- ✅ Tests passing
- ✅ Linting (clippy)
- ✅ Formatting (rustfmt)
- ✅ Documentation builds

## Review Process

### What to Expect

1. **Initial Review**: Within 2-3 days
2. **Feedback**: Constructive comments and suggestions
3. **Iterations**: You may need to make changes
4. **Approval**: At least one maintainer approval required
5. **Merge**: Squash and merge or rebase

### Responding to Feedback

- Be open to suggestions
- Ask questions if unclear
- Make requested changes promptly
- Update the PR description if scope changes

### Review Criteria

Code is evaluated on:
- **Correctness**: Does it work as intended?
- **Performance**: Any performance implications?
- **Safety**: Is it memory-safe and thread-safe?
- **Maintainability**: Is it readable and maintainable?
- **Tests**: Are there adequate tests?
- **Documentation**: Is it well-documented?

## Advanced Topics

### Working on Large Features

For large features:
1. Open an issue first to discuss
2. Break into smaller PRs
3. Create a tracking issue
4. Submit incremental PRs

### Performance Optimization

Before optimizing:
1. Profile first
2. Identify bottlenecks
3. Benchmark before and after
4. Document why optimization is needed

```bash
# Profiling with cargo-flamegraph
cargo install flamegraph
cargo flamegraph --bin dlog

# Benchmarking
cargo bench
```

### Unsafe Code

If you need `unsafe`:
1. Explain why in comments
2. Document safety invariants
3. Keep unsafe blocks minimal
4. Add extra tests

```rust
/// Safety: This is safe because...
unsafe {
    // Minimal unsafe code
}
```

### Adding Dependencies

Before adding a dependency:
1. Check if it's necessary
2. Verify it's maintained
3. Check license compatibility
4. Consider binary size impact

```bash
# Add dependency
cargo add dependency-name

# Add dev dependency
cargo add --dev test-dependency
```

## Getting Help

### Resources

- **Documentation**: https://docs.dlog.io
- **Discord**: https://discord.gg/dlog
- **GitHub Issues**: For bug reports and features
- **GitHub Discussions**: For questions and ideas

### Questions

Don't hesitate to ask:
- Open a GitHub Discussion
- Ask on Discord
- Comment on issues/PRs
- Email: dev@dlog.io

## Recognition

Contributors are recognized:
- In CHANGELOG.md
- In release notes
- On our website
- GitHub contributors page

## License

By contributing to Pyralog, you agree that your contributions will be licensed under:
- **Code contributions**: MIT-0 (MIT No Attribution)
- **Documentation contributions**: CC0-1.0 (Public Domain)

---

Thank you for contributing to Pyralog! Your efforts make this project better for everyone. 🚀

