# Data Structures

## Overview

Helix uses several core data structures to represent repositories, commits, objects, and metadata. This document provides detailed specifications for each data structure.

## Core Data Types

### Commit Object

The commit object is the fundamental unit of version control in Helix.

```rust
struct Commit {
    id: [u8; 32],                    // SHA-256 hash of commit data
    parent_ids: Vec<[u8; 32]>,       // Parent commit hashes (supports merges)
    tree_id: [u8; 32],              // Root tree hash
    author: String,                  // Author name
    email: String,                   // Author email
    timestamp: i64,                  // Unix timestamp
    message: String,                 // Commit message
    signature: [u8; 64],            // Ed25519 signature
    public_key: [u8; 32],           // Author's public key
    files: HashMap<String, FileChange>, // File changes in this commit
}
```

#### File Change Structure
```rust
enum FileChange {
    Added { content_hash: [u8; 32], size: u64 },
    Modified { old_hash: [u8; 32], new_hash: [u8; 32], size: u64 },
    Deleted { old_hash: [u8; 32] },
    Renamed { old_path: String, new_path: String, content_hash: [u8; 32] },
}
```

### Tree Object

Tree objects represent directory structures and file metadata.

```rust
struct Tree {
    id: [u8; 32],                    // SHA-256 hash of tree data
    entries: Vec<TreeEntry>,         // Directory entries
    signature: [u8; 64],            // Ed25519 signature
    public_key: [u8; 32],           // Signer's public key
}

struct TreeEntry {
    name: String,                    // File/directory name
    hash: [u8; 32],                 // Content hash
    mode: u32,                      // File permissions (Unix-style)
    object_type: ObjectType,        // Type of object (blob, tree, etc.)
}
```

### Blob Object

Blob objects store file content.

```rust
struct Blob {
    id: [u8; 32],                    // SHA-256 hash of content
    content: Vec<u8>,               // File content
    size: u64,                      // Content size in bytes
    compression: CompressionType,   // Compression algorithm used
}
```

### Index Node (Hierarchical)

The index uses a hierarchical tree structure for efficient file tracking.

```rust
struct IndexNode {
    name: String,                    // File/directory name
    hash: [u8; 32],                 // Content hash
    mode: u32,                      // File permissions
    children: HashMap<String, IndexNode>, // Subdirectories
    is_directory: bool,             // Directory flag
    staged: bool,                   // Staging status
    modified: bool,                 // Modification status
    last_modified: i64,             // Last modification timestamp
}
```

### Branch Object

Branch objects represent named references to commits.

```rust
struct Branch {
    name: String,                    // Branch name
    head_commit: [u8; 32],          // HEAD commit hash
    upstream: Option<String>,        // Upstream remote branch
    created_at: i64,                // Creation timestamp
    last_updated: i64,              // Last update timestamp
    metadata: HashMap<String, String>, // Additional metadata
}
```

### Remote Object

Remote objects represent external repositories.

```rust
struct Remote {
    name: String,                    // Remote name (e.g., "origin")
    url: String,                    // Remote URL
    fetch_spec: String,             // Fetch specification
    push_spec: String,              // Push specification
    authentication: AuthConfig,     // Authentication configuration
}
```

## Storage Objects

### Object Database

All objects are stored in a content-addressable database.

```rust
enum Object {
    Blob(Blob),                     // File content
    Tree(Tree),                     // Directory structure
    Commit(Commit),                 // Commit metadata
    Delta {                         // Incremental changes
        base_hash: [u8; 32],
        patches: Vec<Patch>,
        target_hash: [u8; 32],
    },
}
```

### Delta Structure

Deltas represent incremental changes between snapshots.

```rust
struct Delta {
    base_hash: [u8; 32],            // Base snapshot hash
    target_hash: [u8; 32],          // Target snapshot hash
    patches: Vec<Patch>,            // List of patches
    metadata: DeltaMetadata,        // Delta metadata
}

struct Patch {
    operation: PatchOperation,      // Type of operation
    path: String,                   // File path
    data: Vec<u8>,                 // Patch data
    size: u64,                     // Patch size
}

enum PatchOperation {
    Add,                            // Add new file
    Modify,                         // Modify existing file
    Delete,                         // Delete file
    Rename { old_path: String },    // Rename file
}
```

## Configuration Structures

### Repository Configuration

