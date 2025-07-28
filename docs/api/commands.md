# Helix Command Reference

## Overview

This document provides a complete reference for all Helix commands, including options, examples, and usage patterns.

## Command Structure

All Helix commands follow the pattern:
```bash
hx <command> [options] [arguments]
```

## Repository Commands

### `hx init`

Initialize a new Helix repository.

**Syntax:**
```bash
hx init [path]
```

**Options:**
- `--bare`: Create a bare repository
- `--template <template>`: Use specified template
- `--shared <permissions>`: Set repository permissions

**Examples:**
```bash
# Initialize in current directory
hx init

# Initialize in specific directory
hx init my-project

# Create bare repository
hx init --bare my-repo.git
```

**Output:**
```
Helix repository initialized successfully!
Repository location: /path/to/repo
Current branch: main

Next steps:
  hx add .     # Stage all files
  hx commit -m "Initial commit"  # Create first commit
```

### `hx clone`

Clone a repository from a remote URL.

**Syntax:**
```bash
hx clone <url> [path]
```

**Options:**
- `--branch <branch>`: Clone specific branch
- `--depth <depth>`: Create shallow clone
- `--bare`: Create bare repository
- `--mirror`: Create mirror repository

**Examples:**
```bash
# Clone repository
hx clone https://github.com/user/repo.git

# Clone to specific directory
hx clone https://github.com/user/repo.git my-repo

# Clone specific branch
hx clone --branch develop https://github.com/user/repo.git
```

**Output:**
```
Repository cloned successfully!
Location: /path/to/repo
Source: https://github.com/user/repo.git
Downloaded: 1234 objects
```

## File Management Commands

### `hx add`

Stage files for commit.

**Syntax:**
```bash
hx add [files...]
```

**Options:**
- `-A, --all`: Stage all files
- `-u, --update`: Stage only tracked files
- `-p, --patch`: Interactive patch mode
- `-f, --force`: Force add ignored files

**Examples:**
```bash
# Stage specific files
hx add file1.txt file2.txt

# Stage all files
hx add .

# Stage only tracked files
hx add -u

# Interactive staging
hx add -p
```

**Output:**
```
Files staged successfully!
Added: 5 files
Total staged: 12 files
```

### `hx status`

Show repository status.

**Syntax:**
```bash
hx status [options]
```

**Options:**
- `--porcelain`: Machine-readable output
- `--branch`: Show branch information
- `--ignored`: Show ignored files
- `--untracked-files <mode>`: Show untracked files

**Examples:**
```bash
# Show status
hx status

# Machine-readable output
hx status --porcelain

# Show ignored files
hx status --ignored
```

**Output:**
```
Repository Status
========================================
On branch: main
HEAD: a1b2c3d4

Changes to be committed:
  + new-file.txt
  ~ modified-file.txt

Changes not staged for commit:
  ~ unstaged-file.txt

Untracked files:
  untracked-file.txt

Summary:
  Staged: 2 files
  Modified: 1 files
  Untracked: 1 files
```

### `hx diff`

Show differences between commits, branches, or working directory.

**Syntax:**
```bash
hx diff [options] [commit1] [commit2]
```

**Options:**
- `--staged`: Show staged changes
- `--cached`: Alias for --staged
- `--name-only`: Show only file names
- `--stat`: Show statistics
- `--color`: Enable colored output

**Examples:**
```bash
# Show working directory changes
hx diff

# Show staged changes
hx diff --staged

# Show changes between commits
hx diff HEAD~1 HEAD

# Show changes in specific file
hx diff file.txt
```

**Output:**
```
Diff View
========================================

File: src/main.rs
@@ -10,7 +10,7 @@
 fn main() {
-    println!("Hello, World!");
+    println!("Hello, Helix!");
     let result = process_data();
     println!("Result: {}", result);
 }
```

## Commit Commands

### `hx commit`

Create a new commit with staged changes.

**Syntax:**
```bash
hx commit [options] [-m <message>]
```

