# Remote Protocol and Push/Pull Implementation

## Overview

The Helix remote protocol has been significantly enhanced to provide a production-ready, efficient, and secure system for distributed version control operations. This document describes the implementation details, features, and usage of the improved remote protocol.

## Key Improvements

### 1. Sophisticated Negotiation Protocol

The new implementation includes a proper negotiation phase that allows the client and server to efficiently determine what objects need to be transferred.

#### Negotiation Request
```rust
pub struct NegotiationRequest {
    pub wants: Vec<String>,           // Objects the client wants
    pub haves: Vec<String>,           // Objects the client already has
    pub shallow: Vec<String>,         // Shallow commits
    pub deepen_since: Option<i64>,    // Deepen since timestamp
    pub deepen_not: Option<Vec<String>>, // Deepen not these commits
    pub filter: Option<String>,       // Object filter
}
```

#### Negotiation Response
```rust
pub struct NegotiationResponse {
    pub acks: Vec<String>,            // Objects the server will send
    pub nak: Vec<String>,             // Objects the server won't send
    pub shallow: Vec<String>,         // Shallow commits
    pub unshallow: Vec<String>,       // Unshallow commits
    pub packfile: Option<String>,     // Pack file ID if available
}
```

### 2. Pack-Based Object Transfer

Instead of transferring individual objects, the new protocol uses pack files for efficient bulk transfer:

#### Pack Structure
```rust
pub struct Pack {
    pub header: PackHeader,
    pub objects: Vec<PackObject>,
    pub index: HashMap<String, usize>,
}
```

#### Benefits
- **Compression**: Objects are compressed together for smaller transfer sizes
- **Delta Encoding**: Similar objects are stored as deltas to reduce size
- **Batch Transfer**: Multiple objects transferred in a single request
- **Resumable**: Large transfers can be resumed if interrupted

### 3. Comprehensive Authentication System

The authentication system supports multiple methods:

#### Supported Authentication Methods
- **Token-based**: Bearer tokens for API access
- **Basic Auth**: Username/password for HTTP servers
- **SSH**: SSH key authentication for SSH URLs
- **OAuth2**: OAuth2 tokens with refresh capability

#### Authentication Configuration
```bash
# Add token authentication
hx auth add github.com --token <your-token>

# Add SSH authentication
hx auth add git@github.com --ssh-key ~/.ssh/id_ed25519

# Add basic authentication
hx auth add gitlab.com --username <user> --password <pass>
```

### 4. Enhanced Push Command

The push command now includes sophisticated negotiation and error handling:

#### Features
- **Pre-push verification**: All commits are verified before push
- **Efficient object transfer**: Only missing objects are uploaded
- **Ref negotiation**: Proper handling of ref updates
- **Error reporting**: Detailed error messages and status reporting
- **Force push protection**: Configurable force push behavior

#### Usage
```bash
# Basic push
hx push

# Force push
hx push --force

# Push to specific remote
hx push --remote upstream

# Push specific refspec
hx push --refspec refs/heads/feature:refs/heads/feature
```

### 5. Enhanced Pull Command

The pull command includes intelligent object fetching and merge strategies:

#### Features
- **Smart object fetching**: Only downloads missing objects
- **Pack-based download**: Efficient bulk object transfer
- **Conflict detection**: Automatic detection of merge conflicts
- **Signature verification**: Verification of downloaded commits
- **Rebase support**: Option to rebase instead of merge

#### Usage
```bash
# Basic pull
hx pull

# Pull with rebase
hx pull --rebase

# Pull from specific remote
hx pull --remote upstream

# Pull specific branch
hx pull --branch feature
```

## Protocol Flow

### Push Flow
1. **Connectivity Check**: Verify connection to remote
2. **Capability Discovery**: Discover remote capabilities
3. **Local Verification**: Verify local commits are valid
4. **Object Collection**: Collect local objects
5. **Remote State Fetch**: Get remote refs and objects
6. **Negotiation**: Determine what needs to be pushed
7. **Pack Creation**: Create pack of missing objects
8. **Upload**: Upload pack to remote
9. **Ref Update**: Update remote refs
10. **Verification**: Verify push was successful

