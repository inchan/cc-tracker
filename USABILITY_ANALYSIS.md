# Prompt Tracking System - Usability Analysis Report

## Executive Summary

The Prompt Tracking System is a well-structured Rust application with solid foundations, but has several practical usability issues that would hinder real-world deployment. Critical issues include incomplete watch mode implementation, inefficient polling patterns, and missing input validation. The codebase has good error handling practices overall but lacks complete documentation.

---

## 1. CLI USABILITY

### 1.1 Watch Mode - Incomplete Implementation (CRITICAL)

**Files:** `src/cli/main.rs` lines 976-1034
**Severity:** CRITICAL

**Issue:** The `cmd_watch` function accepts `category` and `tags` parameters but completely ignores them.

```rust
fn cmd_watch(
    db: &Database,
    config: &Config,
    watch_dir: Option<PathBuf>,
    _category: Option<String>,  // <-- UNUSED (line 980)
    _tags: Option<String>,       // <-- UNUSED (line 981)
) -> Result<(), String> {
    // Parameters are never used in function body
    // ...
}
```

**Impact:** Users cannot apply categories or tags to automatically captured prompts, breaking a major feature workflow.

**Recommendation:** Either remove these parameters from CLI or implement the functionality by:
- Extracting prompts with pre-set category/tags
- Applying automatic tag enrichment to captured files

---

### 1.2 Hardcoded 100ms Polling Interval (CRITICAL)

**File:** `src/cli/main.rs` line 1032
**Severity:** CRITICAL

```rust
loop {
    let captured_ids = watcher.process_events(db)...?;
    // ...
    std::thread::sleep(std::time::Duration::from_millis(100));  // HARDCODED
}
```

**Issues:**
- Creates busy-waiting pattern: 10 iterations per second
- Not configurable, wasting CPU cycles when idle
- Single-threaded blocking: freezes entire application

**Impact:** 
- High CPU usage on systems with many watched files
- Unresponsive CLI during watch mode (Ctrl+C lag)
- Not suitable for daemon/background operation

**Recommendation:** 
- Make polling interval configurable (default to 2-5 seconds)
- Consider event-driven architecture (notify crate already supports this)
- Add `--poll-interval` CLI option

---

### 1.3 Missing Input Validation (HIGH)

**File:** `src/cli/main.rs` line 161, 840
**Severity:** HIGH

```rust
Trends {
    #[arg(short, long, default_value = "30")]
    days: i32,  // <-- Can be negative, zero, or unreasonably large
    // ...
}
```

The `days` parameter accepts any i32 value without validation:
- Negative values produce confusing results
- Zero returns empty results
- Very large values (e.g., 999999) waste database resources

**Affected Commands:**
- `trends` (days parameter)
- `list` (limit parameter could be 0)
- `search` (limit parameter could be 0)
- `report` (no validation of report_type/format combinations)

**Example Validation Gap:**

```rust
fn cmd_trends(db: &Database, days: i32, show_categories: bool) -> Result<(), String> {
    // No validation that days > 0
    let trends = db.get_daily_trends(days)?;
    // ...
}
```

**Recommendation:** Add validation:
```rust
if days <= 0 {
    return Err("Days must be positive".to_string());
}
if days > 36500 { // ~100 years
    return Err("Days must be reasonable (max 36500)".to_string());
}
```

---

### 1.4 Report Format/Type Not Validated (MEDIUM)

**File:** `src/cli/main.rs` lines 593-602
**Severity:** MEDIUM

```rust
fn cmd_report(db: &Database, report_type: &str, format: &str, output: Option<PathBuf>) -> Result<(), String> {
    let rtype = match report_type.to_lowercase().as_str() {
        "weekly" => ReportType::Weekly,
        "monthly" => ReportType::Monthly,
        _ => return Err(format!("Invalid report type: {}...", report_type)),  // User-friendly error
    };
    
    let rformat = format.parse()  // But format uses FromStr trait
        .map_err(|_| format!("Invalid format: {}...", format))?;
```

**Issue:** Inconsistent error handling between manual match and FromStr trait. Should be consistent.

---

### 1.5 CLI Output Clarity Issues (MEDIUM)

