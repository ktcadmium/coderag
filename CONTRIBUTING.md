# Contributing to CodeRAG

Thank you for your interest in contributing to CodeRAG! This document provides guidelines for contributors.

## Development Setup

1. **Prerequisites**
   - Rust 1.70 or later
   - Git
   - 4GB RAM minimum

2. **Clone and Build**
   ```bash
   git clone https://github.com/yourusername/coderag.git
   cd coderag
   cargo build
   cargo test
   ```

3. **Development Tools**
   - Use `cargo fmt` before committing
   - Run `cargo clippy` to check for common issues
   - Ensure all tests pass with `cargo test`

## Project Structure

- `src/` - Source code
- `tests/` - Integration tests
- `memory-bank/` - Project documentation and context
- `CLAUDE.md` - Developer guide for AI assistants

## Making Changes

1. **Read the Memory Bank**: Start by understanding the project context in `memory-bank/`
2. **Create a Branch**: Use descriptive names like `feature/web-crawler` or `fix/search-performance`
3. **Write Tests**: Add tests for new functionality
4. **Update Documentation**: Keep the memory bank current with significant changes
5. **Submit PR**: Include a clear description of changes

## Code Style

- Follow Rust conventions
- Use meaningful variable names
- Add comments for complex logic
- Keep functions focused and small

## Testing

- Unit tests go next to the code they test
- Integration tests go in `tests/`
- Run tests with `cargo test`
- Add tests for bug fixes to prevent regression

## Performance

CodeRAG has strict performance requirements:
- Embedding generation: <5ms
- Search: <10ms for 10k documents
- Startup: <2s

Please ensure your changes don't degrade performance.

## Areas for Contribution

Current priorities:
1. **Web Crawler** (Phase 4) - Help implement documentation crawling
2. **Web UI** (Phase 5) - Create management interface
3. **Documentation** - Improve user guides and examples
4. **Testing** - Increase test coverage
5. **Performance** - Optimize search and embedding generation

## Questions?

- Open an issue for bugs or feature requests
- Join discussions in existing issues
- Check the memory bank for project context

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
