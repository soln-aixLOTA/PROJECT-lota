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

3. **Commit Signing**
   - All commits must be cryptographically signed
   - Supports GPG, SSH, and S/MIME signing
   - Verifies commit authenticity

### Common Commands

```bash
# Normal commit (runs all checks)
git commit -S -m "feat(auth): add JWT refresh"

# Skip hooks (use sparingly)
git commit --no-verify -m "feat(auth): add JWT refresh"

# Update hooks permissions
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/commit-msg

# Configure GPG signing (recommended)
git config --global commit.gpgsign true
git config --global user.signingkey YOUR_KEY_ID

# Configure SSH signing (alternative)
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
```

For more details on:
- Available types and scopes, see [git-hooks.md](git-hooks.md)
- Commit signing setup, see [git-hooks.md#commit-signature-verification](git-hooks.md#commit-signature-verification)
- Troubleshooting, see [git-hooks.md#troubleshooting](git-hooks.md#troubleshooting)
