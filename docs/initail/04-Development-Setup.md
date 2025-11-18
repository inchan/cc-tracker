# 개발 가이드

## 개발 환경 설정 단계별 가이드

### Step 1: 전제 조건 확인

**시스템 요구사항**:
- macOS 12+ / Linux (Ubuntu 20.04+) / Windows 10+ (WSL2)
- RAM: 최소 4GB, 권장 8GB 이상
- 디스크: 최소 2GB 여유 공간
- 인터넷 연결 (초기 설정만 필요)

**버전 확인**:
```bash
# macOS
system_profiler SPSoftwareDataType | grep 'System Version'

# Linux
uname -a
lsb_release -a

# Git
git --version  # 2.30 이상

# Homebrew (macOS)
brew --version
```

### Step 2: Rust 설치

```bash
# Option 1: rustup (권장)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Option 2: Homebrew (macOS)
brew install rust

# 설치 확인
rustc --version
cargo --version
rustup --version
```

**Rust 업데이트**:
```bash
rustup update  # 최신 버전으로 업데이트
rustup toolchain install stable  # 안정 버전 설치
rustup toolchain install nightly  # 야간 버전 설치 (선택)
```

**유용한 도구 설치**:
```bash
# 파일 변경 감시
cargo install cargo-watch

# 매크로 확장 보기
cargo install cargo-expand

# 의존성 트리 보기
cargo install cargo-tree

# 의존성 업데이트 확인
cargo install cargo-outdated

# 보안 감사
cargo install cargo-audit

# 코드 포맷팅 체크
cargo install cargo-fmt

# 린팅
cargo install clippy
```

### Step 3: IDE 설정 (VS Code)

**필수 확장 프로그램**:
1. **rust-analyzer** - Rust 언어 서버
   - 설정: `rust-analyzer.check.command: "clippy"`
   - 호버 정보 활성화

2. **CodeLLDB** - 디버깅
   - macOS에서 원활한 디버깅 지원

3. **Cargo** - Cargo 통합
   - 명령 실행 및 매크로 확장

4. **Better TOML** - TOML 파일 지원
5. **Even Better TOML** - 추가 기능

**VS Code 설정** (`.vscode/settings.json`):
```json
{
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.rulers": [100, 120]
  },
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.checkOnSave.overrideCommand": [
    "cargo",
    "clippy",
    "--workspace",
    "--message-format=json",
    "--all-targets",
    "--all-features"
  ],
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "editor.formatOnSave": true,
  "files.exclude": {
    "**/target": true,
    "**/.git": true
  },
  "[markdown]": {
    "editor.wordWrap": "on",
    "editor.formatOnSave": true
  }
}
```

**Launch Configuration** (`.vscode/launch.json`):
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug CLI",
      "cargo": {
        "args": [
          "build",
          "--bin=prompt-tracking",
          "--package=prompt_tracking"
        ],
        "filter": {
          "name": "prompt-tracking",
          "kind": "bin"
        }
      },
      "args": ["list"],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### Step 4: Git 저장소 설정

```bash
# 프로젝트 디렉토리 생성
mkdir -p ~/Projects/prompt-tracking-system
cd ~/Projects/prompt-tracking-system

# Git 초기화
git init
git config user.name "Your Name"
git config user.email "your.email@example.com"

# 원격 저장소 추가 (GitHub)
git remote add origin https://github.com/YOUR_USERNAME/prompt-tracking-system.git

# 초기 파일 생성 및 커밋
echo "# Prompt Tracking System" > README.md
git add README.md
git commit -m "Initial commit"

# 브랜치 이름 변경 (main으로)
git branch -M main

# 원격 저장소에 푸시
git push -u origin main
```

### Step 5: 프로젝트 구조 생성

```bash
# Cargo 프로젝트 생성
cargo init --name prompt_tracking

# 필수 폴더 생성
mkdir -p src/core/{models,database,capture,analysis,reporting,utils,config}
mkdir -p src/cli/{commands,output}
mkdir -p config/templates
mkdir -p data/{database,prompts,metrics,reports,exports}
mkdir -p tests/{unit,integration,fixtures}
mkdir -p scripts
mkdir -p .github/{workflows,ISSUE_TEMPLATE,PULL_REQUEST_TEMPLATE}
mkdir -p .vscode

# .gitkeep 파일 생성
find . -type d -empty -exec touch {}/.gitkeep \;
```

