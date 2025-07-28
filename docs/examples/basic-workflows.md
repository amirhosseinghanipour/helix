# Basic Workflow Examples

## Introduction

This document provides step-by-step examples of common Helix workflows. Each example is designed to be followed exactly as shown, with expected outputs and explanations.

## Example 1: Creating Your First Repository

### Scenario
You want to create a new project and track it with Helix.

### Step-by-Step Process

```bash
# 1. Create project directory
mkdir my-first-project
cd my-first-project

# 2. Initialize Helix repository
hx init

# Expected output:
# Helix repository initialized successfully!
# Repository location: /path/to/my-first-project
# Current branch: main
# 
# Next steps:
#   hx add .     # Stage all files
#   hx commit -m "Initial commit"  # Create first commit

# 3. Generate your signing key
hx keygen

# Expected output:
# Keypair generated and saved!
# Public key: a1b2c3d4e5f6...

# 4. Configure your identity
hx config user.name "Your Name"
hx config user.email "your.email@example.com"

# 5. Create some project files
echo "# My First Project" > README.md
echo "console.log('Hello, Helix!');" > main.js

# 6. Check status
hx status

# Expected output:
# Repository Status
# ========================================
# On branch: main
# HEAD: No commits yet
# 
# Untracked files:
#   README.md
#   main.js
# 
# Summary:
#   Staged: 0 files
#   Modified: 0 files
#   Untracked: 2 files

# 7. Stage files
hx add .

# Expected output:
# Files staged successfully!
# Added: 2 files
# Total staged: 2 files

# 8. Create first commit
hx commit -m "Initial project setup"

# Expected output:
# Commit created successfully!
# Commit ID: a1b2c3d4
# Message: Initial project setup
# Author: Your Name <your.email@example.com>
# Date: 2024-01-15 10:30:00
# Files: 2 files changed
# Branch: main

# 9. View commit history
hx log

# Expected output:
# HEAD -> a1b2c3d4 VALID Initial project setup
#     Parents: (root)
#     Author: Your Name <your.email@example.com>
#     Date: 2024-01-15 10:30:00
#     Files: 2 files changed
```

### What Happened
- Created a new Helix repository with cryptographic signing
- Generated an Ed25519 keypair for commit signing
- Configured your identity for commits
- Created and committed your first files
- Verified the commit was properly signed and stored

## Example 2: Working with Branches

### Scenario
You want to develop a new feature without affecting the main branch.

### Step-by-Step Process

```bash
# 1. Start from your existing repository
cd my-first-project

# 2. Check current status
hx status

# Expected output:
# Repository Status
# ========================================
# On branch: main
# HEAD: a1b2c3d4
# Working tree clean

# 3. Create a new feature branch
hx checkout -b feature/user-authentication

# Expected output:
# Switched to branch 'feature/user-authentication'
# Current branch: feature/user-authentication
# HEAD: a1b2c3d4

# 4. List all branches
hx branch

# Expected output:
# Branches
# ========================================
#   main
# * feature/user-authentication

# 5. Make changes for your feature
echo "// User authentication module" > auth.js
echo "function login(username, password) {" >> auth.js
echo "  // Implementation here" >> auth.js
echo "}" >> auth.js

# 6. Stage and commit your changes
hx add auth.js
hx commit -m "Add user authentication module"

# Expected output:
# Commit created successfully!
# Commit ID: b2c3d4e5
# Message: Add user authentication module
# Author: Your Name <your.email@example.com>
# Date: 2024-01-15 11:00:00
# Files: 1 files changed
# Branch: feature/user-authentication

# 7. Add more changes
echo "// Password validation" >> auth.js
hx add auth.js
hx commit -m "Add password validation"

# 8. View commit history
hx log

# Expected output:
# HEAD -> b2c3d4e5 VALID Add password validation
#     Parents: a1b2c3d4
#     Author: Your Name <your.email@example.com>
#     Date: 2024-01-15 11:15:00
#     Files: 1 files changed
# 
# a1b2c3d4 VALID Add user authentication module
#     Parents: (root)
#     Author: Your Name <your.email@example.com>
#     Date: 2024-01-15 11:00:00
#     Files: 1 files changed

# 9. Switch back to main branch
hx checkout main

# Expected output:
# Switched to branch 'main'
# Current branch: main
# HEAD: a1b2c3d4

# 10. Verify main branch is unchanged
hx log

# Expected output:
# HEAD -> a1b2c3d4 VALID Initial project setup
#     Parents: (root)
#     Author: Your Name <your.email@example.com>
#     Date: 2024-01-15 10:30:00
#     Files: 2 files changed
```

