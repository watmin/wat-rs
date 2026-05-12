# Arc 170 slice 3 Phase G-console BRIEF — mint walker + sweep doc rot

**Sonnet.** First slice of the retirement-theater purge after let* (`daa973d`). Closes the most acute lie from the audit: `:wat::console::*` was fully annihilated from substrate in slice 1f-η but has NO walker and 12 live-looking doc references that cliff users into cold `UnknownFunction`.

User direction 2026-05-11:
> *"we're pausing 170 forward work to address the backwards claims.... we fix console now"*

See `RETIREMENT-THEATER-INVENTORY.md` for full audit context. This BRIEF scopes to ONLY the console retirement.

## Backstory — what slice 1f-η shipped + the gap

**Slice 1f-η (commit refs in SCORE-SLICE-1F-ETA.md)** retired `:wat::console::*` substrate:
- `wat/console.wat` removed from stdlib
- All `:wat::console::*` dispatch arms removed from src/runtime.rs
- Type registrations removed from src/check.rs
- `BareLegacyConsolePath` variant + validator + 4 call sites all REMOVED from src/check.rs
- 5 Console deftest files deleted; workspace failure floor dropped 10

**The replacement (ambient, per arc 170 TIERS.md):**
- `:wat::kernel::println` — `∀T. T -> :wat::core::nil` — EDN-encode any value, emit to fd 1
- `:wat::kernel::eprintln` — `∀T. T -> :wat::core::nil` — same, to fd 2
- `:wat::kernel::readln` — `() -> :T` — polymorphic read; recipient-typed decode

Registered: src/check.rs:12877-12900. Implemented: src/thread_io.rs.

**The gap:** `BareLegacyConsolePath` variant was REMOVED in slice 1f-η. So:
- User writes `(:wat::console::spawn ...)` today
- Walker doesn't catch it (no variant)
- Type check doesn't recognize it (no scheme)
- Falls through to `UnknownFunction` — generic "call head not found" with no migration hint

Compare to let*/lambda — both fire friendly "retired; use X" diagnostics. Console has NO guardrail.

## Goal — two pieces

### Piece 1 — Mint `BareLegacyConsolePath` walker (Bucket A substrate)

Pattern: clone the `BareLegacyLambda` / `BareLegacyLetStar` template.

1. Add variant in `src/check.rs`:
   ```rust
   BareLegacyConsolePath {
       /// The `:wat::console::*` token (e.g., ":wat::console::spawn").
       path: String,
       span: Span,
   },
   ```

2. Add Display impl that teaches migration:
   ```
   "':wat::console::<verb>' at {span} is retired (arc 170 slice 1f-η). The :wat::console::* namespace has been retired; user code uses the ambient kernel-level stdio ops directly:
     - For output: (:wat::kernel::println v) emits EDN to stdout
     - For error:  (:wat::kernel::eprintln v) emits EDN to stderr
     - For input:  (:wat::kernel::readln -> :T) reads one EDN-decoded value of type :T
   These are EDN-only — any value EDN-encodes; no manual string formatting. See docs/USER-GUIDE.md § Stdio."
   ```

3. Add Diagnostic field emission (matches `BareLegacyLambda` shape).

4. Add walker firing in the existing per-token walker (near `check.rs:2376-2391` where let*/lambda fire). Detection: token starts with `:wat::console::`.

5. Verify probe fires:
   ```bash
   echo '(:wat::console::spawn fn)' > /tmp/probe-console.wat
   ./target/release/wat /tmp/probe-console.wat 2>&1 | head -5
   # Expected: BareLegacyConsolePath diagnostic with migration teaching
   ```

### Piece 2 — Sweep ~20 doc hits (Bucket A docs)

The 12 hits from the audit (Bucket A — these are code-shape changes, not 1:1 text replacement):

