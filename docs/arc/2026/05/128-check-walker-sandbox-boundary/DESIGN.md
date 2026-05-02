# Arc 128 — Check walker respects sandbox boundary

**Status:** **shipped + closed 2026-05-01.** See INSCRIPTION.md
for the close-out (4 questions + verification + what unblocks).
DESIGN below is the as-drafted record kept verbatim.

## Provenance

Arc 126 slice 1 surfaced a structural defect when sonnet's
implementation passed all design criteria but broke the workspace:
2 of 14 tests in `wat-holon-lru` failed
(`HologramCacheService::test_step1_spawn_join` and
`..._test_step2_counted_recv`) because the file-level freeze of
`HologramCacheService.wat` walked the bodies of step3-6 (which
contain the channel-pair-deadlock anti-pattern) and emitted 17
`channel-pair-deadlock` errors at outer freeze time. The errors
broke the file's freeze; ALL deftests in the file failed to run,
not just the deadlock-bearing ones.

The pre-handoff EXPECTATIONS predicted this as the failure mode:
"workspace red because the rule fires on a substrate pattern the
DESIGN's caveats didn't anticipate." The actual cause: not a
substrate pattern false-positive, but a walker scope error — the
walker descending into AST that represents an INNER PROGRAM, not
outer code.

User direction (2026-05-01):

> "it skips execution but does NOT gate file-level freeze.
> how do we attack this problem?"

## The defect

Each `:wat::test::deftest-hermetic` macro expands to:

```
(:wat::core::define (<test-name> -> :wat::test::TestResult)
  (:wat::kernel::run-sandboxed-hermetic-ast
    (:wat::core::forms <prelude> (:wat::core::define :user::main ...)
                       <body>)
    (:wat::core::Vector :wat::core::String)
    :wat::core::None))
```

The body of the deftest is wrapped inside `(:wat::core::forms ...)`
which is the first argument to
`:wat::kernel::run-sandboxed-hermetic-ast`. At runtime, when the
deftest function is invoked, `run-sandboxed-hermetic-ast` freezes
its inner program (a separate freeze unit) and runs it.

But at OUTER freeze of `HologramCacheService.wat`, the structural
check walkers (today: arc 117's `walk_for_deadlock`; with arc 126's
landing: `walk_for_pair_deadlock`) recurse into ALL `WatAST::List`
nodes including the forms-block. They treat the inner program's
AST as outer code, fire their checks, and the outer freeze fails
with errors that belong to the inner program.

This is **a scope error in the walker**, not a check-rule error.
Arc 117 has the same latent defect — no deftest today exercises a
scope-deadlock pattern, so it hasn't surfaced. Arc 126's landing
reveals the latent defect because more deftests exercise its rule.

## The rule

> Structural check walkers MUST NOT descend into the first
> argument (the forms-block) of inner-program-spawning primitive
> calls:
>
> - `:wat::kernel::run-sandboxed-ast`
> - `:wat::kernel::run-sandboxed-hermetic-ast`
> - `:wat::kernel::fork-program-ast`
> - `:wat::kernel::spawn-program-ast`
>
> The trailing arguments (type-list, config args) are still walked
> normally. Inner-program forms are checked at INNER freeze when
> the sandbox-program primitive fires at runtime.

The rule is per-walker. Arc 117's `walk_for_deadlock` gets the
guard. Arc 126's `walk_for_pair_deadlock` (when it relands)
includes the guard from inception. Future structural-check walkers
inherit the convention.

## Impact

After arc 128 lands:

| Today | Post-arc-128 |
|---|---|
| Outer freeze of HologramCacheService.wat fails with errors from step3-6 bodies | Outer freeze passes; step3-6 bodies are skipped at outer level |
| ALL 6 deftests fail to run | Each deftest runs independently |
| Author can't selectively test step1 + step2 | step1, step2 run cleanly; step3-6 panic at INNER freeze with the substring |
| `:should-panic(expected="...")` cannot work — outer freeze fails before deftest runs | `:should-panic(expected="channel-pair-deadlock")` matches the inner-freeze panic |

The inner program's freeze still runs the same checks. Errors are
LOCALIZED to the offending deftest, not propagated to siblings.
This is the correct UX.

## Detection of the boundary

Inner-program-spawning primitives are KNOWN keywords. The walker
matches the form's head against a hard-coded set:

```rust
matches!(head,
    ":wat::kernel::run-sandboxed-ast"
    | ":wat::kernel::run-sandboxed-hermetic-ast"
    | ":wat::kernel::fork-program-ast"
    | ":wat::kernel::spawn-program-ast"
)
```

When matched, the walker recurses into `items.iter().skip(2)` —
skipping `items[0]` (the head keyword) and `items[1]` (the
forms-block). Trailing args (`items[2..]`) are still walked.

## Cost / honest tradeoff

