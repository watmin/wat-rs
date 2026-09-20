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

---

# ORCHESTRATOR'S WEIGH — independent re-run, 2026-09-20

| row | result |
|---|---|
| `scripts/floor.sh` | ✅ **5924/5924 passed** (5 slow), 22 skipped, exit 0 — **+3, exactly as predicted** |
| clippy `-D warnings --all-targets --workspace` | ✅ **0** |
| `census.sh --diff` | ✅ `no STOP-8` |
| fold is INSIDE the stone | ✅ no commit after `a1fa08d36` touches `vocab.rs` |
| clippy fix NOT reverted to dodge the gate | ✅ `*b"::"` survives; the rune sits above it, category `edn` |
| no published history rewritten | ✅ `origin/main` ancestor; 0 `refs/original` |
| **no `.wat` file moved** | ✅ `git diff --stat cc7a923e9..HEAD -- '*.wat'` **empty** — the stone's central safety claim, verified rather than argued |
| golden is the ORACLE'S | ✅ regenerated with `/usr/local/bin/clj` — **byte-identical**, 221 rows |
| corpus generator idempotent | ✅ byte-identical |
| the rule, behaviourally | ✅ **29/29**, incl. `a:/b` refused, `wat::core::x` unmoved, all of 218.7 intact |
| `a/b:c` · `:a/b:c` · `foo#_bar` | ✅ parity, checked against the REPL directly |
| 219 exemption removed | ✅ `exemption("a:b").is_none()` and `("a#b").is_none()` |
| **ward still CATCHES** | ✅ parity-row flip `OK\t42`→`ERR\t42` → RED; restored → green |

## ⭐ TWO THINGS THE COUNTERPART IMPROVED ON THE BRIEF

1. **`a:/b`.** The brief's rule was *"`:` is legal only when neither doubled nor final."* That is a
   **whole-token** rule and it is WRONG: `a:/b` has no `::` and does not end in `:`, yet Clojure
   refuses it because the **prefix** `a:` is colon-final. The counterpart found it and went
   **per-component**. The brief would have shipped a divergence.
2. **The rune's category.** Asked to choose and justify, it picked `edn` on the in-file precedent and
   said why `not-a-name` was arguable but weaker. That is the discipline the category list exists for.

## The chain worth remembering

**clippy's own suggested fix tripped a different gate.** `[b':', b':']` → `*b"::"` satisfied
`clippy::byte_char_slices` and immediately violated `one-variant-separator`, which watches for a
literal `::`. Crate-level clippy passed; only the floor saw it. ⛔ **Two walls can disagree about one
line, and the cheap escape — reverting the clippy fix — would have satisfied one by accident while
re-breaking the other.** The rune answers both.

**VERDICT: ACCEPTED.** Pushing.
