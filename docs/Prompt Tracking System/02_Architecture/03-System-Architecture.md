# 시스템 아키텍처

## 고수준 아키텍처 개요

```
┌─────────────────────────────────────────────────────┐
│           Claude Code (로컬 환경)                  │
│        (프롬프트 입출력 포인트)                   │
└────────────────┬────────────────────────────────────┘
                 │ 감시 (File Watcher)
                 ▼
┌─────────────────────────────────────────────────────┐
│      프롬프트 캡처 레이어                          │
│  • 파일 감시                                       │
│  • 프롬프트 파싱                                   │
│  • 메타데이터 추출                                 │
└────────────────┬────────────────────────────────────┘
                 │ Parsed Prompt + Metadata
                 ▼
┌─────────────────────────────────────────────────────┐
│      데이터 처리 레이어                            │
│  • 중복 검사                                       │
│  • 데이터 검증                                     │
│  • 정규화                                          │
└────────────────┬────────────────────────────────────┘
                 │ Validated Data
                 ▼
┌─────────────────────────────────────────────────────┐
│      데이터 저장소 레이어 (SQLite)                │
│  • prompts 테이블                                 │
│  • metrics 테이블                                 │
│  • feedback 테이블                                │
│  • tags 테이블                                    │
└────────────────┬────────────────────────────────────┘
                 │
    ┌────────────┼────────────┐
    ▼            ▼            ▼
 ┌──────┐   ┌─────────┐  ┌──────────┐
 │분석  │   │보고서   │  │내보내기  │
 │엔진  │   │생성기   │  │          │
 └──────┘   └─────────┘  └──────────┘
    │            │            │
    └────────────┼────────────┘
                 ▼
┌─────────────────────────────────────────────────────┐
│      CLI 인터페이스                                │
│  • 커맨드 파싱                                     │
│  • 출력 포맷팅                                     │
│  • 사용자 상호작용                                 │
└─────────────────────────────────────────────────────┘
```

---

## 계층별 아키텍처 (Layered Architecture)

### 1. 프레젠테이션 계층 (CLI Layer)
**책임**: 사용자 명령 해석 및 결과 표시

**컴포넌트**:
- Command Parser (clap 기반)
- Output Formatter (Markdown, JSON, Table)
- Error Handler

**주요 기능**:
- 커맨드 라우팅
- 인자 파싱 및 검증
- 결과 포맷팅
- 사용자 피드백

### 2. 비즈니스 로직 계층 (Core Layer)
**책임**: 핵심 비즈니스 로직 구현

**컴포넌트**:
- **Capture Module**: 프롬프트 감지 및 파싱
- **Analysis Module**: 품질/효율성 분석
- **Reporting Module**: 보고서 생성
- **Config Module**: 설정 관리

### 3. 데이터 접근 계층 (DAL)
**책임**: 데이터베이스 작업 추상화

**컴포넌트**:
- Database Connection Pool
- DAO (Data Access Objects)
- Query Builder
- Migration Manager

### 4. 데이터 저장소 계층 (Data Layer)
**책임**: 물리적 데이터 저장

**컴포넌트**:
- SQLite Database
- File System (Backup, Export)
- Configuration Files

---

## 주요 컴포넌트 상세

### Capture Module

```
Input: Claude Code Process
   │
   ├─ File Watcher
   │  └─ Detect file changes
   │
   ├─ Prompt Parser
   │  ├─ Extract content
   │  ├─ Parse format
   │  └─ Normalize text
   │
   └─ Metadata Extractor
      ├─ Timestamp
      ├─ Model name
      ├─ Token count
      └─ Context info
   
Output: ParsedPrompt struct
```

**Capture Flow**:
```rust
1. File change event detected
2. Read file content
3. Parse as prompt
4. Extract metadata
5. Validate format
6. Calculate hash
7. Check for duplicates
8. Create PromptData
9. Pass to Database layer
```

### Analysis Module

```
Input: Stored Prompt
   │
   ├─ Quality Analyzer
   │  ├─ Clarity check
   │  ├─ Completeness check
   │  ├─ Specificity check
   │  └─ Guidance check
   │
   ├─ Efficiency Analyzer
   │  ├─ Token count analysis
   │  ├─ Cost calculation
   │  └─ Execution time analysis
   │
   └─ Trend Analyzer
      ├─ Historical comparison
      ├─ Improvement rate
      └─ Pattern detection

Output: AnalysisResult struct
```

**Analysis Pipeline**:
```rust
1. Fetch prompt from DB
2. Run quality analysis
3. Run efficiency analysis
4. Calculate trend
5. Generate recommendations
6. Store results
7. Return AnalysisResult
```

### Reporting Module

```
Input: Analysis Results
   │
   ├─ Weekly Report Generator
   │  ├─ Collect data
   │  ├─ Calculate aggregates
   │  └─ Render template
   │
   ├─ Monthly Report Generator
   │  ├─ Deep analysis
   │  ├─ Trend calculation
   │  └─ Render template
   │
   └─ Format Converters
      ├─ Markdown formatter
      ├─ HTML formatter
      ├─ CSV formatter
      └─ JSON formatter

Output: Report File
```

---

## 데이터 흐름 (Data Flow Diagrams)

### Flow 1: 프롬프트 저장 흐름

