# DearDay Server — Claude 작업 가이드

LLM이 코딩할 때 흔히 저지르는 실수를 줄이기 위한 행동 지침. 프로젝트별 지침과 함께 적용한다.

**트레이드오프:** 이 지침은 속도보다 신중함에 무게를 둔다. 사소한 작업이면 재량껏 판단할 것.

## 1. 코딩 전에 생각하기

**가정하지 말 것. 혼란을 숨기지 말 것. 트레이드오프를 드러낼 것.**

구현 전에:
- 가정은 명시적으로 말한다. 확신이 없으면 묻는다.
- 해석이 여러 개라면 모두 제시한다 — 혼자 결정하지 않는다.
- 더 단순한 방법이 있으면 말한다. 필요하면 밀어붙인다.
- 불분명하면 멈춘다. 무엇이 헷갈리는지 짚는다. 묻는다.

## 2. 단순함 우선

**문제를 푸는 최소한의 코드만. 추측성 코드 금지.**

- 요청되지 않은 기능 추가 금지.
- 한 번만 쓰이는 코드에 추상화 금지.
- 요청되지 않은 "유연성" / "설정 가능성" 금지.
- 일어날 수 없는 상황에 대한 에러 처리 금지.
- 200줄짜리가 50줄이면 충분해 보이면 다시 쓴다.

스스로에게 묻기: "시니어 엔지니어가 이걸 보면 과하게 복잡하다고 할까?" 그렇다면 단순화한다.

## 3. 외과적 수정

**꼭 필요한 부분만 건드린다. 자기가 만든 잔해만 치운다.**

기존 코드를 수정할 때:
- 주변 코드, 주석, 포매팅을 "개선"하지 않는다.
- 망가지지 않은 것을 리팩터하지 않는다.
- 본인 스타일이 더 좋다고 생각하더라도 기존 스타일에 맞춘다.
- 관련 없는 데드 코드를 발견하면 언급만 한다 — 지우지 않는다.

수정으로 인해 고아 코드가 생긴 경우:
- **본인 수정으로 인해** 사용되지 않게 된 import/변수/함수만 제거한다.
- 원래부터 있던 데드 코드는 요청받지 않는 한 지우지 않는다.

판단 기준: 변경된 모든 줄은 사용자 요청과 직접 연결되어야 한다.

## 4. 목표 기반 실행

**성공 기준을 정의한다. 검증될 때까지 반복한다.**

작업을 검증 가능한 목표로 바꿔라:
- "validation 추가" → "잘못된 입력에 대한 테스트를 쓰고, 그걸 통과시킨다"
- "버그 수정" → "재현 테스트를 쓰고, 그걸 통과시킨다"
- "X 리팩터" → "리팩터 전후로 테스트가 통과하는지 확인한다"

다단계 작업이면 짧은 계획을 적는다:
```
1. [단계] → 검증: [확인 방법]
2. [단계] → 검증: [확인 방법]
3. [단계] → 검증: [확인 방법]
```

성공 기준이 명확하면 혼자 루프를 돌 수 있다. 기준이 약하면("동작하게 만들어") 계속 사용자에게 물어봐야 한다.

---

**이 지침이 잘 작동한다는 신호:** diff에 불필요한 변경이 적음, 과한 복잡성 때문에 재작성하는 일이 줄어듦, 실수한 뒤가 아니라 구현 전에 질문이 나옴.

---

이 파일은 Claude(또는 모든 AI 에이전트)가 이 프로젝트를 일관되게 수정하도록 안내한다.
**새 작업을 시작하기 전에 반드시 읽을 것.**

## 프로젝트 개요

- **목적**: DearDay 앱의 백엔드 REST API 서버
- **스택**: Rust + Axum 0.8 + sqlx + PostgreSQL 16
- **워크플로우**: 사용자는 코드를 직접 쓰지 않는다 — 모든 작업이 AI 에이전트를 통해 이루어진다.
  따라서 **가독성과 일관성이 최우선 가치**이다.

## 디렉터리 구조

```
.
├── Cargo.toml
├── docker-compose.yml          # 로컬 Postgres
├── .env                        # 실제 env 변수 (gitignored)
├── migrations/                 # sqlx 마이그레이션 SQL
└── src/
    ├── main.rs                 # 진입점 (얇음 — 설정 읽고 lib::run 호출)
    ├── lib.rs                  # 라이브러리 루트 — 모듈 선언, run() 노출
    ├── config.rs               # env 변수 파싱
    ├── state.rs                # AppState (Arc<inner> 패턴)
    ├── error.rs                # 공용 ApiError + ApiResult<T>
    └── routes/
        ├── mod.rs              # 최상위 라우터 조립
        ├── health.rs           # 작은 라우트 = 파일 하나
        └── users/              # 큰 라우트 = 폴더
            ├── mod.rs          # 라우터 조립 (얇음 — 모듈 선언 + router())
            ├── handlers.rs     # HTTP 핸들러 (요청 → 응답 변환만)
            └── queries.rs      # Users 구조체 + sqlx 쿼리
```

