# Arc 109 Slice 9d — `:wat::std::stream::*` → `:wat::stream::*` (namespace promotion + file path move)

**Status: shipped 2026-05-01.** Substrate (commit `7837262`) +
consumer sweep (`d22bc4f`). 11 files swept (1 stdlib + 10
consumer); **286 rename sites total** (101 stream.wat self-refs
+ 185 consumer); zero substrate-gap fixes. cargo test --release
--workspace 1476/0.

The slice combined three substrate moves with one walker mint:

| What | Outcome |
|---|---|
| `wat/std/stream.wat` → `wat/stream.wat` | git mv (history preserved) |
| `:wat::std::stream::*` → `:wat::stream::*` (101 sites in stream.wat) | sed -i in-place |
| `src/stdlib.rs` `include_str!` path + module doc | path mirrors shipped FQDN |
| `CheckError::BareLegacyStreamPath` variant + `validate_legacy_stream_path` walker | Pattern 3 — third namespace-prefix application |

After the slice, the substrate has zero
`:wat::std::stream::*` references; consumers using the legacy
prefix get a self-describing diagnostic naming the canonical
`:wat::stream::*` replacement at every offending site. § G's
filesystem-path rule (file path mirrors shipped FQDN) is now
honored for stream.

**The walker shape:** simple keyword-prefix detection. Walks
every `WatAST::Keyword(s, span)` node and fires when `s.starts_
with(":wat::std::stream::")`. Catches all positions uniformly
(callable head, type annotation, value position) — no parsed-
TypeExpr inspection needed because this is a pure namespace
prefix retirement, not a type-shape change.

This is the simplest substrate work in arc 109's slice catalog
so far — no special-case dispatchers, no canonicalization map
extension, no AST-grammar exception. Pure filesystem move +
internal sed + walker. Future § K cleanups (K.console / K.
telemetry / K.lru / K.holon-lru) will follow this template's
shape extended with the `Type/method` → namespace-flatten rule.

**Substrate-as-teacher held:** the consumer sweep agent followed
the diagnostic stream cleanly. The agent abandoned its formal
report mid-task (background cargo test waiting); orchestrator
re-checked `git diff --stat` independently and validated the
work — the verify-after-agent protocol from slice 1j tier 1
caught it.

**Originally drafted as a compaction-amnesia anchor mid-slice;
preserved here as the durable record.** Slice 9d is the third
Pattern 3 application after slices 1c/1d/1e (the first three
walker-template applications targeted parsed-TypeExpr shapes;
9d targets keyword prefixes — same mechanism, different
detection level).

## What this slice does

The stream stdlib graduates from `:wat::std::stream::*` to
`:wat::stream::*`. Per INVENTORY § G's three-tier substrate
organization, `:wat::std::*` empties out — every substrate concern
earns its own top-level tier. Stream is "iterable / collection-shaped
HOFs over channels" — it earns its own tier name, same shape as
`:wat::list::*` will get under slice 9a.

**Symbol path:** `:wat::std::stream::*` → `:wat::stream::*` (all 14
HOFs, 4 typealiases, 14 internal helpers — 34 distinct names total).

