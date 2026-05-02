# Arc 128 — INSCRIPTION

## Status

**Shipped + closed 2026-05-01.** Single substrate fix to
`src/check.rs::walk_for_deadlock`; total diff ~120 LOC (15 LOC
guard + 105 LOC of two unit tests). Workspace test passes
(exit=0; 100 `test result: ok` lines; 7 ignored — same set as
pre-arc-128, no regression).

## What this arc adds

Structural check walkers respect the sandbox boundary. The walker
no longer descends into the first argument (the forms-block) of:

- `:wat::kernel::run-sandboxed-ast`
- `:wat::kernel::run-sandboxed-hermetic-ast`
- `:wat::kernel::fork-program-ast`
- `:wat::kernel::spawn-program-ast`

Inner-program forms passed to these primitives have their own
freeze cycle at runtime when the primitive fires; the outer
walker stops at the boundary and recurses only into trailing args
(type-list, config arg).

The fix lands in arc 117's `walk_for_deadlock`. Arc 126's
`walk_for_pair_deadlock` (when it relands) inherits the same
guard pattern from inception.

## Why

Arc 126 slice 1's sonnet sweep produced a correct implementation
of the channel-pair deadlock check, but exposed a structural
defect in the existing walker design: arc 117's
`walk_for_deadlock` (and any sibling structural walker) descends
into ALL `WatAST::List` nodes, including forms-blocks that
represent INNER programs. Those inner forms get checked TWICE —
once at outer file freeze (where errors propagate to siblings),
once at inner freeze when the sandbox primitive fires at runtime.

The outer check is redundant AND scope-conflated: errors
belonging to the inner program break the outer file's freeze,
cascading failures to sibling deftests that have nothing wrong
with them.

User direction (2026-05-01):

> "it skips execution but does NOT gate file-level freeze.
> how do we attack this problem?"

The substrate-level answer: honor the sandbox boundary in the
check walker, mirroring how the substrate already treats inner
programs at runtime.

## The four questions

**Obvious?** Yes. Forms-blocks ARE inner programs (the substrate
runtime treats them this way). The walker descending into them
treats data as code. Once you see the inner/outer distinction,
the boundary is obvious. The fix just makes the check walker
honor what the runtime already enforces.

**Simple?** Yes. ~15 LOC guard in `walk_for_deadlock`: one
`matches!` over four keyword names, one `skip(2)` recursion,
explicit `return`. No new types; no flow analysis; no new AST
shapes.

**Honest?** Yes. The rule names the structural truth:
inner-program forms aren't outer-frozen code. The honest cost —
proactive bug-catching at outer freeze for forms-block contents
disappears — is documented and accepted. Inner freeze still
catches the same bugs at deftest-run time. The trade is correct
because today's "global file freeze fails" UX is strictly worse.

**Good UX?** Phenomenal. Today's UX is broken: sibling deftests
cascade-fail when one has a buggy body. After arc 128, each
deftest is independently runnable; `:should-panic` works
correctly; errors localize to the offending deftest. This is the
UX cargo-test parity has been working toward through arcs
121-124.

## Detection algorithm

In `walk_for_deadlock`, after extracting the form's head keyword,
add an early-match arm:

```rust
if matches!(
    head,
    ":wat::kernel::run-sandboxed-ast"
        | ":wat::kernel::run-sandboxed-hermetic-ast"
        | ":wat::kernel::fork-program-ast"
        | ":wat::kernel::spawn-program-ast"
) {
    for child in items.iter().skip(2) {
        walk_for_deadlock(child, types, errors);
    }
    return;
}
```

`items[0]` is the head keyword; `items[1]` is the forms-block
(skip); `items[2..]` are the trailing args (type-list, optional
config arg) which are still walked.

## Verification

- New unit tests in `src/check.rs::tests`:
  - `arc_128_outer_scope_deadlock_still_fires` — same anti-pattern
    at outer scope (no surrounding sandbox call) STILL fires
    `ScopeDeadlock`. Verifies arc 117 is intact.
  - `arc_128_inner_scope_deadlock_skipped_in_sandboxed_forms` —
    SAME anti-pattern wrapped in `run-sandboxed-hermetic-ast`'s
    forms-block does NOT fire. Verifies the boundary.
  - Both pass (`cargo test --release -p wat --lib check::tests::arc_128`:
    `2 passed; 0 failed`).
- Workspace test (`cargo test --release --workspace`): exit=0;
  100 `test result: ok` lines; 7 ignored (same set as pre-arc-128,
  no regression).

## What this arc unblocks

**Arc 126 reland.** The slice 1 sonnet sweep produced correct
code that broke step1+step2 in `HologramCacheService.wat`
because the file's outer freeze fired the new
`ChannelPairDeadlock` check on step3-6's bodies. Post-arc-128,
the outer freeze skips deftest forms-blocks; the inner freeze
fires the check when each deftest runs; `:should-panic`
annotations on step3-6 will correctly catch the inner-freeze
panic.

Arc 126's reland will be a re-spawn of sonnet on the same
DESIGN+brief; the brief will gain one constraint: the new
walker must include the same sandbox-boundary guard from
inception (mirror arc 117's now-corrected pattern).

## Cross-references

- `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
  — the precedent walker that gets the guard added. Arc 117 had
  the latent same defect (no deftest today exercises it).
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/DESIGN.md`
  — blocked on this arc; reland inherits the guard convention.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/SCORE-SLICE-1.md`
  — the slice 1 sonnet sweep that surfaced this defect.
- `src/check.rs::walk_for_deadlock` — the function that gets the
  guard.
- `src/check.rs::tests` — the two new unit tests (mirror arc 117
  test pattern).

## What this arc does NOT do

- Does NOT change runtime semantics. Sandbox-program primitives
  still freeze inner forms at runtime; the check walker just
  stops redundantly walking the same forms at outer freeze.
- Does NOT touch `.wat` files. The 6 ignored deadlock-class
  tests stay ignored (their bodies still trip the inner freeze
  when run, but the outer freeze passes cleanly).
- Does NOT introduce a new annotation. The boundary is structural
  (the head keyword set), not declared per-deftest.
- Does NOT remove arc 126's blockedness yet. Arc 126's slice 1
  reland follows in a separate session.