### What Happened
- Created a feature branch for isolated development
- Made changes without affecting the main branch
- Committed changes with proper signatures
- Demonstrated branch isolation

## Example 3: Merging Changes

### Scenario
You want to merge your feature branch back into main.

### Step-by-Step Process

```bash
# 1. Continue from previous example
cd my-first-project

# 2. Ensure you're on main branch
hx checkout main

# 3. Merge the feature branch (manual conflict resolution by default)
hx merge feature/user-authentication

# 3b. Merge the feature branch, always taking our branch's version in conflicts
hx merge feature/user-authentication --strategy ours

# 3c. Merge the feature branch, always taking the other branch's version in conflicts
hx merge feature/user-authentication --strategy theirs

# Expected output:
# Merging branch 'feature/user-authentication' into 'main' with strategy: manual
# Merge completed successfully
# Current branch: main

# 4. View the merge result
hx log

# Expected output:
# HEAD -> c3d4e5f6 VALID Merge branch 'feature/user-authentication'
#     Parents: a1b2c3d4, b2c3d4e5
#     Author: Your Name <your.email@example.com>
#     Date: 2024-01-15 12:00:00
#     Files: 1 files changed
# 
# b2c3d4e5 VALID Add password validation
#     Parents: a1b2c3d4
#     Author: Your Name <your.email@example.com>
#     Date: 2024-01-15 11:15:00
#     Files: 1 files changed
# 
# a1b2c3d4 VALID Add user authentication module
#     Parents: (root)
#     Author: Your Name <your.email@example.com>
#     Date: 2024-01-15 11:00:00
#     Files: 1 files changed

# 5. Verify the merge
hx status

# Expected output:
# Repository Status
# ========================================
# On branch: main
# HEAD: c3d4e5f6
# Working tree clean

# 6. List all files
ls -la

# Expected output:
# .helix/
# README.md
# main.js
# auth.js

# 7. Clean up the feature branch
hx branch -d feature/user-authentication

# Expected output:
# Deleted branch 'feature/user-authentication'
```

### What Happened
- Successfully merged feature branch into main
- Created a merge commit with multiple parents
- Verified all changes were properly integrated
- Cleaned up the feature branch

## Example 4: Working with Remotes

### Scenario
You want to share your repository with others via a remote.

### Step-by-Step Process

```bash
# 1. Continue from previous example
cd my-first-project

# 2. Add a remote repository
hx remote add origin https://github.com/your-username/my-first-project.git

# Expected output:
# Added remote 'origin' -> https://github.com/your-username/my-first-project.git

# 3. List remotes
hx remote

# Expected output:
# origin    https://github.com/your-username/my-first-project.git

# 4. Push your repository to remote
hx push origin main

# Expected output:
# Changes pushed successfully!
# Uploaded: 4 objects
# Remote: https://github.com/your-username/my-first-project.git

# 5. Verify push worked
hx status

# Expected output:
# Repository Status
# ========================================
# On branch: main
# HEAD: c3d4e5f6
# Working tree clean
# 
# Your branch is up to date with 'origin/main'

# 6. Make a local change
echo "// Updated documentation" >> README.md
hx add README.md
hx commit -m "Update documentation"

# 7. Push the new commit
hx push origin main

# Expected output:
# Changes pushed successfully!
# Uploaded: 1 objects
# Remote: https://github.com/your-username/my-first-project.git
```

### What Happened
- Added a remote repository for collaboration
- Pushed your local repository to the remote
- Made additional changes and pushed them
- Demonstrated remote synchronization

## Example 5: Handling Conflicts

### Scenario
You need to resolve merge conflicts when branches have conflicting changes.

### Step-by-Step Process

