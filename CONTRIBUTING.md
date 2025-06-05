# Contributing to Editor

Thank you for your interest in contributing to this terminal text editor! We welcome contributions from everyone, whether you're a seasoned Rust developer or just getting started.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70.0 or later
- Git
- A terminal emulator with true color support (recommended)

### Setting up the Development Environment

1. **Fork and clone the repository:**

   ```bash
   git clone https://github.com/your-username/editor.git
   cd editor
   ```

2. **Install dependencies:**

   ```bash
   cargo check
   ```

3. **Run the tests:**

   ```bash
   cargo test
   ```

4. **Run the editor:**
   ```bash
   cargo run -- README.md
   ```

## 🛠️ Development Workflow

### Code Style

We use standard Rust formatting and linting tools:

- **Format code:** `cargo fmt`
- **Check lints:** `cargo clippy`
- **Run tests:** `cargo test`

Please ensure your code passes all checks before submitting a PR.

### Commit Messages

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat: add new feature`
- `fix: resolve bug`
- `docs: update documentation`
- `style: format code`
- `refactor: restructure code`
- `test: add or update tests`
- `chore: maintenance tasks`

### Branch Naming

Use descriptive branch names:

- `feature/command-palette`
- `fix/cursor-positioning`
- `docs/api-documentation`

## 📝 Types of Contributions

### 🐛 Bug Reports

When reporting bugs, please include:

- Operating system and terminal emulator
- Rust version (`rustc --version`)
- Steps to reproduce the issue
- Expected vs. actual behavior
- Any error messages or logs

Use our [bug report template](.github/ISSUE_TEMPLATE/bug_report.md).

### 💡 Feature Requests

For new features, please:

- Check existing issues to avoid duplicates
- Describe the problem you're solving
- Explain your proposed solution
- Consider backwards compatibility

Use our [feature request template](.github/ISSUE_TEMPLATE/feature_request.md).

### 🔧 Code Contributions

1. **Find an issue to work on** (or create one)
2. **Comment** that you'd like to work on it
3. **Fork** the repository
4. **Create a branch** for your changes
5. **Make your changes** following our guidelines
6. **Write tests** for new functionality
7. **Update documentation** if needed
8. **Submit a pull request**

### 📚 Documentation

Documentation improvements are always welcome:

- Fix typos or unclear explanations
- Add examples and use cases
- Improve API documentation
- Update the README or guides

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for specific module
cargo test buffer::tests
```

### Writing Tests

- Add unit tests for new functions
- Add integration tests for complex features
- Include edge cases and error conditions
- Use descriptive test names

### Test Coverage

We aim for high test coverage. You can check coverage with:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## 🏗️ Architecture

### Code Organization

```
src/
├── main.rs          # Application entry point
├── app.rs           # Core application state
├── ui.rs            # UI rendering
├── buffer/          # Text buffer management
├── config/          # Configuration system
├── events.rs        # Event system
├── handlers/        # Event handlers
├── input/           # Input processing
├── plugins/         # Plugin system
├── widgets/         # UI widgets
└── performance.rs   # Performance monitoring
```

### Design Principles

- **Performance**: Maintain 60fps and low memory usage
- **Modularity**: Clear separation of concerns
- **Extensibility**: Plugin-friendly architecture
- **User Experience**: Vim-like interface with modern features
- **Cross-platform**: Work consistently across terminals

## 🔌 Plugin Development

The editor supports a powerful plugin system:

- Plugins are written in Rust
- Use the plugin API for editor integration
- Follow the plugin development guide
- Submit plugins to the plugin registry

See [PLUGIN_DEVELOPMENT.md](docs/PLUGIN_DEVELOPMENT.md) for details.

## 📋 Pull Request Guidelines

### Before Submitting

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (for notable changes)

### PR Description

Include in your PR description:

- **What** changes you made
- **Why** you made them
- **How** to test the changes
- **Screenshots** for UI changes
- **Breaking changes** if any

### Review Process

1. **Automated checks** must pass
2. **Code review** by maintainers
3. **Testing** on different platforms
4. **Merge** after approval

## 🎯 Areas for Contribution

We especially welcome contributions in these areas:

### High Priority

- [ ] Language server protocol (LSP) integration
- [ ] Syntax highlighting improvements
- [ ] Performance optimizations
- [ ] Test coverage improvements

### Medium Priority

- [ ] Additional themes and color schemes
- [ ] Plugin system enhancements
- [ ] Documentation improvements
- [ ] Accessibility features

### Good First Issues

- [ ] Fix typos and documentation
- [ ] Add more unit tests
- [ ] Improve error messages
- [ ] Small UI improvements

Look for issues labeled `good first issue`, `help wanted`, or `documentation`.

## 💬 Communication

- **Discussions:** Use GitHub Discussions for questions and ideas
- **Issues:** Use GitHub Issues for bugs and feature requests
- **Discord:** Join our Discord server for real-time chat
- **Email:** Contact maintainers at editor@example.com

## 📄 License

By contributing to this project, you agree that your contributions will be licensed under the MIT License.

## 🙏 Recognition

Contributors are recognized in:

- CHANGELOG.md for notable contributions
- README.md contributors section
- GitHub contributors graph
- Annual contributor appreciation posts

Thank you for helping make this editor better for everyone! 🎉
