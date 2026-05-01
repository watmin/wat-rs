# Arc 109 Slice K.console — Console grouping noun → namespace flatten + Pattern A canonicalization + file move

**Status: shipped 2026-05-01.** Substrate (commit `6a15b72`) +
consumer sweep + 2 historical-doc fixes. 16 files swept (2
substrate moves + 13 consumer rename + 2 src/ doc-comment
updates); 49/49 pure rename in consumer scope; cargo test
--release --workspace 1476/0.

Three coupled transformations validated atomically:

1. **§ K grouping-noun retirement** — same mechanism as
   K.telemetry; rehearsed.
2. **§ K Pattern A canonicalization** — Tx/Rx → ReqTx/ReqRx;
   ADD ReqChannel + AckChannel typealiases. Console now mirrors
   Telemetry's Pattern A canonical shape.
3. **File move** — `wat/std/service/Console.wat` →
   `wat/console.wat` (subsumes original arc 109 § 9e plan); plus
   `wat-tests/std/service/Console.wat` →
   `wat-tests/console.wat`. After K.console:
   `wat-tests/std/` is FULLY EMPTY (deleted); `wat/std/`
   contains only sandbox.wat + hermetic.wat (K.thread-process
   targets).

**Walker shape:** `validate_legacy_console_path` is K.telemetry's
walker plus a `canonical_console_leaf` helper that maps
`Tx → ReqTx, Rx → ReqRx`. Otherwise identical Pattern 3 shape.

**Originally drafted as a compaction-amnesia anchor mid-slice;
preserved here as the durable record.** Slice K.console is the
fifth Pattern 3 application after slices 1c/1d/1e/9d/K.telemetry.
First slice to bundle § K grouping-noun retirement + Pattern A
channel canonicalization + file move atomically — proves the K
mechanism scales to bigger surfaces with multiple coupled
transformations.

## What this slice does

Three coupled transformations on the Console substrate:

1. **§ K grouping-noun retirement.** `Console` is a label hung on
   top of a namespace (no struct, no value, no kind). Verbs and
   typealiases flatten to `:wat::console::*`. Same shape as
   K.telemetry; mechanism rehearsed.
2. **§ K Pattern A channel canonicalization.** Today's
   `Console::Tx` / `Console::Rx` (no prefix) is a Level 2 mumble
   per the gaze finding — implicit "Request" naming forces
   readers to compare against the prefixed `AckTx` / `AckRx` to
   know what the unprefixed pair carries. Rename to
   `ReqTx` / `ReqRx`; ADD missing `ReqChannel` and `AckChannel`
   typealiases mirroring Telemetry (the Pattern A reference).
3. **File move.** `wat/std/service/Console.wat` →
   `wat/console.wat`. Subsumes the original arc 109 § 9e plan;
   filesystem-path-mirrors-FQDN per § G.

This is the **biggest K slice** (per gaze: Console has more
consumers than Telemetry, and bundles a file move + channel
rename + grouping-noun flatten). The mechanism is rehearsed —
K.telemetry already proved Pattern 3 keyword-prefix detection
on Service. K.console adds the channel-rename axis on top.

## Substrate work scope

### Console-grouping retirement (8 typealiases + 5 verbs flatten)

```
:wat::std::service::Console::Message    → :wat::console::Message
:wat::std::service::Console::Tx         → :wat::console::ReqTx        ;; PATTERN A — rename
:wat::std::service::Console::Rx         → :wat::console::ReqRx        ;; PATTERN A — rename
:wat::std::service::Console::AckTx      → :wat::console::AckTx
:wat::std::service::Console::AckRx      → :wat::console::AckRx
:wat::std::service::Console::Handle     → :wat::console::Handle
:wat::std::service::Console::DriverPair → :wat::console::DriverPair
:wat::std::service::Console::Spawn      → :wat::console::Spawn

:wat::std::service::Console/ack-at      → :wat::console::ack-at
:wat::std::service::Console/err         → :wat::console::err
:wat::std::service::Console/loop        → :wat::console::loop
:wat::std::service::Console/out         → :wat::console::out
:wat::std::service::Console/spawn       → :wat::console::spawn
```