```bash
# 1. Start with a clean repository
cd my-first-project

# 2. Create a new feature branch
hx checkout -b feature/conflicting-changes

# 3. Modify a file
echo "// Feature branch changes" > main.js
hx add main.js
hx commit -m "Update main.js in feature branch"

# 4. Switch back to main and make conflicting changes
hx checkout main
echo "// Main branch changes" > main.js
hx add main.js
hx commit -m "Update main.js in main branch"

# 5. Attempt to merge (this will create conflicts)
hx merge feature/conflicting-changes
# or, to resolve automatically:
hx merge feature/conflicting-changes --strategy ours
hx merge feature/conflicting-changes --strategy theirs

# Expected output (for --strategy ours):
# Merging branch 'feature/conflicting-changes' into 'main' with strategy: ours
# Merge completed with 1 conflicts, resolved automatically using 'ours'.
# Current branch: main

# Expected output (for --strategy theirs):
# Merging branch 'feature/conflicting-changes' into 'main' with strategy: theirs
# Merge completed with 1 conflicts, resolved automatically using 'theirs'.
# Current branch: main

# Expected output (for default/manual):
# Merging branch 'feature/conflicting-changes' into 'main' with strategy: manual
# Merge completed with 1 conflicts.
# Conflicted files:
#   main.js
# Please resolve conflicts and commit the result.
# Current branch: main

# 6. Check status to see conflicted files
hx status

# Expected output:
# Repository Status
# ========================================
# On branch: main
# HEAD: [merge commit hash]
# 
# You have unmerged paths.
#   (fix conflicts and run "hx merge --continue")
# 
# Unmerged paths:
#   main.js

# 7. Resolve the conflict by editing main.js
# The file will contain conflict markers:
# <<<<<<< HEAD
# // Main branch changes
# =======
# // Feature branch changes
# >>>>>>> feature/conflicting-changes

# Edit the file to resolve the conflict:
echo "// Resolved changes from both branches" > main.js

# 8. Stage the resolved file
hx add main.js

# 9. Continue the merge
hx merge --continue

# Expected output:
# Merge completed successfully
# Current branch: main

# 10. Verify the resolution
hx log --oneline

# Expected output:
# [merge commit] Merge branch 'feature/conflicting-changes'
# [feature commit] Update main.js in feature branch
# [main commit] Update main.js in main branch
# [initial commit] Initial project setup
```

### What Happened
- Demonstrated manual and automated conflict resolution using the --strategy flag
- Created conflicting changes in different branches
- Demonstrated conflict detection during merge
- Showed how to resolve conflicts manually
- Completed the merge with resolved conflicts

## Example 6: History Verification

### Scenario
You want to verify the integrity of your repository history.

### Step-by-Step Process

```bash
# 1. Continue from previous example
cd my-first-project

# 2. Verify the entire history
hx verify-history

# Expected output:
# Verifying ancestry for commit: [current HEAD]
# [commit hash] VALID Merge branch 'feature/conflicting-changes'
# [commit hash] VALID Update main.js in feature branch
# [commit hash] VALID Update main.js in main branch
# [commit hash] VALID Initial project setup
# All commits in ancestry are valid!

# 3. Verify a specific commit
hx verify-history a1b2c3d4

# Expected output:
# Verifying ancestry for commit: a1b2c3d4
# [commit hash] VALID Initial project setup
# All commits in ancestry are valid!

# 4. View the commit DAG
hx dag

# Expected output:
# Commit DAG Visualization
# ========================================
# [merge commit] -> [main commit], [feature commit]
#   [main commit] -> [initial commit]
#   [feature commit] -> [initial commit]
#     [initial commit] -> (root)

# 5. Show detailed commit information
hx log --stat

# Expected output:
# HEAD -> [merge commit] VALID Merge branch 'feature/conflicting-changes'
#     Parents: [main commit], [feature commit]
#     Author: Your Name <your.email@example.com>
#     Date: 2024-01-15 14:00:00
#     Files: 1 files changed
# 
# [feature commit] VALID Update main.js in feature branch
#     Parents: [initial commit]
#     Author: Your Name <your.email@example.com>
#     Date: 2024-01-15 13:30:00
#     Files: 1 files changed
# 
# [main commit] VALID Update main.js in main branch
#     Parents: [initial commit]
#     Author: Your Name <your.email@example.com>
#     Date: 2024-01-15 13:45:00
#     Files: 1 files changed
# 
# [initial commit] VALID Initial project setup
#     Parents: (root)
#     Author: Your Name <your.email@example.com>
#     Date: 2024-01-15 10:30:00
#     Files: 2 files changed
```

### What Happened
- Verified the cryptographic integrity of all commits
- Checked specific commits for validity
- Visualized the commit DAG structure
- Demonstrated the tamper-evident nature of Helix

## Example 7: Key Management

### Scenario
You want to manage your cryptographic keys properly.

### Step-by-Step Process