**File:** `src/cli/main.rs` - Multiple locations
**Severity:** MEDIUM

**Issue:** 117 println!/eprintln! calls with no control over verbosity

**Problems:**
- No `--quiet` or `--verbose` flags
- No log level control (no use of logging library)
- All output goes to stdout/stderr, not structured

**Example:** `cmd_watch` output (line 993-996)
```rust
println!("Starting file watcher...");
println!("Watching directory: {}", watch_path.display());
println!("File extensions: .txt, .md, .prompt");
println!("Press Ctrl+C to stop.\n");
```

These are hardcoded and cannot be suppressed even with `--quiet`.

**Recommendation:** Add logging integration:
```bash
RUST_LOG=info prompt-tracking watch  # Shows important info
RUST_LOG=warn prompt-tracking watch  # Only warnings/errors
```

---

## 2. PERFORMANCE CONCERNS

### 2.1 N+1 Query Pattern in Report Generation (HIGH)

**File:** `src/cli/main.rs` lines 614-619
**Severity:** HIGH

```rust
// Get all data
let prompts = db.list_prompts(&PromptFilter::default())?;  // Query 1: All prompts

let quality_scores = db.get_all_quality_scores()?;  // Query 2: All quality scores

// Get efficiency metrics (we need to collect them)
let mut efficiency_metrics = Vec::new();
for prompt in &prompts {  // N+1: One query per prompt!
    if let Ok(Some(metrics)) = db.get_efficiency_metrics(&prompt.id) {
        efficiency_metrics.push(metrics);
    }
}
```

**Impact:** For 1000 prompts = 1003 database queries instead of ~3

**Database Method:** `src/core/database/mod.rs` lines 521-551
```rust
pub fn get_efficiency_metrics(&self, prompt_id: &str) -> Result<Option<EfficiencyMetrics>> {
    // Executes a query for EVERY prompt in the loop above
    let mut stmt = self.conn.prepare(...)?;
    stmt.query_row(params![prompt_id], ...)?;
}
```

**Recommendation:** Add batch query method:
```rust
pub fn get_all_efficiency_metrics(&self) -> Result<Vec<EfficiencyMetrics>> {
    // Single query returns all metrics
}
```

---

### 2.2 Database Tag Fetching - N+1 Pattern (HIGH)

**File:** `src/core/database/mod.rs` lines 334-338
**Severity:** HIGH

```rust
let prompts: Vec<Prompt> = stmt.query_map(...).collect()?;

// Get tags for each prompt (N+1 query pattern)
let mut result = Vec::new();
for mut prompt in prompts {
    prompt.tags = self.get_tags_for_prompt(&prompt.id)?;  // One query per prompt!
    result.push(prompt);
}
```

Each `get_tags_for_prompt` call (lines 654-679) executes a separate SQL query. For listing 100 prompts = 101 queries.

**Recommendation:** Use SQL JOIN to fetch all tags in one query:
```sql
SELECT p.id, p.content, ..., GROUP_CONCAT(t.name) as tags
FROM prompts p
LEFT JOIN prompt_tags pt ON p.id = pt.prompt_id
LEFT JOIN tags t ON pt.tag_id = t.id
GROUP BY p.id
```

---

### 2.3 Cache Lock Failure Handling (MEDIUM)

**File:** `src/core/cache/mod.rs` lines 36-44
**Severity:** MEDIUM

```rust
pub fn get(&self, key: &str) -> Option<T> {
    let data = self.data.read().ok()?;  // <-- Silently returns None on lock poison
    if let Some(entry) = data.get(key) {
        if entry.expires_at > Instant::now() {
            return Some(entry.value.clone());
        }
    }
    None
}
```

**Issue:** If RwLock becomes poisoned (thread panicked while holding lock), all subsequent cache accesses return `None`. No error reporting.

**Impact:** Silent cache degradation; no visibility into why cache isn't working.

**Recommendation:** Log or expose lock failures:
```rust
let data = self.data.read().map_err(|e| {
    eprintln!("Cache lock poisoned: {}", e);
    e
}).ok()?;
```

---

### 2.4 Watcher Duplicate Detection Inefficiency (MEDIUM)

