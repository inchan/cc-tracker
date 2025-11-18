# 프로젝트 준비 단계 완전 가이드

## 1. 프로젝트 개요 및 비전

### 1.1 프로젝트 목표 상세 정의

#### 장기 비전
프롬프트 추적 및 분석 시스템을 통해 **엔터프라이즈급 AI 워크플로우 자동화**를 달성하는 것이 최종 목표입니다. 이는 단순한 프롬프트 저장 도구를 넘어, AI 개발 프로세스 전체에서 품질 관리, 성능 최적화, 비용 절감을 함께 달성하는 통합 시스템입니다.

#### 주요 목표 (SMART)
1. **프롬프트 저장 및 관리**
   - 모든 Claude Code 사용 프롬프트를 자동으로 캡처
   - 구조화된 메타데이터와 함께 저장
   - 빠른 검색 및 필터링 기능 제공
   - 버전 관리 및 히스토리 추적

2. **프롬프트 품질 측정**
   - 자동 품질 점수 산출 (0-100)
   - 다양한 품질 지표 정의 (명확성, 완성도, 효율성 등)
   - 시간 경과에 따른 품질 추이 분석
   - 상위 성능 프롬프트 자동 추출

3. **효율성 분석**
   - 토큰 사용량 추적 및 비용 분석
   - 실행 시간 및 응답 시간 측정
   - 카테고리별 비용 분석
   - 최적화 기회 식별

4. **자동화된 보고서 생성**
   - 주간 분석 보고서 자동 생성
   - 월간 심화 분석 및 트렌드 리포트
   - 카테고리별 성과 대시보드
   - 개선 추천 사항 제시
   - 다양한 포맷 (Markdown, HTML, CSV, PDF)

5. **지속적인 개선 추적**
   - 프롬프트 개선 이력 자동 기록
   - A/B 테스트 지원
   - 프롬프트 라이브러리 진화 추적
   - 팀 협업 및 공유 기능

#### 성공 지표 (KPI)
| 지표 | 목표치 | 측정 방법 |
|------|--------|---------|
| 프롬프트 추적률 | 100% | 수동 확인 vs 자동 캡처 비교 |
| 분석 리포트 생성 주기 | 주 1회 (자동) | 자동화 작업 스케줄러 확인 |
| 프롬프트 품질 개선 속도 | 월 10% 이상 | 월별 평균 품질 점수 추이 |
| 시스템 가용성 | 99% 이상 | 애플리케이션 모니터링 |
| 데이터 정확도 | 95% 이상 | 샘플 검증 및 교차 확인 |
| 사용자 만족도 | 4/5 이상 | 정기 피드백 수집 |

#### 타겟 사용자
- **Primary**: 개인 AI 개발자 (자신의 프롬프트 개선 원하는 사람)
- **Secondary**: 팀 리더 (팀의 AI 사용 패턴 분석 원하는 사람)
- **Future**: 엔터프라이즈 (조직 전체의 AI 워크플로우 최적화 원하는 조직)

### 1.2 핵심 가치 제안
이 프로젝트가 해결하려는 문제:
- **현재 상황**: Claude Code 사용 시 개별 프롬프트의 품질과 효율성을 추적할 방법이 없음
- **문제**: 시간이 지날수록 효과적인 프롬프트를 잊어버리고, 비용 최적화 기회를 놓침
- **솔루션**: 자동화된 추적 및 분석을 통해 지속적 개선과 비용 절감 달성

### 1.3 프로젝트 범위 (Scope)

#### 포함 항목 (In Scope)
- Claude Code에서 사용된 프롬프트 자동 캡처
- 구조화된 메타데이터 저장소 (SQLite 기반)
- CLI 기반 프롬프트 관리 도구
- 자동화된 분석 및 리포팅 엔진
- 로컬 환경에서의 안전한 데이터 관리
- YAML 기반 유연한 설정 시스템

#### 제외 항목 (Out of Scope)
- 클라우드 동기화 (초기 버전)
- 팀 협업 기능 (Phase 2에서)
- 멀티 모델 지원 (Claude 모델만 기본 지원)
- 웹 UI (Phase 3에서 고려)
- 모바일 지원

#### 향후 확장 계획
- Phase 2: API 서버 및 REST 인터페이스
- Phase 3: 웹 대시보드 및 시각화
- Phase 4: 팀 협업 기능
- Phase 5: 멀티 모델 지원 및 엔터프라이즈 기능

### 1.4 프로젝트 제약사항

#### 기술적 제약
- **로컬 우선**: 초기 버전은 로컬 머신에서만 실행
- **데이터 프라이버시**: 모든 프롬프트 데이터는 로컬에 저장, 외부 전송 금지
- **라이선스**: 오픈소스 호환 라이선스만 사용
- **성능**: 초당 10개 프롬프트 처리 가능해야 함

#### 운영 제약
- **팀 규모**: 1인 프로젝트 (개인 시간 활용)
- **개발 기간**: 4-6주 내 MVP 완성
- **유지보수**: 월 8시간 이상 할당
- **예산**: $0 (오픈소스 도구만 사용)

#### 환경 제약
- **OS**: macOS (주 개발 환경)
- **Rust 버전**: 1.70+
- **메모리**: 최소 2GB (권장 4GB 이상)
- **저장소**: 초기 100MB, 장기 1GB+ 예상

---

## 2. 프로젝트 구조 및 아키텍처 개요

### 2.1 고수준 아키텍처