**Lost:** proactive bug-catching for deadlock-class shapes inside
forms-blocks at outer freeze time. Today (today before fix), if
you write a deftest body with a scope-deadlock pattern, the outer
freeze catches it. Post-arc-128, the inner freeze catches it when
the deftest runs.

**Gained:** errors localize to the offending deftest. Sibling
tests in the same file run independently. `:should-panic`
annotations work correctly. The check rule semantics match the
substrate's actual sandboxing semantics.

The trade is worth it: today's "global file freeze fails" UX is
strictly worse than tomorrow's "the offending deftest fails."

## The four questions

**Obvious?** Yes. Forms-blocks are inner programs; inner programs
have their own freeze. The walker descending into them treats data
as code. Once you see the inner/outer distinction, the boundary is
obvious. The substrate's runtime semantics (each
`run-sandboxed-*-ast` call is a separate freeze) IS the truth; the
walker just needs to honor it.

**Simple?** Yes. ~10 LOC per walker — one match arm with a list of
4 keyword names + a `skip(2)` recursion. No new types, no new AST
shapes, no flow analysis. Direct substring match against the form
head.

**Honest?** Yes. The rule names the actual structural truth:
inner-program forms are not outer-frozen code. The substrate
already treats them this way at runtime; arc 128 just makes the
check walker treat them the same way. Mirror the runtime's scope.

The honest cost — proactive bug-catching for forms-block contents
at outer freeze time — is documented and accepted. Inner freeze
still catches the same bugs; the catch happens at deftest-run
time instead of file-load time. This is acceptable per the
substrate-as-teacher discipline (the inner program teaches the
deftest author when it runs).

**Good UX?** Phenomenal. Today's UX is broken — sibling tests
cascade-fail when one has a buggy body. Post-arc-128, tests are
independently runnable. `:should-panic` works. Errors localize.
Each deftest is a unit. This is the UX cargo-test parity has been
working toward through arcs 121-124.

## Implementation plan

### Slice 1 — the boundary guard

`src/check.rs` adds:

- A constant `SANDBOX_PROGRAM_HEADS` listing the 4 keywords (or
  inline the match-list — single-site use).
- An early-match arm in `walk_for_deadlock` (~line 1734): when
  `head` matches the sandbox set, recurse into `items.iter().skip(2)`
  and return; otherwise fall through to existing logic.
- Two unit tests (mirror arc 117's pattern):
  - `scope_deadlock_skips_sandboxed_forms` — anti-pattern inside
    a `(:wat::kernel::run-sandboxed-hermetic-ast (forms ...) ...)`
    block does NOT fire `ScopeDeadlock`.
  - `scope_deadlock_still_fires_at_outer_scope` — the SAME
    anti-pattern at the outer scope (no surrounding sandbox call)
    DOES fire `ScopeDeadlock`. (Already exists; verify still green.)

Estimated total: ~30 LOC (10 LOC fix + 20 LOC tests).

### Slice 2 — closure

INSCRIPTION + cross-reference from arc 126 DESIGN noting that arc
126's `walk_for_pair_deadlock` includes the guard from inception
(arc 126's reland inherits the convention).

058 changelog row.

### Verification

- `cargo test --release -p wat --lib check` — new tests + existing
  arc 117 tests all pass.
- `cargo test --release --workspace` — workspace stays green. No
  regressions. Existing 6 ignored deadlock tests stay ignored
  (their bodies don't fire arc 117's check; arc 126's check isn't
  landed yet).

## Cross-references

- `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
  — the precedent walker that gets the guard added.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/DESIGN.md`
  — blocked on this arc; will inherit the convention from
  inception when it relands.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/SCORE-SLICE-1.md`
  — the slice 1 sonnet sweep that surfaced this defect.
- `docs/arc/2026/04/103-kernel-spawn/INSCRIPTION.md` —
  `spawn-program-ast` provenance.
- `docs/arc/2026/04/104-wat-cli-fork-isolation/INSCRIPTION.md` —
  `fork-program-ast` provenance.
- `docs/arc/2026/04/105-spawn-error-as-data/INSCRIPTION.md` —
  the wat-side `run-sandboxed` migration to wat/std/sandbox.wat.
- `wat/test.wat` lines 304-414 — the deftest macros that expand
  to `run-sandboxed-*-ast` calls; the substrate user-surface that
  motivates the boundary.

## What this arc does NOT do

- Does NOT change runtime semantics. Sandbox-program primitives
  still freeze inner forms at runtime; the check walker just stops
  redundantly walking the same forms at outer freeze.
- Does NOT touch `.wat` files. No deftest body changes; no
  `:ignore` / `:should-panic` annotation changes.
- Does NOT remove arc 126's blockedness. Arc 126 slice 1 needs
  re-spawning AFTER arc 128 lands; the new walker has the guard
  from inception so the previously-collateral failures don't
  occur.
- Does NOT introduce a new annotation. The boundary is structural
  (the head keyword set), not declared per-deftest.
