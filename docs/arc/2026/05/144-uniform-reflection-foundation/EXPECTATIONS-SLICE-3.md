# Arc 144 Slice 3 — Pre-handoff expectations

**Drafted 2026-05-03.** TypeScheme registrations for 15 hardcoded
callables + 6+ new tests + load-bearing length-canary transition.
Predicted MEDIUM-LIGHT slice (Mode A ~60%; Mode B-scheme-shape
~20%; Mode B-canary-still-red ~10%; Mode C ~10%).

**Brief:** `BRIEF-SLICE-3.md`
**Output:** 1 Rust file modified (`src/check.rs` — additive
registrations only) + 1 NEW Rust file (`tests/wat_arc144_hardcoded_primitives.rs`).
~75-150 LOC + ~250-word report.

## Setup — workspace state pre-spawn

- Slice 1 closed (commit 42319ef + drift fix 810129f). Slice 2
  closed (commit 8ff6fd4). Pre-flight baseline tests verified
  clean per FM 9 (slice 1 9/9, slice 2 9/9, arc 143 lookup 11/11,
  manipulation 8/8, define_alias 2/3 — length canary still red).
- 1 in-flight uncommitted file (CacheService.wat — arc 130).
- Workspace baseline failure: slice 6 length canary + the in-
  flight CacheService.wat-induced wat-lru fail.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | `src/check.rs` modified (15 new TypeScheme registrations in `register_builtins`) + new `tests/wat_arc144_hardcoded_primitives.rs`. NO other Rust file changes. NO wat files. |
| 2 | 15 new TypeScheme registrations | Each of the 15 hardcoded callables (Vector, Tuple, HashMap, HashSet, string::concat, assoc, concat, dissoc, keys, values, empty?, conj, contains?, length, get) gains a `env.register` call with TypeScheme. |
| 3 | Audit-first discipline honored | Each registration's shape matches the corresponding `infer_*` handler's actual type-param + param + return signature. Where the brief's table differs from the handler, sonnet preferred the handler + cited the line. |
| 4 | Variadic limitation comments | Each variadic constructor (Vector, Tuple, HashMap, HashSet, concat, string::concat) has a Rust comment above the registration naming the fingerprint limitation + the runtime dispatch site. |
| 5 | New test file | `tests/wat_arc144_hardcoded_primitives.rs` with 6+ tests verifying signature-of returns Some for representative primitives. ALL pass. |
| 6 | **LENGTH CANARY TURNS GREEN** | `cargo test --release --test wat_arc143_define_alias` 3/3 (the `define_alias_length_to_user_size_delegates_correctly` test now PASSES). This is the load-bearing slice 6 canary. |
| 7 | All slice 1 + slice 2 + arc 143 baseline tests still pass | `wat_arc144_special_forms` 9/9; `wat_arc144_lookup_form` 9/9; `wat_arc143_lookup` 11/11; `wat_arc143_manipulation` 8/8. |
| 8 | **`cargo test --release --workspace`** | Failure profile SHRINKS: only the in-flight CacheService.wat-induced wat-lru fail remains (slice 6 length canary closed). ZERO new regressions. |
| 9 | `cargo clippy --release --all-targets` | No new warnings. |
| 10 | Honest report | ~250-word report covers 15-primitive summary + audit deltas + lookup_form verification + length canary transition + test totals + clippy + honest deltas. |

**Hard verdict:** all 10 must pass. Row 6 is THE load-bearing row;
rows 7+8 prove no regression.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | Total slice diff: 75-200 LOC (15 schemes ≈ 5-10 LOC each + ~75 LOC test file). >300 LOC = re-evaluate. |
| 12 | Style consistency | New registrations follow the existing `register_builtins` pattern (helper functions, comment style, alphabetical/grouped placement). |
| 13 | Test pattern consistency | New tests follow `tests/wat_arc144_lookup_form.rs` shape (startup_from_source + invoke_user_main + stdout assertions). |
| 14 | Audit completeness | Sonnet's report names dispatch sites for each registration whose shape differs from the brief's table. |

## Independent prediction

- **Most likely (~60%) — Mode A clean ship.** Brief is detailed +
  pre-flighted; the registration pattern is mechanical; the length
  canary turning green is the high-leverage outcome. ~10-18 min
  wall-clock (smaller scope than slice 2).
- **Surprise on scheme shape (~20%) — Mode B-scheme.** Sonnet's
  audit finds one or more handler signatures differs from the
  brief's table (e.g., `get` may not be `HashMap<K,V> -> Option<V>`
  but polymorphic over multiple containers). Adapts; reports.
- **Length canary STILL red after registration (~10%) — Mode
  B-canary.** The TypeScheme registration alone may be insufficient
  to close the canary (e.g., the macro's quasiquote splice may need
  additional plumbing the brief didn't anticipate). If hit, sonnet
  surfaces the diagnostic CLEAN; orchestrator opens a slice 3b for
  the additional plumbing.
- **clippy / borrow-check friction (~10%) — Mode C.** Polymorphic
  TypeScheme construction may surface lifetime / borrow issues.
  Adapts.

**Time-box: 40 min cap (2× upper-bound 20 min).**

## Methodology

After sonnet returns:
1. Read this file FIRST.
2. Score each row.
3. Diff via `git diff --stat` — 1 Rust file modified + 1 new test
   file expected.
4. Read 3-5 of the new TypeScheme registrations to verify shape.
5. Run the new test file — confirm 6+ pass.
6. **Run `cargo test --release --test wat_arc143_define_alias` —
   confirm 3/3 (length canary green).**
7. Run slice 1 + slice 2 + arc 143 baseline tests — confirm zero
   regression.
8. Run `cargo test --release --workspace` — confirm shrunk failure
   profile.
9. Run `cargo clippy --release --all-targets` — confirm clean.
10. Score; commit `SCORE-SLICE-3.md`.

## What this slice unblocks

- **Slice 4** — verification slice. After slice 3, the length
  canary is already green; slice 4 simplifies to documentation
  prep + arc 109 v1 cross-reference.
- **Slice 5** — closure (INSCRIPTION + 058 row + USER-GUIDE +
  end-of-work-ritual review).
- **Arc 130** — the next stepping stone (`Vector/len` ?) becomes
  the next chain link to either rename or alias. The length canary
  closing means the macro substrate is fully proven for hardcoded
  primitives.
- **Future REPL `(help X)` form** — composes the now-complete
  reflection layer.

The "nothing is special" principle is now substantively complete
across all 5 Binding kinds.
