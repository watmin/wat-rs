# Arc 158a — INSCRIPTION

**Inscribed 2026-05-07 by orchestrator.** All slices shipped.

## What shipped

The pair-deadlock walker family (arcs 117 / 126 / 128 / 131 /
133 / 134) now accepts BOTH `:wat::core::let` binding shapes:

- Legacy `((name :T) rhs)` — read declared `:T` from AST
  (existing path; unchanged)
- New `(name rhs)` — pattern-match RHS via the new
  `derive_type_ann_from_rhs` helper for the closed set of
  channel-related shapes the walker tracks

Both shapes feed the same `PairScopeEntry` downstream; trace
machinery unchanged.

This is the precursor stepping stone arc 158 v1 should have
included. Arc 158 v1 attempted to drop per-binding `:T` and was
reverted post-verification when 24 lib unit tests broke; the
root cause was the walker reading declared `:T` directly from
AST. Arc 158a closes that dependency: walkers no longer require
the declared `:T` for new-shape bindings; they derive type info
from RHS shape recognition.

After arc 158a closes, arc 158b (binding-shape change v2; new
arc directory) ships cleanly atop the migrated walker.

## Slices

| Slice | Commit | What landed |
|---|---|---|
| 1 | `ca43e56` | `parse_binding_for_pair_check` extension + `derive_type_ann_from_rhs` helper + `find_binding_span` extension + `check_let_star_for_scope_deadlock_inferred` extension + 7 new tests |
| 2 | (this commit) | Closure paperwork |

## Settled design

### RHS pattern-match recipe (closed set)

`derive_type_ann_from_rhs` recognizes:

| RHS shape | Returned type-ann |
|---|---|
| `(:wat::kernel::make-bounded-channel TYPE N)` | `:wat::kernel::Channel<TYPE>` (`:` stripped from inner) |
| `(:wat::kernel::make-unbounded-channel TYPE)` | `:wat::kernel::Channel<TYPE>` |
| `(:wat::core::first SOMETHING)` | `:wat::kernel::Sender<wat::core::nil>` placeholder |
| `(:wat::core::second SOMETHING)` | `:wat::kernel::Receiver<wat::core::nil>` placeholder |

For `first`/`second` patterns the placeholder element type
matches arc 133's existing approach in
`extend_pair_scope_with_tuple_destructure` — `type_is_sender_kind`
/ `type_is_receiver_kind` check only the outer parametric head,
not the inner element type. Trace machinery resolves the actual
element type via pair-anchor downstream.

For unrecognized RHS shapes, `derive_type_ann_from_rhs` returns
`None` and the walker conservatively gives up tracking that
binding. No false positives.

### Architecture rationale (vs. inference-side-map)

Two candidates were considered for the migration source:

1. **Inference output** — `infer_let` already computes RHS type;
   could populate a side-map `(let_id, name) → TypeExpr` that the
   walker reads
2. **RHS pattern-match** — for closed set of channel-related
   shapes, derive type-ann directly

Pattern-match wins on the four questions:
- Obvious — same precedent as arc 133's
  `extend_pair_scope_with_tuple_destructure`
- Simple — single function extension; no cross-cutting state
- Honest — closed-set scope matches walker's actual domain
- Good UX — additive; both shapes work; future RHS shapes can
  be added incrementally

Inference-side-map would require new infrastructure used only
by this walker. Heavier than the problem demands.

## Honest deltas surfaced by sonnet

1. **`process_let_binding` silent skip on new shape.** When the
   binder is a Symbol (new shape), the function returns early
   (the substrate path that handles new-shape bindings hasn't
   shipped yet — that's arc 158b's scope). Result: new-shape
   bindings are NOT in the inference `extended` map. The
   post-inference `ScopeDeadlock` walker can't see them; only
   the pre-inference `ChannelPairDeadlock` walker fires (which
   arc 158a migrated to handle new shape).

   **Status:** consistent state. Arc 158a's substrate gives
   new-shape bindings HALF the deadlock detection (pre-inference
   walker fires). Out of arc 158a's scope; tracked in arc 158b
   (binding-shape change) — its `process_let_binding` extension
   populates `extended` for new-shape bindings, restoring full
   deadlock coverage.

2. **Type-ann string format match.** `derive_type_ann_from_rhs`
   produces `:wat::kernel::Channel<wat::core::i64>` (with
   `trim_start_matches(':')` stripping leading `:` from inner
   keyword) — matches `extend_pair_scope_with_tuple_destructure`'s
   format exactly. `parse_type_expr` round-trip works.

3. **Placeholder `nil` element type traces correctly.**
   `Sender<wat::core::nil>` / `Receiver<wat::core::nil>`
   placeholder fires walker correctly because
   `type_is_sender_kind` / `type_is_receiver_kind` check only
   the outer parametric head.

4. **`find_binding_span` + `binding_names` also extended.**
   Sonnet correctly extended these supporting functions in
   `check_let_star_for_scope_deadlock_inferred` so the
   classification loop sees new-shape names. This enables the
   mixed-shape test (test 4) and prepares for arc 158b's
   `process_let_binding` update.

## Tests

`src/check.rs::tests` — 7 new tests:

1. Walker fires on new-shape Channel binding
2. Walker traces `(:wat::core::second pair)` in new shape
3. Legacy shape still works (regression check)
4. Mixed-shape let (legacy + new in one form)
5. Unrecognized new-shape RHS gives up gracefully (no false positive)
6. Arc 128 outer-scope deadlock pattern in new shape
7. Arc 133 destructure-style pattern through new-shape binder

Workspace: 2029 baseline + 7 = 2036 tests; 0 failed; 0 warnings.

## Out of scope (arc 158b will close)

- `process_let_binding` populating `extended` for new-shape
  bindings — arc 158b's substrate work
- Binding-shape change in user code (consumer sweep) — arc 158b
- USER-GUIDE / WAT-CHEATSHEET updates — arc 158b ships those
  alongside the user-visible behavior change

## Cross-references

- **Arc 117** — `ScopeDeadlock` minted; pair-anchor trace
- **Arc 126** — channel-pair deadlock prevention
- **Arc 128** — check walker respects sandbox boundary
- **Arc 131** — `HandlePool` counts as Sender-bearing
- **Arc 133** — closest precedent;
  `extend_pair_scope_with_tuple_destructure` uses the same
  RHS pattern-match recipe
- **Arc 134** — scope-deadlock origin trace
- **Arc 158 v1** — REALIZATIONS at
  `docs/arc/2026/05/158-untyped-let-bindings/REALIZATIONS.md`
  surfaced the walker-vs-binding-shape coupling that prompted
  this arc
- **Arc 158b** — planned next; binding-shape change v2 ships
  on the migrated walker foundation
- **Memory `feedback_stepping_stones_proactive.md`** — proactive
  slicing framework; arc 158a is the explicit stepping stone v1
  skipped. Cite in commit chain.

## Commit chain

- `5b51b67` arc 158 v1 back-out + REALIZATIONS
- `eb7c29e` arc 158a opens (DESIGN + BRIEF + EXPECTATIONS)
- `ca43e56` arc 158a slice 1: substrate (walker pattern-matches RHS)
- (this commit) arc 158a slice 2: closure paperwork
