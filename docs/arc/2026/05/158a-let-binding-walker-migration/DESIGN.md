# Arc 158a — Walker migration: pair-deadlock walkers accept untyped let bindings

**Status:** opened 2026-05-07.

## Background

Arc 158 v1 attempted to drop the per-binding `:T` annotation from
`:wat::core::let` and was reverted post-verification when 24 lib
unit tests failed. REALIZATIONS at
`docs/arc/2026/05/158-untyped-let-bindings/REALIZATIONS.md`
captured the root cause: the pair-anchor scope-deadlock walker
family (arcs 117 / 126 / 128 / 131 / 133 / 134) reads the
let-binding-DECLARED type for identity tracking. v1 stripped the
declared type at inference (per arc 145 lesson) but didn't migrate
the walkers off the declared-type path — so they lost their type
info and stopped firing.

Arc 158a is the precursor stepping stone identified in
REALIZATIONS: migrate the walkers to derive type info from a
shape-independent source BEFORE the binding-shape change ships.
After 158a closes, arc 158b (a new directory; v2 of the binding-
shape change) ships cleanly atop the migrated walkers.

## Goal

Walker family currently extracts type-ann string from declared
`:T` via `parse_binding_for_pair_check` (src/check.rs:3215) —
which only matches the legacy `((name :T) rhs)` shape. Migrate
the walker so it ALSO derives type-ann from RHS pattern-matching
when the binding is in the new `(name rhs)` shape.

After 158a:
- Legacy `((pair :wat::kernel::Channel<wat::core::i64>) (make-bounded-channel ...))` → walker reads declared `:T` (existing path)
- New `(pair (make-bounded-channel ...))` → walker pattern-matches RHS shape, derives `:wat::kernel::Channel<...>` (new path)
- Both shapes feed the SAME `PairScopeEntry` downstream; trace logic unchanged

## Why pattern-match RHS instead of running inference

Two candidate sources for the type info:

1. **Inference output** — `infer_let` already computes the type
   of each RHS. If inference populated a side-map (let_id, name)
   → TypeExpr, the walker could read it.
2. **RHS pattern-match** — for a closed set of RHS shapes
   (`make-bounded-channel`, `make-unbounded-channel`, `first`,
   `second` on a known-pair binding), derive type-ann directly.

Pattern-match wins on the four questions:

- **Obvious?** Walker pattern-matches RHS; same precedent already
  exists in `extend_pair_scope_with_tuple_destructure` (which
  recognizes `make-*-channel` RHS for tuple-destructure bindings).
- **Simple?** Walker is self-contained; no new cross-cutting
  inference-state plumbing.
- **Honest?** The walker only fires on a closed set of channel-
  related types anyway; RHS pattern-match covers the SAME closed
  set without inventing new infrastructure.
- **Good UX?** No regression on any existing test; new shape
  works the same way; future RHS shapes can be added incrementally.

Inference-side-map approach (option 1) would require new
infrastructure (CheckEnv field, populate-then-read protocol)
that's only used by this walker. Heavier than the problem
demands.

## Substrate scope

### `parse_binding_for_pair_check` extension

Current: only accepts legacy `((name :T) rhs)` shape; reads
declared `:T` from AST.

After 158a: accepts BOTH shapes:
- Legacy `((name :T) rhs)` → existing logic
- New `(name rhs)` where name is a bare Symbol → pattern-match
  RHS to derive type-ann string. If RHS is recognizable, return
  `Some((name, type_ann_str, rhs))`. If unrecognizable, return
  `None` (walker conservatively gives up; no false positives).

### RHS patterns to recognize (closed set)

For each pattern, the type-ann string the walker uses:

| RHS shape | Derived type-ann |
|---|---|
| `(:wat::kernel::make-bounded-channel TYPE N)` | `:wat::kernel::Channel<TYPE>` |
| `(:wat::kernel::make-unbounded-channel TYPE)` | `:wat::kernel::Channel<TYPE>` |
| `(:wat::core::first SOMETHING)` where SOMETHING traces to a Channel | `:wat::kernel::Sender<TYPE>` |
| `(:wat::core::second SOMETHING)` where SOMETHING traces to a Channel | `:wat::kernel::Receiver<TYPE>` |