### Step 6: Cargo.toml 설정

**기본 구조**:
```toml
[package]
name = "prompt_tracking"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Enterprise-grade prompt tracking system"
repository = "https://github.com/YOUR_USERNAME/prompt-tracking-system"
license = "MIT"

[lib]
name = "prompt_tracking"
path = "src/core/lib.rs"

[[bin]]
name = "prompt-tracking"
path = "src/cli/main.rs"

[dependencies]
# CLI
clap = { version = "4.4", features = ["derive"] }

# Database
rusqlite = { version = "0.29", features = ["bundled", "chrono"] }
tokio = { version = "1.35", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
regex = "1.10"
anyhow = "1.0"
thiserror = "1.0"

# Logging
log = "0.4"
env_logger = "0.11"

# File handling
walkdir = "2.4"
notify = "6.1"

[dev-dependencies]
criterion = "0.5"
tempfile = "3.8"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## 코드 스타일 및 컨벤션

### 1. Rust 스타일 가이드

**네이밍 컨벤션**:
```rust
// 구조체: PascalCase
pub struct PromptData { }

// 메서드/함수: snake_case
pub fn calculate_quality_score() { }

// 상수: UPPER_SNAKE_CASE
pub const MAX_PROMPT_LENGTH: usize = 10000;

// 모듈: snake_case
mod data_models;
pub mod config_parser;

// 타입 파라미터: PascalCase
pub fn process<T: Serialize>() { }
```

**포맷팅**:
```bash
# 코드 포맷팅 (자동)
cargo fmt

# 린팅
cargo clippy

# 모든 경고 확인
cargo clippy -- -W clippy::all
```

**코드 길이**:
- 최대 라인 길이: 100-120자
- 함수 길이: 50줄 이하 (권장)
- 모듈 길이: 500줄 이상시 분할

### 2. 에러 처리

**에러 타입 정의**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PromptTrackingError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid prompt format")]
    InvalidFormat,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, PromptTrackingError>;
```

**에러 처리 패턴**:
```rust
// 좋은 예
fn process_prompt(content: &str) -> Result<Prompt> {
    let validated = validate_content(content)?;
    let parsed = parse_prompt(&validated)?;
    Ok(parsed)
}

// 나쁜 예 (피할 것)
fn process_prompt(content: &str) -> Result<Prompt> {
    match validate_content(content) {
        Ok(validated) => match parse_prompt(&validated) {
            Ok(parsed) => Ok(parsed),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}
```

### 3. 주석 및 문서화

**Doc Comments**:
```rust
/// 프롬프트 데이터를 데이터베이스에 저장합니다.
///
/// # Arguments
/// * `prompt` - 저장할 프롬프트 데이터
/// * `db_path` - 데이터베이스 경로
///
/// # Returns
/// 저장된 프롬프트의 ID를 반환합니다.
///
/// # Errors
/// 데이터베이스 오류 발생시 에러를 반환합니다.
///
/// # Examples
/// ```
/// let id = save_prompt(&prompt, "prompts.db")?;
/// println!("Saved prompt: {}", id);
/// ```
pub fn save_prompt(prompt: &Prompt, db_path: &str) -> Result<String> {
    // ...
}
```

**내부 주석**:
```rust
// 복잡한 알고리즘 설명 (여러 줄)
// 각 단계를 명확히 설명
// 왜 이렇게 구현했는지 이유 포함
```

---

## 테스트 작성 가이드

### 1. 단위 테스트 (Unit Tests)

**테스트 구조**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_quality_score_valid() {
        let prompt = Prompt {
            content: "Valid prompt".to_string(),
            // ... other fields
        };
        
        let score = calculate_quality_score(&prompt);
        assert!(score >= 0.0 && score <= 100.0);
    }

    #[test]
    #[should_panic(expected = "invalid prompt")]
    fn test_invalid_prompt_panics() {
        let invalid = "";
        validate_prompt(invalid).unwrap();
    }
}
```