```
┌─────────────────────────────────────────────────────────────┐
│                    Claude Code (로컬)                       │
│                    (프롬프트 입출력)                        │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              프롬프트 캡처 레이어 (Rust)                    │
│         CLI Hook / File Watcher / API Interceptor           │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              데이터 처리 레이어 (Rust)                      │
│  - 프롬프트 파싱 및 정규화                                 │
│  - 메타데이터 추출                                         │
│  - 중복 제거 및 데이터 검증                                │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│           데이터 저장소 레이어 (SQLite)                    │
│  - prompts 테이블                                          │
│  - metrics 테이블                                          │
│  - feedback 테이블                                         │
│  - tags 테이블                                             │
└────────────────────────┬────────────────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        ▼                ▼                ▼
    ┌────────┐      ┌────────┐      ┌──────────┐
    │분석엔진 │      │보고서  │      │시각화    │
    │        │      │생성기  │      │엔진      │
    └────────┘      └────────┘      └──────────┘
        │                │                │
        └────────────────┼────────────────┘
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              CLI / 사용자 인터페이스                        │
│    prompt-tracking get                                     │
│    prompt-tracking analyze                                 │
│    prompt-tracking report                                  │
│    prompt-tracking export                                  │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 상세 폴더 구조

```
prompt-tracking-system/
│
├── 📁 docs/                                    # 문서 및 참고자료
│   ├── 📄 PROJECT.md                          # 프로젝트 개요 (이 파일)
│   ├── 📄 REQUIREMENTS.md                     # 기능 요구사항 명세서
│   ├── 📄 ARCHITECTURE.md                     # 시스템 아키텍처 상세 설계
│   ├── 📄 API.md                              # 내부 API 및 인터페이스
│   ├── 📄 DATABASE_SCHEMA.md                  # 데이터베이스 스키마 정의
│   ├── 📄 DEVELOPMENT.md                      # 개발 가이드 및 best practices
│   ├── 📄 DEPLOYMENT.md                       # 배포 및 설치 가이드
│   │
│   └── 📁 references/                         # 외부 레퍼런스 정리
│       ├── 📄 claude-code-reference.md        # Claude Code 기능 정리
│       ├── 📄 mcp-protocol-guide.md           # MCP 스펙시피케이션
│       ├── 📄 rust-ecosystem-guide.md         # Rust 생태계 정보
│       ├── 📄 sqlite-best-practices.md        # SQLite 최적화 가이드
│       ├── 📄 yaml-configuration.md           # YAML 설정 가이드
│       ├── 📄 markdown-standards.md           # 마크다운 작성 표준
│       └── 📄 community-resources.md          # 커뮤니티 자료 링크
│
├── 📁 src/                                    # 소스 코드
│   ├── 📁 core/                               # 핵심 라이브러리
│   │   ├── 📄 lib.rs                          # 라이브러리 진입점
│   │   ├── 📁 models/                         # 데이터 모델
│   │   │   ├── 📄 mod.rs
│   │   │   ├── 📄 prompt.rs                   # Prompt 구조체
│   │   │   ├── 📄 metric.rs                   # Metric 구조체
│   │   │   ├── 📄 feedback.rs                 # Feedback 구조체
│   │   │   └── 📄 tag.rs                      # Tag 구조체
│   │   │
│   │   ├── 📁 database/                       # 데이터베이스 계층
│   │   │   ├── 📄 mod.rs
│   │   │   ├── 📄 connection.rs               # 연결 관리
│   │   │   ├── 📄 migrations.rs               # 마이그레이션
│   │   │   ├── 📄 prompts.rs                  # 프롬프트 CRUD
│   │   │   ├── 📄 metrics.rs                  # 메트릭 CRUD
│   │   │   └── 📄 queries.rs                  # 복잡한 쿼리
│   │   │
│   │   ├── 📁 capture/                        # 프롬프트 캡처
│   │   │   ├── 📄 mod.rs
│   │   │   ├── 📄 claude_code_hook.rs         # Claude Code 훅
│   │   │   ├── 📄 file_watcher.rs             # 파일 감시
│   │   │   └── 📄 parser.rs                   # 파싱 로직
│   │   │
│   │   ├── 📁 analysis/                       # 분석 엔진
│   │   │   ├── 📄 mod.rs
│   │   │   ├── 📄 quality_analyzer.rs         # 품질 분석
│   │   │   ├── 📄 efficiency_analyzer.rs      # 효율성 분석
│   │   │   ├── 📄 trend_analyzer.rs           # 추이 분석
│   │   │   └── 📄 metrics_calculator.rs       # 메트릭 계산
│   │   │
│   │   ├── 📁 reporting/                      # 보고서 생성
│   │   │   ├── 📄 mod.rs
│   │   │   ├── 📄 weekly_report.rs            # 주간 보고서
│   │   │   ├── 📄 monthly_report.rs           # 월간 보고서
│   │   │   ├── 📄 formatters/                 # 포맷터
│   │   │   │   ├── 📄 markdown.rs
│   │   │   │   ├── 📄 html.rs
│   │   │   │   ├── 📄 csv.rs
│   │   │   │   └── 📄 json.rs
│   │   │   └── 📄 templates.rs                # 템플릿 관리
│   │   │
│   │   ├── 📁 utils/                          # 유틸리티
│   │   │   ├── 📄 mod.rs
│   │   │   ├── 📄 validation.rs               # 데이터 검증
│   │   │   ├── 📄 deduplication.rs            # 중복 제거
│   │   │   ├── 📄 serialization.rs            # 직렬화
│   │   │   └── 📄 errors.rs                   # 에러 처리
│   │   │
│   │   └── 📁 config/                         # 설정 관리
│   │       ├── 📄 mod.rs
│   │       ├── 📄 parser.rs                   # YAML 파서
│   │       └── 📄 schema.rs                   # 설정 스키마
│   │
│   └── 📁 cli/                                # CLI 도구
│       ├── 📄 main.rs                         # CLI 진입점
│       ├── 📁 commands/                       # 커맨드
│       │   ├── 📄 mod.rs
│       │   ├── 📄 capture.rs                  # capture 커맨드
│       │   ├── 📄 list.rs                     # list 커맨드
│       │   ├── 📄 search.rs                   # search 커맨드
│       │   ├── 📄 analyze.rs                  # analyze 커맨드
│       │   ├── 📄 report.rs                   # report 커맨드
│       │   ├── 📄 export.rs                   # export 커맨드
│       │   ├── 📄 import.rs                   # import 커맨드
│       │   └── 📄 init.rs                     # init 커맨드
│       │
│       └── 📁 output/                         # 출력 포맷팅
│           ├── 📄 mod.rs
│           ├── 📄 table.rs                    # 테이블 포맷
│           ├── 📄 json.rs                     # JSON 포맷
│           └── 📄 pretty.rs                   # Pretty 포맷
│
├── 📁 config/                                 # 설정 파일
│   ├── 📄 default.yaml                        # 기본 설정
│   ├── 📄 development.yaml                    # 개발 환경 설정
│   ├── 📄 production.yaml                     # 프로덕션 설정
│   ├── 📄 test.yaml                           # 테스트 환경 설정
│   │
│   └── 📁 templates/                          # 템플릿 파일
│       ├── 📄 report-template.md              # 보고서 템플릿
│       ├── 📄 weekly-template.html            # 주간 보고서 HTML
│       └── 📄 metrics-summary.txt             # 메트릭 요약 템플릿
│
├── 📁 data/                                   # 데이터 디렉토리 (gitignore)
│   ├── 📁 database/                           # 데이터베이스
│   │   ├── 📄 prompts.db                      # SQLite 데이터베이스
│   │   └── 📄 .gitkeep
│   │
│   ├── 📁 prompts/                            # 프롬프트 백업
│   │   ├── 📁 2025-11/                        # 연월별 구분
│   │   └── 📄 .gitkeep
│   │
│   ├── 📁 metrics/                            # 측정 데이터 내보내기
│   │   ├── 📁 2025-11/
│   │   └── 📄 .gitkeep
│   │
│   ├── 📁 reports/                            # 생성된 보고서
│   │   ├── 📁 2025-11/
│   │   └── 📄 .gitkeep
│   │
│   └── 📁 exports/                            # 내보낸 데이터
│       ├── 📁 2025-11/
│       └── 📄 .gitkeep
│
├── 📁 tests/                                  # 테스트
│   ├── 📄 common/mod.rs                       # 공통 테스트 유틸
│   │
│   ├── 📁 unit/                               # 단위 테스트
│   │   ├── 📄 models_test.rs
│   │   ├── 📄 database_test.rs
│   │   ├── 📄 analysis_test.rs
│   │   ├── 📄 reporting_test.rs
│   │   └── 📄 utils_test.rs
│   │
│   ├── 📁 integration/                        # 통합 테스트
│   │   ├── 📄 capture_flow_test.rs
│   │   ├── 📄 analysis_flow_test.rs
│   │   ├── 📄 report_generation_test.rs
│   │   └── 📄 end_to_end_test.rs
│   │
│   └── 📁 fixtures/                           # 테스트 데이터
│       ├── 📄 sample_prompts.json
│       ├── 📄 sample_metrics.json
│       └── 📄 sample_config.yaml
│
├── 📁 scripts/                                # 스크립트
│   ├── 📄 setup.sh                            # 초기 설정
│   ├── 📄 build.sh                            # 빌드 스크립트
│   ├── 📄 test.sh                             # 테스트 실행
│   ├── 📄 run-analysis.sh                     # 분석 실행
│   ├── 📄 generate-report.sh                  # 리포트 생성
│   ├── 📄 backup.sh                           # 데이터 백업
│   ├── 📄 install-claude-hook.sh              # Claude Code 훅 설치
│   └── 📄 dev-server.sh                       # 개발 서버 실행
│
├── 📁 .github/                                # GitHub 관련
│   ├── 📁 workflows/                          # CI/CD 파이프라인
│   │   ├── 📄 test.yml                        # 테스트 자동화
│   │   ├── 📄 build.yml                       # 빌드 자동화
│   │   └── 📄 release.yml                     # 릴리스 자동화
│   │
│   ├── 📁 ISSUE_TEMPLATE/
│   │   ├── 📄 bug_report.md
│   │   ├── 📄 feature_request.md
│   │   └── 📄 question.md
│   │
│   └── 📁 PULL_REQUEST_TEMPLATE/
│       └── 📄 pull_request_template.md
│
├── 📁 .vscode/                                # VSCode 설정
│   ├── 📄 settings.json
│   ├── 📄 extensions.json
│   └── 📄 launch.json
│
├── 📄 Cargo.toml                              # Rust 프로젝트 설정
├── 📄 Cargo.lock                              # 의존성 락 파일
├── 📄 README.md                               # 프로젝트 소개
├── 📄 CONTRIBUTING.md                         # 기여 가이드
├── 📄 LICENSE                                 # 라이선스 (MIT)
├── 📄 .gitignore                              # Git 무시 파일
├── 📄 .gitattributes                          # Git 속성
├── 📄 CHANGELOG.md                            # 변경 로그
├── 📄 .env.example                            # 환경변수 예제
└── 📄 ROADMAP.md                              # 개발 로드맵
```

### 2.3 주요 모듈별 책임

| 모듈 | 책임 | 입력 | 출력 |
|------|------|------|------|
| **capture** | Claude Code에서 프롬프트 감지 및 추출 | 사용자 입력 / 파일 변경 | Prompt 구조체 |
| **database** | 데이터 영속성 관리 | CRUD 명령 | 쿼리 결과 |
| **analysis** | 품질/효율성 분석 | Prompt 데이터 | Analysis 결과 |
| **reporting** | 분석 결과를 리포트로 변환 | Analysis 데이터 | 마크다운/HTML/CSV |
| **cli** | 사용자 명령 해석 및 실행 | CLI 입력 | 명령 실행 결과 |
| **config** | 설정 관리 | YAML 파일 | 설정 객체 |

---

## 3. 상세 문서 목록 및 작성 계획

### 3.1 필수 핵심 문서

#### 📄 PROJECT.md (이 파일)
**목적**: 프로젝트의 전체 개요와 비전을 정의
**포함 내용**:
- 프로젝트 목표 및 성공 지표
- 타겟 사용자 및 가치 제안
- 프로젝트 범위
- 고수준 아키텍처

**작성 기한**: 준비 단계 초기

#### 📄 REQUIREMENTS.md
**목적**: 구체적인 기능 요구사항 명세
**포함 내용**:
- 기능 요구사항 (Functional Requirements)
  - 프롬프트 캡처: 자동 감지, 메타데이터 추출, 버전 관리
  - 저장소 관리: CRUD, 검색, 필터링, 분류
  - 분석 기능: 품질 점수, 효율성 지표, 추이 분석
  - 보고서 생성: 자동 생성, 다양한 포맷, 스케줄링
  - 설정 관리: YAML 기반, 프로필 지원

- 비기능 요구사항 (Non-Functional Requirements)
  - 성능: 초당 10 프롬프트 처리
  - 확장성: 연 10만 프롬프트 저장 가능
  - 신뢰성: 99% 가용성
  - 보안: 로컬 데이터 암호화 (Phase 2)
  - 유지보수성: 모듈화된 구조

- 사용 사례 (Use Cases)
  - UC1: 프롬프트 자동 캡처 및 저장
  - UC2: 저장된 프롬프트 검색
  - UC3: 품질 분석 실행
  - UC4: 주간 보고서 생성

**작성 기한**: 준비 단계 중기

#### 📄 ARCHITECTURE.md
**목적**: 상세한 시스템 아키텍처 설계
**포함 내용**:
- 시스템 아키텍처 다이어그램
- 컴포넌트 상호작용
- 데이터 흐름도
- 계층별 설계 (Layered Architecture)
- 주요 설계 패턴 (Strategy, Repository, Factory 등)
- 확장성 고려사항

**작성 기한**: 준비 단계 말기

#### 📄 DATABASE_SCHEMA.md
**목적**: 데이터베이스 설계 정의
**포함 내용**:
- 테이블 스키마 (SQL DDL)
  - prompts 테이블
  - metrics 테이블
  - feedback 테이블
  - tags 테이블
  - prompt_tags 조인 테이블

- 인덱스 전략
- 제약사항 (Constraints)
- 마이그레이션 전략
- 성능 고려사항

**작성 기한**: 준비 단계 말기

#### 📄 API.md
**목적**: 내부 API 및 인터페이스 명세
**포함 내용**:
- 라이브러리 공개 API
- CLI 커맨드 명세
  - `prompt-tracking init`
  - `prompt-tracking capture <prompt>`
  - `prompt-tracking list [--filter]`
  - `prompt-tracking search <query>`
  - `prompt-tracking analyze [--period]`
  - `prompt-tracking report [--format]`
  - `prompt-tracking export [--range]`

- 각 명령의 입출력 형식
- 에러 코드 및 메시지
- 예제

**작성 기한**: 개발 단계 초기

### 3.2 참고 및 레퍼런스 문서

#### 📄 DEVELOPMENT.md
**목적**: 개발자를 위한 가이드
**포함 내용**:
- 개발 환경 설정 상세 가이드
- 코드 스타일 및 컨벤션 (Rust clippy)
- 테스트 작성 가이드 및 전략
- 디버깅 팁
- 성능 프로파일링
- 로깅 전략
- 문서 작성 표준

**작성 기한**: 개발 단계 초기

#### 📄 DEPLOYMENT.md
**목적**: 배포 및 설치 가이드
**포함 내용**:
- 시스템 요구사항
- 설치 단계별 가이드
- 초기 구성
- Claude Code와의 통합
- 백업 및 복구
- 문제 해결

**작성 기한**: 개발 단계 중기

#### 📄 CONTRIBUTING.md
**목적**: 프로젝트 기여 가이드
**포함 내용**:
- 기여 방법
- Pull Request 프로세스
- 코드 리뷰 기준
- 이슈 보고 방법
- 라이선스 및 저작권

**작성 기한**: 오픈소스 공개 시

### 3.3 외부 레퍼런스 정리

#### 📄 references/claude-code-reference.md
**수집 내용**:
- Claude Code 공식 문서 요약
- 환경변수 설정
- 플러그인/확장 메커니즘
- API 엔드포인트
- 제한사항 및 주의사항
- 커뮤니티 예제 링크

**소스**:
- Anthropic 공식 문서
- Claude Code GitHub 저장소
- 커뮤니티 포럼 및 블로그

#### 📄 references/mcp-protocol-guide.md
**수집 내용**:
- MCP 스펙시피케이션 요약
- 프로토콜 버전 및 호환성
- 메시지 형식
- 구현 예제
- 성능 최적화

**소스**:
- MCP 공식 스펙
- 커뮤니티 구현체
- GitHub 토론

#### 📄 references/rust-ecosystem-guide.md
**수집 내용**:
- 추천 라이브러리
  - CLI: clap, structopt
  - 데이터베이스: rusqlite, sqlx
  - 직렬화: serde, serde_json, serde_yaml
  - 비동기: tokio, async-std
  - 로깅: log, tracing, env_logger
  - 테스트: criterion (벤치마크)

- 성능 최적화 팁
- 메모리 관리
- 에러 처리 best practices

**소스**:
- Rust Book
- Are We (Game/Web/Async) Yet?
- crates.io
- GitHub awesome-rust

#### 📄 references/sqlite-best-practices.md
**수집 내용**:
- SQLite 최적화 전략
- 인덱싱 가이드
- 트랜잭션 관리
- 성능 튜닝
- 동시성 제어
- 백업 전략

**소스**:
- SQLite 공식 문서
- SQLite 성능 튜닝 가이드
- 커뮤니티 사례

#### 📄 references/yaml-configuration.md
**수집 내용**:
- YAML 문법 기본
- 설정 파일 작성 패턴
- 환경변수 override
- 프로필 관리
- 검증 전략

#### 📄 references/community-resources.md
**수집 내용**:
- 유용한 링크 모음
- GitHub 저장소
- 커뮤니티 포럼
- 튜토리얼 및 가이드
- 코드 예제
- 질문할 수 있는 커뮤니티

---

## 4. 레퍼런스 수집 및 구성 가이드

### 4.1 Claude Code 관련 자료 수집

**필수 수집 항목**:
```yaml
claude_code_resources:
  official:
    - url: "https://www.anthropic.com/claude-code"
      title: "Claude Code 공식 페이지"
      priority: "critical"
      items_to_collect:
        - 기본 기능 설명
        - 명령어 목록
        - API 엔드포인트
        - 제한사항

  documentation:
    - url: "https://docs.anthropic.com"
      title: "Anthropic 공식 문서"
      priority: "critical"
      items_to_collect:
        - API 레퍼런스
        - 환경 변수
        - 인증 방법
        - 베스트 프랙티스

  community:
    - source: "GitHub Discussions"
      title: "Claude Code 커뮤니티 토론"
      priority: "high"
      focus_areas:
        - 실제 사용 사례
        - 일반적인 문제 및 해결책
        - 팁과 트릭

    - source: "Reddit r/Claude"
      priority: "medium"
      focus_areas:
        - 사용자 경험 공유
        - 팁과 트릭

  examples:
    - source: "GitHub awesome-claude-code"
      priority: "high"
      collect:
        - 오픈소스 예제
        - 통합 가이드