### Pattern A canonicalization — add missing channel typealiases

Mirror Telemetry's shape (the Pattern A reference). Console isn't
generic over a payload type (Message is fixed), so the typealiases
are non-generic:

```
:wat::console::ReqChannel = :(wat::console::ReqTx, wat::console::ReqRx)
:wat::console::AckChannel = :(wat::console::AckTx, wat::console::AckRx)
```

These are NEW typealiases (not flattened from existing names) —
they fill in the Pattern A reference shape that Console was
missing. Today's spawn-helper code constructs the tuples inline;
post-K.console, those construction sites can use the named
aliases.

### File move

```
wat/std/service/Console.wat → wat/console.wat
```

`src/stdlib.rs` `include_str!` path updates (one entry).

After K.console ships, `wat/std/` is empty (post-9d + 9f-9g; the
last file was Console.wat). `:wat::std::*` namespace also empties
(stream and Console were the last residents). § G's "wat/std/
empties out + namespace flattens" goal lands.

## Pattern 3 walker

**`CheckError::BareLegacyConsolePath`** — fires on any keyword
starting with `:wat::std::service::Console::` (typealias path)
or `:wat::std::service::Console/` (verb path). Replacement logic
handles both grouping-noun strip AND the Tx/Rx → ReqTx/ReqRx
rename:

```rust
fn canonical_console_replacement(legacy: &str) -> String {
    if let Some(tail) = legacy.strip_prefix(":wat::std::service::Console::") {
        let canonical_tail = match tail {
            "Tx" => "ReqTx",
            "Rx" => "ReqRx",
            other => other,
        };
        format!(":wat::console::{}", canonical_tail)
    } else if let Some(tail) = legacy.strip_prefix(":wat::std::service::Console/") {
        format!(":wat::console::{}", tail)
    } else {
        legacy.to_string()  // unreachable; walker checks prefix
    }
}
```

Same overall shape as K.telemetry's walker; the special-case for
Tx/Rx is the only addition. Single CheckError variant; single
walker.

## What to ship

### Substrate (Rust + wat-stdlib)

1. **Move file**: `git mv wat/std/service/Console.wat wat/console.wat`.
2. **Internal renames in wat/console.wat** — apply the
   canonical_console_replacement transformation via sed:
   - `:wat::std::service::Console::Tx` → `:wat::console::ReqTx`
   - `:wat::std::service::Console::Rx` → `:wat::console::ReqRx`
   - `:wat::std::service::Console::` → `:wat::console::` (catches
     remaining typealiases)
   - `:wat::std::service::Console/` → `:wat::console::` (verbs)
3. **Add `ReqChannel` and `AckChannel` typealiases** to
   wat/console.wat at the typealias section. Update spawn helper
   code if it benefits from the new aliases (judgment call;
   minimal change is fine).
4. **Update `src/stdlib.rs`**:
   - Path `wat/std/service/Console.wat` → `wat/console.wat`
   - `include_str!` argument matches
   - Module doc comments referencing the old path — update.
5. **Mint `CheckError::BareLegacyConsolePath`** in src/check.rs:
   variant + Display + Diagnostic + walker
   `validate_legacy_console_path` + canonical_console_replacement
   helper.
6. **Wire walker into `check_program`** alongside slice 9d's
   stream walker and K.telemetry's service walker.

### Verification

Probe coverage:
- `(:wat::std::service::Console/spawn ...)` → fires
- `(:wat::console::spawn ...)` → silent
- `:wat::std::service::Console::Tx` → fires; canonical is `:wat::console::ReqTx`
- `:wat::std::service::Console::AckTx` → fires; canonical is `:wat::console::AckTx`
- `:wat::console::ReqTx` → silent
- `:my::pkg::Console::*` (user paths) → silent

## Sweep order

Same four-tier discipline.