**File:** `src/core/watcher/mod.rs` lines 128-133
**Severity:** MEDIUM

```rust
// Inside handle_event - called for EVERY file change
match self.capture_service.capture_from_file(path) {
    Ok(prompt) => {
        if db.find_by_hash(&prompt.content_hash)?.is_some() {
            continue;  // Skip if duplicate
        }
        db.create_prompt(&prompt)?;
    }
}
```

**Issue:** Calls `find_by_hash()` for every captured file, even if most are new. Database lookup overhead.

**Better Approach:**
- Add configuration option for duplicate checking
- Use content hash indexing (already should be indexed)
- Consider batch insert with duplicate handling

---

## 3. ERROR HANDLING QUALITY

### 3.1 Silent Error Suppression in Watch Mode (HIGH)

**File:** `src/cli/main.rs` line 139, `src/core/watcher/mod.rs` line 139
**Severity:** HIGH

```rust
// In watcher handle_event
match self.capture_service.capture_from_file(path) {
    Ok(prompt) => { ... }
    Err(_) => continue,  // <-- ERROR SILENTLY IGNORED
}
```

**Problems:**
- Users don't know why files aren't being captured
- No distinction between file read errors vs. parse errors
- Makes debugging impossible

**User-Visible Impact:**
```
$ prompt-tracking capture --watch ./my-prompts/
Starting file watcher...
Watching directory: ./my-prompts/
# User adds file: permission_denied_file.txt
# Nothing happens - no error message
# User has no idea why
```

**Recommendation:** Log errors with context:
```rust
Err(e) => {
    eprintln!("Failed to capture from {}: {}", path.display(), e);
    continue;
}
```

Or with structured logging:
```rust
Err(e) => {
    log::error!("Capture failed for {}: {}", path.display(), e);
    continue;
}
```

---

### 3.2 Config Loading Error Handling (MEDIUM)

**File:** `src/cli/main.rs` lines 224-231
**Severity:** MEDIUM

```rust
let config = if let Some(path) = &cli.config {
    Config::load(path).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config: {}. Using defaults.", e);
        Config::default()
    })
} else {
    Config::default()
};
```

**Issues:**
- Silently falls back to defaults if config file is invalid
- User might think their config was applied
- No indication which settings reverted to defaults
- Typos in config file go unnoticed

**Better Approach:**
```rust
let config = if let Some(path) = &cli.config {
    Config::load(path).map_err(|e| {
        format!("Failed to load config from {}: {}", path.display(), e)
    })?
} else {
    Config::default()
};
```

---

### 3.3 Date Parsing Fallback (MEDIUM)

**File:** `src/core/database/mod.rs` lines 481-483, 602-607
**Severity:** MEDIUM

```rust
chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
    .map(|dt| dt.with_timezone(&chrono::Utc))
    .unwrap_or_else(|_| chrono::Utc::now())  // <-- Silently uses current time on parse error
```

**Issues:**
- If stored datetime is invalid, silently uses current time
- Data corruption goes unnoticed
- Next `select` returns wrong timestamp

**Impact:** Historical data becomes useless if date parsing fails.

**Recommendation:** Return error instead of silent fallback:
```rust
chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
    .map(|dt| dt.with_timezone(&chrono::Utc))
    .map_err(|e| PromptTrackingError::DatabaseError(format!("Invalid date: {}", e)))?
```

---

## 4. DOCUMENTATION STATUS

### 4.1 Incomplete Docstrings (HIGH)

**File:** `src/core/database/mod.rs`
**Severity:** HIGH

Public API documentation:
- 27 public functions
- Only 1 doc comment found (`///`)
- **Documentation coverage: 3.7%**

**Example:** Undocumented functions
```rust
// No docs - what does this do? What if hash doesn't exist?
pub fn find_by_hash(&self, hash: &str) -> Result<Option<Prompt>> { ... }

// No docs - what happens if prompt doesn't exist?
pub fn archive_prompt(&self, id: &str) -> Result<()> { ... }

// No docs - is this destructive? Is there a timeout?
pub fn restore_version(&self, prompt_id: &str, version: i32) -> Result<Prompt> { ... }
```

