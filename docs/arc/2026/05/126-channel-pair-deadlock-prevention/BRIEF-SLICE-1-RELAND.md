# Arc 126 Slice 1 RELAND — Sonnet Brief

**This is the reland brief.** The first sonnet sweep
(`a37104bfc10e4c6fa`, ~13.5 min) produced a correct
implementation that PASSED 5 of 6 hard scorecard rows but FAILED
on workspace green — the new check fired on deftest bodies
(wrapped in `run-sandboxed-hermetic-ast` forms) at outer file
freeze, cascading failures to sibling tests in the same file.
Sonnet's diff is preserved at
`/tmp/arc-126-slice-1-sonnet-a37104bf.patch` (604 lines).

The reland follows after **arc 128 ships the sandbox-boundary
fix to the structural check walker**. Arc 117's
`walk_for_deadlock` now includes a guard that skips the first
argument (the forms-block) of `:wat::kernel::run-sandboxed-ast`
/ `run-sandboxed-hermetic-ast` / `fork-program-ast` /
`spawn-program-ast` calls. Arc 126's new `walk_for_pair_deadlock`
MUST INHERIT THIS GUARD FROM INCEPTION.

Read the original brief at `BRIEF-SLICE-1.md` first — the
algorithm + diagnostic substring lock + read-in-order anchors +
NOT-do list all carry over. THIS document amends three rows:

## Amendment 1 — read arc 128 first

Add to the read-in-order list (before reading
src/check.rs's arc-117 functions):

1. `docs/arc/2026/05/128-check-walker-sandbox-boundary/INSCRIPTION.md`
   — the boundary doctrine your `walk_for_pair_deadlock` MUST
   honor.
2. `docs/arc/2026/05/126-channel-pair-deadlock-prevention/SCORE-SLICE-1.md`
   — the previous attempt's scorecard. Your reland inherits the
   structural choices that scored well; the boundary guard fixes
   the row that failed.

Then continue with arc 126 DESIGN, arc 117 DESIGN+INSCRIPTION,
src/check.rs functions in their original order.

## Amendment 2 — mandatory boundary guard

`walk_for_pair_deadlock` MUST include the arc 128 boundary
guard. Read `walk_for_deadlock` in `src/check.rs` (currently
~line 1750-1770 post-arc-128) — the canonical pattern is:

```rust
// Arc 128 — sandbox-boundary guard.
if matches!(
    head,
    ":wat::kernel::run-sandboxed-ast"
        | ":wat::kernel::run-sandboxed-hermetic-ast"
        | ":wat::kernel::fork-program-ast"
        | ":wat::kernel::spawn-program-ast"
) {
    for child in items.iter().skip(2) {
        walk_for_pair_deadlock(child, types, errors);
    }
    return;
}
```

Mirror this verbatim in `walk_for_pair_deadlock` immediately
after head extraction, before any other dispatch logic.

This is non-negotiable. Without the guard, slice 1 will
recapitulate the previous failure: outer freeze of
`HologramCacheService.wat` will fire on step3-6 deftest bodies
and break step1+step2.

## Amendment 3 — add a boundary unit test

Add a fifth unit test (in addition to the four named in the
original brief):

- **`channel_pair_deadlock_skipped_in_sandboxed_forms`** — mirror
  arc 128's `arc_128_inner_scope_deadlock_skipped_in_sandboxed_forms`
  test. Construct a wat source containing the channel-pair
  anti-pattern wrapped in a `run-sandboxed-hermetic-ast`
  forms-block. Assert NO `ChannelPairDeadlock` fires at outer
  freeze. Verifies the boundary guard works for arc 126's check.

Read arc 128's two unit tests (`arc_128_outer_scope_deadlock_still_fires`
and `arc_128_inner_scope_deadlock_skipped_in_sandboxed_forms`) at
the end of `src/check.rs::tests` for the construction pattern.
Mirror their `check(src)` helper-call structure.

## What's the same

Everything else from the original brief carries over:
- ONE file changes (`src/check.rs`).
- The diagnostic substring `channel-pair-deadlock` MUST appear
  verbatim. (Slice 2's `:should-panic` annotations match against
  it.)
- All other unit tests from the original brief
  (canonical-anti-pattern fires; two-different-pairs silent;
  HandlePool-pop silent; substring assertion).
- Workspace stays GREEN.
- ~200 LOC budget; >300 LOC = stop and report.
- No commits, no `.wat` edits, no docs sweep.

## Specific expected outcomes (post-reland)

After your work:

- `cargo test --release -p wat --lib check`: all check tests
  pass (existing arc 117 + arc 128 + your 5 new arc 126).
- `cargo test --release --workspace`: exit=0; same 7 ignored
  tests as today (no regression).
- The 6 deadlock-bearing tests stay ignored. Slice 2 converts
  them to `:should-panic` in a separate session.
- Your patch is committable as-is on a green workspace.

## Reporting back

Same shape as the original brief (target ~150 words):

1. File:line refs for variant + Display + each function.
2. Unit test count: 5 added; all passing. Specifically confirm
   the boundary test (`..._skipped_in_sandboxed_forms`) passes.
3. Workspace test totals (passed / failed / ignored).
4. The exact panic message. Slice 2's substring source.
5. Any honest delta from arc 117's walker pattern — if you
   needed a non-trivial deviation, surface it.

## Substrate-as-teacher check

This brief is the second test of the same discipline. The first
sweep produced a correct implementation that surfaced a
substrate gap (arc 128). This sweep should produce the same
implementation + the boundary guard, on a workspace where the
boundary already holds. If the discipline is intact, the only
delta from sweep 1 to sweep 2 is the guard mirror + the
boundary unit test.

Working directory: `/home/watmin/work/holon/wat-rs/`.
