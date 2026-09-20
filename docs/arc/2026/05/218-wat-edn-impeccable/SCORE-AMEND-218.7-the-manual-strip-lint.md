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

---

# ORCHESTRATOR'S WEIGH — independent re-run, 2026-09-20

| row | result |
|---|---|
| `scripts/floor.sh` | ✅ **5921/5921 passed** (3 slow), 22 skipped, exit 0 |
| clippy `-D warnings --all-targets --workspace` | ✅ **0** |
| `census.sh --diff` | ✅ `no STOP-8` |
| fold is INSIDE the stone | ✅ **no commit after `fccc7b45d` touches `clj_oracle_parity.rs`** |
| no published history rewritten | ✅ `origin/main` is an ancestor; **0** `refs/original` refs |
| no `#[allow]` smuggled | ✅ `&t[1..]` gone tree-wide; `strip_prefix` form, `::foo` short-circuit intact |
| golden is the ORACLE'S | ✅ regenerated with `/usr/local/bin/clj` — **byte-identical** |
| corpus generator idempotent | ✅ byte-identical, 190 rows |
| 19 behavioural rows (4 fixes · 7 spec-strict refusals · 6 non-vacuity controls) | ✅ 19/19 |
| **the ward CATCHES** | ✅ injected `ERR\t42` → RED `"42" clj:ERR wat:OK`; restored → green |

## ⚠ THE ORCHESTRATOR RAN A PROBE THAT COULD NOT FAIL — recorded, not hidden

Re-proving the ward after the classifier rewrite, the orchestrator flipped **`a:b`**'s golden verdict
`OK`→`ERR` and expected a red. **The ward passed, and that was nearly filed as a finding
("exemptions mask the row").**

**The probe was mis-aimed.** `a:b` is `clj:OK / wat:ERR`. Flipping the golden to `ERR` makes it
`clj:ERR / wat:ERR` — **artificial parity** — so the loop `continue`s at `if wat == clj` and never
reaches `exemption()`. The ward behaved correctly; the probe asserted nothing.

⛔ **Second occurrence this session of `[[feedback_a_green_from_a_mis_aimed_probe_is_indistinguishable_from_a_working_gate]]`** —
in a session where the orchestrator wrote that exact warning into another brief. The valid probe
flips a **parity** row (`42`), and that one goes red.

## One real, MINOR weakness — reported, not fixed

An **exempted** row is skipped once verdicts differ, so if `clojure.edn`'s verdict ever changes such
that a divergence *disappears*, the row becomes plain parity and the exemption lingers **with no
signal**. Stale exemptions are therefore undetectable. This is housekeeping, not a correctness hole:
every *unexempted* divergence is still caught, proven above. A cure would assert that each
`exemption()` entry corresponds to an actual current divergence. **Not in this stone.**

**VERDICT: ACCEPTED.** Pushing.
