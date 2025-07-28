# Development Workflows

## Overview

This guide covers common development workflows and best practices for using Helix in different scenarios. These workflows are designed to maximize productivity while maintaining code quality and security.

## Basic Development Workflow

### Daily Development Cycle

```bash
# 1. Start your day
hx status                    # Check current state
hx pull origin main          # Get latest changes

# 2. Create feature branch
hx checkout -b feature/new-feature

# 3. Make changes and commit frequently
hx add .
hx commit -m "Implement user authentication"

# 4. Push your branch
hx push origin feature/new-feature

# 5. Create pull request (via web interface)
# 6. After review, merge to main
hx checkout main
hx pull origin main
hx branch -d feature/new-feature  # Clean up
```

### Commit Frequency

Make small, focused commits:

```bash
# Good: Small, focused commits
hx add auth/models.rs
hx commit -m "Add User model with validation"

hx add auth/controllers.rs
hx commit -m "Add authentication controller"

hx add auth/routes.rs
hx commit -m "Add authentication routes"

# Bad: Large, unfocused commits
hx add .
hx commit -m "Add authentication system"
```

## Feature Development Workflow

### 1. Feature Branch Workflow

```bash
# Start from main
hx checkout main
hx pull origin main

# Create feature branch
hx checkout -b feature/user-dashboard

# Develop feature
# ... make changes ...

# Commit frequently
hx add .
hx commit -m "Add dashboard layout"

# Push branch
hx push origin feature/user-dashboard

# Continue development
# ... more changes ...
hx add .
hx commit -m "Add dashboard widgets"

# Push updates
hx push origin feature/user-dashboard
```

### 2. Feature Completion

```bash
# Ensure all tests pass
# ... run tests ...

# Final commit
hx add .
hx commit -m "Complete user dashboard feature"

# Push final version
hx push origin feature/user-dashboard

# Create pull request for review
# After approval, merge via web interface
```

### 3. Cleanup

```bash
# Switch back to main
hx checkout main
hx pull origin main

# Delete feature branch
hx branch -d feature/user-dashboard

# Delete remote branch
hx push origin --delete feature/user-dashboard
```

## Bug Fix Workflow

### 1. Hotfix Workflow

```bash
# Create hotfix branch from main
hx checkout main
hx checkout -b hotfix/critical-bug

# Fix the bug
# ... make changes ...

# Commit fix
hx add .
hx commit -m "Fix critical authentication bug"

# Push hotfix
hx push origin hotfix/critical-bug

# Merge to main
hx checkout main
hx merge hotfix/critical-bug
hx push origin main

# Cleanup
hx branch -d hotfix/critical-bug
hx push origin --delete hotfix/critical-bug
```

### 2. Bug Fix in Feature Branch

```bash
# If bug is found in feature branch
hx checkout feature/user-dashboard

# Fix the bug
# ... make changes ...

# Commit fix
hx add .
hx commit -m "Fix validation error in user form"

# Continue with feature development
```

## Release Workflow

### 1. Release Preparation

```bash
# Ensure main is stable
hx checkout main
hx pull origin main

# Create release branch
hx checkout -b release/v1.2.0

# Update version numbers
# ... update version files ...

# Commit version bump
hx add .
hx commit -m "Bump version to v1.2.0"

# Push release branch
hx push origin release/v1.2.0
```

### 2. Release Testing

```bash
# Test the release
# ... run tests, manual testing ...

# Fix any issues found
hx add .
hx commit -m "Fix release issues"

# Push fixes
hx push origin release/v1.2.0
```

### 3. Release Completion

```bash
# Merge to main
hx checkout main
hx merge release/v1.2.0
hx push origin main

# Create tag
hx tag v1.2.0
hx push origin v1.2.0

# Cleanup
hx branch -d release/v1.2.0
hx push origin --delete release/v1.2.0
```

## Collaborative Workflow

### 1. Team Development

```bash
# Start with latest main
hx checkout main
hx pull origin main

# Create your feature branch
hx checkout -b feature/team-feature

# Develop and commit
# ... development work ...

# Push your work
hx push origin feature/team-feature

# Keep branch updated with main
hx checkout main
hx pull origin main
hx checkout feature/team-feature
hx merge main
hx push origin feature/team-feature
```

### 2. Code Review Process

```bash
# After pushing feature branch
# Create pull request via web interface

# Address review comments
# ... make requested changes ...

# Update branch
hx add .
hx commit -m "Address review comments"
hx push origin feature/team-feature

# After approval, merge
hx checkout main
hx pull origin main
hx branch -d feature/team-feature
```

