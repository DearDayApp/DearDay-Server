# DearDay Server

DearDay 앱의 백엔드 REST API 서버.

## 기술 스택

| 레이어 | 선택 |
|---|---|
| 언어 | Rust 1.95.0 (`rust-toolchain.toml`로 핀) |
| 웹 프레임워크 | [axum](https://github.com/tokio-rs/axum) 0.8 |
| 비동기 런타임 | [tokio](https://github.com/tokio-rs/tokio) |
| 데이터베이스 | PostgreSQL 16 |
| DB 클라이언트 | [sqlx](https://github.com/launchbadge/sqlx) (컴파일 타임 쿼리 검증) |
| 인메모리 캐시 | [moka](https://github.com/moka-rs/moka) |
| 로깅 | tracing + tracing-subscriber |
| 에러 처리 | thiserror + anyhow |
| HTTP 미들웨어 | tower-http (trace, cors) |

## 아키텍처

핵심 패턴 세 가지로 요약됩니다. 자세한 컨벤션은 [CLAUDE.md](CLAUDE.md) 참고.

### 1. `AppState` — Arc로 감싼 공유 상태

DB 풀, 캐시 등 공유 자원을 한 곳에 모은 뒤 `Arc`로 감싸 핸들러에 주입한다. clone이 저렴(포인터 + 참조 카운터 증가)하므로 매 요청 복사해도 비용 무시 가능.

### 2. 도메인별 Queries 구조체 + `FromRef<AppState>`

각 도메인은 자기에게 필요한 자원만 들고 있는 구조체를 갖는다 (예: `Users { db: PgPool }`). 핸들러는 `State<AppState>` 대신 **`State<Users>` 같은 서브 상태**를 받는다.

```rust
async fn list(State(users): State<Users>) -> ApiResult<Json<Vec<User>>> {
    Ok(Json(users.list().await?))
}
```

→ **핸들러 시그니처 자체가 의존성 명세**가 된다. DB만 쓰는지, 캐시도 쓰는지, 시그니처만 보고 안다.

### 3. 단일 에러 타입 — `ApiError`

모든 핸들러는 `ApiResult<T>` (`= Result<T, ApiError>`)를 반환. `sqlx::Error → ApiError` 변환은 자동:

| sqlx 에러 | HTTP |
|---|---|
| `RowNotFound` | 404 |
| Unique violation | 409 |
| 그 외 | 500 |

## 디렉터리 구조

```
src/
├── main.rs              # 진입점 (얇음 — config 읽고 lib::run 호출)
├── lib.rs               # 라이브러리 루트
├── config.rs            # env 파싱
├── state.rs             # AppState
├── error.rs             # ApiError + ApiResult
└── routes/
    ├── mod.rs           # 최상위 라우터 조립
    ├── health.rs        # 작은 라우트 = 파일 하나
    └── users/           # 큰 라우트 = 폴더
        ├── mod.rs       # 라우터 조립
        ├── handlers.rs  # HTTP I/O 변환
        └── queries.rs   # Queries 구조체 + sqlx 쿼리

migrations/              # sqlx 마이그레이션
tests/api/               # 통합 테스트 (#[sqlx::test] 격리 DB)
```

## 로컬 실행

```bash
docker compose up -d                # Postgres 시작
cp .env.example .env                # 없으면 직접 작성
sqlx migrate run                    # 마이그레이션 적용
cargo run                           # 서버 실행 (기본 :3000)
```

`.env` 예시:

```
BIND_ADDR=0.0.0.0:3000
RUST_LOG=dearday=debug,tower_http=debug
DATABASE_URL=postgres://dearday:dearday@localhost:5432/dearday
```

## 테스트

```bash
cargo test
```

`#[sqlx::test]` 매크로가 매 테스트마다 격리된 임시 DB를 만들고 마이그레이션을 적용한 뒤 풀을 주입한다. 테스트 끝나면 DB가 자동 정리된다.

## CI

`develop` 브랜치로 PR이 열리면 [GitHub Actions](.github/workflows/ci.yml)가 자동 실행:

- Postgres 16 service 컨테이너 기동
- `sqlx migrate run`
- `cargo test`

fmt/clippy는 CI가 아니라 로컬 Stop hook에서 강제 ([.claude/hooks/cargo-build.sh](.claude/hooks/cargo-build.sh)).

## 문서

- [CLAUDE.md](CLAUDE.md) — AI 에이전트 작업 가이드 (코딩 컨벤션, 새 리소스 추가법, 하지 말 것)
- [docs/agents/](docs/agents/) — 이슈 트래커, 트리아지 라벨, 도메인 문서
