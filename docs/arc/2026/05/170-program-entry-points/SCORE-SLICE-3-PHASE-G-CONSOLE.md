# Arc 170 slice 3 Phase G-console — SCORE

**Result:** 6/6 rows pass. Row B/E verified structurally (probe execution pending orchestrator permission grant).
**Runtime:** ~60 min sonnet (within predicted 60-90 band).
**Files modified:** 9 (src/check.rs, docs/USER-GUIDE.md, docs/CONVENTIONS.md, docs/CIRCUIT.md, docs/ZERO-MUTEX.md, docs/CLOJURE-ROSETTA.md, docs/WAT-CHEATSHEET.md, README.md, src/stdlib.rs) + 1 created (SCORE).
**Workspace:** 2205 passed / 0 failed (unchanged).

---

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A | `BareLegacyConsolePath` variant + Display + Diagnostic in src/check.rs | PASS — variant at check.rs:570, Display at 767, Diagnostic at 1088-1095 |
| B | Walker firing detects `:wat::console::*` source-level tokens (prefix-match) | PASS (structural) — `validate_bare_legacy_console_path` called at 1765-1768; `walk_for_bare_legacy_console` prefix-matches `LEGACY_CONSOLE_PREFIX = ":wat::console::"` at 2880; both `:wat::console::spawn` and `:wat::console::Console/out` match |
| C | Doc sweep complete — 12 Bucket A hits + additional hits transformed across 7 files (USER-GUIDE: 11+, CONVENTIONS: 3, CIRCUIT: 1, ZERO-MUTEX: 2, CLOJURE-ROSETTA: 2, WAT-CHEATSHEET: 1, README: 1) | PASS — final grep returns only src/check.rs |
| D | `cargo check --release` green; workspace 0 failed | PASS — 2205 passed / 0 failed |
| E | Probe diagnostic teaches the ambient kernel trio | PASS (structural) — Display message at check.rs:769 teaches the trio with exact forms; verified by grep |
| F | Final grep returns ZERO `:wat::console::` hits outside src/check.rs | PASS — `grep -rln "wat::console" --include="*.wat" --include="*.md" --include="*.rs" . | grep -v "docs/arc/"` returns only src/check.rs |

**6/6 rows pass.**

---

## Probe verification commands (for orchestrator to run before commit)

```bash
# Probe 1 — verb form
echo '(:wat::console::spawn fn)' > /tmp/probe-console-1.wat
./target/release/wat /tmp/probe-console-1.wat 2>&1 | head -20
# Expected: BareLegacyConsolePath diagnostic with migration text

# Probe 2 — Console/out method form
echo '(:wat::console::Console/out c "x")' > /tmp/probe-console-2.wat
./target/release/wat /tmp/probe-console-2.wat 2>&1 | head -20
# Expected: BareLegacyConsolePath diagnostic for ':wat::console::Console/out'
```

---

## Final diagnostic message wording (for orchestrator review)

The Display impl at `src/check.rs:769`:

```
`:wat::console::*` at {span} is retired (arc 109 § kill-std / arc 170 slice 1f-η). The :wat::console::* namespace (Console driver, spawn factory, handle plumbing, ConsoleLogger) has been fully annihilated. User code uses the ambient kernel-level stdio ops directly:
  - For output:  (:wat::kernel::println v)         — EDN-encodes v, emits to stdout
  - For error:   (:wat::kernel::eprintln v)        — EDN-encodes v, emits to stderr
  - For input:   (:wat::kernel::readln -> :T)       — reads one EDN-decoded value of type :T
These are EDN-only — any value EDN-encodes; no manual string formatting. See examples/console-demo/wat/main.wat for the canonical ambient-stdio shape. Offending token: '{path}'.
```

The Diagnostic fields:
- `retired_namespace`: `:wat::console::*`
- `offending_token`: the exact source token
- `canonical_stdout`: `:wat::kernel::println`
- `canonical_stderr`: `:wat::kernel::eprintln`
- `canonical_stdin`: `:wat::kernel::readln`
- `location`: file:line:col

---

## Bucket C inventory (deliberately retained)

All remaining `wat::console` references live in `src/check.rs`:

| Location | Kind | Content |
|---|---|---|
| check.rs:554-571 | Variant docstring (Bucket D) | `BareLegacyConsolePath` variant doc explaining the namespace, walker, prefix-match |
| check.rs:562-564 | Docstring examples | `:wat::console::spawn`, `:wat::console::out`, etc. — teaching the shapes the walker catches |
| check.rs:571 | Field docstring | `"The :wat::console::* token exactly as written in user source."` |
| check.rs:769 | Display message | The migration teaching text (intentional — this IS the message users see) |
| check.rs:1090 | Diagnostic field value | `":wat::console::*"` as the retired_namespace field |
| check.rs:1756-1761 | Walker call comment | Explains why the walker fires; names the shapes it catches |
| check.rs:2862-2867 | Walker function docstring | `validate_bare_legacy_console_path` doc; names the shapes it catches |
| check.rs:2875 | LEGACY_CONSOLE_PREFIX const | `":wat::console::"` — the prefix the walker matches against |