**Options:**
- `-m, --message <message>`: Commit message
- `-a, --all`: Stage and commit all tracked files
- `--amend`: Amend previous commit
- `--no-verify`: Skip pre-commit hooks
- `--signoff`: Add sign-off line

**Examples:**
```bash
# Commit with message
hx commit -m "Add user authentication"

# Stage and commit all tracked files
hx commit -a -m "Update documentation"

# Amend previous commit
hx commit --amend -m "Fix typo in commit message"
```

**Output:**
```
Commit created successfully!
Commit ID: a1b2c3d4
Message: Add user authentication
Author: Your Name <your.email@example.com>
Date: 2024-01-15 10:30:00
Files: 3 files changed
Branch: main
```

### `hx log`

Show commit history.

**Syntax:**
```bash
hx log [options] [commit] [--] [path]
```

**Options:**
- `-n, --max-count <number>`: Limit number of commits
- `--oneline`: One line per commit
- `--graph`: Show commit graph
- `--stat`: Show statistics
- `--author <pattern>`: Filter by author
- `--since <date>`: Show commits since date
- `--until <date>`: Show commits until date

**Examples:**
```bash
# Show recent commits
hx log

# Show last 5 commits
hx log -n 5

# Show with graph
hx log --graph

# Show commits by author
hx log --author "John Doe"

# Show commits since date
hx log --since "2024-01-01"
```

**Output:**
```
HEAD -> a1b2c3d4 VALID Add user authentication
    Parents: b2c3d4e5
    Author: Your Name <your.email@example.com>
    Date: 2024-01-15 10:30:00
    Files: 3 files changed

b2c3d4e5 VALID Update documentation
    Parents: c3d4e5f6
    Author: Your Name <your.email@example.com>
    Date: 2024-01-15 09:15:00
    Files: 1 files changed
```

## Branch Commands

### `hx branch`

List, create, or delete branches.

**Syntax:**
```bash
hx branch [options] [branch-name]
```

**Options:**
- `-a, --all`: Show all branches
- `-r, --remotes`: Show remote branches
- `-d, --delete`: Delete branch
- `-D, --force-delete`: Force delete branch
- `-m, --move`: Rename branch

**Examples:**
```bash
# List branches
hx branch

# Create new branch
hx branch feature-branch

# Delete branch
hx branch -d feature-branch

# Rename branch
hx branch -m old-name new-name
```

**Output:**
```
Branches
========================================
* main
  feature-branch
  hotfix-branch
```

### `hx checkout`

Switch between branches or restore files.

**Syntax:**
```bash
hx checkout [options] <branch>
```

**Options:**
- `-b, --branch`: Create and switch to new branch
- `-B, --force-branch`: Create/reset and switch to branch
- `--track`: Set up tracking mode
- `--no-track`: Do not set up tracking mode

**Examples:**
```bash
# Switch to branch
hx checkout feature-branch

# Create and switch to new branch
hx checkout -b new-feature

# Switch to previous branch
hx checkout -
```

**Output:**
```
Switched to branch 'feature-branch'
Current branch: feature-branch
HEAD: a1b2c3d4
```

### `hx merge`

Merge branches.

**Syntax:**
```bash
hx merge [options] <branch>
```

**Options:**
- `--strategy <strategy>`: Conflict resolution strategy (`ours`, `theirs`, `manual`). Default: `manual`.
- `--no-ff`: Create merge commit even if fast-forward possible
- `--squash`: Squash commits into single commit
- `--abort`: Abort merge
- `--continue`: Continue merge after resolving conflicts

**Examples:**
```bash
# Merge branch into current branch (manual conflict resolution)
hx merge feature-branch

# Merge and always take our branch's version in conflicts
hx merge feature-branch --strategy ours

# Merge and always take the other branch's version in conflicts
hx merge feature-branch --strategy theirs

# Abort merge
hx merge --abort

# Continue merge after resolving conflicts
hx merge --continue
```

