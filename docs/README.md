# LotaBots Documentation

## Git Hooks

LotaBots uses Git hooks to maintain code quality and consistency. For detailed information, see [git-hooks.md](git-hooks.md).

### Quick Start

The repository includes two main hooks:

1. **Pre-commit Hook**
   - Formats and lints Rust code
   - Checks for security issues
   - Validates configurations
   - Runs tests
   - Ensures documentation

2. **Commit Message Hook**
   - Enforces conventional commits format
   - Supports LotaBots-specific types
   - Validates commit message structure

### Common Commands

```bash
# Normal commit (runs all checks)
git commit -m "feat(auth): add JWT refresh"

# Skip hooks (use sparingly)
git commit --no-verify -m "feat(auth): add JWT refresh"

# Update hooks permissions
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/commit-msg
```

For more details on available types, scopes, and troubleshooting, refer to [git-hooks.md](git-hooks.md).