**테스트 명명 규칙**:
- `test_function_name_condition`
- `test_save_prompt_valid_input`
- `test_calculate_score_empty_prompt`
- `test_database_connection_fails`

### 2. 통합 테스트 (Integration Tests)

**위치**: `tests/` 디렉토리

```rust
// tests/integration_tests.rs
use prompt_tracking::*;

#[test]
fn test_full_capture_flow() {
    // 1. 프롬프트 캡처
    let prompt = capture_prompt("test_prompt.txt").unwrap();
    
    // 2. 데이터 검증
    assert!(!prompt.content.is_empty());
    
    // 3. 데이터베이스 저장
    let id = save_to_db(&prompt).unwrap();
    
    // 4. 데이터 검색
    let retrieved = fetch_from_db(&id).unwrap();
    assert_eq!(prompt.content, retrieved.content);
}
```

### 3. 테스트 실행

```bash
# 모든 테스트 실행
cargo test

# 특정 테스트 실행
cargo test test_save_prompt

# 테스트 출력 표시
cargo test -- --nocapture

# 벤치마크 실행
cargo test --release

# 테스트 커버리지 (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

---

## 디버깅 팁

### 1. 로깅 활성화

```bash
# 환경변수로 로그 레벨 설정
RUST_LOG=debug cargo run

# 모듈별 로깅
RUST_LOG=prompt_tracking::database=debug cargo run

# 모든 로그 출력
RUST_LOG=trace cargo run
```

### 2. 디버거 사용

```bash
# VS Code에서 디버깅
# - F5 또는 Run > Start Debugging
# - 중단점 설정
# - Watch 창에서 변수 모니터링

# lldb 직접 사용
lldb ./target/debug/prompt-tracking
(lldb) run list
(lldb) breakpoint set -n calculate_quality_score
(lldb) continue
```

### 3. 프로파일링

```bash
# Flamegraph 생성 (성능 분석)
cargo install flamegraph
cargo flamegraph

# 메모리 프로파일링
cargo install valgrind
valgrind ./target/debug/prompt-tracking
```

---

## 커밋 메시지 가이드

**형식**: `type(scope): subject`

**Types**:
- `feat`: 새로운 기능
- `fix`: 버그 수정
- `docs`: 문서 변경
- `style`: 코드 스타일 변경 (형식, 세미콜론 등)
- `refactor`: 코드 리팩토링
- `perf`: 성능 개선
- `test`: 테스트 추가/변경
- `chore`: 빌드, 의존성 등

**예시**:
```
feat(capture): add automatic prompt detection

Add file watcher to automatically detect and capture prompts
from Claude Code. Implements SHA-256 hashing for duplicate detection.

Fixes #42
```

---

## 성능 프로파일링

### 1. 벤치마크 작성

```rust
#[bench]
fn bench_save_prompt(b: &mut Bencher) {
    let prompt = create_test_prompt();
    
    b.iter(|| {
        save_prompt(&prompt)
    });
}
```

**벤치마크 실행**:
```bash
cargo bench
```

### 2. 메모리 사용량 분석

```bash
# macOS
/usr/bin/time -lp ./target/release/prompt-tracking list

# Linux
/usr/bin/time -v ./target/release/prompt-tracking list
```

---

## 공통 문제 해결

### 문제 1: 컴파일 오류
```bash
# 캐시 제거
cargo clean

# 다시 빌드
cargo build

# 린팅 및 체크
cargo check
cargo clippy
```

### 문제 2: 테스트 실패
```bash
# 상세 출력으로 실행
cargo test test_name -- --nocapture

# 로그 활성화
RUST_LOG=debug cargo test
```

### 문제 3: 의존성 문제
```bash
# 의존성 업데이트 확인
cargo outdated

# 의존성 충돌 해결
cargo tree

# 보안 감사
cargo audit
```

---

## 다음 단계

1. [[Timeline & Milestones]] - 개발 타임라인 확인
2. [[Database Schema]] - 데이터베이스 구현 시작
3. [[Technical Decisions]] - 기술 선택 검토

---

마지막 업데이트: 2025-11-18