```
Claude Code
    │
    ▼
File saved to disk
    │
    ▼
File Watcher detects
    │
    ▼
Read file content
    │
    ▼
Parse prompt content
    │
    ▼
Extract metadata
    │
    ▼
Calculate hash
    │
    ▼
Check for duplicates ──► Found similar ──► User action
    │                                    (New/Update)
    │
    No duplicates
    │
    ▼
Validate data
    │
    ▼
Store in DB
    │
    ▼
Success notification
```

### Flow 2: 분석 및 보고 흐름

```
Schedule: Weekly Monday 09:00
    │
    ▼
Start analysis job
    │
    ▼
Fetch last 7 days prompts
    │
    ▼
For each prompt:
  ├─ Quality analysis
  ├─ Efficiency analysis
  └─ Store results
    │
    ▼
Aggregate statistics
    │
    ▼
Generate insights
    │
    ▼
Render report templates
    │
    ▼
Convert to formats
    │
    ▼
Save to file
    │
    ▼
Send notification
```

---

## 컴포넌트 상호작용 (Component Interaction)

```
┌──────────────┐
│     CLI      │
└──────┬───────┘
       │
       ├─────────────────┬─────────────────┬─────────────────┐
       │                 │                 │                 │
       ▼                 ▼                 ▼                 ▼
   ┌───────┐         ┌─────────┐      ┌────────┐       ┌──────────┐
   │Capture│         │Analysis │      │Report  │       │  Config  │
   │Module │         │ Module  │      │Module  │       │  Module  │
   └───┬───┘         └────┬────┘      └───┬────┘       └────┬─────┘
       │                  │               │                 │
       │              ┌───▼───┐           │            ┌────▼──────┐
       │              │       │           │            │  Config   │
       │              │  DB   │◄──────────┘            │   File    │
       └──────────────►       │                        └───────────┘
                      │       │
                      └───────┘
```

---

## 설계 패턴 (Design Patterns)

### 1. Repository Pattern (DAL)
**목적**: 데이터 접근 로직 추상화

```rust
pub trait PromptRepository {
    fn create(&self, prompt: &Prompt) -> Result<String>;
    fn read(&self, id: &str) -> Result<Prompt>;
    fn update(&self, prompt: &Prompt) -> Result<()>;
    fn delete(&self, id: &str) -> Result<()>;
    fn list(&self, filter: &Filter) -> Result<Vec<Prompt>>;
}
```

### 2. Strategy Pattern (Analysis)
**목적**: 다양한 분석 전략 구현

```rust
pub trait Analyzer {
    fn analyze(&self, prompt: &Prompt) -> Result<AnalysisResult>;
}

pub struct QualityAnalyzer;
pub struct EfficiencyAnalyzer;
pub struct TrendAnalyzer;
```

### 3. Builder Pattern (Report)
**목적**: 복잡한 보고서 생성

```rust
pub struct ReportBuilder {
    data: ReportData,
    format: ReportFormat,
}

impl ReportBuilder {
    pub fn with_data(mut self, data: ReportData) -> Self { ... }
    pub fn with_format(mut self, format: ReportFormat) -> Self { ... }
    pub fn build(self) -> Report { ... }
}
```

### 4. Factory Pattern (Config)
**목적**: 환경별 설정 생성

```rust
pub trait ConfigFactory {
    fn create_config(env: &str) -> Result<Config>;
}
```

---

## 확장성 고려사항

### 1. 수직 확장 (Vertical)
- 더 많은 메모리로 더 큰 데이터셋 처리
- 병렬 처리 강화 (Tokio workers)
- 데이터베이스 최적화 (인덱싱, 분할)

### 2. 수평 확장 (Horizontal)
- Phase 2: API 서버로 분리
- 데이터베이스 레플리케이션
- 로드 밸런싱

### 3. 기능 확장
- 플러그인 아키텍처 지원
- 커스텀 분석기 추가
- 외부 데이터 소스 연동

---

## 보안 고려사항

### 1. 데이터 접근 제어
- 파일 시스템 권한 (644: rw-r--r--)
- 데이터베이스 트랜잭션
- 감사 로그

### 2. 데이터 무결성
- 체크섬 검증
- 트랜잭션 관리
- 정기 백업

### 3. 입력 검증
- SQL 인젝션 방지
- 명령 인젝션 방지
- 타입 검증

---

## 배포 아키텍처

### 개발 환경
```
Local Machine
├── Source code (Git)
├── SQLite DB (로컬)
├── Config files (YAML)
└── Test data
```

### 프로덕션 환경
```
Local Machine
├── Compiled binary
├── SQLite DB (암호화, 백업)
├── Config files (프로필)
└── Report output dir
```

---

## 성능 고려사항

### 1. 데이터베이스 최적화
- 적절한 인덱싱
- 쿼리 최적화
- 연결 풀링

### 2. 메모리 최적화
- 구조체 크기 최소화
- 메모리 풀 사용
- 가비지 컬렉션 최소화 (Rust는 자동)

### 3. I/O 최적화
- 배치 처리
- 캐싱
- 비동기 작업 (Tokio)

---

## 장애 복구 (Disaster Recovery)

### 1. 자동 백업
- 일일 1회 전체 백업
- 트랜잭션 로그
- WAL (Write-Ahead Logging)

### 2. 에러 처리
- 포괄적 에러 타입
- Graceful degradation
- 사용자 알림

### 3. 데이터 검증
- 무결성 검사
- 일관성 검증
- 자동 복구

---

## 다음 단계

1. [[Database Schema]] - 상세 데이터베이스 설계
2. [[Development Setup]] - 개발 환경 구성
3. [[Timeline & Milestones]] - 개발 타임라인

---

마지막 업데이트: 2025-11-18
