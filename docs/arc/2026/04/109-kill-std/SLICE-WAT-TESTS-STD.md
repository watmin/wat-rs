# Arc 109 Slice — `wat-tests/std/` flatten

**Status: shipped 2026-05-01.** 4 `git mv` operations + 1
`rmdir` (empty form/) + header self-refs in 4 moved files +
README.md + 4 live docs (SERVICE-PROGRAMS, CONVENTIONS,
ZERO-MUTEX, plus crates/wat-telemetry-sqlite's Sqlite.wat
header). cargo test --release --workspace 1476/0.

This slice surfaced the **two-axis drift principle** that 9f-9g
clarified: filesystem renames cause runtime drift (symbol
lookup) AND documentation drift (path mentions in comments,
READMEs, design docs). The former requires no sweep when symbols
don't change; the latter ALWAYS does, however small.

## What this slice does

Mirror slice 9f-9g + 9d's filesystem cleanup on the test side.
The test tree's `wat-tests/std/` subdirectory parallels `wat/std/`
historically; with `wat/std/` emptying out, the test tree's
`std/` segment becomes equally dishonest. Per § G's
filesystem-path-mirrors-FQDN rule applied to tests:
`wat-tests/<ns>/X.wat` tests `:wat::<ns>::*` symbols; the path
should never carry an `std/` segment that the symbol path doesn't
have.

| File | Symbol-path tested | Target | Reason |
|---|---|---|---|
| `wat-tests/std/test.wat` | `:wat::test::*` | `wat-tests/test.wat` | matches new `wat/test.wat` |
| `wat-tests/std/stream.wat` | `:wat::stream::*` | `wat-tests/stream.wat` | matches new `wat/stream.wat` |
| `wat-tests/std/struct-to-form.wat` | `:wat::core::struct->form` | `wat-tests/core/struct-to-form.wat` | joins option-expect.wat + result-expect.wat under `wat-tests/core/` |
| `wat-tests/std/service-template.wat` | canonical service pattern (kernel + Console) | `wat-tests/service-template.wat` (top-level) | not tied to a single substrate file — it's a teaching artifact; lives at top-level alongside `wat-tests/time.wat` |
| `wat-tests/std/form/` (empty dir) | — | delete | `rmdir` — leftover from a removed test |
| `wat-tests/std/service/Console.wat` | `:wat::std::service::Console::*` | defer to **K.console** | rides with the Console flatten + symbol rename |

After this slice, `wat-tests/std/` contains only `service/Console.wat`,
which K.console moves with the symbol rename. After K.console,
`wat-tests/std/` is gone entirely.

## Why this is mostly trivial

- **No symbol changes** — these are tests; they reference
  substrate symbols. Symbols already migrated in prior slices
  (9f/9g for test.wat + edn.wat; 9d for stream).
- **No walker** — the substrate doesn't track test-file paths.
- **No consumer migration** — `wat-tests/` files are leaves of
  the dependency tree (the test harness reads them; nothing
  imports from them).

The work IS:
- 4 `git mv` operations
- delete 1 empty directory
- update file-header comments (each file references its own old path)
- update `wat-tests/README.md` layout-mapping section
- verify `cargo test --release --workspace` 1476/0

## What to ship

### Filesystem moves

```bash
git mv wat-tests/std/test.wat            wat-tests/test.wat
git mv wat-tests/std/stream.wat          wat-tests/stream.wat
git mv wat-tests/std/struct-to-form.wat  wat-tests/core/struct-to-form.wat
git mv wat-tests/std/service-template.wat wat-tests/service-template.wat
rmdir wat-tests/std/form
# wat-tests/std/service/Console.wat stays for K.console
```

### Header-comment fixes

Each moved file's first line is `;; wat-tests/<old-path>.wat — <description>`.
Update to the new path so the source identifies its own location
honestly.

### README update

`wat-tests/README.md` line 25-27 layout-mapping table — update
to reflect the new pairings.

### Verification

- `cargo build --release` clean
- `cargo test --release --workspace` 1476/0
- `ls wat-tests/std/` shows only `service/`

## Closure (slice step N)

When the moves are structurally complete:

1. Update `INVENTORY.md` § G — add a note that `wat-tests/std/`
   flattens alongside `wat/std/`; only `wat-tests/std/service/`
   remains until K.console.
2. Update `J-PIPELINE.md` — slice done; remove from
   independent-sweeps backlog.
3. Update `SLICE-WAT-TESTS-STD.md` — flip from anchor to durable
   shipped-record.
4. Add 058 changelog row noting the test-tree mirror cleanup.

## Cross-references

- `docs/arc/2026/04/109-kill-std/SLICE-9F-9G.md` — substrate-side
  precedent (file move only, no symbol changes).
- `docs/arc/2026/04/109-kill-std/SLICE-9D.md` — substrate-side
  precedent for stream's matching test file.
- `wat-tests/README.md` — layout convention this slice realigns.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § G — filesystem
  doctrine.

## Estimated scope

- 4 `git mv` operations
- 1 `rmdir`
- 4 file-header comment fixes
- 1 README layout-mapping update

Total <15 minutes. Smallest substrate slice in arc 109's
catalog after 9f-9g.
