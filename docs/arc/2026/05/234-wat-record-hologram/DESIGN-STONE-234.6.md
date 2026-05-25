# DESIGN — Stone 234.6 — `:wat::holon::defrecord` migration + HARD CUT retirement

**Status:** ACTIVE (2026-05-25). Arc 234 RESUMES per spawn-block winding (arc 236 CLOSED at `1e24907f`; Stone 234.4.match SHIPPED at `bf329ebe`).

**Scope:** Migrate all `:wat::holon::defrecord` callers in wat-rs to `:wat::Record::def` (the Stone 234.2b replacement). HARD CUT retirement: delete `wat/holon/defrecord.wat` macro source; remove registry entries in `src/stdlib.rs`; update `wat/Record.wat:76` D12 comment (no co-existence claim). After this stone ships, `:wat::holon::defrecord` is STRUCTURALLY UNREPRESENTABLE in source (per `feedback_inscription_immutable` + failure-engineering ✅✅✅ HARD CUT discipline).

Brings arc 234 to ONE stone (Stone 234.7 INSCRIPTION) from closure.

---

## Origin

Stone 234.2b shipped `:wat::Record::def` macro at `wat/Record.wat`. Stone 234.5 shipped `:wat::holon::*` auto-dispatch on `Value::wat__Record` making holon-form access semantically reachable from new-substrate records. Together they make `:wat::holon::defrecord` redundant: callers can use the new macro AND the auto-dispatch surface provides equivalent holon-form behavior.

Per `feedback_no_known_defect_left_unfixed` + `feedback_inscription_immutable`: arc 234's thesis is "the wat-record hologram LANDS." If the legacy `:wat::holon::defrecord` surface remains in the codebase as a parallel option, the new substrate has not actually landed — it's added a competing surface to the old one. The migration IS the landing. Per user direction this session (correcting the "scope-different separate arc 238" framing): the migration stays inside arc 234 because the arc's thesis requires it.

---

## Audit (substrate truth as of 2026-05-25 pre-stone)

| File | References | Type |
|---|---|---|
| `tests/probe_arc227_stone2_defrecord.rs` | 56 | arc 227 v3 macro probe; tests OLD macro extensively |
| `wat/holon/defrecord.wat` | 6 | the OLD macro source itself (definition + docstring examples) |
| `tests/probe_diagnostic_typed_entities_reflection.rs` | 4 | arc 232.0a probe |
| `tests/probe_diagnostic_defprotocol_dispatch.rs` | 4 | arc 232 work (likely future-arc probe) |
| `src/stdlib.rs` | 2 | registry registration sites |
| `wat/Record.wat` | 1 | D12 comment ("co-exists with `:wat::holon::defrecord`") |
| `tests/probe_arc234_stone2b_defrecord_macro.rs` | 1 | Stone 234.2b probe; predecessor of the new macro |
| `tests/probe_diagnostic_polymorphic_type.rs` | 1 | arc 234.0 probe |

**Total: ~75 references across 8 files.** ~70% in one probe file (arc 227 v3 probe).

---

## Locked decisions

### D1 — Migration shape: mechanical find-replace `:wat::holon::defrecord` → `:wat::Record::def`

