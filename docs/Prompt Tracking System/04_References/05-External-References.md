# 외부 레퍼런스 모음

## 🔗 공식 문서 (Official Documentation)

### Claude Code
- **공식 페이지**: https://www.anthropic.com/claude-code
- **API 문서**: https://docs.anthropic.com
- **모델 정보**: Claude 3.5 Sonnet
- **최신 뉴스**: https://anthropic.com/news

**수집 항목**:
- [ ] 모델 스펙 및 토큰 제한
- [ ] API 엔드포인트
- [ ] 환경 변수 설정
- [ ] 에러 핸들링 가이드
- [ ] 레이트 제한

---

### Rust
- **공식 사이트**: https://www.rust-lang.org
- **Rust Book**: https://doc.rust-lang.org/book/
- **Rustlings**: https://github.com/rust-lang/rustlings (학습 자료)
- **Rust By Example**: https://doc.rust-lang.org/rust-by-example/

**주요 주제**:
- Ownership and Borrowing
- Error Handling (Result, Option)
- Trait and Generics
- Async/Await

---

### Tokio (비동기 런타임)
- **문서**: https://tokio.rs/tokio/tutorial
- **API Docs**: https://docs.rs/tokio/
- **GitHub**: https://github.com/tokio-rs/tokio

**학습 자료**:
- [ ] Tasks and Spawning
- [ ] Channels (MPSC)
- [ ] Timers
- [ ] Runtime Configuration

---

### SQLite
- **공식 문서**: https://www.sqlite.org/docs.html
- **CLI Tutorial**: https://www.sqlite.org/cli.html
- **Best Practices**: https://www.sqlite.org/bestpractices.html

**주요 주제**:
- 스키마 설계
- 인덱싱 전략
- 쿼리 최적화
- 트랜잭션 관리
- 백업 및 복구

---

## 📚 Rust 라이브러리 가이드

### CLI 프레임워크: clap

**리소스**:
- GitHub: https://github.com/clap-rs/clap
- 문서: https://docs.rs/clap/
- 예제: https://github.com/clap-rs/clap/tree/master/examples

**주요 기능**:
- Derive macros
- Subcommands
- Argument validation
- Help formatting

**사용 예시**:
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "prompt-tracking")]
#[command(about = "Prompt tracking CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List { #[arg(long)] limit: Option<usize> },
    Capture { content: String },
    Analyze { id: String },
}
```

---

### 데이터베이스: rusqlite

**리소스**:
- GitHub: https://github.com/rusqlite/rusqlite
- 문서: https://docs.rs/rusqlite/
- 예제: https://github.com/rusqlite/rusqlite/tree/master/examples

**주요 기능**:
- Type-safe queries
- Bundled SQLite
- Transactions
- Connection pooling

**사용 예시**:
```rust
use rusqlite::Connection;

fn create_prompt(conn: &Connection, content: &str) -> Result<String> {
    let mut stmt = conn.prepare(
        "INSERT INTO prompts (content) VALUES (?1) RETURNING id"
    )?;
    let id = stmt.query_row([content], |row| row.get(0))?;
    Ok(id)
}
```

---

### 직렬화: serde

**리소스**:
- 문서: https://docs.rs/serde/
- Serde Book: https://serde.rs
- 데이터 포맷: https://serde.rs/#data-formats

**지원 포맷**:
- serde_json (JSON)
- serde_yaml (YAML)
- serde_toml (TOML)
- bincode (바이너리)

**사용 예시**:
```rust
use serde::{Serialize, Deserialize};
use serde_yaml;

#[derive(Serialize, Deserialize, Debug)]
pub struct Prompt {
    pub id: String,
    pub content: String,
    pub created_at: String,
}

// YAML로 직렬화
let prompt = Prompt { /* ... */ };
let yaml = serde_yaml::to_string(&prompt)?;

// YAML에서 역직렬화
let parsed: Prompt = serde_yaml::from_str(&yaml)?;
```

---

### 에러 처리: thiserror & anyhow

**thiserror**:
- 문서: https://docs.rs/thiserror/
- 구조화된 에러 타입 정의

**anyhow**:
- 문서: https://docs.rs/anyhow/
- 동적 에러 처리

**사용 패턴**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PromptError {
    #[error("Invalid format")]
    InvalidFormat,
    
    #[error("Database error: {0}")]
    DbError(#[from] rusqlite::Error),
}
```

---

### 테스트: criterion & proptest

**criterion** (벤치마크):
- 문서: https://docs.rs/criterion/
- 용도: 성능 측정