**File path:** `wat/std/stream.wat` → `wat/stream.wat` (path mirrors
shipped FQDN per § G's filesystem-path rule).

**Substrate doc reference + `include_str!` site:** `src/stdlib.rs`
line 86-87 (`path: "wat/std/stream.wat"` / `include_str!("../wat/std/stream.wat")`)
plus the doc comment at line 8 mentioning `:wat::std::stream::*` as
an example.

## Why this is mechanical (Pattern 3)

Pure namespace prefix migration. No grammar shift, no shape break,
no API change, no value type change. Every `:wat::std::stream::X`
becomes `:wat::stream::X`. Substrate-as-teacher Pattern 3 (dedicated
CheckError variant + walker) catches OLD usage in consumer code
post-rename.

This is the THIRD application of Pattern 3 to namespace migrations
(after slices 1c/1d for primitive type names; slice 1e extended for
parametric heads). The walker template is mature.

## What to ship

### Substrate (Rust + wat-stdlib)

1. **Rename inside `wat/std/stream.wat`** — every
   `:wat::std::stream::X` literal becomes `:wat::stream::X` (101
   self-references in this file alone — define heads, internal
   helper calls, typealiases, doc comments).

2. **Move the file** — `git mv wat/std/stream.wat wat/stream.wat`
   so the path mirrors the shipped FQDN.

3. **Update `src/stdlib.rs`**:
   - Line 86: `path: "wat/stream.wat"` (was `"wat/std/stream.wat"`)
   - Line 87: `include_str!("../wat/stream.wat")` (was `"../wat/std/stream.wat"`)
   - Line 8 doc comment: update `:wat::std::stream::*` example to
     `:wat::stream::*`.

4. **Mint `CheckError::BareLegacyStreamPath { old, new, span }`** in
   `src/check.rs`:
   - `old`: the OLD `:wat::std::stream::X` literal the user wrote
   - `new`: the canonical `:wat::stream::X` form
   - `span`: source position
   - `Display` IS the migration brief, naming arc 109 slice 9d and
     INVENTORY § G's tier-naming doctrine.

5. **Add walker `validate_legacy_stream_path`** that walks the
   program AST recognizing:
   - **Callable heads** — `WatAST::Keyword(k, _)` at list-head
     position when `k` starts with `":wat::std::stream::"`.
   - **Type annotations** — `TypeExpr::Path(":wat::std::stream::X")`
     and `TypeExpr::Parametric { head: "wat::std::stream::X", ... }`
     (the four typealiases plus any user references).
   - **Keyword values** — bare `:wat::std::stream::X` keyword nodes
     used as values (e.g., passed to a HOF or stored in a Vec).
   - Walk Lists / Tuples / Parametric children recursively.

   Emit one `BareLegacyStreamPath` per occurrence with the suggested
   canonical form.

6. **Wire walker into `check_program`** alongside the existing slice
   1c/1d/1e walkers + scope-deadlock walker.

### Verification

Probe coverage:
- `(:wat::std::stream::map ...)` → fires (retired)
- `(:wat::stream::map ...)` → silent (canonical)
- `:wat::std::stream::Stream<i64>` (type annotation) → fires
- `:wat::stream::Stream<i64>` → silent
- User namespace `:my::pkg::stream::*` → silent (different prefix)

## Sweep order

Same four-tier discipline as slices 1c-1j.

1. **Substrate stdlib** — `wat/stream.wat` (the renamed file)
   plus any other `wat/` files that mention the old prefix
   (e.g., `wat/kernel/queue.wat` if it references stream in
   doc comments).
2. **Lib + early integration tests** — `src/check.rs` walker
   doc comment, `src/stdlib.rs` line 8 example, `src/runtime.rs`
   embedded test wat strings if any reference stream paths.
3. **`wat-tests/`** + **`crates/*/wat-tests/`** —
   `wat-tests/std/stream.wat`,
   `crates/wat-telemetry-sqlite/wat-tests/telemetry/reader.wat`.
4. **`tests/`**, **`examples/`**, **`crates/*/wat/`** —
   `tests/wat_stream.rs`, `tests/wat_names_are_values.rs`,
   `examples/interrogate/wat/main.wat`,
   `crates/wat-telemetry-sqlite/wat/telemetry/Reader.wat`,
   `crates/wat-telemetry-sqlite/src/cursor.rs`.

Verification gate after each tier: `cargo build --release` clean +
`grep -rln ':wat::std::stream::' <swept-tier>` returns empty.
Final gate: `cargo test --release --workspace` 1476/0.

### Tier order rationale

The substrate stdlib HAS to flip first — `wat/stream.wat` is the
shipped binary's stream stdlib. If consumers rename to
`:wat::stream::map` while the stdlib still defines
`:wat::std::stream::map`, every consumer call fires UnknownFunction.
Substrate flip + walker shipped together; THEN consumer sweep
follows the diagnostic stream.

## Estimated scope

- `wat/std/stream.wat` self-references: **101 sites**
- Consumer files: **10 files**, **186 sites total**
- Total: **~287 sites across 11 files**

Bigger than slice 1g (~86 sites) but much smaller than slice 1f
(772) or 1h (542). Sweep should run ~30-60 minutes via sonnet.

## What does NOT change

- The 14 stream HOFs' shapes, signatures, or semantics — pure
  rename.
- The 4 typealias bodies — still `Receiver<...>` /
  `(:fn(:Sender<T>) -> :())` etc.; just the typealias name's
  prefix rebinds.
- Internal helper functions (`map-worker`, `chunks-step`, etc.) —
  same names, just new prefix.
- `:wat::kernel::Sender<T>` / `Receiver<T>` types stream wraps
  around — unaffected.

## Closure (slice 9d step N)

When sweep is structurally complete:

1. Update `INVENTORY.md` § G — strike the `:wat::std::stream::*`
   row from "What :wat::std::* becomes" + the file-move row from
   the dishonest-layout table; mark ✓ shipped slice 9d.
2. Update `J-PIPELINE.md` — slice 9d done; remove from
   independent-sweeps backlog.
3. Update `SLICE-9D.md` — flip from anchor to durable
   shipped-record.
4. Add 058 changelog row noting the namespace promotion + file
   move + walker pattern (third Pattern 3 namespace-prefix
   application).

## Cross-references

- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § G — three-tier
  substrate organization; § H — list-tier promotion (parallel
  shape).
- `docs/arc/2026/04/109-kill-std/SLICE-1C.md`, `SLICE-1D.md`,
  `SLICE-1E.md` — Pattern 3 walker template precedent.
- `docs/SUBSTRATE-AS-TEACHER.md` — Pattern 3 mechanism.
- `src/stdlib.rs` line 86-87 — substrate's stream registration.
- `wat/std/stream.wat` (pre-9d) → `wat/stream.wat` (post-9d) —
  the moved file.