```

**수집 방법**:
1. 각 소스에서 중요 정보 추출
2. Markdown 형식으로 요약
3. 출처 명시 (URL, 날짜)
4. 카테고리별 분류

### 4.2 MCP (Model Context Protocol) 자료 수집

**필수 수집 항목**:
- 프로토콜 스펙시피케이션 (공식 문서)
- 메시지 형식 및 타입
- 에러 처리
- 실제 구현 예제
- 성능 최적화 팁
- 커뮤니티 라이브러리

**저장 형식**:
```yaml
mcp_protocol:
  version: "1.0"  # 수집 대상 버전
  key_concepts:
    - name: "Request/Response"
      description: ""
      example: ""
    
    - name: "Streaming"
      description: ""
      example: ""
  
  implementation_checklist:
    - [ ] 프로토콜 버전 선택
    - [ ] 메시지 핸들링
    - [ ] 에러 처리
    - [ ] 타임아웃 관리
```

### 4.3 Rust 생태계 자료 수집

**필수 라이브러리 조사**:

| 카테고리 | 후보 라이브러리 | 비교 항목 |
|---------|--------------|---------|
| **CLI** | clap / structopt / argh | 기능, 문서, 커뮤니티 |
| **DB** | rusqlite / sqlx | 성능, 타입 안전성, 비동기 지원 |
| **직렬화** | serde / serde_yaml / serde_json | 성능, 기능, 호환성 |
| **비동기** | tokio / async-std | 생태계, 성능 |
| **로깅** | log / tracing / env_logger | 기능, 성능 |
| **테스트** | criterion / proptest | 벤치마크, 속성 테스트 |

**수집 작업**:
1. 각 라이브러리의 공식 문서 검토
2. GitHub 저장소 활동도 확인
3. crates.io 다운로드 수 및 평점 확인
4. 커뮤니티 평가 및 리뷰 수집
5. 실제 프로젝트에서의 사용 사례 찾기

### 4.4 레퍼런스 저장 구조

**References 폴더 구성**:
```
docs/references/
├── claude-code/
│   ├── official-docs-summary.md
│   ├── api-reference.md
│   ├── environment-setup.md
│   ├── community-tips.md
│   └── common-issues.md
│
├── mcp-protocol/
│   ├── specification-summary.md
│   ├── message-formats.md
│   ├── implementation-guide.md
│   └── examples.md
│
├── rust-ecosystem/
│   ├── recommended-libraries.md
│   ├── cli-frameworks.md
│   ├── database-libraries.md
│   ├── async-runtimes.md
│   └── testing-frameworks.md
│
├── sqlite/
│   ├── schema-design.md
│   ├── optimization-guide.md
│   ├── indexing-strategy.md
│   └── backup-recovery.md
│
└── external-links.md
```

**각 파일의 기본 구조**:
```markdown
# [주제] 레퍼런스

