# 카카오 로그인 수동 테스트

`tests/api/auth_real_kakao.rs`는 실제 카카오 API와 통신해서 `/auth/kakao/*` 엔드포인트 전체 플로우를 검증한다. 기본적으로 `#[ignore]` 되어 있어 `cargo test`에서 실행되지 않는다 — 수동으로 토큰을 받아서 실행한다.

## 1회: Kakao Developers 콘솔 설정

1. https://developers.kakao.com → 내 애플리케이션 → DearDay 앱 선택
2. **카카오 로그인** 메뉴 → 활성화 ON
3. **Redirect URI** 등록 — 아래 중 하나를 사용 (이미 등록돼 있으면 스킵):
   - `https://example.com/oauth`  ← 가장 간단 (코드를 URL에서 직접 복사)
4. **동의항목**에서 최소 `프로필 정보(닉네임/프로필 사진)` 활성화

## 매번: Access Token 받기

Access token은 약 6시간만 유효하다. 만료되면 다시 받는다.

REST API 키는 Kakao Developers 콘솔 → 내 애플리케이션 → 앱 설정 → 앱 키 → **REST API 키**에서 확인.

### 단계 1 — 인가 코드 받기

브라우저에서 다음 URL 열기 (`$KAKAO_REST_API_KEY`는 본인 키로 교체):

```
https://kauth.kakao.com/oauth/authorize?client_id=$KAKAO_REST_API_KEY&redirect_uri=https://example.com/oauth&response_type=code
```

카카오 로그인 + 동의 진행 후, 브라우저가 `https://example.com/oauth?code=XXXXXXXX...`로 리다이렉트된다. URL의 `code=` 뒤 값을 복사.

### 단계 2 — 코드를 토큰으로 교환

```bash
curl -X POST https://kauth.kakao.com/oauth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code" \
  -d "client_id=$KAKAO_REST_API_KEY" \
  -d "redirect_uri=https://example.com/oauth" \
  -d "code=<위에서 복사한 code>"
```

응답 예:
```json
{
  "access_token": "abc123...",
  "token_type": "bearer",
  "refresh_token": "xyz789...",
  "expires_in": 21599,
  "refresh_token_expires_in": 5183999,
  "scope": "profile_image profile_nickname"
}
```

## 테스트 실행

```bash
export KAKAO_TEST_ACCESS_TOKEN=<access_token 값>
cargo test --test api real_kakao -- --ignored --nocapture
```

`--nocapture`는 테스트 안의 `println!` 출력을 보여줘서 각 단계의 응답을 확인할 수 있다.

## 무엇을 검증하나

`real_kakao_full_flow` 테스트는 한 번 실행으로 전체 플로우를 돈다:

1. `POST /auth/kakao/check` → `{registered: false}` (신규 DB)
2. `POST /auth/kakao/sign-up` → 201 + access/refresh 토큰
3. `POST /auth/kakao/check` 다시 → `{registered: true}`
4. `POST /auth/kakao/login` → 200 + 새 토큰 페어
5. 이전 sign-up 시 받은 refresh로 `/auth/reissue` → 401 (단일 세션 — login으로 무효화됨)
6. login 시 받은 refresh로 `/auth/reissue` → 200 + 새 페어
7. 새 refresh로 `/auth/logout` → 204
8. 로그아웃한 refresh로 `/auth/reissue` → 401 (블랙리스트)

전부 통과해야 카카오 로그인 로직이 정상.

## 일반 테스트 (Mock 기반)

`tests/api/auth.rs`는 wiremock으로 카카오 API를 가짜 응답하도록 mock한다. 토큰 없이도 실행 가능하고 CI에서 항상 돈다:

```bash
cargo test --test api
```