**File:** `src/core/config/mod.rs`
**Severity:** MEDIUM
- 8 public structs
- Only 6 doc comments
- **Coverage: 75%**

Missing docs for:
```rust
pub struct DatabaseConfig { ... }  // What are the defaults?
pub struct AnalysisConfig { ... }  // What do these weights mean?
```

**Recommendation:** Add doc comments with examples:
```rust
/// Archive a prompt by ID.
///
/// Changes the prompt status to "archived" without deleting data.
/// Archived prompts are excluded from list/search by default.
///
/// # Arguments
///
/// * `id` - The UUID of the prompt to archive
///
/// # Returns
///
/// Returns `Ok(())` if archived successfully.
/// Returns `DatabaseError` if prompt doesn't exist.
///
/// # Examples
///
/// ```no_run
/// let db = Database::new("~/.prompt-tracking/prompts.db")?;
/// db.archive_prompt("550e8400-e29b-41d4-a716-446655440000")?;
/// ```
pub fn archive_prompt(&self, id: &str) -> Result<()> { ... }
```

---

### 4.2 Missing Examples in Lib.rs (MEDIUM)

**File:** `src/core/lib.rs` lines 16-37
**Severity:** MEDIUM

Library documentation example is outdated or incomplete:
```rust
/// ```rust,no_run
/// let db = Database::new("~/.prompt-tracking/prompts.db").unwrap();
/// let service = CaptureService::default();
/// let prompt = service.process_content("Write a function...").unwrap();
/// db.create_prompt(&prompt).unwrap();
/// ```
```

**Issues:**
- Uses `unwrap()` (discouraged in examples)
- Doesn't show error handling
- Doesn't demonstrate quality analysis
- Doesn't show how to integrate with CLI

**Recommendation:** Provide complete, idiomatic example with proper error handling.

---

### 4.3 Configuration Documentation (MEDIUM)

**File:** `src/core/config/mod.rs`
**Severity:** MEDIUM

```rust
/// Capture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub watch_directory: String,  // <-- What format? Absolute? Relative?
    pub auto_capture: bool,        // <-- What does this do?
    pub deduplicate: bool,         // <-- Only in watch mode?
    pub similarity_threshold: f64,  // <-- Range? Default units?
}
```

Missing clarity on:
- What `watch_directory` accepts (absolute? relative? env vars?)
- When `auto_capture` applies
- Similarity threshold range (0.0-1.0?)
- Backup behavior details

---

## 5. REAL-WORLD INTEGRATION ISSUES

### 5.1 Database Path Expansion Not Portable (HIGH)

**File:** `src/core/config/mod.rs` lines 102-107
**Severity:** HIGH

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                path: "~/.local/share/prompt-tracking/prompts.db".to_string(),
                // ...
            },
            capture: CaptureConfig {
                watch_directory: "$HOME/.claude-code-data".to_string(),  // NOT EXPANDED!
                // ...
            },
            // ...
        }
    }
}
```

**Issues:**
1. `watch_directory` uses literal `$HOME` - environment variable NOT expanded
2. `~` is only expanded via `shellexpand::tilde()` at database initialization (line 57 in database/mod.rs)
3. No expansion for watch_directory - fails if literal "$HOME/.claude-code-data" doesn't exist

**Real-world scenario:**
```bash
$ ls -la '$HOME/.claude-code-data'  # Looks for literal directory named "$HOME"
ls: cannot access '$HOME/.claude-code-data': No such file or directory

$ prompt-tracking capture --watch
Error: Failed to create watcher: failed to watch path
```

**Recommendation:** 
1. Expand environment variables for all paths at config load time
2. Provide docs on supported path variables
3. Validate paths exist before starting watch mode

```rust
pub fn watch_directory(&self) -> Result<PathBuf> {
    let expanded = shellexpand::tilde(&self.capture.watch_directory).to_string();
    let expanded = shellexpand::env(&expanded).unwrap_or_else(|_| expanded.into());
    Ok(PathBuf::from(expanded))
}
```

---

### 5.2 Hardcoded Directory Assumptions (MEDIUM)

**File:** `src/core/config/mod.rs` line 102
**Severity:** MEDIUM

