# BRIEF — STONE F: a type parameter does not launder impurity into a pure aggregate

Read `DESIGN-stone-F-a-type-parameter-is-not-purity.md` first. It hands you the INVARIANT and a
two-layer shape; the exact choke points are yours to find, and that finding is part of the work.

## First, reproduce — including the one case the orchestrator could not

The three record rows in the DESIGN, plus a **generic `:Pure` enum** holding a handle. The
orchestrator's own enum repro was malformed syntax, so it is not evidence; stone E's probe
(`tests/diagnostics/probe_ex003_hashability_looks_inside__enum_*.wat`) has a working enum shape.

## Rooms

1. `src/check.rs:15035` — `is_pure_type`, especially the `Path` arm's `None => true`.
2. `src/check.rs` — `validate_aggregate_containment` and its enum sibling
   (`ImpureVariantFieldInPureEnum`): they run over DECLARATIONS, where `T` is still `T`.
3. Where the checker types a constructor call and an annotation `(Head :- [args])` — find where
   type arguments get bound for a registered aggregate/enum. **Report whether there is one choke
   point or several.**
4. Where the runtime evaluates a pure aggregate / enum constructor — for layer 2.
5. `src/runtime.rs` — `value_is_hashable` (stone E) shows the deep-walk shape; the impurity test
   for a VALUE is the question "does this value contain an opaque handle / interior-mutable value",
   which `key_eligibility()`'s `NeverAKey(..)` reasons already classify per variant.

## STOP triggers

1. **Layer 1 would refuse a generic function that builds `Box<T>` for a pure `T`.** That outlaws a
   truth; the DESIGN assigns that case to layer 2. Report the shape; do not ship the over-refusal.
2. **No usable choke point for constructor typing** — the binding of type arguments happens in many
   unrelated places. Report the list; do not patch one and call it closed.
3. **The corpus has a pure aggregate that legitimately holds something `is_pure_type` calls impure.**
   Report it with the site; do not weaken the invariant to make the floor green.
4. **Any gate reddens that you did not add** — capture whole, name the arm. ⛔ Do not re-run first.

## Prove it

A probe: annotated, inferred, generic-fn-built, and enum — each with a handle (refused, at the layer
you say) and each with a pure payload (accepted — the positive controls matter here, because this
change can over-refuse). Plus the EDN row: a pure record can no longer be written with a handle
silently turned into `nil`. **Mutation per layer:** disable each layer alone and say which rows red.

## Mechanics — ⛔ read

- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks. Do not end your turn while it
  runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines, never piped exit codes.
- `cargo fmt` reformats the whole workspace — `rustfmt <file>` or neither.
- `git add` BEFORE `git ls-files`-based gates. New files under `tests/` answer to
  `no_loose_string_assert`, `no_inlined_edn`, `no_inlined_wat_in_tests`, `every_tracked_wat_parses`,
  `every_wat_bad_fixture_actually_fails`.
- A runtime-error golden that embeds a `.rs` line in a `:message` STRING cannot be masked by
  `blank_rust_source_lines` — stone E re-parsed tagged-EDN `:message` strings; reuse that.