Same call shape across both macros (per Stone 234.2b's design); semantic equivalence via Stone 234.5's auto-dispatch on `Value::wat__Record`. Mechanical find-replace works because:

- Both macros take same args: `(:wat::*::def-form :ns::Type [field <- :type] ...)`
- New substrate makes `:wat::holon::*` verbs (`to-holon`, `from-holon`, etc.) auto-dispatch on `Value::wat__Record`, so callers using holon-form access continue to work without source change beyond the macro head

### D2 — HARD CUT: delete OLD macro source + registry entries

After migration:
- DELETE `wat/holon/defrecord.wat` entirely (not deprecated; deleted)
- REMOVE `:wat::holon::defrecord` registry entries in `src/stdlib.rs` (2 sites per audit)
- UPDATE `wat/Record.wat:76` D12 comment — strike "co-exists with `:wat::holon::defrecord`"; replace with affirmative naming (`:wat::Record::def` is THE record-defining macro)

No transitional alias. No deprecation warning. No `defrecord-deprecated` form. The legacy surface becomes UNREPRESENTABLE in source. Future-orchestrator opening a source file CANNOT reach for `:wat::holon::defrecord` because it doesn't exist.

### D3 — Workspace boundary: wat-rs only

Per `feedback_workspace_boundaries`: lab repos (`holon-lab-trading`, `holon-lab-baseline`, `holon-lab-ddos`) are SEPARATE repos. Lab callers (if any) are lab-repo work, not arc 234.6 work. Arc 234.6 ships the substrate retirement; lab callers (if surfaced as broken by lab tests) are independent investigation in those repos.

### D4 — Probe test bodies: behavior preservation verified

Stone 234.2b's new macro + Stone 234.5's auto-dispatch should make every existing probe test pass against the NEW macro. The probe test bodies assert behavior; behavior is preserved by the new substrate. Sonnet runs all probes after migration; ANY behavior change requires investigation INSIDE this stone (test-body adjustment if the change traces to substrate-equivalent behavior; STOP if it traces to a substrate gap).

### D5 — Order of operations (substrate-as-teacher cascade safety)

1. **First**: bulk find-replace `:wat::holon::defrecord` → `:wat::Record::def` across 7 caller files (everything except `wat/holon/defrecord.wat` which gets deleted)
2. **Second**: run cargo test; verify probes pass (substrate still has both macros registered; new callers use new macro; old macro still registered but no callers)
3. **Third**: update `wat/Record.wat:76` D12 comment (affirmative; no co-existence claim)
4. **Fourth**: delete `wat/holon/defrecord.wat`
5. **Fifth**: remove registry entries in `src/stdlib.rs`
6. **Sixth**: run cargo test + cargo build; verify NO source references the deleted file or removed registry entry

This order keeps the substrate functional at each step; cascade-as-teacher catches any missed callers immediately at step 6 if the registry removal leaves dangling references.

### D6 — Three+ file constraint (substrate ship)

Touch:
- 7 caller files (find-replace)
- `wat/Record.wat` (D12 comment update; 1-line)
- `wat/holon/defrecord.wat` (DELETED)
- `src/stdlib.rs` (registry retirement)

Plus possibly:
- Loader / file-list registry IF the substrate has a hard-coded list of `wat/holon/*.wat` files to load (sonnet checks; may need update)

DO NOT touch:
- Any other Rust source file (only `src/stdlib.rs` for registry)
- Any arc 234 historical artifacts (DESIGNs/BRIEFs/SCOREs for shipped stones)
- Any arc 236 artifacts
- holon-rs (STOP-4)
- Lab repos (D3 — workspace boundary)

### D7 — clippy ≤ 54 (current baseline 52)

No new warnings. The migration is deletion + find-replace; should not introduce lints. May DROP some warnings if dead-code paths are eliminated (`:wat::holon::defrecord` registry entries + macro source).

### D8 — Lib baseline preservation

Expected: 827/0 (unchanged). All arc 234/236/232 regression probes GREEN. No behavior change; only surface rename + macro source deletion.

### D9 — No probe file deletions (per `feedback_test_rot_audit` discipline)

The arc 227 probe (`probe_arc227_stone2_defrecord.rs`) tests behaviors that still matter — the probe TESTS the macro produces correct behavior; after migration, the test asserts the NEW macro produces correct behavior. Same test contract; different macro under test. KEEP the probe file. The 56 references inside it get find-replaced; the test assertions stay.

### D10 — Verification sequence (load-bearing post-migration)

After ALL six order-of-operations steps complete:
1. `cargo build --release -p wat` — 0 errors (no dangling references to deleted file / removed registry)
2. `cargo test --release --test probe_arc227_stone2_defrecord` — 28/28 PASS (or current count; behavior preserved against new macro)
3. `cargo test --release --lib -p wat --no-fail-fast` — 827/0
4. Defensive grep: `grep -rn ":wat::holon::defrecord" src/ wat/ wat-tests/ tests/ crates/ examples/` returns 0 results
5. `cargo clippy --release --lib -p wat -- -D warnings | grep -c "warning"` — ≤ 54

---

## Trap-door audit

### T1 — arc 227 probe behavior preservation

The arc 227 probe (`probe_arc227_stone2_defrecord.rs`) was authored to test the OLD macro's behavior in detail. After find-replace, the probe tests the NEW macro. Stone 234.5's auto-dispatch should make holon-form access work identically — but the probe MAY have test assertions that depend on subtle behavior differences (e.g., the OLD macro produced HolonAST directly; the NEW macro produces Value::wat__Record with auto-dispatch reaching holon-form). Sonnet runs the probe after find-replace; if it passes → mechanical migration done; if it fails → investigate INSIDE this stone (test-body adjustment IF the difference is substrate-equivalent + the assertion is incidental; STOP if substrate behavior actually differs).

### T2 — Registry removal cascade safety

`src/stdlib.rs` has 2 registry entries for `:wat::holon::defrecord` (per audit). Removing them makes the symbol unresolvable. Order-of-operations D5 puts registry removal LAST so all callers are migrated first. If any caller is missed, cargo build at step 6 surfaces the failure (`":wat::holon::defrecord" not found in registry`) — substrate-as-teacher cascade catches it.

### T3 — File-list loader (if exists)

Some substrate code may have a hard-coded list of `wat/holon/*.wat` files to load at startup. Sonnet checks `src/lib.rs` / `src/stdlib.rs` / `src/runtime.rs` for any file-list that mentions `defrecord.wat`. If found: remove the entry. If not found: the loader probably enumerates the directory dynamically.

### T4 — `wat/Record.wat:76` D12 comment

The comment currently reads "Co-exists with `:wat::holon::defrecord` (DIFFERENT behavior: that macro → HolonAST; ...)" — strike the co-exists claim. Replace with affirmative naming. Example replacement:

```
;; D12: :wat::Record::def is THE record-defining macro. Mints
;; Value::wat__Record with dual-form (struct + holon). Holon-form
;; access via :wat::holon::* auto-dispatch (Stone 234.5).
```

### T5 — Probe file naming inconsistency

`tests/probe_arc227_stone2_defrecord.rs` has "defrecord" in its filename and references the OLD macro name in its module-level docstring (per audit lines 1, 11, 62 from grep). After migration:
- The FILE name stays (history preservation; renaming would break git blame + churn the test file)
- The docstring SHOULD be updated to reflect "tests user-defined types via `:wat::Record::def` (formerly `:wat::holon::defrecord`)" — affirmative historical reference + current macro

Same for any other probe whose docstring mentions the old macro by name.

### T6 — Cross-probe regression

After migration, run ALL arc 234 + arc 236 + arc 232 probes (not just the 5 from EXPECTATIONS). Some probes I haven't enumerated may use the macro indirectly through a wat helper that uses the macro. Sonnet runs the full lib test suite + arc-prefix probes to surface any cross-probe regression.

### T7 — Deleted file's git history

`wat/holon/defrecord.wat` gets deleted. The git history preserves the file's prior content; INSCRIPTION (Stone 234.7) will reference the deleted file's prior path for historical record. Future readers can `git log --all -- wat/holon/defrecord.wat` to see the file's history. Per `feedback_inscription_immutable`: deletion is honest; preservation in git history is sufficient; no need to keep the file as deprecated-stub.

### T8 — D12 comment in wat/Record.wat — co-existence assertion was honest at Stone 234.2b ship

The wat/Record.wat:76 comment was honest at Stone 234.2b ship time — the two macros DID co-exist. Updating the comment to "is THE record-defining macro" is honest NOW (after Stone 234.6 retires the old one). The comment evolution is part of the substrate's truthful self-description. No revisionism; the comment reflects current state.

### T9 — wat-loader: does `wat/holon/defrecord.wat` load eagerly at startup?

If the substrate loads `wat/holon/*.wat` eagerly via a glob OR a hard-coded list, deleting `wat/holon/defrecord.wat` might leave a "file not found" error at startup. Sonnet checks the loader behavior; if the file is loaded eagerly + a list exists, remove the list entry alongside the file deletion.

### T10 — Lab repo impact (out of scope but noted)

Lab repos (`holon-lab-trading`, etc.) may have callers of `:wat::holon::defrecord`. After Stone 234.6 ships, if a lab repo tries to use the old macro, it FAILS at parse/check time (macro not registered). This is the HARD CUT working as intended — the lab repos must migrate to `:wat::Record::def` in their own repos. Stone 234.6 does NOT proactively migrate lab repos (D3 workspace boundary); Stone 234.7 INSCRIPTION notes the affirmative-out-of-scope: "lab repos migrate independently in their own repos."

---

## STOP triggers

- STOP-1 unexpected compile errors not tracing to find-replace / registry retirement / file deletion
- STOP-2 lib baseline regresses below 827 by even 1
- STOP-3 **120 min elapsed** (Mode A target 60-90 min; STOP-3 is 2× upper-bound)
- STOP-4 holon-rs touched
- STOP-5 Rust changes outside `src/stdlib.rs` (or wherever the loader file-list lives, if discovered)
- STOP-6 arc 234 OR arc 236 OR arc 232 regression in probe tests beyond test-body adjustment expected from migration
- STOP-7 clippy > 54
- STOP-8 transitional alias / deprecation warning / "defrecord-deprecated" form minted (D2 HARD CUT)
- STOP-9 `wat/holon/defrecord.wat` preserved as deprecated-stub (D2 — full delete; no stub)
- STOP-10 lab repos touched (D3 workspace boundary)
- STOP-11 probe arc 227 test assertions modified BEYOND what's required by the macro shape change (per T1 — test-body adjustment is acceptable if traceable to substrate-equivalent behavior; UNJUSTIFIED test changes are STOP)

Each STOP REJECTION.

---

## Calibration

**Target:** 60-90 min Mode A. **Upper:** 120 min (STOP-3).

Surface:
- Find-replace: 8 files; ~75 references; mechanical
- D12 comment update: 1 line in wat/Record.wat
- File delete: 1 (`wat/holon/defrecord.wat`)
- Registry retirement: 2 sites in `src/stdlib.rs`
- Optional loader file-list update (if discovered)
- Docstring updates in probe files referencing old macro (T5; ~2-3 file headers)

Net: ~150-200 line touch across ~10 files (mostly find-replace + small surgical edits).

Cascade depth: 1-2 compile rounds expected. Step-6 (`cargo build` after registry retirement) is the substrate-as-teacher catch for any missed caller.

Confidence: HIGH. Mechanical migration with clear order-of-operations + substrate-as-teacher cascade safety net + verified semantic equivalence via Stone 234.5's auto-dispatch.

Risks:
- T1 (arc 227 probe behavior preservation) — most likely place for surprise; sonnet investigates if probe fails
- T9 (file-list loader) — may or may not exist; sonnet checks
- T6 (cross-probe regression) — uses full test suite as safety net

---

## What this unblocks

- **Stone 234.7 INSCRIPTION + arc 234 closure** — Stone 234.6 is the LAST substrate work before arc 234 can close
- **Lab repos** (downstream) — once Stone 234.6 ships, lab repos see the macro as unavailable + must migrate; that's lab-repo work in those repos
- **Future record-related substrate work** — the surface is unified at `:wat::Record::*`; future stones (arc 232.1 defprotocol consumer-side, etc.) work against the canonical macro

After Stone 234.6 ships + Stone 234.7 INSCRIPTION closes arc 234: **arc 235 (PROPOSED) opens for records with rich VSA encodings**.

---

## Cross-references

- `wat/Record.wat` — Stone 234.2b's `:wat::Record::def` (the replacement macro)
- `wat/holon/defrecord.wat` — Stone 227.2 v3's `:wat::holon::defrecord` (the OLD macro to delete)
- `src/stdlib.rs` — registry sites for both macros (retirement target)
- `tests/probe_arc227_stone2_defrecord.rs` — heaviest caller probe (56 references)
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2b.md` — the replacement macro's sub-DESIGN
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — replacement macro shipment record
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.5.md` — auto-dispatch shipment (the semantic-equivalence mechanism)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.4.match.md` — most recent arc 234 ship (template for SCORE shape)
- `docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.2.md` — migration cascade precedent (47-fn sweep + substrate-as-teacher cascade)
- `docs/arc/2026/05/234-wat-record-hologram/PAUSE-CONTEXT.md` — arc 234 pause + resume framing
- `feedback_inscription_immutable` — HARD CUT discipline; no transitional aliases
- `feedback_no_known_defect_left_unfixed` — the discipline that puts 234.6 inside arc 234 (not separate arc 238)
- `feedback_workspace_boundaries` — wat-rs only; lab repos out of scope (D3)
- `feedback_stone_briefs_cite_prior_score` — BRIEF cites Stone 236.2 + Stone 234.4.match SCOREs for sonnet to mirror
- Task #402 — already closed by Stone 234.4.match; this stone is independent

After this stone ships: Stone 234.7 INSCRIPTION authoring (orchestrator-direct per `feedback_sonnet_no_realization_voice`).