Default config assumes standard Linux directories:
```rust
path: "~/.local/share/prompt-tracking/prompts.db".to_string()
```

**Issues:**
- Windows: Should use `%APPDATA%` or similar
- macOS: Should use `~/Library/Application Support/`
- Container/CI: These directories might be read-only
- NixOS: Different filesystem layout entirely

**Recommendation:** Use `dirs` crate properly (already imported):
```rust
pub fn default() -> Self {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("prompt-tracking");
    
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("prompt-tracking");
    
    Self {
        database: DatabaseConfig {
            path: data_dir.join("prompts.db").to_string_lossy().to_string(),
            // ...
        },
        // ...
    }
}
```

---

### 5.3 Watch Mode Not Suitable for Background Daemon (HIGH)

**File:** `src/cli/main.rs` lines 976-1034
**Severity:** HIGH

Current design makes watch mode unsuitable for daemon operation:

**Problems:**
1. Blocks in infinite loop (line 1006)
2. Hardcoded sleep(100ms) means no ability to gracefully shutdown
3. Thread-blocking means CLI is frozen until Ctrl+C
4. Not background-able without wrapper script

**Real-world usage:**
```bash
# User expects this to work like systemd service
prompt-tracking capture --watch &  # Background job

# But it's just blocking - not suitable for daemon
# No signal handling, no config reload, no health checks
```

**Impact:** Cannot integrate with:
- systemd services
- Supervisor/runit
- Docker containers
- Kubernetes pods

**Recommendation:**
1. Add async/tokio support (already in dependencies)
2. Implement signal handling (SIGTERM)
3. Add `--daemonize` flag for proper background operation
4. Add health check endpoint

---

### 5.4 Auto-Analyze Configuration Not Honored in Watch Mode (MEDIUM)

**File:** `src/cli/main.rs` lines 1015-1027
**Severity:** MEDIUM

Watch mode re-analyzes prompts every time, but doesn't respect config:

```rust
if config.analysis.auto_analyze {  // <-- Only in watch mode!
    if let Ok(Some(prompt)) = db.get_prompt(&id) {
        // Auto-analysis happens here
    }
}
```

**Problem:** 
- Regular `capture` command (line 378) also has auto_analyze logic
- Inconsistent behavior between watch and manual capture
- No way to disable expensive analysis in high-volume watch scenarios

**Recommendation:** Document that auto_analyze affects both paths, or add separate `watch.auto_analyze` config.

---

### 5.5 Similarity Threshold Not Configurable in Watch Mode (MEDIUM)

**File:** `src/cli/main.rs` lines 986-991
**Severity:** MEDIUM

```rust
let watcher_config = WatcherConfig {
    watch_path: watch_path.clone(),
    recursive: true,
    file_extensions: vec!["txt".to_string(), "md".to_string(), "prompt".to_string()],
    similarity_threshold: config.capture.similarity_threshold,  // Uses global config
};
```

**Issue:** Watch mode always uses global similarity_threshold from config. No way to:
- Disable duplicate detection in watch mode
- Use different threshold for watched files
- Override per command

**Real-world scenario:**
```bash
# User wants to collect all prompts including near-duplicates for analysis
prompt-tracking capture --watch --similarity-threshold 0.5  # ERROR: not supported
```

**Recommendation:** Allow `--similarity-threshold` flag:
```rust
Capture {
    // ...
    #[arg(long)]
    similarity_threshold: Option<f64>,
}
```

---

## 6. MISSING VALIDATION & EDGE CASES

### 6.1 Empty Database Handling (LOW)

**File:** `src/cli/main.rs` lines 421-424, 518-521, 654-657
**Severity:** LOW

Multiple commands handle empty results but messaging could be clearer:

```rust
let prompts = db.list_prompts(&filter)?;

if prompts.is_empty() {
    println!("No prompts found.");
    return Ok(());
}
```

**Minor Issue:** Doesn't distinguish between:
- No data exists at all
- Filters matched nothing
- Database query failed

**Example:** User runs `list --category unknown` gets same message as empty database.

