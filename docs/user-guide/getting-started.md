# Getting Started with Helix

## Introduction

Helix is a modern, secure version control system that combines the familiarity of Git with advanced cryptographic security features. This guide will help you get started with Helix, from installation to your first commit.

## Installation

### Prerequisites

- **Rust**: Helix is written in Rust and requires Rust 1.70 or later
- **Git**: For cloning the repository (optional, for development)
- **OpenSSL**: For cryptographic operations (usually pre-installed)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/amirhosseinghanipour/helix.git
cd helix

# Build the project
cargo build --release

# The binary will be available at target/release/hx
```

### Installing Globally

```bash
# Install globally (requires Rust)
cargo install --path .

# Verify installation
hx --version
```

### System Packages

*Note: System packages are not yet available. Build from source for now.*

## First Repository

### Initializing a Repository

```bash
# Create a new directory for your project
mkdir my-project
cd my-project

# Initialize a new Helix repository
hx init

# This creates the .helix directory with repository metadata
```

### Repository Structure

After initialization, your repository will have this structure:

```
my-project/
├── .helix/
│   ├── config              # Repository configuration
│   ├── objects/            # Content-addressable storage
│   ├── index/              # Hierarchical file index
│   ├── branches.json       # Branch information
│   └── remotes.json        # Remote repository configuration
├── .helixignore            # Ignore patterns (optional)
└── [your project files]
```

### Ignoring Files with .helixignore

Helix supports a `.helixignore` file to specify which files and directories should be ignored. This works similarly to `.gitignore`:

```bash
# Create a .helixignore file
touch .helixignore

# Add patterns to ignore
echo "target/" >> .helixignore
echo "*.log" >> .helixignore
echo ".env" >> .helixignore
```

**Common .helixignore patterns:**
- `target/` - Ignore build directories
- `*.log` - Ignore log files
- `.env` - Ignore environment files
- `node_modules/` - Ignore dependencies
- `*.tmp` - Ignore temporary files

**Pattern syntax:**
- `*.ext` - Ignore files with specific extension
- `directory/` - Ignore entire directories
- `/file` - Ignore specific file from repository root
- `# comment` - Comments (lines starting with #)

### Setting Up Your Identity

Before making your first commit, you need to set up your identity:

```bash
# Generate a new Ed25519 keypair
hx keygen

# This creates ~/.helix/keys/ed25519.key
# Your public key will be displayed
```

### Configuration

You can configure your identity globally or per repository:

```bash
# Global configuration
hx config --global user.name "Your Name"
hx config --global user.email "your.email@example.com"

# Repository-specific configuration
hx config user.name "Your Name"
hx config user.email "your.email@example.com"
```

## Basic Workflow

### Staging Files

```bash
# Stage specific files
hx add file1.txt file2.txt

# Stage all files in current directory
hx add .

# Stage all tracked files
hx add -u

# Check what's staged
hx status
```

### Making Commits

```bash
# Create a commit with a message
hx commit -m "Initial commit"

# The commit will be automatically signed with your Ed25519 key
```

### Viewing History

```bash
# View commit history
hx log

# View with graph visualization
hx log --graph

# View specific number of commits
hx log -n 5
```

### Checking Status

```bash
# Check repository status
hx status

# This shows:
# - Current branch
# - Staged changes
# - Modified files
# - Untracked files
```

## Working with Branches

### Creating Branches

```bash
# Create a new branch
hx branch feature-branch

# Create and switch to new branch
hx checkout -b feature-branch

# List all branches
hx branch
```

### Switching Branches

```bash
# Switch to an existing branch
hx checkout main

# Switch to the previous branch
hx checkout -
```

### Merging Branches

```bash
# Merge a branch into current branch
hx merge feature-branch

# This creates a merge commit if there are conflicts
```

## Working with Remotes

### Adding Remotes

```bash
# Add a remote repository
hx remote add origin https://github.com/user/repo.git

# List remotes
hx remote

# Remove a remote
hx remote remove origin
```