```rust
struct RepositoryConfig {
    version: u32,                   // Configuration version
    author: String,                 // Default author name
    email: String,                  // Default author email
    signing: SigningConfig,         // Signing configuration
    storage: StorageConfig,         // Storage configuration
    remotes: HashMap<String, Remote>, // Remote repositories
    hooks: HashMap<String, String>, // Git-style hooks
}
```

### Signing Configuration

```rust
struct SigningConfig {
    key_path: PathBuf,              // Path to signing key
    verify_commits: bool,           // Whether to verify commits
    required: bool,                 // Whether signing is required
    algorithm: SignatureAlgorithm,  // Signature algorithm
}
```

### Storage Configuration

```rust
struct StorageConfig {
    compression: CompressionType,   // Compression algorithm
    delta_encoding: bool,           // Enable delta encoding
    snapshot_interval: u32,         // Snapshot frequency
    garbage_collection: GCConfig,   // Garbage collection settings
}
```

## Memory Management

### Object Cache

```rust
struct ObjectCache {
    blobs: LruCache<[u8; 32], Blob>,      // Blob cache
    trees: LruCache<[u8; 32], Tree>,      // Tree cache
    commits: LruCache<[u8; 32], Commit>,  // Commit cache
    max_size: usize,                       // Maximum cache size
}
```

### Index Cache

```rust
struct IndexCache {
    nodes: HashMap<String, IndexNode>,    // Index node cache
    dirty: HashSet<String>,               // Dirty nodes
    max_entries: usize,                   // Maximum entries
}
```

## Serialization Formats

### Commit Serialization

Commits are serialized using a binary format for efficiency:

```
[32 bytes]  - Commit ID (SHA-256)
[4 bytes]   - Number of parents
[N * 32 bytes] - Parent commit IDs
[32 bytes]  - Tree ID
[4 bytes]   - Author name length
[N bytes]   - Author name
[4 bytes]   - Email length
[N bytes]   - Email
[8 bytes]   - Timestamp
[4 bytes]   - Message length
[N bytes]   - Message
[64 bytes]  - Ed25519 signature
[32 bytes]  - Public key
[4 bytes]   - Number of file changes
[N bytes]   - File changes (serialized)
```

### Tree Serialization

```
[32 bytes]  - Tree ID (SHA-256)
[4 bytes]   - Number of entries
[N bytes]   - Tree entries (serialized)
[64 bytes]  - Ed25519 signature
[32 bytes]  - Public key
```

### Tree Entry Serialization

```
[4 bytes]   - Name length
[N bytes]   - Entry name
[32 bytes]  - Content hash
[4 bytes]   - File mode
[1 byte]    - Object type
```

## Performance Considerations

### Memory Usage

- **Commit Objects**: ~1KB per commit (typical)
- **Tree Objects**: ~100-500 bytes per tree
- **Blob Objects**: Variable (file content size)
- **Index Nodes**: ~200 bytes per file

### Storage Efficiency

- **Compression**: LZ4 reduces size by 60-80%
- **Deduplication**: 95%+ reduction for repeated content
- **Delta Encoding**: 70-90% reduction for incremental changes

### Cache Performance

- **Object Cache**: LRU eviction for memory management
- **Index Cache**: Dirty tracking for efficient persistence
- **Hash Lookups**: O(1) average case with good hash distribution

## Security Properties

### Cryptographic Integrity

- **Content Hashing**: SHA-256 for all content
- **Signature Verification**: Ed25519 for all commits and trees
- **Chain Verification**: Recursive verification of parent commits
- **Tamper Detection**: Any modification breaks the hash chain

### Data Validation

- **Type Safety**: Strong typing prevents invalid operations
- **Bounds Checking**: All array accesses are bounds-checked
- **Format Validation**: Serialization format validation
- **Consistency Checks**: Cross-reference validation

## Future Extensions

### Planned Enhancements

- **Large File Support**: Streaming for files > 100MB
- **Binary Delta**: Binary diff algorithms for better compression
- **Parallel Processing**: Concurrent object operations
- **Custom Metadata**: Extensible metadata system

### Research Areas

- **Quantum-Resistant Hashing**: Post-quantum hash functions
- **Advanced Compression**: Machine learning-based compression
- **Distributed Storage**: Multi-node object storage
- **Blockchain Integration**: Immutable history verification 