**원칙**: 한 도메인의 변경은 한 폴더 안에서 끝나야 한다.
새 리소스를 추가할 때 가능한 한 적은 폴더만 건드린다.

## 세 가지 핵심 패턴

### 1. AppState — Arc로 감싼 공유 상태

```rust
// src/state.rs
pub struct AppState { inner: Arc<AppStateInner> }
pub struct AppStateInner { pub db: PgPool }
```

`Clone`이 저렴하다 (Arc clone). 필드 접근은 `Deref`를 통해 이루어진다.

### 2. Queries 구조체 + FromRef — 도메인별 서브 상태

각 도메인은 `FromRef<AppState>`를 구현하는 자체 구조체를 가진다.
핸들러는 `State<AppState>` 대신 **`State<Users>` 같은 서브 상태**를 받는다.

```rust
// queries.rs
#[derive(Clone)]
pub struct Users { db: PgPool }

impl FromRef<AppState> for Users {
    fn from_ref(state: &AppState) -> Self {
        Self { db: state.db.clone() }
    }
}

impl Users {
    pub async fn list(&self) -> sqlx::Result<Vec<User>> { ... }
    pub async fn find_by_id(&self, id: Uuid) -> sqlx::Result<User> { ... }
    pub async fn create(&self, input: CreateUser) -> sqlx::Result<User> { ... }
}

// mod.rs (핸들러)
async fn list(State(users): State<Users>) -> ApiResult<Json<Vec<User>>> {
    Ok(Json(users.list().await?))
}
```

**왜?** 핸들러 시그니처만 봐도 의존성이 명확하고, DB 코드가 한곳에 모이며, 나중에 테스트 쓰기가 쉽다.

### 3. 단일 에러 타입 — ApiError

모든 핸들러는 `ApiResult<T>` (= `Result<T, ApiError>`)를 반환한다.
`sqlx::Error → ApiError` 변환은 자동:

| sqlx 에러 | HTTP |
|---|---|
| `RowNotFound` | 404 Not Found |
| Unique violation | 409 Conflict |
| 그 외 전부 | 500 Internal Server Error |

validation 에러가 필요해지면 `ApiError`에 `BadRequest(String)` variant를 추가하고 400으로 매핑한다 (현재는 미사용이라 정의돼 있지 않음).

## 새 리소스 추가하는 법

리소스 이름이 `posts`라고 가정. **순서대로 모든 단계를 따른다**:

### Step 1. 마이그레이션 작성

```bash
sqlx migrate add create_posts
# migrations/<timestamp>_create_posts.sql 생성됨
```

SQL을 작성한다 (`migrations/20260524000001_create_users.sql` 참고). 컨벤션:
- 기본키: `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`
- 타임스탬프: `created_at`, `updated_at`을 `TIMESTAMPTZ NOT NULL DEFAULT NOW()`로
- 자주 조회하는 컬럼에 인덱스 추가

### Step 2. 도메인 폴더 만들기

```
src/routes/posts/
├── mod.rs
├── handlers.rs
└── queries.rs
```

가장 안전한 방법: **`src/routes/users/`를 복사한 뒤 식별자만 바꾼다**.

### Step 3. `queries.rs` 작성

다음을 포함해야 한다:
- `pub struct Posts { db: PgPool }`, `#[derive(Clone)]` 포함
- `impl FromRef<AppState> for Posts { ... }`
- `pub struct Post { ... }`, `#[derive(Debug, Serialize, sqlx::FromRow)]` 포함
- `pub struct CreatePost { ... }`, `#[derive(Debug, Deserialize)]` 포함
- `impl Posts { pub async fn ... }` — DB 메서드

### Step 4. `handlers.rs` 작성

```rust
use super::queries::{CreatePost, Post, Posts};
use crate::error::ApiResult;

pub(super) async fn list(State(posts): State<Posts>) -> ApiResult<Json<Vec<Post>>> {
    Ok(Json(posts.list().await?))
}
// get_one, create ...
```

핸들러는 **HTTP I/O 변환에만 집중**한다. 비즈니스 로직이나 sqlx 쿼리는 들어가지 않는다.

### Step 5. `mod.rs` 작성

```rust
use axum::{routing::get, Router};
use crate::state::AppState;

mod handlers;
mod queries;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list).post(handlers::create))
        .route("/{id}", get(handlers::get_one))
}
```

`mod.rs`는 **라우터 조립만**. 다른 코드는 넣지 않는다.

### Step 6. 최상위 라우터에 등록

`src/routes/mod.rs`:
```rust
mod posts;

// router() 내부
.nest("/posts", posts::router())
```

### Step 7. 마이그레이션 적용 후 실행

```bash
sqlx migrate run
cargo run
# curl로 검증
```

## 코딩 컨벤션