## 개요
간단한 설명

## 주요 개념
- 개념 1
- 개념 2

## 구체적 정보
### 항목 1
설명 및 예제

## 출처
- 링크 1 (날짜)
- 링크 2 (날짜)

## 마지막 업데이트
2025-11-18
```

---

## 5. 개발 환경 설정 상세 가이드

### 5.1 전제 조건 및 시스템 요구사항

**OS 및 하드웨어**:
- **OS**: macOS 12+, Linux (Ubuntu 20.04+), Windows 10+ (WSL2)
- **CPU**: Intel/Apple Silicon (M1/M2 이상 권장)
- **RAM**: 최소 4GB, 권장 8GB 이상
- **디스크**: 초기 2GB, 장기 5GB 이상 여유 공간
- **네트워크**: 초기 설정 시 필요, 이후 로컬 전용

**필수 소프트웨어**:
- Git 2.30+
- Rust 1.70+ (rustup으로 설치)
- Claude Code (로컬 설치)
- 선택사항: Docker (테스트/배포용)

### 5.2 Rust 개발 환경 설정

**Step 1: Rust 설치**
```bash
# rustup 설치 (macOS/Linux)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 또는 Homebrew (macOS)
brew install rust

# 버전 확인
rustc --version
cargo --version
```

**Step 2: Rust 업데이트 및 추가 도구 설치**
```bash
# Rust 업데이트
rustup update

