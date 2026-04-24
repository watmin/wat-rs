# Arc 045 — INSCRIPTION

**Closed:** 2026-04-24.
**Commits:**
- `1eac166` — DESIGN + BACKLOG opened
- `6c6482a` — Slice 1: substrate rename `:abort` → `:panic`
- `6abcb85` — Slice 2: user-facing demo cleanup + :abort prose
  rename + Vec<HolonAST>→Holons typealias fold-in
- `<this commit>` — Slice 3: INSCRIPTION + cross-references

## Why this arc existed

Builder's directive (verbatim, with mid-arc clarification):

> if we see the form (set-capacity-mode! :error) explicitly - it
> can now be omitted. if we see the form (set-capacity-mode!
> :abort) it should be kept and :abort swapped to :panic
> ...
> the default mode is :error - we don't need to state the default
> anymore
> ...
> [later] its :abort - but maybe we should named it :panic?....
> no one is using it outside of tests

Three coupled changes:
1. **Substrate rename `:abort` → `:panic`.** `:panic` matches Rust's
   `panic!()` macro behavior (which unwinds); `:abort` connoted
   `std::process::abort` (no unwinding). Survey showed zero active
   downstream callers — only test fixtures.
2. **Demo cleanup `:error` line removal.** `:error` is the default
   per arc 037; demos showing `(set-capacity-mode! :error)`
   redundantly state the default.
3. **(Mid-slice fold-in)** Replace `:Vec<wat::holon::HolonAST>`
   with the `:wat::holon::Holons` typealias (arc 033) in
   user-facing example signatures. Builder enqueued this during
   the slice-2 sweep as another "true up" item.

## What shipped per slice

### Slice 1 — Substrate rename `:abort` → `:panic`

Five source files touched:

`src/config.rs`:
- Enum variant `CapacityMode::Abort` → `CapacityMode::Panic`.
- Parse arm `":abort" => CapacityMode::Abort` →
  `":panic" => CapacityMode::Panic`.
- Error message strings updated to `"expected :error / :panic"`
  with arc 045 historical note.
- Module-level doc comment updated with rename history.
- 4 unit-test fixture strings: `:abort` → `:panic`.
- `parent_config()` helper: `CapacityMode::Abort` → `::Panic`.

`src/runtime.rs`:
- Match arm `CapacityMode::Abort => panic!` → `CapacityMode::Panic`.
- Panic message: `"capacity exceeded under :abort"` →
  `"under :panic"` with note about unwinding.
- Doc comment for `eval_algebra_bundle` updated.

`src/check.rs:3713`: doc comment updated.

Tests:
- `tests/wat_bundle_capacity.rs`: module header gets arc 045 rename
  note; both test fns renamed (`under_abort_mode` →
  `under_panic_mode`); 2 wat fixture strings updated.
- `tests/wat_run_sandboxed.rs`: comment + 1 wat fixture string
  updated (this fixture would have failed at parse time without
  the rename).

Verification: `cargo test --release` on `wat-rs/Cargo.toml` —
**39 binaries, 850 tests pass, zero failures, zero clippy warnings.**

### Slice 2 — User-facing demo cleanup + :abort prose rename + Holons fold-in

Five files touched:

`README.md` (5 sites):
- 2 demo blocks (echo.wat in §wat binary, deftest example in
  §Self-hosted testing) lose their `(set-capacity-mode! :error)`
  line.
- §Capacity-guard prose: `:error / :abort` → `:error / :panic`;
  mode table row updated; closing paragraph adds rename history.

`docs/CONVENTIONS.md` (2 sites):
- §Namespaces table: `:wat::config::*` description names current
  modes with rename note.
- §Sandbox Config inheritance: example preamble dropped its
  `:error` line; surrounding prose rephrased to "needed only when
  overriding defaults."

`docs/USER-GUIDE.md` (8 sites):
- 6 demo blocks lose `:error` lines (§1 minimal + multi-file +
  test example, §2 echo, §12 canonical shape, §13 deftest).
- §Config setters bullet for `set-capacity-mode!` rewritten to
  show `:panic` as the override, default `:error` named.
- §12 capacity-mode list: `:abort` → `:panic` with arc 045 note.
- 2 example signatures use `:wat::holon::Holons` typealias
  instead of `:Vec<wat::holon::HolonAST>` (arc 033 fold-in).

`docs/README.md` (1 site):
- arc 040 entry: `:error / :abort` → `:error / :panic` with
  rename parenthetical.

`wat/std/test.wat` (3 sites):
- Three usage-comment example blocks lose redundant
  `:error` setter lines; preamble prose rephrased to frame
  setters as override targets.

Final post-sweep grep audit: zero `(set-capacity-mode! :error)`
demo lines remain in user-facing docs; zero raw
`:Vec<wat::holon::HolonAST>` in user-facing docs (typealias
definition sites in `src/types.rs` + arc 033 records keep both
names for clarity).

## What this arc proves

**Substrate rename costs are bounded by survey discipline.**
Pre-rename survey identified 0 active downstream callers (in
`wat/`, `wat-tests/`, `examples/`, `crates/`, lab repo proper),
so the substrate edit was 5 source files + 2 test files; the
docs sweep was prose alignment, not migration. Rename when
the cost is cheapest — that was now.

**Demo discipline = show-the-minimum.** Each `:error` line
removed makes the substrate's opinionated default more visible
to fresh readers. Setters appear in demos only when they're
overriding the default. Same principle as arc 018's
opinionated-defaults move applied at finer granularity.

**Mid-arc directives that compound cleanly.** Builder's three
coupled directives (substrate rename + demo cleanup + Holons
typealias fold-in) all touched the same surfaces (mostly
USER-GUIDE.md examples). Folding them in one slice avoided
re-touching files; the commit message names which fixes belong
to which directive so the audit trail stays clean.

## What stays unchanged (frozen historical)

- All earlier arc INSCRIPTIONs / DESIGN / BACKLOG that mention
  `:abort` (including arc 037 which retired `:silent`/`:warn`,
  arcs 038-044 which described the substrate at slice-close).
  Their `:abort` references are accurate to those slice-close
  moments.
- `arc 005 INVENTORY.md` (preserved per builder's earlier call).
- `scripts/arc_037_slice_5_sweep.py` — frozen tool, ran once.
- Test fixtures verifying that `:silent` / `:warn` / `:abort`
  variant attempts are now rejected (those test the rejection;
  they need the retired strings).

## Files touched

- `src/config.rs`, `src/runtime.rs`, `src/check.rs` — Slice 1.
- `tests/wat_bundle_capacity.rs`, `tests/wat_run_sandboxed.rs` —
  Slice 1.
- `README.md`, `docs/CONVENTIONS.md`, `docs/USER-GUIDE.md`,
  `docs/README.md`, `wat/std/test.wat` — Slice 2.
- `docs/arc/2026/04/045-capacity-mode-rename/{DESIGN,BACKLOG,INSCRIPTION}.md`
  — the arc record.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — cross-repo audit trail row.

## Going forward

The lab repo (`holon-lab-trading/`) will pick up the new name
naturally if/when a future lab arc commits a capacity mode. None
currently do — the lab's `wat/` sources don't override
`:capacity-mode` (they inherit `:error` from the substrate
default).

This is the last polish-class arc planned. Per builder's stance
"treat wat as stable until we find it isn't — the polish is for
the next llm to observe": further drift catches will surface
when the next caller hits something that doesn't work, not from
preemptive iteration.
