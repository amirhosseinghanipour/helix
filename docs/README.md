# Helix Documentation

Welcome to the Helix documentation. This guide provides comprehensive information about the Helix version control system, from basic usage to advanced architecture details.

## Documentation Structure

### 📚 [User Guide](./user-guide/)
- **Getting Started**: Installation, first repository, basic commands
- **Core Concepts**: Understanding commits, branches, remotes
- **Workflows**: Common development workflows and best practices
- **Troubleshooting**: Common issues and solutions

### 🏗️ [Architecture](./architecture/)
- **System Overview**: High-level architecture and design principles
- **Data Structures**: Detailed explanation of core data models
- **Storage System**: How objects, commits, and metadata are stored
- **Cryptographic Design**: Security model and cryptographic primitives
- **Performance Characteristics**: Scalability and optimization details

### 🔧 [API Reference](./api/)
- **Command Reference**: Complete CLI command documentation
- **Configuration**: Repository and global configuration options
- **File Formats**: Internal file formats and specifications
- **Protocol**: Remote communication protocol details

### 💡 [Examples](./examples/)
- **Basic Workflows**: Step-by-step examples for common tasks
- **Advanced Scenarios**: Complex workflows and edge cases
- **Integration**: Examples with external tools and CI/CD
- **Migration**: Examples of migrating from other VCS systems

## Quick Start

### Installation
```bash
# Clone the repository
git clone https://github.com/amirhosseinghanipour/helix.git
cd helix

# Build from source
cargo build --release

# Install globally (optional)
cargo install --path .
```

### First Repository
```bash
# Initialize a new repository
hx init my-project
cd my-project

# Generate your signing key
hx keygen

# Add and commit files
hx add .
hx commit -m "Initial commit"

# View status
hx status
```

### Key Features
- **Cryptographic Signatures**: Every commit is signed with Ed25519
- **Tamper-Evident History**: SHA-256 Merkle DAG ensures integrity
- **High Performance**: Hierarchical index for fast operations
- **Decentralized**: No central authority required

## Contributing to Documentation

We welcome contributions to improve our documentation! Please:

1. Follow the existing style and structure
2. Include code examples where appropriate
3. Test all examples before submitting
4. Update the table of contents when adding new sections

## Documentation Standards

- Use clear, concise language
- Include practical examples
- Maintain consistent formatting
- Keep information up-to-date with code changes
- Provide both basic and advanced content

---

**Need Help?** If you can't find what you're looking for, please [open an issue](https://github.com/amirhosseinghanipour/helix/issues) or check our [FAQ](./user-guide/faq.md). 