```bash
# 1. Show your current public key
hx key show

# Expected output:
# Public Key (Ed25519):
# a1b2c3d4e5f6...
# 
# SSH Format:
# ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI... your.email@example.com

# 2. Export your keypair for backup
hx key export ~/backup/helix-key-$(date +%Y%m%d).key

# Expected output:
# Keypair exported!
# Location: ~/backup/helix-key-20240115.key

# 3. Generate a new key (rotation)
hx keygen --new

# Expected output:
# Keypair generated and saved!
# Public key: f6e5d4c3b2a1...

# 4. Test the new key
echo "// Test commit with new key" >> test.js
hx add test.js
hx commit -m "Test new signing key"

# Expected output:
# Commit created successfully!
# Commit ID: [new commit hash]
# Message: Test new signing key
# Author: Your Name <your.email@example.com>
# Date: 2024-01-15 15:00:00
# Files: 1 files changed
# Branch: main

# 5. Verify the commit was signed with new key
hx verify-history

# Expected output:
# Verifying ancestry for commit: [new commit hash]
# [new commit hash] VALID Test new signing key
# [previous commits] VALID [previous commit messages]
# All commits in ancestry are valid!

# 6. Import a key from backup (if needed)
hx key import ~/backup/helix-key-20240115.key

# Expected output:
# Keypair imported!
# Public key: a1b2c3d4e5f6...
```

### What Happened
- Displayed your public key in multiple formats
- Created a secure backup of your keypair
- Generated a new key for rotation
- Tested the new key with a commit
- Demonstrated key import functionality

## Example 8: Using .helixignore

### Scenario
You want to ignore certain files and directories in your repository.

### Step-by-Step Process

```bash
# 1. Continue from previous example
cd my-first-project

# 2. Create a .helixignore file
cat > .helixignore << 'EOF'
# Build artifacts
target/
build/
dist/

# Dependencies
node_modules/
vendor/

# IDE files
.vscode/
.idea/

# OS files
.DS_Store
Thumbs.db

# Log files
*.log
logs/

# Environment files
.env
.env.local

# Temporary files
*.tmp
*.temp
EOF

# 3. Create some files that should be ignored
mkdir target
echo "build artifact" > target/app.exe
echo "log message" > app.log
echo "secret" > .env
mkdir node_modules
echo "dependency" > node_modules/package.json

# 4. Check status (ignored files should not appear)
hx status

# Expected output:
# Repository Status
# ========================================
# On branch: main
# HEAD: [commit hash]
# Working tree clean
# 
# Note: Ignored files are not shown in status

# 5. Try to add ignored files
hx add target/app.exe app.log .env

# Expected output:
# No files to add
# (These files are ignored by .helixignore)

# 6. Add the .helixignore file itself
hx add .helixignore
hx commit -m "Add .helixignore file"

# Expected output:
# Commit created successfully!
# Commit ID: [commit hash]
# Message: Add .helixignore file
# Author: Your Name <your.email@example.com>
# Date: 2024-01-15 16:00:00
# Files: 1 files changed
# Branch: main

# 7. Verify that ignored files are not tracked
hx status

# Expected output:
# Repository Status
# ========================================
# On branch: main
# HEAD: [commit hash]
# Working tree clean
```

### What Happened
- Created a `.helixignore` file with common ignore patterns
- Demonstrated that ignored files are not tracked
- Showed that ignored files don't appear in status
- Verified that the ignore file itself can be tracked

### Advanced .helixignore Patterns

```bash
# More complex patterns
cat >> .helixignore << 'EOF'

# Ignore specific file types
*.bak
*.backup
*.old

# Ignore files in any directory
**/temp/
**/cache/

# Ignore files with specific names
config.local
secrets.json

# Ignore everything in a directory except specific files
docs/*
!docs/README.md
!docs/API.md

# Ignore files by size (if supported)
# Large files
*.iso
*.zip
*.tar.gz
EOF
```

### Testing Ignore Patterns

```bash
# Test if a file would be ignored
hx check-ignore path/to/file

# Expected output:
# path/to/file
# (if the file would be ignored)

# Or no output if the file would not be ignored
```

This example demonstrates how `.helixignore` provides fine-grained control over which files are tracked in your repository, similar to Git's `.gitignore` functionality.

## Best Practices Summary

### Do's
- ✅ Make small, focused commits
- ✅ Use descriptive commit messages
- ✅ Create feature branches for development
- ✅ Verify history regularly
- ✅ Backup your keys securely
- ✅ Test changes before merging

### Don'ts
- ❌ Commit directly to main
- ❌ Use vague commit messages
- ❌ Ignore merge conflicts
- ❌ Share private keys
- ❌ Skip verification steps

### Security Checklist
- [ ] Verify commit signatures
- [ ] Backup keys regularly
- [ ] Rotate keys periodically
- [ ] Monitor for suspicious activity
- [ ] Keep software updated

These examples provide a solid foundation for using Helix effectively. Practice these workflows to become comfortable with the system's features and security model. 