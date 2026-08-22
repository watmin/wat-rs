# EXPECTATIONS — `--check` as loud as the runtime on a malformed `fn`

Written BEFORE the strike. Every row independently re-run by the orchestrator.

| # | what | expected |
|---|---|---|
| 1 | `(fn :foo [x <- :i64] -> :i64 x)` | `--check` **FAILS**, located, message matches the runtime's |
| 2 | `(fn 42 …)` · `(fn "s" …)` | **FAIL**, same |
| 3 | `(fn)` · `(fn [x <- :i64])` · `(fn [x <- :i64] ->)` | **FAIL**, reason mirrors `eval.rs:49` verbatim |
| 4 | ★ well-formed fn | still CHECKS |
| 5 | ★ `(fn :- [T] [x <- :T] -> :T x)` — the γ-i binder | still CHECKS |
| 6 | ★ `(fn {:doc "m"} […] -> :i64 x)` — metadata preamble | still CHECKS |
| 7 | the runtime is UNCHANGED — same message, same span, for every row-1..3 form | identical to today |
| 8 | `SigParse::SilentReject` no longer exists | `grep SilentReject src/` returns nothing |
| 9 | `src/check.rs` diff | **EMPTY** |
| 10 | floor (orchestrator, central) | **4855/4855** — or a NAMED set of macro-template reds |
| 11 | clippy `-D warnings` | 0 |

## Independent prediction

**Runtime: 15-30 min** if the exemption is pure debt — it is four deletions and a message copy.
**60+ and a re-decide** if STOP-1 fires. **2× box: 60 min** on the optimistic path.

## Trap doors, named before the strike

1. ★ **Rows 5 and 6 are the real gate.** The binder and the metadata map are both peeled BEFORE the
   guard, so each LOOKS like a non-Vector in slot 0 until its peel runs. A check placed one line too
   early turns both into false positives — and row 5 would silently undo γ-i.
2. ★ **STOP-1 is an OUTCOME, not a failure.** Six macro templates quasiquote a `fn` form. If they
   light up, the exemption was load-bearing for unquote nodes and the answer becomes a slot rule
   (diagnose unless slot 0 is an unquote). A rider that special-cases past this hides the finding.
   **The floor is the only instrument that can answer it; I could not, and said so in the DESIGN.**
3. **Row 7 exists because this stone must not change the runtime.** The two sequences are supposed to
   AGREE; a change that makes them agree by moving the runtime has fixed the wrong twin.
4. **Row 8 is how "delete the exemption" is distinguished from "add a check".** If `SilentReject`
   survives, something still routes to it and the silent state is still representable.
5. **A green floor is not sufficient on its own.** Rows 1-3 must be seen FAILING. A stone whose only
   evidence is "nothing broke" has not demonstrated that anything now fires.
   `[[feedback_a_green_test_can_prove_nothing]]`

## Scoring method

Written after the orchestrator's own re-run. Rows 1-3 are checked for FAILURE first, then 4-6 for
success; a stone that passes 4-6 and does not fail 1-3 has changed nothing.
