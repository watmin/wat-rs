# SCORE — E5, weighed against the orchestrator's own re-run

> Re-run here at `c9cdd9d32`.

## The scorecard, re-run

| # | pre-value at HEAD | after |
|---|---|---|
| 1 | `rust_caller_span!()` at both sites | ✅ the caller's `list_span`; the probe pins the refusal to `probe_arc278_export.wat:224` — the `(fire-rules …)` form itself, not merely "some .wat" |
| 2 | ★ the lint could not see either fn | ✅ **re-driven by me**, below |
| 3 | all three entries hold `list_span` | ✅ all pass it |
| 4 | I said 9 + 1 | ⚠ **10 + 1** — my table was wrong; see thin spot A |
| 5 | — | ✅ 8 test callers pass synthetic spans, unruned |
| 6 | — | ✅ the lint gains a **doc note only**; predicate untouched |
| 7 | — | ✅ 3 fire files + 5 test modules + the lint doc + the probe + one comment fix. **`validate/typing.rs` absent** — E1 held |
| 8 | lint 116/116 | ✅ 116/116 |
| 9 | floor 5219/5219 | ✅ `Summary [ 422.173s] 5220 tests run: 5220 passed (3 slow), 21 skipped`, zero FAIL rows |
| 10 | clippy rc=0 | ✅ rc=0 |

**Row 2, re-driven here.** Re-introducing `rust_caller_span!()` in the threaded
`fire_rules_on_session`:

```
FAIL  wat::lint span_substitution_justified::span_substitutions_are_justified
  🔥🔥🔥 SPAN SUBSTITUTION — 1 site(s) mint a RUST location while a real wat span was
  in scope.
  Offenders:  src/rete/kernel/fire/rules.rs:672  (fn fire_rules_on_session)
```

The ★ decision holds: **the gate that already existed now sees the site.** The rider drove it on
*both* bodies rather than one, on its own initiative — "one mutation cannot prove a two-site claim",
which is this arc's own lesson applied back at me.

## ⛔ Where MY brief was thin — and one figure was unfalsifiable

- **A. ★ My call-site table names a caller that does not exist.** DESIGN lists `fire/mod.rs` among
  `fire_rules_on_session`'s real callers. It is not one — it calls `fire_once_session`. The true
  count is **10 + 1**, not 9 + 1. The "count them yourself and report the number you find" hedge is
  the only reason this did not mislead, and a hedge is not a substitute for a correct table.
- **B. ★★ MY 534/71 DOES NOT REPRODUCE, AND IT WAS LOAD-BEARING.** It was the entire justification
  for not widening the lint. The rider ported the lint's **own** walker with `carries_span`
  inverted — the correct instrument — and got **494/69**, at my stated commit, both times; a raw
  macro grep gives 557/72 and `src/`+`tests/` gives 778. **No definition yields 534/71.** Mine came
  from an ad-hoc regex script I ran in my terminal and never committed. The decision survives on the
  measured figure, but for a day the stone carried a number that could not be checked and therefore
  could not be wrong. **The cure the rider shipped is the right one:** the lint's doc now records
  494/69 *and names the instrument*, so the next reader rechecks instead of inheriting folklore.
  **Second instance of this failure; the memory is updated rather than duplicated.**
- **C. Row 1 as written collides with `no_loose_string_assert`.** "A probe asserts the span names a
  `.wat` file" reads as `ends_with(".wat")`, which is a lint violation. The rider hit exactly that
  red and fixed it by *tightening* to an exact `assert_eq!` on `(basename, line)` derived from the
  `.wat` at runtime — so the probe pins the specific form and cannot rot when the fixture is edited.
  Better than what I asked for.
- **D. Trap 4 (clippy arity) never fired.** Four params is far under the ceiling. Not wrong, just
  unnecessary — and an unnecessary trap costs the rider a check.

## The rider's own red, and how it handled it

Its first `wat::lint` run went red on `no_loose_string_assert` (trap 5, as predicted) — **and it
truncated its own capture with `| tail -25`, losing the failure block.** It said so, unprompted, and
did not blind-re-run: it reconstructed the cause by reading the lint instead. That is the exact
mistake `CLAUDE.md` names first, self-reported rather than buried, and the recovery was the one the
doctrine prescribes.

## A hole this strike found and did not close

**Nothing gates `file:line` citations in comments.** The rider's doc block shifted two accurate
references in `src/rete/kernel/arm.rs` (`fire/rules.rs:629`→642, `:814`→828). `no_stale_path_in_doc`
checks **paths, not lines**, so both rotted silently and would have stayed rotted. Refreshed here;
the general gap stands open and is worth its own row — every edit above a cited line does this,
undetected.

## Arms not driven, named

**The `fire-once` runtime refusal (`mod.rs:1047`)** — **reachable but not driven.** Its trigger needs
`rete_arm_lookup(...).is_none()` on a network with live productions, and `:wat::rete::import`
registers the arm, so the empty-deps poke that reaches the `fire-rules` wall cannot reach this one.
Its **lint** arm is proven (mutation 1, arm B); its **span-value** arm is not. Named rather than
claimed.