### 네이밍
- **입력 DTO**: `Create<X>`, `Update<X>` (예: `CreateUser`)
- **응답 DTO**: 모델을 그대로 사용 (`User`). 모양이 다를 때만 `<X>Response`.
- **Queries 구조체**: 복수형 명사 (`Users`, `Posts`)
- **Queries 메서드**: `list`, `find_by_id`, `find_by_email`, `create`, `update`, `delete`
- **핸들러 함수**: `list_*`, `get_*`, `create_*`, `update_*`, `delete_*` —
  모듈 내부에서는 문맥이 명확하므로 `list`, `get_one`, `create`도 괜찮다.

### 파일 크기
- 파일이 **200줄을 넘으면 분리 신호**.
  `queries.rs`가 너무 커지면 `queries/users.rs`, `queries/auth.rs` 식으로 쪼갠다.

### sqlx 쿼리
- 항상 `sqlx::query_as!` 매크로 사용 (컴파일 타임 검증)
- 여러 줄 쿼리: `r#" ... "#` raw string에 들여쓰기로 작성
- **`SELECT *` 금지** — 모든 컬럼을 명시한다
- 컴파일 시점에 DB가 떠 있어야 한다. DB가 없는 환경(CI 등)에서는
  `cargo sqlx prepare`로 쿼리 캐시(`.sqlx/`)를 생성해두고
  `SQLX_OFFLINE=true`로 빌드한다.

### 에러 처리
- 핸들러는 `ApiResult<T>` 반환
- `?` 자유롭게 사용 (`From<sqlx::Error>` 구현되어 있음)
- validation 실패: `BadRequest` variant를 추가한 뒤 `Err(ApiError::BadRequest("...".into()))` 반환
- 로깅: `tracing::{info, warn, error, debug}!` — **`println!` 금지**

### 주석
- **WHAT을 문서화하지 말 것**. 식별자 이름으로 충분해야 한다.
- **WHY가 분명하지 않을 때만 주석**을 단다 — 외부 제약, 워크어라운드, 반직관적 결정 등.

## 자주 쓰는 명령어

```bash
# 개발
docker compose up -d              # Postgres 시작
docker compose down               # Postgres 중지 (데이터 유지)
docker compose down -v            # 중지 + 데이터 삭제

cargo run                         # 서버 실행 (debug)
cargo run --release               # 최적화 빌드
cargo check                       # 타입 체크만 (가장 빠름)
cargo clippy                      # 린트
cargo fmt                         # 포맷

# 마이그레이션
sqlx migrate add <name>           # 새 마이그레이션 파일 생성
sqlx migrate run                  # 보류 중인 마이그레이션 적용
sqlx migrate info                 # 마이그레이션 상태 확인

# sqlx 오프라인 모드 (CI 등)
cargo sqlx prepare                # 쿼리 캐시 생성 (.sqlx/)
SQLX_OFFLINE=true cargo build     # 캐시 사용해 빌드

# 컨테이너 안에서 psql
docker exec -it dearday-postgres psql -U dearday -d dearday
```

## 환경 변수

`.env`에 정의 (gitignored):

```
BIND_ADDR=0.0.0.0:3000
RUST_LOG=dearday=debug,tower_http=debug
DATABASE_URL=postgres://dearday:dearday@localhost:5432/dearday
```

새 env 변수를 추가할 때는 **항상 `src/config.rs`의 `Config` 구조체에 필드를 추가**한다.
`std::env::var(...)` 호출을 코드베이스 여기저기 흩뿌리지 않는다.

## 하지 말 것

- ❌ 핸들러 안에서 sqlx 쿼리 직접 작성 → 항상 `Queries 구조체` 메서드를 거친다
- ❌ `unwrap()` / `expect()` (`main.rs` / `lib.rs` 부트스트랩에서만 허용) → `?`와 `ApiError` 사용
- ❌ `service` 레이어 성급하게 추가 — 진짜 비즈니스 로직(트랜잭션 합성, 외부 API 호출 등)이 생긴 뒤에만 도입
- ❌ `pub` 남용 — 모듈 밖에서 쓰지 않으면 비공개로 둔다
- ❌ 로깅에 `println!` → `tracing` 매크로 사용
- ❌ `.env`를 git에 커밋

## 참고 자료

이 프로젝트의 패턴은 다음에서 영향을 받았다:
- [launchbadge/realworld-axum-sqlx](https://github.com/launchbadge/realworld-axum-sqlx) — sqlx 메인테이너의 레퍼런스 구현
- [Zero To Production In Rust](https://github.com/lukemathwalker/zero-to-production) — Rust 웹 책의 동반 저장소
- [Axum + sqlx queries pattern](https://www.joshka.net/axum-sqlx-queries-pattern/) — `FromRef` queries 구조체 패턴의 출처

## Agent 스킬

### 이슈 트래커

이슈는 `DearDayApp/DearDay-Server` GitHub 저장소에 있으며, `gh` CLI로 관리한다. `docs/agents/issue-tracker.md` 참고.

### 트리아지 라벨

다섯 개의 표준 역할에 기본 라벨 이름을 그대로 사용한다 (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). `docs/agents/triage-labels.md` 참고.

### 도메인 문서

단일 컨텍스트 구성: `CONTEXT.md`와 `docs/adr/`가 저장소 루트에 위치. `docs/agents/domain.md` 참고.