**Recommendation:** Provide context-aware messages:
```rust
if prompts.is_empty() {
    if filter.category.is_some() {
        println!("No prompts found matching category '{}'", filter.category.unwrap());
    } else if filter.tags.is_empty() {
        println!("No prompts found in database");
    } else {
        println!("No prompts found with tags: {}", filter.tags.join(", "));
    }
    return Ok(());
}
```

---

### 6.2 File Read Errors Not Detailed (MEDIUM)

**File:** `src/core/capture/mod.rs` lines 65-68
**Severity:** MEDIUM

```rust
pub fn capture_from_file(&self, path: &Path) -> Result<Prompt> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        PromptTrackingError::FileNotFound(format!("Failed to read file: {}", e))
    })?;
    // ...
}
```

**Issue:** All filesystem errors are mapped to `FileNotFound`, even:
- Permission denied
- File too large
- Path not a file (is a directory)
- Encoding errors

**Recommendation:** Create more specific error type:
```rust
pub enum CaptureError {
    FileNotFound(String),
    PermissionDenied(String),
    FileTooLarge(String),
    InvalidEncoding(String),
}
```

---

### 6.3 Large File Handling Not Addressed (MEDIUM)

**Files:** `src/core/capture/mod.rs`, `src/cli/main.rs`
**Severity:** MEDIUM

**Issue:** No checks for file size limits

Potential problems:
```rust
// Reading into memory without size check
let content = std::fs::read_to_string(path)?;  // Could be GB

// Token estimation doesn't bound output
let estimated_tokens = (content.len() as f64 / 4.0).ceil() as u32;  // u32 overflow?
```

**Real scenario:**
```bash
# User accidentally watches a directory with large log files
prompt-tracking capture --watch ./logs/
# System runs out of memory trying to read 10GB debug.log
```

**Recommendation:** Add file size limits:
```rust
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

pub fn capture_from_file(&self, path: &Path) -> Result<Prompt> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(PromptTrackingError::FileError(
            format!("File too large: {} (max {})", metadata.len(), MAX_FILE_SIZE)
        ));
    }
    // ...
}
```

---

## Summary Table

| Category | Issue | Severity | File(s) | Line(s) |
|----------|-------|----------|---------|---------|
| CLI Usability | Watch mode ignores category/tags | CRITICAL | main.rs | 980-981 |
| CLI Usability | Hardcoded 100ms polling | CRITICAL | main.rs | 1032 |
| CLI Usability | Missing input validation | HIGH | main.rs | 161, 840 |
| Performance | N+1 queries in report generation | HIGH | main.rs | 614-619 |
| Performance | N+1 tag fetching | HIGH | database/mod.rs | 334-338 |
| Error Handling | Silent error suppression in watch | HIGH | watcher/mod.rs | 139 |
| Documentation | Database docstring coverage 3.7% | HIGH | database/mod.rs | various |
| Integration | Config path expansion issues | HIGH | config/mod.rs | 102-107 |
| Integration | Watch mode not daemon-suitable | HIGH | main.rs | 976-1034 |
| Error Handling | Config loading error ignored | MEDIUM | main.rs | 224-231 |
| Performance | Cache lock poisoning ignored | MEDIUM | cache/mod.rs | 37-44 |
| Integration | Hardcoded directory assumptions | MEDIUM | config/mod.rs | 102 |
| CLI Usability | No output verbosity control | MEDIUM | main.rs | various |
| Error Handling | Date parsing fallback | MEDIUM | database/mod.rs | 481-483, 602-607 |

---

## Recommendations Priority

### Immediate (Before Production)
1. Fix watch mode to use category/tags parameters
2. Implement configurable polling interval for watch mode
3. Add input validation for numeric parameters
4. Log errors in watch mode (stop silent failures)

### Short-term (Next Sprint)
1. Fix N+1 query patterns (batch efficiency metrics query)
2. Add docstrings to public API
3. Fix config path expansion issues
4. Make watch mode async/daemon-compatible

### Medium-term (Quality)
1. Add output verbosity levels
2. Refactor error handling to be more specific
3. Add file size limits
4. Improve empty result messaging

### Nice-to-have
1. Additional performance benchmarks
2. More detailed configuration documentation
3. CLI usage examples in README
4. Integration tests for real-world scenarios

