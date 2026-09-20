# SCORE — AMEND 218.8: the `one-variant-separator` gate

Folded into the stone. **No follow-up commit.** Stone is now `a1fa08d36` (was `1bb428bf9`).
AMEND brief remains `cbce16a88` on top. **Not pushed.** Floor / workspace clippy / census:
orchestrator's row.

## The red

`wat::lint one_variant_separator::only_identifier_rs_spells_the_variant_separator` at
`crates/wat-edn/src/vocab.rs:225` — `*b"::"` from the clippy `byte_char_slices` rewrite.
`[b':', b':']` did not match the gate; the clippy-blessed form did. Did **not** revert the
clippy fix.

## The rune

Category **`edn`**, matching `translate_and_validate_ns` in the same file. This site refuses a
doubled colon inside an EDN symbol/keyword body; it does not separate an enum from a variant.
`not-a-name` was arguable; `edn` is the precedent and the job.

```rust
// rune:lint(one-variant-separator, edn) — EDN constituent rule: refuse a
// doubled colon in a symbol/keyword body. Not an enum/variant split.
if bytes.windows(2).any(|w| w == *b"::") {
```

## Walls I ran (not the floor)

- `cargo clippy --release --all-targets -p wat-edn --offline -- -D warnings` — exit 0.
- `cargo test --release -p wat-edn --offline` — 352 unit+integration + 3 doctests, all pass.
- `one_variant_separator::only_identifier_rs_spells_the_variant_separator` — **1 passed**, 5945 skipped.

Floor + workspace clippy + census: **not run** (brief: orchestrator, uncontended). Do not push.
8d does not start.
