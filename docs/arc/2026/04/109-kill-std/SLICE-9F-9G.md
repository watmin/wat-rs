# Arc 109 Slice 9f-9g — Pure file-path moves for edn + test

**Status: shipped 2026-05-01.** Two `git mv` operations + 2
`WatSource` entries updated in `src/stdlib.rs` + 1 doc comment
in `src/assertion.rs`. cargo test --release --workspace 1476/0.
The simplest substrate slice in arc 109's catalog: pure file
move, zero symbol changes, zero walker, zero consumer sweep.

## What this slice does

Two pure file moves where the symbols are ALREADY aligned with
the namespace segment, so the file basename matches the symbol
path's leaf. § G's filesystem-path-mirrors-FQDN rule is satisfied
by moving the file alone — no symbol renames needed.

| Slice | From | To | Shipped FQDN (unchanged) |
|---|---|---|---|
| 9f | `wat/std/edn.wat` | `wat/edn.wat` | `:wat::edn::*` (basename `edn` = namespace `edn`) |
| 9g | `wat/std/test.wat` | `wat/test.wat` | `:wat::test::*` (basename `test` = namespace `test`) |

## Scope split out from 9h-9i

The original slice 9f-9i plan bundled four moves. Two of them
(sandbox + hermetic) needed deeper work — gaze ward (2026-05-01)
flagged `run-sandboxed-hermetic-ast` as Level 1 (doubly-named for
one property) and noted the two files are twins differing only
in transport (Thread vs Process). They've been split out as
slice K.thread-process per the user direction:

> i support this renaming - add this kernel work to the backlog
> - do the other std/ files first

Slice K.thread-process retires `:wat::kernel::run-sandboxed*` →
`:wat::kernel::Thread/run-ast` + `:wat::kernel::Process/run-ast`,
moves files to `wat/kernel/thread.wat` + `wat/kernel/process.wat`,
and adds a shared `Program/drive` (poly over Program<I,O>). Bigger
work than file move; tracked separately.

9f-9g stays as a pure move because edn / test already satisfy the
basename-equals-namespace-leaf invariant.

## Why this is trivial

- **No symbol changes** — every shipped path stays exactly as is.
  No walker; no `CheckError` variant; no consumer migration.
- **No type-checker work** — wat code referring to
  `:wat::edn::read` keeps working with zero churn.
- **No test sweep** — programs use the FQDN names; nothing
  references the file path at runtime.
- The ONLY thing that changes: the substrate's bundled stdlib
  registration in `src/stdlib.rs` (the `path:` field + the
  `include_str!` argument), plus any source-tree references to
  the old filesystem path in comments / docstrings.

## What to ship

### Substrate (Rust + filesystem)

1. **Two `git mv` operations** (already done; staged):
   ```bash
   git mv wat/std/edn.wat   wat/edn.wat
   git mv wat/std/test.wat  wat/test.wat
   ```

2. **Update `src/stdlib.rs`** — two `WatSource` entries change.
   For each: `path:` field + `include_str!` argument updated
   correspondingly.

3. **Sweep filesystem-path references** elsewhere:
   - Doc comments mentioning `wat/std/edn.wat` / `wat/std/test.wat`
   - Module-level docs if any cite the old paths
   - Possibly arc INSCRIPTIONs that quote specific bundled paths
   - Run: `grep -rln 'wat/std/edn\|wat/std/test'`

4. **Verify**:
   - `cargo build --release` clean
   - `cargo test --release --workspace` 1476/0
   - `ls wat/std/` shows: `hermetic.wat`, `sandbox.wat`,
     `service/` (the three remaining K.thread-process +
     K.console targets).

### What does NOT change

- Every `:wat::edn::*`, `:wat::test::*` shipped symbol path.
- Any user wat code calling `:wat::edn::read`,
  `:wat::test::deftest`, etc.
- The runtime registration order (preserved by keeping the
  `STDLIB_FILES` array order).
- Comments inside the moved files.

## Closure (slice 9f-9g step N)

When the moves are structurally complete:

1. Update `INVENTORY.md` § G "Dishonest layout" table — strike
   the edn + test rows; mark ✓ shipped slice 9f-9g. Update
   sandbox/hermetic rows to point at K.thread-process.
2. Update `J-PIPELINE.md` — slice 9f-9g done; add
   K.thread-process line item; remove from independent-sweeps
   backlog.
3. Update `SLICE-9F-9G.md` — flip from anchor to durable
   shipped-record.
4. Add 058 changelog row noting two-file move + scope-split note
   for K.thread-process inheriting the deeper work.

## Cross-references

- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § G "Filesystem
  path mirrors FQDN" — the doctrine this slice applies.
- `docs/arc/2026/04/109-kill-std/SLICE-9D.md` — stream did file
  move + symbol rename together; 9f-9g does file move only
  because symbols already align.
- `src/stdlib.rs` — where `STDLIB_FILES` registers each
  bundled file's path + `include_str!`.

## Estimated scope

- 2 `git mv` operations (already done)
- 2 `WatSource` entries updated in `src/stdlib.rs`
- <10 source-tree references to old paths

Total <15 minutes. Smallest substrate slice in arc 109's
catalog.