**`docs/USER-GUIDE.md` — 11 hits:**
- `:586` — tier 1 namespaces list: drop `:wat::console::*`; the trio is now in `:wat::kernel::*`
- `:743` — review context
- `:896, 908, 909` — Console-as-gateway section; needs full rewrite to teach the ambient trio
- `:1876` — likely a Console reference
- `:2359, 2365, 2470` — same; rewrite each example
- `:3021, 3027` — same
- `:3423` — reference table entry for `:wat::console::*`; replace with the kernel trio

**`docs/CONVENTIONS.md` — 3 hits:**
- `:428, 586, 645` — references to `:wat::console::spawn`, exempt list, type table

**`docs/CIRCUIT.md` — 1 hit:**
- `:30` — code example with `:wat::console::spawn`

**`docs/ZERO-MUTEX.md` — 2 hits:**
- `:188, 313` — references to `:wat::console` as active gateway

**`docs/CLOJURE-ROSETTA.md` — 2 hits:**
- `:213, 215` — code example with `:wat::console::Console` + `println!`

**`docs/WAT-CHEATSHEET.md` — 1 hit:**
- `:93` — code example with `:wat::console::log`

**For each hit:** the OLD shape was `(:wat::console::Console/out console "string")` or similar — service/struct with methods. The NEW shape is `(:wat::kernel::println value)` — ambient verb taking EDN value. **This is not 1:1 text replacement.** Each example needs rewrite that preserves the example's pedagogical intent.

Sonnet's judgment per hit:
- Is the example demonstrating "how to print"? → rewrite to `(:wat::kernel::println v)`
- Is it demonstrating "Console as a service shape"? → may need to remove the example or repurpose to a different service pattern
- Is it a tier 1/2 namespace list? → remove `:wat::console::*`; add note about ambient kernel trio

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/RETIREMENT-THEATER-INVENTORY.md`** — the full audit context
2. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-1F-ETA.md`** — what slice 1f-η actually shipped + the architectural observation about EDN-only ambient stdio
3. **`docs/SUBSTRATE-AS-TEACHER.md`** — Pattern 3 (symbol migration) — the recipe being applied here
4. **`docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 14** — the discipline this purge enforces
5. **`src/check.rs:2376-2391`** — the active walker firing for let*/lambda; pattern to mirror for console
6. **`src/check.rs:261-303`** (`BareLegacyLetStar` + `BareLegacyLambda` variant definitions) — template
7. **`src/check.rs:655-668, 949-965`** — Display + Diagnostic field for the prior retirements
8. **`src/check.rs:12877-12900`** — registration of the ambient kernel trio (println/eprintln/readln) — the migration target
9. **`examples/console-demo/wat/main.wat`** — working example of the new surface
10. **`docs/arc/2026/05/170-program-entry-points/TIERS.md`** — the architecture doctrine

## Implementation path

### Phase 1 — Mint walker (15-20 min)

1. Add `BareLegacyConsolePath` variant in `src/check.rs` matching the BareLegacyLambda shape
2. Add Display impl with the migration teaching above
3. Add Diagnostic field emission
4. Add walker firing detection (token starts with `:wat::console::`)
5. Probe: `(:wat::console::spawn fn)` should fire the diagnostic
6. Probe: `(:wat::console::Console/out c "x")` should also fire (any `:wat::console::*`)
7. Run `cargo check --release` to ensure no breakage

### Phase 2 — Documentation sweep (45-70 min)

Per file, per hit, judgment-driven rewrite. Don't mechanically transform; preserve teaching intent.

For tier-list entries (`docs/USER-GUIDE.md:586`, similar): simply drop `:wat::console::*` from the list; the ambient kernel trio is already documented elsewhere.

For code examples: rewrite the example to use `(:wat::kernel::println v)` / `(:wat::kernel::eprintln v)` / `(:wat::kernel::readln -> :T)`. Use `examples/console-demo/wat/main.wat` as the reference for shape.

For service-pattern Console docs: remove (Console-as-service was killed; the architecture pivoted to ambient kernel stdio per TIERS.md).

For typealias tables (`docs/CONVENTIONS.md`): remove Console entries; the trio doesn't have user-facing typealiases (they're direct ops).

### Phase 3 — Verify

```bash
# 1. Workspace stays green
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2205 passed / 0 failed (or matched baseline)