# 유용한 도구 설치
cargo install cargo-watch        # 파일 변경 감시
cargo install cargo-expand       # 매크로 확장 보기
cargo install cargo-tree         # 의존성 트리
cargo install cargo-outdated     # 의존성 업데이트 확인
cargo install cargo-audit        # 보안 감사
```

**Step 3: IDE/Editor 설정 (VS Code)**
```bash
# 필수 확장 프로그램 설치
# 1. Rust-analyzer 확장
# 2. CodeLLDB 또는 lldb-vscode (디버깅)
# 3. crates (의존성 관리)
# 4. Better TOML (설정 파일)
# 5. Even Better TOML

# .vscode/settings.json
{
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.inlayHints.enable": true
}
```

### 5.3 Claude Code 로컬 설치 및 설정

**Step 1: Claude Code 설치**
```bash
# 공식 문서에서 최신 버전 다운로드
# macOS의 경우:
# - App Store에서 설치하거나
# - https://www.anthropic.com/claude-code에서 직접 다운로드

# 설치 확인
claude-code --version
```

**Step 2: 환경 변수 설정**
```bash
# ~/.zshrc 또는 ~/.bash_profile에 추가
export CLAUDE_API_KEY="your-api-key-here"
export CLAUDE_CODE_DATA_DIR="$HOME/.claude-code-data"
export CLAUDE_CODE_LOG_LEVEL="info"

# 변경사항 적용
source ~/.zshrc
```

**Step 3: 초기 설정**
```bash
# Claude Code 초기화
claude-code init

# 기본 구성 확인
claude-code config show

# 프롬프트 저장소 위치 확인
ls -la $CLAUDE_CODE_DATA_DIR/prompts
```

### 5.4 프로젝트 저장소 초기화

**Step 1: GitHub 저장소 생성**
```bash
# GitHub에서 새 저장소 생성 (web ui)
# Repository name: prompt-tracking-system
# Description: An enterprise-grade prompt tracking and analysis system
# Visibility: Public (또는 Private)
```

**Step 2: 로컬 저장소 초기화**
```bash
# 프로젝트 디렉토리 생성
mkdir -p ~/Projects/prompt-tracking-system
cd ~/Projects/prompt-tracking-system

# Cargo 프로젝트 초기화
cargo init --name prompt_tracking

# Git 저장소 초기화
git init
git config user.name "Your Name"
git config user.email "your.email@example.com"

# 원격 저장소 연결
git remote add origin https://github.com/YOUR_USERNAME/prompt-tracking-system.git

# 초기 커밋
echo "# Prompt Tracking System

An enterprise-grade prompt tracking and analysis system for Claude Code.

## Status
🚧 Pre-development phase

## Getting Started
See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for setup instructions.

## License
MIT" > README.md

git add README.md
git commit -m "Initial commit: project setup"
git branch -M main
git push -u origin main
```

**Step 3: 필수 폴더 구조 생성**
```bash
# docs 디렉토리
mkdir -p docs/references

# src 디렉토리 구조
mkdir -p src/core/{models,database,capture,analysis,reporting,utils,config}
mkdir -p src/cli/commands
mkdir -p src/cli/output

# 기타 디렉토리
mkdir -p config/templates
mkdir -p data/{database,prompts,metrics,reports,exports}
mkdir -p tests/{unit,integration,fixtures}
mkdir -p scripts
mkdir -p .github/{workflows,ISSUE_TEMPLATE,PULL_REQUEST_TEMPLATE}
mkdir -p .vscode

