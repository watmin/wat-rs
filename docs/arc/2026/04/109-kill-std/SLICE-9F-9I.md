# Arc 109 Slice 9f-9i — File-path moves for already-honest-symbol files

**Compaction-amnesia anchor.** Read this first if you're picking
up these four file moves mid-flight.

## What this slice does

Four pure file moves. The symbols inside each file are ALREADY
honest — they ship FQDN paths under `:wat::edn::*`,
`:wat::test::*`, `:wat::kernel::*`. The dishonesty is purely at
the filesystem layer: the files live under `wat/std/` even
though their shipped namespaces have nothing to do with `:wat::std::*`.
Slice 9f-9i moves each file to its honest location per § G's
filesystem-path-mirrors-FQDN rule.

| Slice | From | To | Shipped FQDN (unchanged) |
|---|---|---|---|
| 9f | `wat/std/edn.wat` | `wat/edn.wat` | `:wat::edn::*` |
| 9g | `wat/std/test.wat` | `wat/test.wat` | `:wat::test::*` |
| 9h | `wat/std/sandbox.wat` | `wat/kernel/sandbox.wat` | `:wat::kernel::run-sandboxed*` |
| 9i | `wat/std/hermetic.wat` | `wat/kernel/hermetic.wat` | `:wat::kernel::run-sandboxed-hermetic*` |

Bundled as one slice because they share an identical shape (file
move + `src/stdlib.rs` `include_str!` path update + one consistency
sweep over comments / docstrings / paths in tests).

## Why this is trivial

- **No symbol changes** — every shipped path stays exactly as is.
  No walker needed; no `CheckError` variant; no consumer migration
  via Pattern 2 / Pattern 3 mechanism.
- **No type-checker work** — wat code referring to
  `:wat::edn::read` keeps working with zero churn.
- **No test sweep** — programs use the FQDN names; nothing
  references the file path at runtime.
- The ONLY thing that changes: the substrate's bundled stdlib
  registration in `src/stdlib.rs` (the `path:` field + the
  `include_str!` argument), plus any source-tree references to
  the old filesystem path in comments / READMEs / module docs.

## What to ship

### Substrate (Rust + filesystem)

1. **Four `git mv` operations**:
   ```bash
   git mv wat/std/edn.wat       wat/edn.wat
   git mv wat/std/test.wat      wat/test.wat
   git mv wat/std/sandbox.wat   wat/kernel/sandbox.wat
   git mv wat/std/hermetic.wat  wat/kernel/hermetic.wat
   ```

2. **Update `src/stdlib.rs`** — four `WatSource` entries change.
   For each: `path:` field updated to the new location AND the
   `include_str!` macro argument updated correspondingly. The
   substrate's bundled stdlib registration is what binds the
   wat-source to the runtime.

3. **Sweep filesystem-path references** elsewhere in the source
   tree:
   - Doc comments in `src/stdlib.rs` mentioning `wat/std/edn.wat`
     etc.
   - Module-level doc strings if any reference the old paths
   - Possibly READMEs / arc INSCRIPTIONs if they cite specific
     bundled file paths.

   Run `grep -rln 'wat/std/edn\|wat/std/test\|wat/std/sandbox\|wat/std/hermetic'`
   to find every reference.

4. **Verify**:
   - `cargo build --release` clean
   - `cargo test --release --workspace` 1476/0
   - `ls wat/std/` shows zero `.wat` files (only the `service/`
     subdir remains, which K.console will move).

### What does NOT change

- Every `:wat::edn::*`, `:wat::test::*`, `:wat::kernel::*` shipped
  symbol path.
- Any user wat code calling `:wat::edn::read`, `:wat::test::deftest`,
  etc.
- The runtime registration order (preserved by keeping the
  `STDLIB_FILES` array order).
- Comments inside the moved files (the files themselves don't
  reference their own filesystem path).

## Closure (slice 9f-9i step N)

When the moves are structurally complete:

1. Update `INVENTORY.md` § G "Dishonest layout" table — strike
   four rows; mark ✓ shipped slice 9f-9i.
2. Update `J-PIPELINE.md` — slice 9f-9i done; remove from
   independent-sweeps backlog.
3. Update `SLICE-9F-9I.md` — flip from anchor to durable
   shipped-record.
4. Add 058 changelog row noting four-file move + filesystem-
   doctrine application.

After 9f-9i closes, `wat/std/` contains only `service/Console.wat`;
slice K.console will move that one too, fully emptying `wat/std/`.

## Cross-references

- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § G "Filesystem
  path mirrors FQDN" — the doctrine this slice applies.
- `docs/arc/2026/04/109-kill-std/SLICE-9D.md` — the precedent
  slice for filesystem moves (stream did file move + symbol
  rename together; 9f-9i does file move only because symbols
  are already honest).
- `src/stdlib.rs` — where `STDLIB_FILES` registers each
  bundled file's path + `include_str!`.

## Estimated scope

- 4 `git mv` operations
- 4 `WatSource` entries updated in `src/stdlib.rs`
- Probably <10 source-tree references to the old paths (mostly
  doc comments)

Total <30 minutes. Smallest substrate slice in arc 109's
catalog. No agent delegation needed — orchestrator does it
directly.