# 2. Probe: console fires walker
echo '(:wat::console::spawn fn)' > /tmp/probe-console.wat
./target/release/wat /tmp/probe-console.wat 2>&1 | head -10
# Expected: BareLegacyConsolePath with migration teaching

# 3. Final grep — :wat::console:: hits remain ONLY in historical context
grep -rln "wat::console" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/"
# Expected: only src/check.rs (variant + walker + Display references the legacy name to teach migration); zero hits in user-facing docs
```

## Scope (what's IN)

- `BareLegacyConsolePath` variant + Display + Diagnostic + walker firing
- ~20 doc hits across 6 files transformed
- Probe fires correctly on `:wat::console::*` tokens
- Workspace stays at 0 failed

## Scope (what's OUT)

- Other retirement-theater items from the inventory (`wat/std/` paths, lambda docstrings, stream namespace, fork-program walker notes) — separate Phase G-* slices
- `eval_kernel_wait_child` cleanup — folds into Slice 4 (queued)
- Phase E V3, Phase F — paused per user direction
- Anything labeled INSCRIPTION-class — this is a slice, not arc closure
- Renaming `BareLegacyConsolePath` to any specific verb-named variant — generic path-prefix walker per the audit (catches all `:wat::console::*`)

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `BareLegacyConsolePath` variant + Display + Diagnostic in src/check.rs | grep + read |
| B | Walker firing detects `:wat::console::*` source-level tokens | probe fires |
| C | Doc sweep complete — 12 Bucket A hits transformed across 6 files | grep |
| D | `cargo check --release` green; workspace 0 failed | full test |
| E | Probe: `(:wat::console::spawn fn)` fires BareLegacyConsolePath with migration text | manual probe |
| F | Final grep returns ZERO `:wat::console::` hits outside Bucket C (variant + walker + Display teaching the legacy name) | grep |

**6 rows.** All must PASS.

## Predicted runtime

**60-90 min sonnet.** Walker mint is 15-20 min (mechanical template clone); doc sweep is 45-70 min (judgment per hit, not mechanical).

**Hard cap:** 180 min.

## Constraints (hard)

- DO NOT touch anything under `docs/arc/` (FM 11 immutable inscriptions)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT use --no-verify or skip hooks
- DO NOT add new substrate features beyond the walker
- DO NOT rewrite TIERS.md or arc-170 DESIGN docs (those describe the locked architecture)
- Walker firing detection: match prefix `:wat::console::`, not whole-string equality — must catch every verb in the namespace
- Workspace must stay at 0 failed

## Honest delta categories (anticipated)

1. **Walker positioning** — where in `check.rs` walker chain BareLegacyConsolePath fits relative to the other BareLegacy* arms; surface the choice
2. **Doc rewrite judgment calls** — examples where the OLD shape's teaching intent doesn't trivially map to the NEW shape; surface specific cases for review
3. **Tier-list cleanup vs. add-trio** — for namespace tier lists that mentioned `:wat::console::*`, do we just remove or do we add a pointer to ambient kernel I/O? Surface the choice
4. **Diagnostic-message wording** — the migration text needs to teach the trio AND explain "EDN-only"; surface the final wording for review
5. **Workspace impact** — should be zero; surface anything unexpected

## Cross-references

- `RETIREMENT-THEATER-INVENTORY.md` — the audit
- `SCORE-SLICE-1F-ETA.md` — the slice that killed Console substrate
- `SCORE-SLICE-3-LET-STAR-PURGE.md` (commit `daa973d`) — the precedent purge SCORE shape
- `BRIEF-SLICE-3-LET-STAR-PURGE.md` — the precedent BRIEF shape
- `TIERS.md` — the architecture doctrine the retirement aligns with
- `examples/console-demo/wat/main.wat` — working example of new surface