# .gitkeep 파일로 빈 디렉토리 추적
find . -type d -empty -exec touch {}/.gitkeep \;
```

**Step 4: .gitignore 설정**
```bash
cat > .gitignore << 'EOF'
# Rust
/target/
Cargo.lock
**/*.rs.bk
*.pdb

# IDE
.vscode/*
!.vscode/settings.json
!.vscode/extensions.json
!.vscode/launch.json
.idea/
*.swp
*.swo
*~
.DS_Store

# 데이터 (gitignore하되, 예제는 추적)
/data/database/*.db
/data/prompts/*
!/data/prompts/.gitkeep
/data/metrics/*
!/data/metrics/.gitkeep
/data/reports/*
!/data/reports/.gitkeep
/data/exports/*
!/data/exports/.gitkeep

# 환경
.env
.env.local
*.log

# 빌드 아티팩트
*.o
*.a
*.so
*.dylib

# Python (분석용 스크립트)
__pycache__/
*.py[cod]
*$py.class
venv/
EOF

git add .gitignore
git commit -m "docs: add .gitignore"
```

### 5.5 Cargo.toml 기본 설정

```toml
[package]
name = "prompt_tracking"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Enterprise-grade prompt tracking and analysis system for Claude Code"
readme = "README.md"
repository = "https://github.com/YOUR_USERNAME/prompt-tracking-system"
license = "MIT"
keywords = ["prompt", "tracking", "analysis", "claude", "ai"]
categories = ["command-line-utilities", "development-tools"]

[lib]
name = "prompt_tracking"
path = "src/core/lib.rs"

[[bin]]
name = "prompt-tracking"
path = "src/cli/main.rs"

[dependencies]
# CLI
clap = { version = "4.4", features = ["derive"] }

# 데이터베이스
rusqlite = { version = "0.29", features = ["bundled", "chrono"] }
tokio = { version = "1.35", features = ["full"] }

# 직렬화
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# 유틸리티
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
regex = "1.10"
anyhow = "1.0"
thiserror = "1.0"

# 로깅
log = "0.4"
env_logger = "0.11"
tracing = "0.1"
tracing-subscriber = "0.3"

# 파일 처리
walkdir = "2.4"
notify = "6.1"  # 파일 감시

[dev-dependencies]
criterion = "0.5"        # 벤치마크
proptest = "1.4"         # 속성 테스트
tempfile = "3.8"         # 테스트 임시 파일
mockall = "0.12"         # 목 객체

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.dev]
opt-level = 0
debug = true
```

---

## 6. 데이터 모델 및 저장소 설계

### 6.1 핵심 데이터 구조 (Rust)

**Prompt 구조체**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,                    // UUID
    pub content: String,               // 프롬프트 원본 텍스트
    pub created_at: DateTime<Utc>,    // 생성 시간
    pub updated_at: DateTime<Utc>,    // 수정 시간
    pub model: String,                 // 사용된 모델 (claude-3-5-sonnet 등)
    pub category: String,              // 카테고리 (분류)
    pub tags: Vec<String>,             // 태그 목록
    pub description: Option<String>,   // 선택적 설명
    pub metrics: Metrics,              // 성능 메트릭
    pub feedback: Option<Feedback>,    // 사용자 피드백
    pub status: PromptStatus,          // 상태 (active, archived 등)
}

pub enum PromptStatus {
    Active,
    Archived,
    Deprecated,
    Testing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub quality_score: f32,            // 품질 점수 (0-100)
    pub efficiency_score: f32,         // 효율성 점수 (0-100)
    pub execution_time_ms: f32,        // 실행 시간 (밀리초)
    pub tokens: TokenUsage,            // 토큰 사용량
    pub usage_count: u32,              // 사용 횟수
    pub last_used: DateTime<Utc>,      // 마지막 사용 시간
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    pub estimated_cost_usd: f32,       // 추정 비용
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub user_rating: u8,               // 1-5 점수
    pub notes: String,                 // 사용자 노트
    pub created_at: DateTime<Utc>,
}
```

### 6.2 SQLite 데이터베이스 스키마

**prompts 테이블**:
```sql
CREATE TABLE IF NOT EXISTS prompts (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    model TEXT NOT NULL,
    category TEXT,
    description TEXT,
    status TEXT DEFAULT 'active',
    
    -- 메트릭
    quality_score REAL,
    efficiency_score REAL,
    execution_time_ms REAL,
    
    -- 토큰 사용
    input_tokens INTEGER,
    output_tokens INTEGER,
    total_tokens INTEGER,
    estimated_cost_usd REAL,
    
    -- 사용 통계
    usage_count INTEGER DEFAULT 0,
    last_used DATETIME,
    
    -- 인덱스
    CONSTRAINT status_check CHECK (status IN ('active', 'archived', 'deprecated', 'testing'))
);

CREATE INDEX idx_prompts_created_at ON prompts(created_at);
CREATE INDEX idx_prompts_category ON prompts(category);
CREATE INDEX idx_prompts_status ON prompts(status);
CREATE INDEX idx_prompts_model ON prompts(model);
```

**tags 테이블**:
```sql
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS prompt_tags (
    prompt_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (prompt_id, tag_id),
    FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX idx_prompt_tags_tag_id ON prompt_tags(tag_id);
```

**feedback 테이블**:
```sql
CREATE TABLE IF NOT EXISTS feedback (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    prompt_id TEXT NOT NULL,
    user_rating INTEGER NOT NULL CHECK (user_rating BETWEEN 1 AND 5),
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE
);

CREATE INDEX idx_feedback_prompt_id ON feedback(prompt_id);
```

**metrics 테이블**:
```sql
CREATE TABLE IF NOT EXISTS metrics_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    prompt_id TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    quality_score REAL,
    efficiency_score REAL,
    execution_time_ms REAL,
    input_tokens INTEGER,
    output_tokens INTEGER,
    FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE
);

CREATE INDEX idx_metrics_history_prompt_id ON metrics_history(prompt_id);
CREATE INDEX idx_metrics_history_timestamp ON metrics_history(timestamp);
```

### 6.3 분석 메트릭 정의

**품질 점수 계산**:
```
품질 점수 = (명확성 × 0.3) + (완성도 × 0.3) + (구체성 × 0.2) + (안내성 × 0.2)

- 명확성 (30%): 프롬프트가 얼마나 명확한가
  * 키워드 밀도, 문장 구조, 모호한 표현 분석
  
- 완성도 (30%): 프롬프트가 얼마나 완전한가
  * 컨텍스트 포함도, 예제 제시, 형식 지정 분석
  
- 구체성 (20%): 프롬프트가 얼마나 구체적인가
  * 수치 지정, 제약사항 명시 분석
  
- 안내성 (20%): 프롬프트가 얼마나 좋은 결과를 유도하는가
  * 역사적 결과 분석, 사용자 평점 활용
```

**효율성 점수 계산**:
```
효율성 점수 = 100 - (정규화된_토큰수 + 정규화된_실행시간 + 정규화된_비용)

- 낮은 토큰 사용량: 더 효율적 (50점)
- 빠른 실행 속도: 더 효율적 (30점)
- 낮은 비용: 더 효율적 (20점)
```

**분석 지표**:
| 지표 | 설명 | 수집 빈도 |
|------|------|---------|
| 평균 프롬프트 길이 | 프롬프트 평균 길이 (토큰) | 일일 |
| 카테고리별 분포 | 각 카테고리의 프롬프트 수 | 일일 |
| 시간대별 사용 패턴 | 시간대별 프롬프트 사용량 | 일일 |
| 품질 점수 추이 | 월별/주별 평균 품질 변화 | 주간 |
| 효율성 개선율 | 월 대비 효율성 개선 % | 월간 |
| 토큰 비용 분석 | 월별 총 토큰 사용 및 비용 | 월간 |
| 상위 성능 프롬프트 | 품질/효율성 상위 10개 | 월간 |

---

## 7. 개발 타임라인 및 마일스톤

### 7.1 프로젝트 페이즈 (Phase별 4-6주)

#### **Phase 1: 기초 구축 (1주)**
**목표**: 프로젝트 기반 완성

**세부 작업**:
- [ ] 폴더 구조 및 파일 생성
- [ ] Cargo.toml 및 의존성 설정
- [ ] 데이터 모델 정의 (Rust 구조체)
- [ ] SQLite 데이터베이스 스키마 생성
- [ ] 기본 에러 처리 구조 설정
- [ ] 로깅 시스템 초기화
- [ ] 첫 번째 단위 테스트 작성

**결과물**:
- 컴파일 가능한 기본 프로젝트
- 데이터 모델 및 스키마 확정
- 개발 환경 완성

**진행 확인**:
```bash
cargo build
cargo test
```

---

#### **Phase 2: 핵심 기능 (1.5주)**
**목표**: 프롬프트 저장 및 관리 기능 완성

**세부 작업**:
- [ ] 프롬프트 캡처 모듈 구현
  - [ ] Claude Code 파일 감시 기능
  - [ ] 프롬프트 파싱 로직
  - [ ] 메타데이터 추출

- [ ] 데이터베이스 CRUD 작업 구현
  - [ ] Create: 새 프롬프트 저장
  - [ ] Read: 프롬프트 조회 및 검색
  - [ ] Update: 프롬프트 수정
  - [ ] Delete: 프롬프트 삭제/아카이브

- [ ] 기본 CLI 커맨드 구현
  - [ ] `prompt-tracking init`
  - [ ] `prompt-tracking capture`
  - [ ] `prompt-tracking list`
  - [ ] `prompt-tracking search`

- [ ] 데이터 검증 및 중복 제거 로직

**결과물**:
- 프롬프트 저장 및 조회 기능
- 기본 CLI 도구
- 통합 테스트

**진행 확인**:
```bash
cargo build --release
./target/release/prompt-tracking list
```

---

#### **Phase 3: 분석 및 리포팅 (1.5주)**
**목표**: 분석 엔진 및 자동 리포팅 완성

**세부 작업**:
- [ ] 품질 점수 계산 엔진 구현
  - [ ] 명확성, 완성도, 구체성, 안내성 분석
  - [ ] 점수 산출 알고리즘

- [ ] 효율성 분석 엔진 구현
  - [ ] 토큰 사용량 분석
  - [ ] 실행 시간 분석
  - [ ] 비용 계산

- [ ] 추이 분석 엔진
  - [ ] 시간 경과에 따른 변화 추적
  - [ ] 추세 선형화

- [ ] 보고서 생성 기능
  - [ ] 주간 보고서 템플릿
  - [ ] 월간 보고서 템플릿
  - [ ] 다양한 포맷 (Markdown, HTML, CSV)

- [ ] 자동 스케줄링
  - [ ] 주간 보고서 자동 생성
  - [ ] 월간 분석 자동 실행

- [ ] CLI 커맨드 추가
  - [ ] `prompt-tracking analyze`
  - [ ] `prompt-tracking report`
  - [ ] `prompt-tracking export`

**결과물**:
- 완전한 분석 엔진
- 자동 보고서 생성
- 데이터 내보내기 기능

**진행 확인**:
```bash
prompt-tracking analyze --period monthly
prompt-tracking report --format markdown
```

---

#### **Phase 4: 최적화 및 배포 (0.5-1주)**
**목표**: 프로덕션 준비

**세부 작업**:
- [ ] 성능 최적화
  - [ ] 데이터베이스 인덱싱
  - [ ] 쿼리 최적화
  - [ ] 메모리 사용량 감소

- [ ] 테스트 커버리지 증대
  - [ ] 단위 테스트 (>80%)
  - [ ] 통합 테스트
  - [ ] 엔드-투-엔드 테스트

- [ ] 문서 완성
  - [ ] README.md 완성
  - [ ] API 문서
  - [ ] 설치 가이드
  - [ ] 사용 가이드

- [ ] 버그 수정 및 안정화
  - [ ] 버그 리포트 처리
  - [ ] 엣지 케이스 처리

- [ ] 배포 준비
  - [ ] 빌드 자동화
  - [ ] 릴리스 프로세스
  - [ ] 버전 태깅

**결과물**:
- v0.1.0 릴리스
- 완전한 문서
- 설치 가능한 바이너리

---

### 7.2 주간 스프린트 계획 예시

**Week 1 (Phase 1)**
```
Mon: 프로젝트 구조 생성, Cargo.toml 설정
Tue: 데이터 모델 정의, DB 스키마 작성
Wed: 에러 처리, 로깅 시스템
Thu: 단위 테스트 작성
Fri: 통합 및 최종 검수
```

**Week 2-3 (Phase 2)**
```
Mon-Tue: 파일 감시 및 캡처 구현
Wed-Thu: CRUD 작업 구현
Fri: CLI 커맨드 (list, search) 구현
```

**Week 4-5 (Phase 3)**
```
Mon-Tue: 분석 엔진 구현 (품질, 효율성)
Wed: 보고서 템플릿 및 생성 로직
Thu-Fri: 자동 스케줄링, CLI 커맨드 추가
```

**Week 6 (Phase 4)**
```
Mon-Tue: 성능 최적화 및 버그 수정
Wed-Thu: 테스트 커버리지 증대
Fri: 문서 최종 검수, v0.1.0 릴리스
```

---

## 8. 기술 결정사항 및 정당성

### 8.1 언어 선택: Rust

**선택 이유**:
- **성능**: C/C++에 가까운 성능, 오버헤드 없음
- **메모리 안전성**: 컴파일 타임에 메모리 안전성 보장
- **동시성**: 안전한 동시성 패턴 (Ownership)
- **생태계**: 성숙한 CLI, 데이터베이스, 비동기 라이브러리
- **커뮤니티**: 활발한 커뮤니티, 풍부한 자료

**대안 검토**:
| 언어 | 장점 | 단점 | 이유 |
|------|------|------|------|
| Go | 빠른 개발, 쉬운 배포 | 메모리 안전성 부족 | MVP 이후 고려 |
| Python | 빠른 프로토타이핑 | 성능 제약, 의존성 관리 어려움 | 분석 스크립트용으로만 사용 |
| TypeScript | Node.js 생태계 | 성능, 타입 안전성 부족 | 웹 UI 계층용으로 고려 |

---

### 8.2 데이터베이스: SQLite

**선택 이유**:
- **로컬 우선**: 파일 기반, 서버 불필요
- **간단성**: SQL 문법, 스키마 정의 직관적
- **성능**: 일반적 사용 수준에서 충분한 성능
- **확장성**: 연 10만 프롬프트 저장 가능
- **백업**: 파일 기반 백업 용이

**대안 검토**:
| 데이터베이스 | 장점 | 단점 | 사용 시점 |
|-------------|------|------|---------|
| PostgreSQL | 고급 기능, 확장성 | 서버 필요 | Phase 3 (API 서버) |
| MongoDB | 스키마 유연성 | 트랜잭션 약함 | 비정형 프롬프트 저장 필요시 |
| JSON 파일 | 가장 단순 | 성능 한계 | 초기 프로토타입 |

---

### 8.3 설정 관리: YAML

**선택 이유**:
- **읽기 쉬움**: 들여쓰기 기반, 가독성 좋음
- **유연성**: 배열, 객체, 변수 interpolation 지원
- **도구**: 풍부한 파서 라이브러리
- **표준**: 많은 도구에서 사용 (Docker, Kubernetes 등)

**대안 검토**:
| 형식 | 장점 | 단점 |
|------|------|------|
| JSON | 표준, 파싱 쉬움 | 가독성 떨어짐, 주석 불가 |
| TOML | 읽기 쉬움 | YAML만큼 유연하지 않음 |
| INI | 매우 단순 | 기능 제약 |

---

### 8.4 CLI 프레임워크: clap

**선택 이유**:
- **기능**: 풍부한 기능 (subcommand, argument parsing, validation)
- **derive 매크로**: 간편한 정의 (declarative)
- **생태계**: 많은 프로젝트에서 사용
- **문서**: 상세한 문서와 예제

**대안 검토**:
| 프레임워크 | 장점 | 단점 |
|----------|------|------|
| structopt | derive 편리 | 유지보수 중단 (clap v4로 통합) |
| argh | 가볍고 간단 | 기능 제약 |
| getopts | 간단 | 저수준 API |

---

## 9. 위험 관리 및 대응 전략

### 9.1 식별된 위험 요소

| 위험 | 발생 가능성 | 영향도 | 우선순위 | 대응 방안 |
|------|-----------|--------|---------|---------|
| Claude Code API 변경 | 중간 | 높음 | P1 | 공식 문서 정기 모니터링, 추상화 계층 활용 |
| 프롬프트 파싱 실패 | 높음 | 중간 | P2 | 폴백 메커니즘, 로깅, 사용자 리뷰 |
| 데이터 마이그레이션 | 낮음 | 높음 | P2 | 초기 스키마 신중한 설계, 마이그레이션 도구 |
| 성능 병목 | 중간 | 중간 | P3 | 초기 벤치마크, 프로파일링 |
| 통합 테스트 복잡성 | 중간 | 중간 | P3 | 모듈화 설계, 테스트 픽스처 |
| 보안 이슈 (프롬프트 유출) | 낮음 | 높음 | P1 | 로컬 저장소만 사용, 접근 제어 |

### 9.2 대응 전략 상세

**Risk: Claude Code API 변경**
```
대응:
1. 공식 문서 주간 검토
2. API 변경사항 모니터링 (GitHub Issues, 포럼)
3. 추상화 계층 생성 (Adapter 패턴)
4. 호환성 테스트 스위트 작성
5. 버전 관리 전략 (semantic versioning)
```

**Risk: 프롬프트 파싱 실패**
```
대응:
1. 여러 프롬프트 형식에 대한 파서 작성
2. 파싱 실패 시 원본 저장 및 경고
3. 사용자가 수동으로 메타데이터 추가 가능하게
4. 파싱 통계 추적 (성공률, 실패율)
5. 커뮤니티 피드백 수집
```

---

## 10. 성공 기준 및 평가 지표

### 10.1 프로젝트 성공 기준

#### 기능적 성공
- [ ] 모든 Phase 1-3 목표 달성
- [ ] 프롬프트 추적률 95% 이상
- [ ] 자동 리포트 생성 정확도 90% 이상
- [ ] 시스템 안정성 99% 이상

#### 품질 기준
- [ ] 테스트 커버리지 80% 이상
- [ ] 코드 리뷰 완료율 100%
- [ ] 버그 해결률 95% 이상
- [ ] 보안 취약점 0개

#### 사용성 기준
- [ ] 설치 소요 시간 < 5분
- [ ] 사용자 만족도 4/5 이상
- [ ] 문서 완성도 100%
- [ ] 커뮤니티 피드백 긍정 비율 80% 이상

### 10.2 정량적 평가 지표 (KPI)

```yaml
technical_metrics:
  performance:
    - 프롬프트 저장 속도: < 100ms
    - 검색 응답 시간: < 500ms
    - 분석 엔진 처리 시간: < 30초
    - 메모리 사용량: < 200MB

  reliability:
    - 시스템 가용성: 99%
    - 데이터 정합성: 100%
    - 에러 복구율: 99%

  scalability:
    - 최대 저장 가능 프롬프트: 100,000개
    - 최대 일일 처리: 10,000개
    - 동시 접근: 10개

code_quality_metrics:
  - 테스트 커버리지: > 80%
  - Cyclomatic Complexity: < 10
  - 코드 중복도: < 5%
  - 정적 분석 이슈: < 10개

user_metrics:
  - 설치 성공률: > 95%
  - 사용 지속률 (30일): > 80%
  - 피드백 응답 시간: < 24시간
```

---

## 최종 체크리스트

프로젝트 시작 직전 최종 준비 확인:

### 준비 단계
- [ ] 이 문서 전체 검토 완료
- [ ] 모든 폴더 구조 생성
- [ ] Git 저장소 초기화 및 연결
- [ ] Rust 개발 환경 설정 완료
- [ ] Claude Code 설치 및 설정 완료
- [ ] IDE (VS Code) 설정 완료

### 문서 준비
- [ ] PROJECT.md 작성 완료
- [ ] REQUIREMENTS.md 작성 완료
- [ ] ARCHITECTURE.md 작성 완료
- [ ] DATABASE_SCHEMA.md 작성 완료
- [ ] references/ 폴더의 주요 문서 작성 완료

### 기술 검토
- [ ] 데이터 모델 설계 검토 완료
- [ ] 데이터베이스 스키마 최종 승인
- [ ] API 명세 확정
- [ ] CLI 커맨드 목록 확정
- [ ] 의존성 라이브러리 선택 완료

### 팀/환경 준비
- [ ] 모든 팀원이 개발 환경 설정 완료 (있는 경우)
- [ ] GitHub 저장소 접근 권한 확인
- [ ] 커뮤니케이션 채널 설정 (필요시)
- [ ] 개발 일정 공지 (필요시)

### 최종 검수
- [ ] 모든 준비 작업 최종 확인
- [ ] 리스크 평가 완료
- [ ] 타임라인 및 마일스톤 재확인
- [ ] 성공 기준 및 KPI 최종 검토

---

**준비가 완료되었습니까? 그렇다면 Phase 1을 시작하세요!**

마지막 업데이트: 2025-11-18