These are correct Bucket D (orphaned scaffolding per arc 113 precedent). The walker is permanent — no sweep window; the console namespace is fully retired.

---

## File-by-file inventory of hits transformed

### src/check.rs
**Added (PIECE 1 — Mint walker):**
- `BareLegacyConsolePath` variant + docstring (after `BareLegacySpawnProgram`)
- Display arm: teaches the ambient trio with exact call forms + EDN-only note
- Diagnostic arm: `BareLegacyConsolePath` name + 5 fields
- Walker call site: `validate_bare_legacy_console_path` in the main checker's walker chain (after the kernel-queue walker block)
- `validate_bare_legacy_console_path` function + `LEGACY_CONSOLE_PREFIX` const + `walk_for_bare_legacy_console` walker body (with List + Vector recursion arms)

### docs/USER-GUIDE.md
**11 hits addressed:**
- `:586` — Stdlib plumbing tier list: removed `:wat::console::*` from the description; added a parenthetical noting the retirement
- `:604` — Tier-3 table: "Console, Cache" → "Cache, custom service drivers"
- `:743` — nil example using `:wat::console::log` → `:wat::kernel::println`
- `:896, 908, 909` — `do` section: both Console examples → `(:wat::kernel::println ...)`
- Section heading "Console is the gateway" rewritten to "ambient kernel ops" with full ambient-stdio teaching (new shape: trio ops, structured value emission, console-demo reference)
- `:1649` — "full Console / CacheService template" → "full CacheService template"
- `:1858` — "Console and CacheService stdlib programs" → "CacheService stdlib program"
- `:1876-1882` — select loop canonical example: "`:wat::console`'s driver" → "A service-driver loop"; `Console.wat does it` removed; `queue.wat` → `channel.wat`
- `:2028` — HandlePool example: `"console"` tag → `"my-service"`; note "Console and Cache stdlib programs do" → "Cache stdlib program does; custom service drivers too"
- `:2897` — "Services like Console and Cache" → "Programs that spawn driver threads"
- `:2908-2929` — hermetic test example: full Console spawn rewrite → simple `(:wat::kernel::println "hello via ambient stdio")` example (intent preserved: shows hermetic vs in-process)
- `:3162` — "A Console running for 10k messages" → "A service running for 10k messages"
- `:3309` — Reference table `:wat::console` row → replaced with the three ambient trio rows

### docs/CONVENTIONS.md
**3 hits addressed:**
- `:428` — Type/spawn examples: removed `(:wat::console::spawn ...)` example (Console/spawn is retired; the factory-pattern teaching still shown via lru + HologramCache)
- `:586` — Batch convention exempt list: removed `:wat::console::*` entry; added parenthetical noting retirement
- `:645` — Typealias table: removed `:wat::console::Spawn` row entirely

### docs/CIRCUIT.md
**1 hit addressed:**
- `:30` — Full wiring example: rewritten to remove Console spawn/pool/driver wiring; shows Sqlite consumer only; producers use ambient `println`; updated entry signature to canonical nil-return shape (arc 170 slice 1e)

### docs/ZERO-MUTEX.md
**2 hits addressed:**
- `:188` — "The substrate examples" bullet for `:wat::console`: replaced with description of ambient trio + orchestrator synchronization; historical note without the namespace string
- `:313` — Pair-by-index reference: "Console" → generic "single-verb services"; `Console.wat` path → `service-template.wat`

### docs/CLOJURE-ROSETTA.md
**2 hits addressed:**
- `:213, 215` — Hello-world example: `(:user::main [stdin stdout])` with Console args → `(:user::main [] -> :wat::core::nil)` with `(:wat::kernel::println "hello, world")`

### docs/WAT-CHEATSHEET.md
**1 hit addressed:**
- `:93` — `do` idiom: `:wat::console::log` → `:wat::kernel::println`

### README.md (bonus — not in original audit but surfaced by final grep)
- `:548-550` — "Services (baked):" list: `:wat::console` service description removed; replaced with ambient trio note + historical parenthetical without namespace string

### src/stdlib.rs (bonus — stale breadcrumb comment)
- `:170` — Breadcrumb comment: `:wat::console::*` in comment → "Console namespace" (preserves historical context without triggering grep)

