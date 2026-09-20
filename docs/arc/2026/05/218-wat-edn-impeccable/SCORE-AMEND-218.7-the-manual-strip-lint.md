# SCORE — AMEND 218.7: the manual-strip lint

Folded into the stone. **No follow-up commit.** Stone is now `fccc7b45d` (was `49c9900d6`).
AMEND brief remains `f81f1a297` on top. **Not pushed.** Floor/clippy workspace: orchestrator's row.

## The red

`clippy::manual_strip` at `clj_oracle_parity.rs:152` (`let body = &t[1..]` after `t.starts_with(':')`).
Captured at workspace clippy; crate-only clippy was the same one error. No `#[allow]`.

## The rewrite

Clippy's suggested shape, the `::foo` arm:

```rust
if let Some(body) = t.strip_prefix(':') {
    // `::foo` is Clojure auto-resolve, not the 219 constituent-char ruling.
    if body.starts_with(':') {
        return false;
    }
    return body.contains(':') || body.contains('#');
}
```

`every_exemption_names_a_reason` still asserts `exemption("::foo").is_none()` and
`exemption("a:b").is_some()`. Both held after the fold.

## Walls I ran (not the floor)

- `cargo clippy --release --all-targets -p wat-edn --offline -- -D warnings` — exit 0 (the 1 error is gone).
- `cargo test --release -p wat-edn --offline` — 349 unit+integration + 3 doctests, all pass.
  `every_exemption_names_a_reason` and `wat_edn_matches_clj_oracle` both ok.

Floor + workspace clippy + census: **not run** (brief: orchestrator, uncontended). Do not push. 8d does not start.
