# Arc 133 Slice 1 — Sonnet Brief

**Goal:** close the structural-enforcement bypass that hides
tuple-destructure bindings from arc 117/131's
`ScopeDeadlock` check (and arc 126's `ChannelPairDeadlock`
check). After this slice, a let* binding of shape
`((pool driver) (:wat::lru::spawn ...))` fires the same
diagnostic as the equivalent typed-name shape.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** Slice 1 = walker extension + unit tests in
`src/check.rs`. NO `.wat` files modified. If existing
wat-tests newly fire the check, that's slice-2 work
(separate session). NO commits, NO pushes.

## Read-in-order anchor docs

1. `docs/arc/2026/05/133-tuple-destructure-binding-check/DESIGN.md`
   — the rule, the bypass, the four-questions framing,
   the failure-engineering provenance. Source of truth.
2. `docs/arc/2026/05/131-handlepool-scope-deadlock/SCORE-SLICE-2.md`
   § "Latent gap surfaced" — the SCORE doc that named this
   bypass after the slice 2 sonnet sweep.
3. `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
   — the parent rule. Section "The diagnostic" shows the
   pre/post canonical-fix shape; the same shape applies
   uniformly post-arc-133.
4. `docs/arc/2026/05/126-channel-pair-deadlock-prevention/INSCRIPTION.md`
   — the sibling rule with the SAME bypass. Both checks'
   binding-parse functions skip tuple-destructure today.

## The substrate gap

Two functions in `src/check.rs` parse let* bindings to feed
the structural-deadlock checks:

- **`parse_binding_for_typed_check`** (line 1940) — used by
  arc 117/131's `ScopeDeadlock`. Returns
  `(name, type_keyword_string, span)` for typed-name shape
  `((name :type-keyword) rhs)`. Returns `None` for any
  other shape — including tuple-destructure
  `((name1 name2 ...) rhs)`.
- **`parse_binding_for_pair_check`** (line 2382) — used by
  arc 126's `ChannelPairDeadlock`. Same constraint: typed-
  name only; tuple-destructure returns `None`.

Both bypass the same shape. Tests that bind via tuple-
destructure pattern — for example,
`((pool driver) (:wat::lru::spawn 16 1 ...))` — get NO
classification feedback into the checks. The bindings are
silently invisible.

The result: tests with the deadlock-prone STRUCTURE but
the tuple-destructure SHAPE are "correct by accident"
today — the check doesn't see them, so no fire. They might
deadlock at runtime; the structural enforcement doesn't
catch them.

Arc 131 slice 2's sonnet surfaced this: 12 of 15 surveyed
files were canonical, but some "canonical 12" might have
non-canonical shape that's just shielded by the bypass.
We don't know which. Arc 133 closes the gap so the check
sees every binding shape.

## What changes

The substrate already runs inference over let* bindings
including tuple-destructure. Look at
`src/check.rs::process_let_binding` (line 5413+) for how
inference handles each shape:

- Lines 5433-5478: typed-single shape. Parses the type
  annotation, infers the RHS, unifies, binds the name.
- Lines 5480-5508: tuple-destructure shape. Generates fresh
  tuple-vars per name, infers the RHS, unifies against the
  fresh tuple, binds each name to its post-substitution
  element type. The `out_scope: HashMap<String, TypeExpr>`
  receives one entry per destructured name.

Inference already produces the per-name TypeExpr we need.
The check needs to consume it.

### Recommended approach (investigate; override if better)

Add a per-let-star post-inference deadlock check inside
`infer_let_star` (line 5048+). After all bindings are
processed (`extended` HashMap is fully populated), run the
classification using `extended` directly:

```rust
fn infer_let_star(...) -> Option<TypeExpr> {
    // ... existing binding-processing loop ...

    // Arc 133 — post-inference scope-deadlock check.
    // Uses inferred types directly, covering all binding
    // shapes (typed-name + tuple-destructure) uniformly.
    check_let_star_for_scope_deadlock_inferred(
        bindings,                 // for span info per name
        &args[1],                 // body
        &extended,                // name → inferred TypeExpr
        env.types(),
        errors,
    );
    check_let_star_for_pair_deadlock_inferred(
        bindings,
        &args[1],
        &extended,
        env.types(),
        errors,
    );

    infer(&args[1], env, &extended, fresh, subst, errors)
}
```

Where the new functions are post-inference siblings of the
existing structural walkers — same classification logic
(`type_is_thread_kind`, `type_contains_sender_kind`,
`type_is_sender_kind`, `type_is_receiver_kind`) but reading
from the inferred-types map instead of re-parsing source-
text annotations.

Decide whether to RETIRE the structural walkers
(`validate_scope_deadlock` + `validate_channel_pair_deadlock`)
once the inference-time check covers their work, OR keep both
as belt + suspenders. The structural walkers run BEFORE
inference at top level — their advantage is robustness against
inference-time errors. The inference-time check has access to
inferred types — its advantage is uniform shape coverage. The
former probably retires; the latter is the load-bearing path
post-arc-133.

**Alternative path** (if the in-place approach hits an
unexpected blocker): extend `parse_binding_for_typed_check`
+ `parse_binding_for_pair_check` to recognize tuple-
destructure shape. For each name, look up the RHS-call's
return-type via a passed-in CheckEnv reference; expand
aliases via TypeEnv; bind each name to its tuple-element
type. This requires plumbing CheckEnv into the walker
signatures (currently `&TypeEnv` only). More invasive but
preserves the two-pass structure.

Pick the path that fits the substrate cleanly. Investigate
both before committing to one. Surface the trade-off in your
report.

## Required unit tests

Add to `src/check.rs::tests`:

1. **`arc_133_typed_name_binding_still_fires`** — sanity: the
   existing typed-name shape continues to fire ScopeDeadlock
   (regression guard). Hand-craft a let* with
   `((pool :HandlePool<MyHandle<...>>) (...))` sibling to
   `((thr :Thread<...>) (...))` + `(Thread/join-result thr)`
   in body. Assert error fires, kind = "HandlePool".
2. **`arc_133_tuple_destructure_with_handlepool_fires`** —
   the new path. Hand-craft a let* with
   `((pool driver) (some-spawn-fn))` where the spawn returns
   `(HandlePool<MyHandle<K,V>>, Thread<unit, unit>)` (use a
   typealias) and `(Thread/join-result driver)` in body.
   Assert ScopeDeadlock fires, offending_binding = "pool",
   offending_kind = "HandlePool".
3. **`arc_133_tuple_destructure_silent_when_clean`** — the
   negative path. Tuple-destructure where RHS returns
   `(i64, Thread<unit,unit>)` (no Sender-bearing). Assert
   NO error fires.
4. **`arc_133_tuple_destructure_pair_check_fires`** — the
   sibling check. Hand-craft a let* binding
   `((tx rx) (:wat::kernel::make-bounded-channel :i64 1))`
   followed by a function call passing both `tx` and `rx`
   to one helper. Assert `ChannelPairDeadlock` fires.

Mirror the existing arc-117 + arc-131 unit-test patterns at
the end of `src/check.rs::tests`.

## Constraints

- ONE Rust file changes: `src/check.rs`. No `.wat` files in
  this slice. No other Rust files. No documentation.
- Workspace test MAY FAIL on existing wat-tests that newly
  fire the check (correct-by-accident tuple-destructure
  shapes). DO NOT fix those tests in this slice — slice 2
  handles the sweep. Run
  `cargo test --release -p wat --lib check` to verify the
  unit tests pass; the workspace test failures are
  EXPECTED data for slice 2.
- The new offending_kind values stay consistent with arc 117
  + 131: "HandlePool", "Sender", "QueueSender". No new kind
  strings for arc 133 (the rule is the same; the binding
  shape changes).
- No new public API. No new types. Minimize surface area.
- NO commits, NO pushes. Working tree stays modified.

## What success looks like

1. `cargo test --release -p wat --lib check` exit=0; new
   unit tests pass; existing arc 117 + arc 131 tests still
   pass.
2. The four arc_133_* unit tests prove all three flow paths
   (typed-name unchanged + tuple-destructure with
   HandlePool fires + tuple-destructure clean is silent +
   ChannelPairDeadlock fires on tuple-destructure).
3. Workspace test prediction (DO NOT RUN unless you can
   keep it under 60s with timeouts): grep
   `wat-tests/ crates/*/wat-tests/` for tuple-destructure
   spawn patterns to estimate; surface the count.
4. No commits.
5. Honest report.

## Reporting back

Target ~250 words:

1. Approach chosen (in-place inside `infer_let_star`,
   walker-with-CheckEnv, or a third path you found). Why.
2. File:line refs for each new / changed function.
3. Whether any existing structural walker retired (and if
   so, what the diff looked like).
4. Unit test count + pass status (`cargo test --release -p
   wat --lib check`).
5. Workspace test prediction: grep counts for tuple-
   destructure patterns + spawn-call shapes that might
   newly fire. Confidence: rough-only.
6. Honest deltas: anything you needed to invent (e.g.
   the exact Display message for tuple-destructure
   binding spans, fresh helper functions, etc.).

## What this brief is testing (meta)

Per the failure-engineering chain, arc 133 is the gap
surfaced by arc 131 slice 2's sonnet sweep. The discipline:
every observed gap becomes an arc; the artifacts teach the
next agent without conversation context. This brief tests
whether the discipline scales when the substrate fix
requires a path-choice (in-place vs walker-extension)
rather than a literal mechanical edit.

The substrate already runs inference over the binding
shape we want to check. Reading
`src/check.rs::process_let_binding` carries the lesson:
the inference machinery answers what the structural walker
can't see directly. Arc 133's clean implementation
recognizes that the inferred-types map IS the migration
brief for the walker.

Begin by reading the DESIGN, then arc 131's SCORE-SLICE-2
§ "Latent gap surfaced", then arc 117's INSCRIPTION
§ "The diagnostic", then `parse_binding_for_typed_check` +
`parse_binding_for_pair_check` + `process_let_binding` +
`infer_let_star`. Then pick the approach. Then implement.
Then unit-test. Then report.