### Pull Flow
1. **Connectivity Check**: Verify connection to remote
2. **Capability Discovery**: Discover remote capabilities
3. **Remote State Fetch**: Get remote refs and objects
4. **Local Object Collection**: Collect local objects
5. **Negotiation**: Determine what needs to be fetched
6. **Pack Download**: Download pack or individual objects
7. **Object Extraction**: Extract objects from pack
8. **Ref Update**: Update local refs
9. **Verification**: Verify downloaded commits

## Error Handling

### Network Errors
- **Connection failures**: Automatic retry with exponential backoff
- **Timeout handling**: Configurable timeouts for different operations
- **Partial transfers**: Resume capability for interrupted transfers

### Authentication Errors
- **Invalid credentials**: Clear error messages with resolution steps
- **Expired tokens**: Automatic token refresh for OAuth2
- **SSH key issues**: Detailed SSH connection diagnostics

### Protocol Errors
- **Version mismatches**: Graceful degradation to compatible protocols
- **Capability conflicts**: Fallback to basic operations
- **Object corruption**: Automatic retry and verification

## Performance Optimizations

### 1. Parallel Operations
- **Concurrent downloads**: Multiple objects downloaded simultaneously
- **Background verification**: Signature verification in background
- **Async I/O**: Non-blocking network operations

### 2. Caching
- **Object cache**: Frequently accessed objects cached in memory
- **Connection pooling**: Reuse HTTP connections
- **Capability cache**: Remote capabilities cached locally

### 3. Compression
- **LZ4 compression**: Fast compression/decompression
- **Delta encoding**: Efficient storage of similar objects
- **Thin packs**: Minimal pack files for efficiency

## Security Features

### 1. Cryptographic Verification
- **Commit signatures**: All commits verified before push/pull
- **Object integrity**: SHA-256 verification of all objects
- **Chain of trust**: Complete ancestry verification

### 2. Authentication Security
- **Secure storage**: Credentials stored with proper permissions
- **Token rotation**: Support for automatic token refresh
- **SSH key management**: Secure SSH key handling

### 3. Transport Security
- **HTTPS/TLS**: Encrypted transport for HTTP operations
- **SSH encryption**: Secure shell for SSH operations
- **Certificate verification**: Proper SSL certificate validation

## Configuration

### Remote Configuration
```toml
[remote "origin"]
url = "https://github.com/user/repo"
fetch = "+refs/heads/*:refs/remotes/origin/*"
push = "refs/heads/*:refs/heads/*"
```

### Authentication Configuration
```json
{
  "github.com": {
    "method": "Token",
    "host": "github.com",
    "token": "ghp_..."
  },
  "git@github.com": {
    "method": "SSH",
    "host": "github.com",
    "key_path": "~/.ssh/id_ed25519"
  }
}
```

### Protocol Configuration
```toml
[protocol]
timeout = 30
retry_attempts = 3
parallel_downloads = 4
compression_level = 6
```

## Future Enhancements

### Planned Features
1. **Smart HTTP**: Full Git smart HTTP protocol support
2. **Bundle support**: Git bundle format for offline operations
3. **Incremental sync**: Efficient incremental synchronization
4. **Multi-remote**: Enhanced multi-remote support
5. **Mirror support**: Repository mirroring capabilities

### Performance Improvements
1. **Streaming**: Streaming object transfer for large repositories
2. **Predictive fetching**: Pre-fetch objects likely to be needed
3. **Distributed caching**: Peer-to-peer object sharing
4. **Compression improvements**: Advanced compression algorithms

## Migration from Legacy Protocol

The new protocol maintains backward compatibility with the legacy HTTP-based protocol. Existing repositories will continue to work, but users are encouraged to upgrade to take advantage of the new features.

### Migration Steps
1. Update Helix to latest version
2. Configure authentication for remotes
3. Test connectivity with `hx auth test <host>`
4. Perform initial sync with new protocol
5. Verify all operations work correctly

## Troubleshooting

### Common Issues

#### Authentication Problems
```bash
# Test authentication
hx auth test github.com

# Re-add authentication
hx auth remove github.com
hx auth add github.com --token <new-token>
```

#### Network Issues
```bash
# Check connectivity
hx remote test origin

# Increase timeout
export HELIX_TIMEOUT=60
```

#### Protocol Issues
```bash
# Force legacy protocol
export HELIX_USE_LEGACY_PROTOCOL=1

# Enable debug logging
export HELIX_LOG_LEVEL=debug
```

This enhanced remote protocol provides a solid foundation for distributed version control operations, with room for future enhancements and optimizations. 