**Output:**
```
Merging branch 'feature-branch' into 'main' with strategy: ours
Merge completed with 1 conflicts, resolved automatically using 'ours'.
Current branch: main
```

## Remote Commands

### `hx remote`

Manage remote repositories.

**Syntax:**
```bash
hx remote <command> [options] [name] [url]
```

**Commands:**
- `add`: Add remote
- `remove`: Remove remote
- `rename`: Rename remote
- `set-url`: Set remote URL
- `show`: Show remote information
- `prune`: Remove stale remote references

**Examples:**
```bash
# Add remote
hx remote add origin https://github.com/user/repo.git

# List remotes
hx remote

# Show remote information
hx remote show origin

# Remove remote
hx remote remove origin
```

**Output:**
```
Added remote 'origin' -> https://github.com/user/repo.git
```

### `hx push`

Push commits to remote repository.

**Syntax:**
```bash
hx push [options] [remote] [branch]
```

**Options:**
- `--force`: Force push
- `--set-upstream`: Set upstream branch
- `--all`: Push all branches
- `--tags`: Push tags
- `--dry-run`: Show what would be pushed

**Examples:**
```bash
# Push to origin
hx push origin main

# Push and set upstream
hx push --set-upstream origin feature-branch

# Force push (use with caution)
hx push --force origin main
```

**Output:**
```
Changes pushed successfully!
Uploaded: 5 objects
Remote: https://github.com/user/repo.git
```

### `hx pull`

Pull changes from remote repository.

**Syntax:**
```bash
hx pull [options] [remote] [branch]
```

**Options:**
- `--rebase`: Rebase instead of merge
- `--ff-only`: Fast-forward only
- `--no-ff`: Never fast-forward
- `--all`: Pull all branches

**Examples:**
```bash
# Pull from origin
hx pull origin main

# Pull with rebase
hx pull --rebase origin main

# Pull all branches
hx pull --all
```

**Output:**
```
Changes pulled successfully!
Downloaded: 3 objects
Remote: https://github.com/user/repo.git
Current branch: main

All pulled commits are valid!
```

## Key Management Commands

### `hx keygen`

Generate a new Ed25519 keypair.

**Syntax:**
```bash
hx keygen [options]
```

**Options:**
- `--new`: Generate new key (rotate existing)
- `--output <path>`: Specify output path
- `--force`: Overwrite existing key

**Examples:**
```bash
# Generate new keypair
hx keygen

# Generate new key and rotate
hx keygen --new

# Generate key to specific path
hx keygen --output ~/my-key.key
```

**Output:**
```
Keypair generated and saved!
Public key: a1b2c3d4e5f6...
Key location: ~/.helix/keys/ed25519.key
```

### `hx key show`

Display your public key.

**Syntax:**
```bash
hx key show [options]
```

**Options:**
- `--format <format>`: Output format (hex, base64, ssh)
- `--key <path>`: Specify key file

**Examples:**
```bash
# Show public key
hx key show

# Show in SSH format
hx key show --format ssh

# Show specific key
hx key show --key ~/my-key.key
```

**Output:**
```
Public Key (Ed25519):
a1b2c3d4e5f6...

SSH Format:
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI... your.email@example.com
```

### `hx key import`

Import a keypair from file.

**Syntax:**
```bash
hx key import <path>
```

**Examples:**
```bash
# Import keypair
hx key import ~/backup/my-key.key

# Import from different location
hx key import /path/to/key.file
```

**Output:**
```
Keypair imported!
Public key: a1b2c3d4e5f6...
```

### `hx key export`

Export your keypair to file.

**Syntax:**
```bash
hx key export <path>
```

**Examples:**
```bash
# Export keypair
hx key export ~/backup/helix-key.key

# Export with timestamp
hx key export ~/backup/helix-key-$(date +%Y%m%d).key
```

**Output:**
```
Keypair exported!
Location: ~/backup/helix-key.key
```

## Verification Commands

### `hx verify-history`

Verify commit history integrity.