For `first`/`second` patterns, the binding-scope (with prior
entries) must contain SOMETHING already, so the walker traces
through to the parent Channel binding. This is the same
mechanism `trace_to_pair_anchor` already uses; we're just
flipping which input it reads from.

### Out of scope for 158a

- Function-call RHS (e.g. `(my-app::make-pair :i64)`) — the
  walker can't recognize these without full inference. After
  arc 158b ships, ANY new RHS pattern users introduce that
  needs walker tracking will need explicit recognition added
  here. For now: legacy shape stays valid for those edge cases;
  arc 158b's sweep can leave them legacy with a follow-up arc
  if needed.
- Walker pattern-match for arbitrary user-defined types — only
  the channel-related closed set above.

The closed-set approach is honest about the walker's scope: it
fires on Channel/Sender/Receiver detection, nothing else.

## Slice plan

### Slice 1 — substrate

- Extend `parse_binding_for_pair_check` to accept both shapes;
  add RHS pattern-match for `make-*-channel` cases
- Keep the existing legacy-shape path unchanged (zero regression
  on tests using legacy shape)
- Add new tests verifying walker fires on NEW shape:
  - `(pair (:wat::kernel::make-bounded-channel :wat::core::i64 1))`
    → `pair` recognized as `Channel<:wat::core::i64>`
  - `(rx (:wat::core::second pair))` → `rx` recognized as
    `Receiver<:wat::core::i64>` via trace through `pair`
  - Combined scope-deadlock pattern using new shape → walker fires
- 5-7 tests covering both legacy compatibility AND new shape
- Workspace: 2029/0/0 must hold (legacy paths unaffected; new
  paths are additive)

### Slice 2 — closure paperwork

- INSCRIPTION
- 058 changelog row
- (No USER-GUIDE / WAT-CHEATSHEET updates needed — this arc is
  internal substrate; user-facing behavior unchanged. Arc 158b
  will update those.)

## Cross-references

- **Arc 117** — `ScopeDeadlock` minted; pair-anchor trace
- **Arc 126** — channel-pair deadlock prevention
- **Arc 128** — check walker respects sandbox boundary
- **Arc 131** — `HandlePool` counts as Sender-bearing
- **Arc 133** — tuple-destructure bindings honor scope-deadlock
  (closest precedent — already pattern-matches `make-*-channel`
  RHS via `extend_pair_scope_with_tuple_destructure`)
- **Arc 134** — scope-deadlock origin trace
- **Arc 158 v1 (REALIZATIONS)** — surfaced the walker-vs-binding-
  shape coupling
- **Arc 158b (planned)** — binding-shape change v2; depends on
  158a closing

## Four questions

- **Obvious?** YES — RHS pattern-match recipe exists in arc 133;
  this is an extension of the same approach.
- **Simple?** YES — single function extension + new test cases.
- **Honest?** YES — closed-set scope matches walker's actual
  domain (Channel-related detection); doesn't invent broader
  type tracking.
- **Good UX?** YES — both shapes work post-158a; no regression
  on any existing test.

## Stepping-stones

- **Tractability of next steps?** Arc 158b becomes mechanical:
  binding-shape change + sweep. Walker concerns are settled.
- **Dependencies?** Arc 117 / 126 / 133's existing infrastructure
  is sufficient; 158a only EXTENDS the pattern-match.
- **Composition?** Single substrate slice; no consumer sweep
  needed (no user-visible behavior change). Atomic.

## Estimated effort

- Slice 1: ~30-45 min Sonnet (one function extension + 5-7 tests)
- Slice 2: ~15 min orchestrator (closure paperwork)
- Total: ~45-60 min wall-clock if Mode A clean
