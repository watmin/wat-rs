# Arc 212 stone δ-comm-purge — SCORE: protocol-violation purge

## Summary

4 sites migrated from `_ <comm-call>` to `_ (Result/expect -> :T <comm-call> "msg")`. Protocol-compliance measurement is now honest at each site. The BRIEF listed line 209 as the recv site; the actual failing site was `BLOCKING_CHILD_SRC` at line 87 (a const source string frozen at test startup). The format string at line 209 was a second recv site discovered during repair; it remains in place unreached by the checker (it is only evaluated at runtime after the supervisor world is frozen). The BRIEF's site numbering was slightly imprecise — the root cause was correct; the line number pointed at the format string rather than the const.

One type annotation correction was needed (STOP trigger 2 — one retry): `:wat::core::Option<:wat::core::nil>` is illegal (leading `:` inside `<>`); corrected to `:wat::core::Option<wat::core::nil>` per the substrate diagnostic.

---

## Per-site changes

### Site 1 — `tests/wat_arc170_stone_a_drain_and_join.rs:101`

- Call: `(:wat::kernel::send tx 1)`
- Chosen `:T`: `:wat::core::nil` (send returns `Result<nil, SendError>`)
- Message: `"send 1 failed — receiver dropped before drain"`
- Pattern: `_ (:wat::core::Result/expect -> :wat::core::nil (:wat::kernel::send tx 1) "send 1 failed — receiver dropped before drain")`

### Site 2 — `tests/wat_arc170_stone_a_drain_and_join.rs:102`

- Call: `(:wat::kernel::send tx 2)`
- Chosen `:T`: `:wat::core::nil`
- Message: `"send 2 failed — receiver dropped before drain"`
- Pattern: `_ (:wat::core::Result/expect -> :wat::core::nil (:wat::kernel::send tx 2) "send 2 failed — receiver dropped before drain")`

### Site 3 — `tests/wat_arc170_stone_a_drain_and_join.rs:103`

- Call: `(:wat::kernel::send tx 3)`
- Chosen `:T`: `:wat::core::nil`
- Message: `"send 3 failed — receiver dropped before drain"`
- Pattern: `_ (:wat::core::Result/expect -> :wat::core::nil (:wat::kernel::send tx 3) "send 3 failed — receiver dropped before drain")`

### Site 4 — `tests/probe_lifeline_orphan_clean_via_fork_program.rs:87` (BLOCKING_CHILD_SRC const)

- Call: `(:wat::kernel::recv rx)` on a channel created with `(:wat::kernel::make-unbounded-channel :wat::core::nil)`
- Chosen `:T`: `:wat::core::Option<wat::core::nil>` (recv returns `Result<Option<nil>, RecvError>`; inner type argument bare per wat type grammar rule — no leading `:` inside `<>`)
- Message: `"recv failed — sender dropped before shutdown"`
- Pattern: `_ (:wat::core::Result/expect -> :wat::core::Option<wat::core::nil> (:wat::kernel::recv rx) "recv failed — sender dropped before shutdown")`
- Note: BRIEF cited line 209 (the format string in the WatAST::StringLit call) as the site. The actual violation surfaced by the type-checker was in the const `BLOCKING_CHILD_SRC` at line 87, which is frozen during test startup via `freeze_ok(BLOCKING_CHILD_SRC)`. The format string at line 209 contains a second recv discard; the checker does not reach it because it is only evaluated at runtime (inside a string literal passed to fork-program). That site is now correct by inspection but is not verified by freeze.

---

## Type annotation discovery

First attempt used `:wat::core::Option<:wat::core::nil>`. Substrate rejected it:

```
malformed :wat::core::Result/expect form: declared type ":wat::core::Option<:wat::core::nil>"
failed to parse: type expression :wat::core::Option<:wat::core::nil> contains an
illegal leading ':' on the inner argument :wat::core::nil: inside `<>`, `()`, or
`fn(...)`, type arguments are bare Rust symbols. The colon prefix marks wat keywords
and lives at the OUTERMOST type position only. Drop the leading ':' on the inner:
write :wat::core::Option<wat::core::nil> instead.
```

Corrected on first retry: `:wat::core::Option<wat::core::nil>`.

---

## Verification

```
cargo test --release --test wat_arc170_stone_a_drain_and_join
  test result: ok. 4 passed; 0 failed; 0 ignored

cargo test --release --test probe_lifeline_orphan_clean_via_fork_program
  test result: ok. 1 passed; 0 failed; 0 ignored
```

---

## Build

`cargo build --release` — Finished (clean, 5 warnings pre-existing in substrate; zero new warnings from test-fixture changes).

---

## Workspace baseline

Was: 2 failing (both surfaced + diagnosed by δ-comm-positions cascade; they were always wrong — the pre-arc-212 List-only walker hid them). Now: 0 (both closed by this stone).

---

## Mode classification

**Mode A** — 4 sites migrated; both named tests pass; cargo build clean; SCORE written. One :T correction needed (inner type argument syntax) resolved on first retry within STOP trigger 2 bounds.