1. **Substrate stdlib** — `wat/console.wat` (just moved + flattened)
   plus any other `wat/` files that mention Console.
2. **Lib + early integration tests** — `src/freeze.rs`,
   `src/stdlib.rs` doc comments, embedded wat strings in
   `src/runtime.rs` lib tests.
3. **`wat-tests/`** + **`crates/*/wat-tests/`** —
   `wat-tests/std/service/Console.wat` (which itself moves; see
   below), `crates/wat-telemetry/wat-tests/telemetry/Console.wat`.
4. **`tests/`**, **`examples/`**, **`crates/*/wat/`** —
   `tests/wat_tco.rs`, `examples/console-demo/wat/main.wat`,
   `crates/wat-telemetry/wat/telemetry/Console.wat`,
   `crates/wat-telemetry/wat/telemetry/ConsoleLogger.wat`,
   `crates/wat-lru/wat-tests/lru/CacheService.wat`,
   `crates/wat-telemetry/src/lib.rs`.

### wat-tests file move

`wat-tests/std/service/Console.wat` is the matching test file.
Per the wat-tests README convention `wat/<ns>/X.wat ↔
wat-tests/<ns>/X.wat`, after K.console it lives at
`wat-tests/console.wat` (top-level, mirroring the new
`wat/console.wat`). The K.console slice handles this rename
alongside the substrate move.

After K.console: `wat-tests/std/` is fully empty (the prior
slice already moved the other 4 files; this slice closes
service/). The `wat-tests/std/` directory deletes.

## Estimated scope

- Substrate Console.wat self-references: ~35 sites
- Consumer files (production code): ~10 files
- Total occurrences across consumers: 141 (per pre-slice grep)
- New typealiases: 2 (`ReqChannel`, `AckChannel`)
- Combined: ~176 rename sites + 2 new typealiases + 1 file move
  + 1 wat-tests file move + walker

Comparable to K.telemetry (196/196). Sonnet-tractable.

## What does NOT change

- The 5 verbs' shapes / semantics — pure rename + namespace move.
- The ack-channel typealias bodies (still `QueueSender<unit>` /
  `QueueReceiver<unit>`).
- The Tx/Rx body shapes — `QueueSender<Message>` /
  `QueueReceiver<Message>`; just the alias names rename.
- Internal helper structure — every define keeps its body.
- Consumer call shapes mechanically transform; argument order
  and types unchanged.

## Closure (slice K.console step N)

When sweep is structurally complete:

1. Update `INVENTORY.md` § K — strike Console rows in both the
   grouping-noun cleanup table AND the channel-naming-patterns
   table; mark ✓ shipped K.console. Note that § G's
   "wat/std/ empties out" goal is now fully landed (only
   K.thread-process's wat/std/{sandbox,hermetic}.wat remain;
   those move under K.thread-process).
2. Update `J-PIPELINE.md` — slice K.console done; remove from
   independent-sweeps backlog.
3. Update `SLICE-K-CONSOLE.md` — flip from anchor to durable
   shipped-record.
4. Add 058 changelog row noting:
   - First slice to bundle § K grouping-noun + Pattern A channel
     canonicalization + file move atomically
   - Validates that the K mechanism scales to bigger surfaces
   - Final closure of the `:wat::std::*` namespace (everything
     else has either flattened already or is K.thread-process
     territory)

## Cross-references

- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § K — the
  "/ requires a real Type" doctrine + channel-naming-patterns
  subsection (Pattern A reference).
- `docs/arc/2026/04/109-kill-std/SLICE-K-TELEMETRY.md` — the
  precedent slice; same § K mechanism on a smaller surface.
- `docs/arc/2026/04/109-kill-std/SLICE-9D.md` — Pattern 3 walker
  precedent (keyword-prefix detection).
- `docs/SUBSTRATE-AS-TEACHER.md` — the migration mechanism.
- `wat/std/service/Console.wat` (pre-K.console) →
  `wat/console.wat` (post-K.console).