**proptest** (속성 기반 테스트):
- 문서: https://docs.rs/proptest/
- 용도: 무작위 입력으로 테스트

---

## 🎯 학습 자료

### Rust 심화
1. **Traits & Generics**
   - https://doc.rust-lang.org/book/ch10-00-generic-types-values-and-functions.html
   - https://doc.rust-lang.org/book/ch19-03-advanced-traits.html

2. **Macro System**
   - https://doc.rust-lang.org/book/ch19-06-macros.html
   - https://danielkeep.github.io/tlborm/book/

3. **Async Programming**
   - https://rust-lang.github.io/async-book/
   - https://tokio.rs/tokio/tutorial

---

### 소프트웨어 설계
1. **Design Patterns in Rust**
   - https://rust-lang.github.io/api-guidelines/
   - https://github.com/rust-unofficial/patterns

2. **Architecture Patterns**
   - Hexagonal Architecture (Ports & Adapters)
   - CQRS (Command Query Responsibility Segregation)
   - Event Sourcing

3. **데이터베이스 설계**
   - Third Normal Form (3NF)
   - 인덱싱 전략
   - 쿼리 최적화

---

## 🛠️ 개발 도구

### 필수 도구
| 도구 | 링크 | 용도 |
|------|------|------|
| rustup | https://rustup.rs/ | Rust 버전 관리 |
| cargo | https://doc.rust-lang.org/cargo/ | 패키지 매니저 |
| rust-analyzer | https://rust-analyzer.github.io/ | LSP 구현 |
| clippy | https://github.com/rust-lang/rust-clippy | 린터 |
| rustfmt | https://rust-lang.github.io/rustfmt/ | 코드 포매터 |

### 추가 도구
| 도구 | 설치 | 용도 |
|------|------|------|
| cargo-watch | `cargo install cargo-watch` | 파일 감시 |
| cargo-tree | `cargo install cargo-tree` | 의존성 시각화 |
| cargo-audit | `cargo install cargo-audit` | 보안 감사 |
| cargo-expand | `cargo install cargo-expand` | 매크로 확장 |
| flamegraph | `cargo install flamegraph` | 성능 분석 |

---

## 📖 커뮤니티 리소스

### 포럼 및 토론
- **Rust Users Forum**: https://users.rust-lang.org/
- **r/rust**: https://www.reddit.com/r/rust/
- **Stack Overflow**: https://stackoverflow.com/questions/tagged/rust
- **GitHub Discussions**: 각 저장소의 Discussions 탭

### 블로그 및 뉴스
- **This Week in Rust**: https://this-week-in-rust.org/
- **Rust Blog**: https://blog.rust-lang.org/
- **Tokio Blog**: https://tokio.rs/blog/
- **Inside Rust**: https://blog.rust-lang.org/inside-rust/

### 비디오 및 팟캐스트
- **RustConf Videos**: https://www.youtube.com/user/RustConferences
- **Rustacean Station**: https://rustacean-station.org/ (팟캐스트)
- **Tokio Tutorial Videos**: YouTube Tokio 채널

---

## 🔍 조사 결과 요약

### Claude Code 통합
**현황**:
- Claude Code는 로컬 파일 시스템에 프롬프트 저장
- 파일 감시 방식으로 접근 가능
- Phase 2에서 API 직접 통합 검토

**권장사항**:
- 초기: 파일 시스템 기반 감시
- 장기: MCP (Model Context Protocol) 활용

---

### 성능 고려사항
**SQLite**:
- 최대 100,000개 레코드까지 충분한 성능
- 적절한 인덱싱으로 조회 속도 < 500ms 달성 가능
- WAL (Write-Ahead Logging) 활용 추천

**Rust**:
- Tokio로 비동기 처리 가능
- 메모리 오버헤드 최소 (C/C++ 수준)
- 컴파일 타임 최적화로 런타임 성능 극대화

---

### 라이브러리 선택 근거

| 선택 | 대안 | 사유 |
|------|------|------|
| clap | structopt, argh | 가장 기능이 풍부하고 문서가 좋음 |
| rusqlite | sqlx | 번들된 SQLite, 간단한 API |
| serde | 기타 직렬화 | 생태계 표준, 타입 안전 |
| tokio | async-std | 커뮤니티 규모, 생태계 풍부 |
| thiserror | custom Error | 매크로로 간편한 구현 |

---

## 📝 다음 단계

1. [[Claude Code Setup]] - Claude Code 환경 구성
2. [[Database Schema]] - 데이터베이스 설계 검토
3. [[Rust Tips & Tricks]] - Rust 개발 팁 수집

---

마지막 업데이트: 2025-11-18
