# Contributing to Leeca Proxmox VE SDK

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/) specification with [gitmoji](https://gitmoji.dev/).

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters
- Reference issues and pull requests liberally after the first line

### Types

- ✨ feat: A new feature
- 🐛 fix: A bug fix
- 📚 docs: Documentation only changes
- 💄 style: Changes that do not affect the meaning of the code
- ♻️ refactor: A code change that neither fixes a bug nor adds a feature
- ⚡️ perf: A code change that improves performance
- 🧺 test: Adding missing tests or correcting existing tests
- 🔧 chore: Changes to the build process or auxiliary tools

### Examples

```bash
git commit -m "✨ feat(auth): add login functionality"
git commit -m "🐛 fix(validation): handle empty host names"
git commit -m "📚 docs(readme): update installation instructions"
```

## Development Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and lints

   ```bash
   cargo test
   cargo clippy
   cargo fmt --all -- --check
   ```

5. Commit your changes following our convention
6. Push to your fork
7. Open a Pull Request

## Pull Request Process

1. Update documentation if needed
2. Update `CHANGELOG.md` following semantic versioning
3. Ensure CI passes
4. Get review from maintainers

## Development Setup

```bash
# Clone your fork
git clone https://github.com/<your-username>/leeca-proxmox.git

# Add upstream remote
git remote add upstream https://github.com/original/leeca-proxmox.git

# Install development dependencies
cargo install cargo-llvm-cov cargo-audit

# Install project dependencies
cargo build
```

## Testing Standards

- Write tests for all new features
- Maintain or improve code coverage
- Run the full test suite before submitting

To run tests, you need to set up the `.env` file with the values from the [example](.env.example) targeting to a real testing instance of Proxmox VE.
