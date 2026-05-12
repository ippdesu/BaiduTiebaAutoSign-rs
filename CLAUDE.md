# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build --release     # Build
cargo run --release       # Build and run (requires BDUSS env var)
cargo check               # Fast compile check without codegen
```

There are no tests defined.

## Architecture

Single-crate, single-binary Rust project (`src/main.rs`, ~420 lines). Async Tokio runtime.

**Flow:**

1. Read `BDUSS` env var, split by `#` to support multiple accounts
2. For each account: fetch a TBS token (anti-CSRF), then fetch the followed forums list (paginated), then sign each forum
3. All three HTTP endpoints hit `tieba.baidu.com` / `c.tieba.baidu.com`

**Key functions in `src/main.rs`:**

- `encode_data()` — Signs a `BTreeMap<String, String>` with MD5 using the hardcoded `SIGN_KEY` (`"tiebaclient!!!"`). The signature is `md5(sorted_kv_pairs_joined + SIGN_KEY)` uppercased, inserted as `sign` into the map.
- `get_tbs()` — GETs `http://tieba.baidu.com/dc/common/tbs` with a `BDUSS` cookie, extracts `tbs` from the JSON response.
- `get_favorite()` — POSTs to `http://c.tieba.baidu.com/c/f/forum/like` with form data to paginate through followed forums. Returns a flattened `Vec<Value>`.
- `client_sign()` — POSTs to `http://c.tieba.baidu.com/c/c/forum/sign` with form data to sign a single forum.
- `validate_bduss()` — Checks BDUSS token looks valid (>20 chars, no `=`).

All three network functions have inline 3-attempt retry loops with exponential backoff (`1.5 * 2^attempt + random 0..1` seconds).

**Rate limiting in main loop:** At least 1–2.5s between sign requests (randomized), plus an extra 5–10s pause every 10 forums.

## BDUSS Token

Required env var. Multiple accounts separated by `#`. Obtain from browser cookies after logging into tieba.baidu.com.

## CI/CD

GitHub Actions (`.github/workflows/main.yml`) runs `cargo run --release` daily at 3 AM UTC. BDUSS stored as a GitHub Secret.