### 3. Conflict Resolution

```bash
# When conflicts occur during merge
hx merge feature/conflicting-branch

# Resolve conflicts in files
# ... edit conflicted files ...

# Stage resolved files
hx add .

# Complete merge
hx commit -m "Resolve merge conflicts"

# Push resolved merge
hx push origin main
```

## Maintenance Workflow

### 1. Repository Maintenance

```bash
# Check repository health
hx verify-history

# Clean up old branches
hx branch -d old-feature-branch

# Update remotes
hx remote update

# Garbage collection (if implemented)
# hx gc
```

### 2. Key Rotation

```bash
# Generate new key
hx keygen --new

# Update repository to use new key
hx config signing.key ~/.helix/keys/ed25519-new.key

# Test with new key
hx add .
hx commit -m "Test new signing key"

# Backup old key
hx key export ~/backup/old-key.key
```

## Advanced Workflows

### 1. Rebase Workflow

```bash
# Keep feature branch updated with main
hx checkout feature/feature-branch
hx rebase main

# If conflicts occur, resolve them
# ... resolve conflicts ...
hx add .
hx rebase --continue

# Force push (use with caution)
hx push origin feature/feature-branch --force
```

### 2. Cherry-pick Workflow

```bash
# Apply specific commit to current branch
hx cherry-pick <commit-hash>

# If conflicts occur, resolve them
# ... resolve conflicts ...
hx add .
hx cherry-pick --continue
```

### 3. Stash Workflow

```bash
# Save current work temporarily
hx stash push -m "WIP: authentication feature"

# Switch to different branch
hx checkout main

# Do urgent work
# ... urgent changes ...
hx add .
hx commit -m "Urgent fix"

# Return to feature branch
hx checkout feature/feature-branch

# Restore stashed work
hx stash pop
```

## Security Workflows

### 1. Secure Development

```bash
# Verify all commits in history
hx verify-history

# Check specific commit
hx verify-history <commit-hash>

# Verify before pushing
hx push origin main --verify
```

### 2. Key Management

```bash
# Regular key backup
hx key export ~/backup/keys/$(date +%Y%m%d)-helix-key.key

# Key rotation schedule
# - Generate new keys quarterly
# - Backup keys monthly
# - Test keys weekly
```

### 3. Audit Trail

```bash
# Generate audit report
hx log --audit > audit-report.txt

# Verify specific time period
hx verify-history --since="2024-01-01" --until="2024-01-31"
```

## Performance Workflows

### 1. Large Repository Management

```bash
# For large repositories, use sparse checkouts
hx config core.sparseCheckout true

# Add specific directories
echo "src/" > .helix/info/sparse-checkout
echo "docs/" >> .helix/info/sparse-checkout

# Update working directory
hx read-tree -m HEAD
```

### 2. Efficient Cloning

```bash
# Shallow clone for large repositories
hx clone --depth 1 <repository-url>

# Clone specific branch only
hx clone --branch main <repository-url>
```

## Troubleshooting Workflows

### 1. Recovery from Mistakes

```bash
# Undo last commit (keep changes)
hx reset --soft HEAD~1

# Undo last commit (discard changes)
hx reset --hard HEAD~1

# Recover deleted branch
hx reflog
hx checkout -b recovered-branch <commit-hash>
```

### 2. Repository Repair

```bash
# Verify repository integrity
hx verify-history

# Repair corrupted objects (if implemented)
# hx fsck --repair

# Rebuild index
hx reset --hard HEAD
```

## Best Practices Summary

### Do's
- ✅ Make small, focused commits
- ✅ Use descriptive branch names
- ✅ Write clear commit messages
- ✅ Verify commits regularly
- ✅ Keep branches updated
- ✅ Use feature branches for development
- ✅ Review code before merging

### Don'ts
- ❌ Commit directly to main
- ❌ Make large, unfocused commits
- ❌ Use vague commit messages
- ❌ Ignore merge conflicts
- ❌ Force push to shared branches
- ❌ Commit without testing
- ❌ Share private keys

### Security Checklist
- [ ] Verify commit signatures
- [ ] Backup keys regularly
- [ ] Rotate keys periodically
- [ ] Use strong passphrases
- [ ] Monitor for suspicious activity
- [ ] Keep software updated
- [ ] Audit access regularly

These workflows provide a solid foundation for productive and secure development with Helix. Adapt them to your team's specific needs and project requirements. 