### Pushing and Pulling

```bash
# Push changes to remote
hx push origin main

# Pull changes from remote
hx pull origin main

# Note: All commits are verified for cryptographic integrity
```

## Key Management

### Generating Keys

```bash
# Generate a new keypair
hx keygen

# This creates the key file and displays your public key
```

### Managing Keys

```bash
# Show your public key
hx key show

# Export your keypair
hx key export /path/to/backup.key

# Import a keypair
hx key import /path/to/key.file
```

### Key Security

- **Backup your keys**: Store them securely
- **Don't share private keys**: Keep them confidential
- **Use different keys**: Consider separate keys for different projects
- **Rotate keys**: Generate new keys periodically

## Advanced Features

### DAG Visualization

```bash
# Visualize the commit DAG
hx dag

# This shows the commit graph in ASCII art
```

### History Verification

```bash
# Verify commit history integrity
hx verify-history

# Verify specific commit
hx verify-history <commit-hash>
```

### Diff Viewing

```bash
# View changes in working directory
hx diff

# View staged changes
hx diff --staged

# View changes between commits
hx diff commit1..commit2
```

## Best Practices

### Commit Messages

Write clear, descriptive commit messages:

```bash
# Good commit message
hx commit -m "Add user authentication system

- Implement JWT token generation
- Add password hashing with bcrypt
- Create login/logout endpoints
- Add session management"

# Bad commit message
hx commit -m "fix stuff"
```

### Branch Naming

Use descriptive branch names:

```bash
# Good branch names
hx branch feature/user-authentication
hx branch bugfix/login-validation
hx branch hotfix/security-patch

# Bad branch names
hx branch temp
hx branch test
```

### Regular Commits

Make small, focused commits:

```bash
# Instead of one large commit, make several small ones
hx add auth/models.rs
hx commit -m "Add user model"

hx add auth/controllers.rs
hx commit -m "Add authentication controllers"

hx add auth/routes.rs
hx commit -m "Add authentication routes"
```

### Security Practices

- **Verify commits**: Always verify commit signatures
- **Use strong keys**: Generate keys with sufficient entropy
- **Backup regularly**: Keep secure backups of your keys
- **Monitor integrity**: Regularly verify repository integrity

## Troubleshooting

### Common Issues

#### "No keypair found"
```bash
# Generate a new keypair
hx keygen
```

#### "Repository not initialized"
```bash
# Initialize the repository
hx init
```

#### "No changes to commit"
```bash
# Stage files first
hx add .
hx commit -m "Your message"
```

#### "Remote not configured"
```bash
# Add a remote
hx remote add origin <url>
```

### Getting Help

```bash
# Show help for a command
hx help <command>

# Show general help
hx --help

# Show version
hx --version
```

## Next Steps

Now that you have the basics, explore these advanced topics:

- [Advanced Workflows](./workflows.md)
- [Branching Strategies](./branching.md)
- [Remote Collaboration](./collaboration.md)
- [Security Best Practices](./security.md)
- [Performance Optimization](./performance.md)

## Examples

### Complete Workflow Example

```bash
# Initialize repository
mkdir my-project && cd my-project
hx init
hx keygen

# Configure identity
hx config user.name "Your Name"
hx config user.email "your.email@example.com"

# Create some files
echo "# My Project" > README.md
echo "console.log('Hello, World!');" > main.js

# Stage and commit
hx add .
hx commit -m "Initial project setup"

# Create a feature branch
hx checkout -b feature/new-feature

# Make changes
echo "// New feature" >> main.js
hx add main.js
hx commit -m "Add new feature"

# Merge back to main
hx checkout main
hx merge feature/new-feature

# Add remote and push
hx remote add origin https://github.com/user/repo.git
hx push origin main
```

This completes your introduction to Helix! You now have the foundation to start using Helix for version control with cryptographic security. 