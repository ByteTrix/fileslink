# 🏗️ FilesLink Architecture Documentation

## 📋 Table of Contents
- [System Overview](#system-overview)
- [Architecture Diagrams](#architecture-diagrams)
- [Component Details](#component-details)
- [Data Flow](#data-flow)
- [Storage Model](#storage-model)
- [Security Architecture](#security-architecture)
- [Technology Stack](#technology-stack)
- [Performance Characteristics](#performance-characteristics)

---

## System Overview

FilesLink is a cloud-native file sharing system that leverages Telegram's infrastructure as a storage backend. The architecture follows a microservices-inspired design with clear separation of concerns between the bot interface, HTTP server, and storage management layers.

### Key Architectural Principles
- **Cloud-First**: Telegram as primary storage eliminates local disk dependencies
- **Stream-Based**: Memory-efficient file handling through streaming
- **Queue-Driven**: Asynchronous processing prevents system overload
- **Stateless Server**: HTTP layer remains stateless for horizontal scalability
- **Event-Driven**: Message-based communication between components

---

## Architecture Diagrams

### High-Level System Architecture

```mermaid
graph TB
    subgraph "Client Layer"
        U1[Telegram User]
        U2[Web Browser]
    end
    
    subgraph "Application Layer"
        BOT[Bot Service<br/>Teloxide]
        HTTP[HTTP Server<br/>Axum]
        QUEUE[Message Queue<br/>MPSC Channel]
    end
    
    subgraph "Business Logic Layer"
        PM[Process Message<br/>Handler]
        QP[Queue Processor]
        FS[File Storage<br/>Manager]
    end
    
    subgraph "Data Layer"
        JSON[file_mappings.json<br/>Metadata Store]
        TG[Telegram Storage<br/>Channel]
    end
    
    U1 -->|Upload File| BOT
    BOT -->|Parse Message| PM
    PM -->|Enqueue| QUEUE
    QUEUE -->|Process| QP
    QP -->|Forward File| TG
    QP -->|Save Metadata| FS
    FS -->|Persist| JSON
    QP -->|Send Link| BOT
    BOT -->|Download Link| U1
    
    U2 -->|GET /files/:id| HTTP
    HTTP -->|Lookup Metadata| FS
    FS -->|Read| JSON
    HTTP -->|Fetch File| TG
    TG -->|Stream| HTTP
    HTTP -->|Response| U2
    
    style BOT fill:#4CAF50
    style HTTP fill:#2196F3
    style TG fill:#FF9800
    style JSON fill:#9C27B0
```

### Component Architecture

```mermaid
graph LR
    subgraph "Bot Module (bot/)"
        B1[bot.rs<br/>TeloxideBot]
        B2[process_message.rs<br/>Message Handler]
        B3[queue.rs<br/>Queue Processor]
    end
    
    subgraph "Server Module (src/)"
        S1[main.rs<br/>Application Entry]
        S2[server.rs<br/>HTTP Routes]
    end
    
    subgraph "Shared Module (shared/)"
        SH1[config.rs<br/>Configuration]
        SH2[file_storage.rs<br/>Storage Manager]
        SH3[chat_config.rs<br/>Permissions]
        SH4[utils.rs<br/>Utilities]
    end
    
    B1 -->|Uses| B2
    B2 -->|Enqueues| B3
    B3 -->|Manages| SH2
    
    S1 -->|Initializes| B1
    S1 -->|Creates| S2
    S2 -->|Fetches| SH2
    
    B1 -.->|Config| SH1
    S2 -.->|Config| SH1
    B2 -.->|Permissions| SH3
    
    style B1 fill:#81C784
    style B2 fill:#81C784
    style B3 fill:#81C784
    style S1 fill:#64B5F6
    style S2 fill:#64B5F6
    style SH1 fill:#BA68C8
    style SH2 fill:#BA68C8
```

### Upload Flow Sequence

```mermaid
sequenceDiagram
    actor User
    participant Bot
    participant Queue
    participant Processor
    participant Storage as Telegram Storage
    participant DB as file_mappings.json
    
    User->>Bot: Send File/Photo/Video
    Bot->>Bot: Validate Permissions
    Bot->>Queue: Add to Queue
    Bot->>User: Queue Position: N
    
    Queue->>Processor: Process Next Item
    Processor->>Bot: Update Status: Processing
    
    alt Upload from Telegram
        Processor->>Storage: Forward File to Channel
        Storage-->>Processor: Return file_id
    else Upload from URL
        Processor->>Processor: Download from URL
        Processor->>Storage: Upload to Channel
        Storage-->>Processor: Return file_id
    end
    
    Processor->>Processor: Generate unique_id
    Processor->>DB: Save Metadata
    DB-->>Processor: Confirm
    
    Processor->>Bot: Edit Message with Link
    Bot->>User: Download Link Ready
    
    Note over User,DB: File now available for download
```

### Download Flow Sequence

```mermaid
sequenceDiagram
    actor User
    participant Browser
    participant Server as HTTP Server
    participant FS as File Storage
    participant DB as file_mappings.json
    participant TG as Telegram API
    
    User->>Browser: Click Download Link
    Browser->>Server: GET /files/{unique_id}
    
    Server->>FS: get_file_metadata(unique_id)
    FS->>DB: Read Mapping
    DB-->>FS: Return FileMetadata
    FS-->>Server: FileMetadata
    
    alt File Found
        Server->>TG: get_file(telegram_file_id)
        TG-->>Server: File Info
        Server->>TG: download_file(file_path)
        TG-->>Server: File Stream
        Server->>Browser: Stream Response
        Browser->>User: File Downloaded
    else File Not Found
        Server->>Browser: 404 Not Found
        Browser->>User: Error Page
    end
```

### Data Model

```mermaid
erDiagram
    FILE_METADATA {
        string unique_id PK
        string telegram_file_id
        string file_name
        string mime_type
        int file_size
        int uploaded_at
    end
    
    TELEGRAM_STORAGE {
        string file_id PK
        binary file_data
        string caption
    end
    
    QUEUE_ITEM {
        Message message
        Message queue_message
        string file_id
        string file_name
        string url
    }
    
    FILE_METADATA ||--|| TELEGRAM_STORAGE : "maps to"
    QUEUE_ITEM }|--|| FILE_METADATA : "creates"
```

---

## Component Details

### 1. Bot Service (`bot/`)

**Responsibilities:**
- Telegram message reception and parsing
- User permission validation
- File type identification (document, photo, video, animation)
- URL command processing (`/url`)
- Queue management and coordination

**Key Files:**
- `bot.rs` - Main bot implementation using Teloxide
- `process_message.rs` - Message parsing and routing logic
- `queue.rs` - Asynchronous queue processing and file forwarding

**Technologies:**
- Teloxide (Telegram Bot API wrapper)
- Tokio MPSC channels for queue communication
- Async/await for concurrent message handling

### 2. HTTP Server (`src/server.rs`)

**Responsibilities:**
- REST API endpoint serving
- File metadata lookup
- Telegram file fetching and streaming
- Content-Type negotiation
- Download header management

**Endpoints:**
| Endpoint | Method | Description | Response |
|----------|--------|-------------|----------|
| `/` | GET | Health check / Homepage | HTML |
| `/files/:id` | GET | Download file by unique_id | Binary stream |
| `/files` | GET | List all files (optional) | HTML |

**Technologies:**
- Axum web framework
- Tower middleware
- Tokio async runtime

### 3. File Storage Manager (`shared/src/file_storage.rs`)

**Responsibilities:**
- Metadata persistence and retrieval
- Thread-safe concurrent access (RwLock)
- CRUD operations for file mappings
- JSON serialization/deserialization

**Data Structure:**
```rust
pub struct FileMetadata {
    pub unique_id: String,           // 8-char nanoid
    pub telegram_file_id: String,    // Telegram's internal ID
    pub file_name: String,           // Original filename
    pub mime_type: Option<String>,   // Content-Type
    pub file_size: u32,              // Size in bytes
    pub uploaded_at: u64,            // Unix timestamp
}
```

**Storage Format:**
```json
{
  "files": {
    "a7k3m9x2": {
      "unique_id": "a7k3m9x2",
      "telegram_file_id": "BQACAgIAAxkBAAI...",
      "file_name": "document.pdf",
      "mime_type": "application/pdf",
      "file_size": 1234567,
      "uploaded_at": 1697472000
    }
  }
}
```

### 4. Configuration Management (`shared/src/config.rs`)

**Environment Variables:**

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `BOT_TOKEN` | ✅ | - | Telegram bot token from BotFather |
| `STORAGE_CHANNEL_ID` | ✅ | - | Private channel ID for file storage |
| `SERVER_PORT` | ❌ | 8080 | HTTP server port |
| `APP_FILE_DOMAIN` | ❌ | `http://localhost:{PORT}/files` | Base URL for download links |
| `TELEGRAM_API_URL` | ❌ | `https://api.telegram.org` | Telegram API endpoint |
| `ENABLE_FILES_ROUTE` | ❌ | false | Enable file listing endpoint |

---

## Data Flow

### Upload Pipeline

```mermaid
flowchart TD
    A[User Sends File] --> B{File Type?}
    B -->|Document/Photo/Video| C[Extract file_id]
    B -->|URL Command| D[Download from URL]
    
    C --> E[Add to Queue]
    D --> F[Upload to Telegram]
    F --> E
    
    E --> G[Queue Processor]
    G --> H{Source?}
    
    H -->|Telegram| I[Forward to Storage Channel]
    H -->|URL| J[Send to Storage Channel]
    
    I --> K[Get new file_id]
    J --> K
    
    K --> L[Generate unique_id<br/>nanoid 8 chars]
    L --> M[Create FileMetadata]
    M --> N[Save to file_mappings.json]
    N --> O[Generate Download URL]
    O --> P[Send Link to User]
    
    style A fill:#E3F2FD
    style P fill:#C8E6C9
```

### Download Pipeline

```mermaid
flowchart TD
    A[User Clicks Link] --> B[HTTP GET /files/:id]
    B --> C[Extract unique_id]
    C --> D[Lookup in file_mappings.json]
    
    D --> E{Found?}
    E -->|No| F[Return 404 Page]
    E -->|Yes| G[Get telegram_file_id]
    
    G --> H[Call Telegram API<br/>get_file]
    H --> I{Success?}
    I -->|No| J[Return 500 Error]
    I -->|Yes| K[Get file path]
    
    K --> L[Stream Download<br/>from Telegram]
    L --> M[Set Headers<br/>Content-Type, Disposition]
    M --> N[Stream to Browser]
    N --> O[User Downloads File]
    
    style A fill:#E3F2FD
    style O fill:#C8E6C9
    style F fill:#FFCDD2
    style J fill:#FFCDD2
```

---

## Storage Model

### Architecture Pattern: Hybrid Storage

```mermaid
graph TB
    subgraph "FilesLink Application"
        APP[Application Logic]
    end
    
    subgraph "Metadata Layer (Local)"
        META[file_mappings.json]
    end
    
    subgraph "Binary Storage Layer (Cloud)"
        TG_CHANNEL[Telegram Storage Channel]
        TG_CDN[Telegram CDN Network]
    end
    
    APP -->|Write Metadata| META
    APP -->|Read Metadata| META
    APP -->|Upload Files| TG_CHANNEL
    APP -->|Download Files| TG_CDN
    TG_CHANNEL -.->|Replicated to| TG_CDN
    
    style META fill:#9C27B0,color:#fff
    style TG_CHANNEL fill:#FF9800,color:#fff
    style TG_CDN fill:#FF9800,color:#fff
```

### Storage Characteristics

| Aspect | Local (Metadata) | Cloud (Binary Files) |
|--------|------------------|----------------------|
| **Type** | JSON file | Telegram channel |
| **Size** | ~1KB per 100 files | Up to 2GB per file |
| **Speed** | Instant | Network-dependent |
| **Backup** | Manual required | Auto by Telegram |
| **Redundancy** | Single point | Telegram's multi-DC |
| **Cost** | Disk space | Free (Telegram) |

---

## Security Architecture

### Security Layers

```mermaid
graph TB
    subgraph "Perimeter Security"
        AUTH[User Authentication]
        PERM[Permission Check]
    end
    
    subgraph "Application Security"
        VAL[Input Validation]
        RATE[Rate Limiting]
        UNIQ[Unique ID Generation]
    end
    
    subgraph "Data Security"
        PRIV[Private Storage Channel]
        MAP[Metadata Isolation]
        STREAM[Secure Streaming]
    end
    
    subgraph "Infrastructure Security"
        TLS[TLS/HTTPS]
        ENV[Environment Variables]
        TOKEN[Token Protection]
    end
    
    AUTH --> VAL
    PERM --> VAL
    VAL --> UNIQ
    UNIQ --> PRIV
    PRIV --> STREAM
    STREAM --> TLS
    
    style AUTH fill:#F44336,color:#fff
    style PRIV fill:#F44336,color:#fff
    style TLS fill:#F44336,color:#fff
```

### Access Control Matrix

| Actor | Upload | Download | Manage Channel | Config Access |
|-------|--------|----------|----------------|---------------|
| Authorized User | ✅ | ✅ | ❌ | ❌ |
| Anonymous | ❌ | ✅ (with link) | ❌ | ❌ |
| Bot | ✅ | ✅ | ✅ | ✅ |
| Admin | ✅ | ✅ | ✅ | ✅ |

### Threat Mitigation

| Threat | Mitigation | Status |
|--------|------------|--------|
| Unauthorized Upload | Permission system via chat_config.json | ✅ Implemented |
| Link Guessing | 8-char random IDs (62^8 combinations) | ✅ Implemented |
| Token Exposure | Environment variables only | ✅ Implemented |
| Data Loss | Telegram's redundancy + JSON backup | ✅ Recommended |
| DDoS | Queue system + Rate limiting | ⚠️ Partial |

---

## Technology Stack

### Core Technologies

```mermaid
mindmap
  root((FilesLink))
    Language
      Rust 2021
      Cargo Package Manager
    Runtime
      Tokio Async
      Multi-threaded
    Bot Framework
      Teloxide
      Telegram Bot API
    Web Framework
      Axum
      Tower Middleware
    Storage
      Telegram Channels
      JSON Files
    Utilities
      Serde JSON
      nanoid
      mime_guess
      reqwest
```

### Dependency Graph

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.38.0 | Async runtime |
| `teloxide` | Latest | Telegram bot API |
| `axum` | 0.7.5 | HTTP server |
| `serde` | 1.0 | Serialization |
| `serde_json` | 1.0 | JSON handling |
| `nanoid` | 0.4 | ID generation |
| `mime_guess` | 2.0 | MIME detection |
| `reqwest` | Latest | HTTP client |
| `log` | 0.4 | Logging facade |
| `pretty_env_logger` | 0.5 | Log formatting |

---

## Performance Characteristics

### Throughput Analysis

```mermaid
graph LR
    subgraph "Upload Performance"
        U1[Sequential Queue]
        U2[Network Bound]
        U3[Telegram API Limit]
    end
    
    subgraph "Download Performance"
        D1[Concurrent Streams]
        D2[Memory Efficient]
        D3[CDN Cached]
    end
    
    U1 --> U2
    U2 --> U3
    D1 --> D2
    D2 --> D3
    
    style U1 fill:#FFF9C4
    style D1 fill:#C8E6C9
```

### Performance Metrics

| Operation | Latency | Throughput | Memory | Scalability |
|-----------|---------|------------|--------|-------------|
| **Upload (Small <1MB)** | 1-3s | Sequential | ~10MB | Queue-limited |
| **Upload (Large 100MB)** | 10-30s | Sequential | ~20MB | Queue-limited |
| **Download (Small <1MB)** | 100-500ms | 100+ concurrent | ~5MB per | Horizontal |
| **Download (Large 100MB)** | 2-10s | 50+ concurrent | ~10MB per | Horizontal |
| **Metadata Lookup** | <1ms | 1000+/s | Minimal | Memory-bound |

### Optimization Strategies

1. **Caching Layer**
   - Implement Redis for hot file metadata
   - Cache Telegram file_info responses
   - Reduce JSON file reads

2. **Connection Pooling**
   - Maintain persistent Telegram API connections
   - HTTP/2 for better multiplexing
   - Connection reuse for downloads

3. **Parallel Processing**
   - Multiple queue workers for uploads
   - Async streaming for downloads
   - Background metadata persistence

4. **CDN Integration**
   - Cloudflare/nginx reverse proxy
   - Static asset caching
   - Geographic distribution

---

## Scalability Patterns

### Horizontal Scaling

```mermaid
graph TB
    LB[Load Balancer<br/>nginx/Cloudflare]
    
    subgraph "Application Tier"
        APP1[FilesLink Instance 1]
        APP2[FilesLink Instance 2]
        APP3[FilesLink Instance N]
    end
    
    subgraph "Shared Storage"
        FS[Shared file_mappings.json<br/>NFS/S3]
        TG[Telegram Storage Channel]
    end
    
    LB --> APP1
    LB --> APP2
    LB --> APP3
    
    APP1 --> FS
    APP2 --> FS
    APP3 --> FS
    
    APP1 --> TG
    APP2 --> TG
    APP3 --> TG
    
    style LB fill:#3F51B5,color:#fff
    style FS fill:#9C27B0,color:#fff
    style TG fill:#FF9800,color:#fff
```

### Bottleneck Analysis

| Component | Bottleneck | Solution |
|-----------|------------|----------|
| Queue Processing | Sequential | Multiple workers |
| Telegram API | Rate limits | Request queuing + backoff |
| Metadata Storage | File I/O | Redis/Database |
| Download Bandwidth | Server uplink | CDN + caching |

---

## Deployment Architecture

### Production Setup

```mermaid
graph TB
    subgraph "Internet"
        USER[Users]
        CF[Cloudflare CDN]
    end
    
    subgraph "Edge Layer"
        NGINX[Nginx Reverse Proxy<br/>SSL Termination]
    end
    
    subgraph "Application Layer"
        SYSTEMD[Systemd Service]
        APP[FilesLink Application]
    end
    
    subgraph "Storage Layer"
        LOCAL[file_mappings.json]
        BACKUP[Automated Backups<br/>S3/rsync]
    end
    
    subgraph "External Services"
        TG_API[Telegram API]
        TG_STORAGE[Storage Channel]
    end
    
    USER --> CF
    CF --> NGINX
    NGINX --> APP
    SYSTEMD --> APP
    APP --> LOCAL
    LOCAL --> BACKUP
    APP --> TG_API
    APP --> TG_STORAGE
    
    style CF fill:#FF9800,color:#fff
    style NGINX fill:#4CAF50,color:#fff
    style APP fill:#2196F3,color:#fff
```

---

## Monitoring and Observability

### Key Metrics

```mermaid
graph LR
    subgraph "Business Metrics"
        M1[Files Uploaded/day]
        M2[Downloads/day]
        M3[Storage Usage]
    end
    
    subgraph "System Metrics"
        M4[Response Time]
        M5[Error Rate]
        M6[Queue Depth]
    end
    
    subgraph "Infrastructure Metrics"
        M7[CPU Usage]
        M8[Memory Usage]
        M9[Network I/O]
    end
    
    M1 --> DASH[Dashboard]
    M2 --> DASH
    M3 --> DASH
    M4 --> DASH
    M5 --> DASH
    M6 --> DASH
    M7 --> DASH
    M8 --> DASH
    M9 --> DASH
    
    style DASH fill:#673AB7,color:#fff
```

### Health Checks

- Bot connectivity to Telegram
- Storage channel accessibility
- HTTP server responsiveness  
- file_mappings.json read/write
- Disk space availability
- Queue processing status

---

## Disaster Recovery

### Backup Strategy

```mermaid
graph TB
    PRIMARY[Primary System]
    
    subgraph "Critical Data"
        JSON[file_mappings.json]
        ENV[.env configuration]
    end
    
    subgraph "Backup Destinations"
        S3[S3 Bucket<br/>Daily]
        GIT[Git Repository<br/>On Change]
        LOCAL_BACKUP[Local Backup<br/>Hourly]
    end
    
    PRIMARY --> JSON
    PRIMARY --> ENV
    
    JSON --> S3
    JSON --> GIT
    JSON --> LOCAL_BACKUP
    
    ENV --> S3
    ENV --> GIT
    
    style JSON fill:#F44336,color:#fff
    style ENV fill:#F44336,color:#fff
```

### Recovery Procedures

| Scenario | Impact | Recovery Time | Steps |
|----------|--------|---------------|-------|
| file_mappings.json loss | Links broken | 5 min | Restore from backup |
| Storage channel deleted | All files lost | N/A | Recreate, start fresh |
| Bot token compromised | Security risk | 10 min | Revoke, update, restart |
| Server crash | Service down | 2 min | Auto-restart (systemd) |
| Database corruption | Partial data loss | 30 min | Restore, validate integrity |

---

## Future Enhancements


### Planned Features

- [ ] **Caching System**: Redis for hot files
- [ ] **Admin Dashboard**: Web UI for management
- [ ] **File Expiration**: Auto-delete after N days
- [ ] **Compression**: Compress before upload
- [ ] **Encryption**: E2E encryption option
- [ ] **Multiple Channels**: Load balancing across channels
- [ ] **Statistics**: Download tracking and analytics
- [ ] **Rate Limiting**: Per-user upload/download limits
- [ ] **Search**: Full-text file search
- [ ] **Tags**: File categorization system

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines and architecture decisions.

## License

This project architecture is documented under the same license as the project.

---

**Document Version**: 2.0  
**Last Updated**: October 2025  
**Author**: FilesLink Development Team