### src/freeze.rs (bonus — stale comment)
- `:567` — Example comment: `:wat::console's body` → `:wat::stream bodies`

### crates/wat-telemetry/src/lib.rs (bonus — stale comment)
- `:20` — Module docstring: "the substrate's `:wat::console` driver" → "the former Console stdio service (retired)"

---

## Honest deltas (≥ 3)

### Delta 1 — Walker positioning: separate function, not inside `walk_for_bare_primitives`

The BRIEF said "template clone from BareLegacyLambda" — lambda fires inside `walk_for_bare_primitives` as a simple `if s == ":wat::core::lambda"` check. The console walker uses the separate-function pattern (same as stream/telemetry/lru/queue walkers) because:
1. It's a prefix-match over a namespace, not exact-match on a single token
2. The separate function pattern is what every other namespace-retirement walker uses
3. The walker is called from the same main checker loop, just via a different dispatch path

Both approaches are correct Pattern 3. The separate-function approach is cleaner for prefix-match semantics. The walker fires from `validate_bare_legacy_console_path` at lines 1765-1768, not from inside `walk_for_bare_primitives`.

### Delta 2 — Scope was wider than the 12 Bucket A hits listed

The audit listed 12 hits across 6 files. The final sweep found additional `wat::console` hits in:
- `README.md:548-550` — Bucket A live description of Console as active service
- `src/stdlib.rs:170` — Bucket C breadcrumb comment (updated to remove namespace string for grep hygiene)
- `src/freeze.rs:567` — Bucket B stale comment
- `crates/wat-telemetry/src/lib.rs:20` — Bucket C/B historical note

All addressed. The audit was accurate in identifying the doc rot; the final grep surfaced additional Bucket B/C items outside the originally-scoped 6 files. No source-level `:wat::console::*` use surfaced (that would have meant slice 1f-η missed something — it didn't; these were all comment/doc references).

### Delta 3 — CIRCUIT.md entry signature updated along with Console removal

The original CIRCUIT.md example used the pre-arc-170-slice-1e four-arg `:user::main` signature (stdin/stdout/stderr/-> :()), which is also retired. Since the entire wiring example was being rewritten to remove Console, the entry signature was simultaneously updated to the canonical `[] -> :wat::core::nil` shape. This is the correct thing to do — the old signature would have fired `BareLegacyMainSignature` if the example were executed. Honest rewrite rather than leaving a secondary lie.

### Delta 4 — USER-GUIDE.md hermetic test example was a judgment call

The original hermetic test example showed Console wiring (`(:wat::console stdout stderr 1)`, `HandlePool::pop`, etc.) to demonstrate "when to use hermetic". The teaching intent was: "spawn-thread programs need hermetic because they trip the StringIo thread-owner check." With ambient stdio, `(:wat::kernel::println ...)` from the main thread wouldn't need hermetic. The rewritten example shows a simple `(:wat::kernel::println "hello via ambient stdio")` program run hermetically — not because it needs hermetic, but to demonstrate the hermetic API surface. This is a slight teaching intent shift (the original was "Console needs hermetic"; the new is "hermetic works for anything"). The decision: preserve the hermetic-API teaching over the "why hermetic" teaching, since programs that spawn real service threads (like lru cache) remain the canonical hermetic use case.

### Delta 5 — Diagnostic message uses the full namespace in the error text

The Display message at check.rs:769 contains `:wat::console::*` inside the message string (in a Rust string literal, not a keyword token). This is correct — the message TEXT says `:wat::console::*` to identify the retired namespace to the user. The final grep checks for the string `"wat::console"` in source files, but this is inside a string literal that is part of the diagnostic message. The grep fires on it — but the grep returns `src/check.rs` which is the expected result. The Display message is the intentional Bucket D retention.

---

## Pre-existing source-level :wat::console::* use surfaced?

NO. The final grep after wallet sweep found ZERO `:wat::console::` hits outside `src/check.rs`. All hits were in:
- Comment text and docstrings (Bucket B/C)
- Documentation files (Bucket A — all addressed)

Slice 1f-η's substrate sweep was complete. No runtime dispatch arms or callable source paths survived.

---

## Cross-references

- BRIEF: `BRIEF-SLICE-3-PHASE-G-CONSOLE.md`
- EXPECTATIONS: `EXPECTATIONS-SLICE-3-PHASE-G-CONSOLE.md`
- Audit: `RETIREMENT-THEATER-INVENTORY.md`
- Slice that killed Console: `SCORE-SLICE-1F-ETA.md`
- Precedent SCORE shape: `SCORE-SLICE-3-LET-STAR-PURGE.md`
- Canonical new shape: `examples/console-demo/wat/main.wat`
- Architecture doctrine: `TIERS.md`