**Syntax:**
```bash
hx verify-history [options] [commit]
```

**Options:**
- `--since <date>`: Verify since date
- `--until <date>`: Verify until date
- `--verbose`: Show detailed output
- `--quiet`: Suppress output

**Examples:**
```bash
# Verify all history
hx verify-history

# Verify specific commit
hx verify-history a1b2c3d4

# Verify since date
hx verify-history --since "2024-01-01"
```

**Output:**
```
Verifying ancestry for commit: a1b2c3d4
a1b2c3d4 VALID Add user authentication
b2c3d4e5 VALID Update documentation
All commits in ancestry are valid!
```

### `hx dag`

Visualize commit DAG.

**Syntax:**
```bash
hx dag [options] [commit]
```

**Options:**
- `--depth <depth>`: Limit depth
- `--format <format>`: Output format (ascii, dot, json)

**Examples:**
```bash
# Show DAG
hx dag

# Show specific commit DAG
hx dag a1b2c3d4

# Limit depth
hx dag --depth 3
```

**Output:**
```
Commit DAG Visualization
========================================
a1b2c3d4 -> b2c3d4e5, c3d4e5f6
  b2c3d4e5 -> d4e5f6g7
  c3d4e5f6 -> d4e5f6g7
    d4e5f6g7 -> (root)
```

## Configuration Commands

### `hx config`

Get or set configuration values.

**Syntax:**
```bash
hx config [options] [name] [value]
```

**Options:**
- `--global`: Use global configuration
- `--local`: Use repository configuration
- `--list`: List all configurations
- `--unset`: Remove configuration

**Examples:**
```bash
# Set user name
hx config user.name "Your Name"

# Set user email
hx config user.email "your.email@example.com"

# List configurations
hx config --list

# Remove configuration
hx config --unset user.name
```

**Output:**
```
user.name=Your Name
user.email=your.email@example.com
core.repositoryformatversion=0
```

## Utility Commands

### `hx help`

Show help information.

**Syntax:**
```bash
hx help [command]
```

**Examples:**
```bash
# Show general help
hx help

# Show command help
hx help commit

# Show all commands
hx help --all
```

### `hx version`

Show version information.

**Syntax:**
```bash
hx version
```

**Output:**
```
hx version 0.1.0
```

## Exit Codes

Helix uses the following exit codes:

- `0`: Success
- `1`: General error
- `2`: Command line error
- `3`: Repository error
- `4`: Cryptographic error
- `5`: Network error
- `6`: Configuration error

## Environment Variables

- `HX_CONFIG`: Path to global configuration file
- `HX_EDITOR`: Default editor for commit messages
- `HX_PAGER`: Pager for command output
- `HX_DEBUG`: Enable debug output

## Configuration Files

- `~/.helix/config`: Global configuration
- `.helix/config`: Repository configuration
- `~/.helix/keys/`: Key storage directory
- `.helix/objects/`: Object storage directory
- `.helix/index/`: Index storage directory

## Ignore Files

### `.helixignore`

Helix supports a `.helixignore` file to specify which files and directories should be ignored during operations like `add`, `status`, and `commit`.

**File Location**: `.helixignore` in the repository root

**Pattern Syntax**:
- `*.ext` - Ignore files with specific extension
- `directory/` - Ignore entire directories
- `/file` - Ignore specific file from repository root
- `file*` - Ignore files starting with "file"
- `*file` - Ignore files ending with "file"
- `*file*` - Ignore files containing "file"
- `# comment` - Comments (lines starting with #)

**Example .helixignore**:
```
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
```

**Built-in Ignore Patterns**:
Helix automatically ignores these patterns even without a `.helixignore` file:
- `.helix/` - Repository metadata
- `.git/` - Git repositories
- `target/` - Rust build artifacts
- `node_modules/` - Node.js dependencies
- `.DS_Store` - macOS system files
- `*.tmp`, `*.log` - Common temporary files
- IDE and editor files (`.vscode/`, `.idea/`, `*.swp`, etc.) 