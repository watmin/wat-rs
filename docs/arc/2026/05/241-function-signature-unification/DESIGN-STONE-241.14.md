# DESIGN — Stone 241.14 — `:wat::core::def-restricted` + `:wat::core::defn-restricted` ABSORB INTO METADATA-MAP (Enemy 4 of 4)

**Status:** STRIKE-READY (2026-05-29 very late). Enemy 4 in the define-family death campaign. Honors broken Stone 241.6 D10 + line-182 commitment that was orphaned when Stone 241.10's scope shifted to remedy apparatus. **def and defn are the ONLY definers post-stone** — restriction declarations migrate to metadata-map on def/defn.

## User direction (load-bearing)

User direction 2026-05-29 very late: *"study the dungeon - the enemy is weakened - finish them - their name is rendered illegal - def and defn are the only ways."*

The "only ways" framing tightens Enemy 4's scope: NOT just retire `def-restricted`, ALSO retire `defn-restricted` macro. Both forms die. Restrictions live as `:restricted-to` key in metadata-map on def/defn.

## The orphaned commitment this honors

Stone 241.6 DESIGN explicitly committed (D10 + line 182):
- *"Subsumes arc 203's `def-restricted` / `defn-restricted` / `struct-restricted` legacy"* (line 11)
- *"HARD CUT of `def-restricted` / `defn-restricted` (that's Stone 241.10)"* (line 182)

Stone 241.10 absorbed remedy-apparatus scope (mid-arc scope shift); the def-restricted/defn-restricted HARD CUT was orphaned silently. No SCORE/INSCRIPTION re-stated the commitment with a new target stone number. **Stone 241.14 lands that work — 25 days late.** Per `feedback_defer_by_naming`: mid-arc scope shift requires explicit redirect; this stone's INSCRIPTION acknowledges the orphan explicitly.

## What this stone delivers

### S1 — Migrate restriction enforcement to metadata-map READS

Current state (parallel storages):
- `SymbolTable.defined_value_restrictions: HashMap<String, Vec<String>>` (arc 198; populated by def-restricted parser + RestrictionEntry inventory)
- `SymbolTable.binding_metadata: HashMap<String, HashMap<String, WatAST>>` (Stone 241.6; populated by def's metadata-map clause + reflection)

Target state (single storage):
- `binding_metadata` is the sole store. The `:restricted-to` key (value: Vector of prefix-keywords) carries the whitelist.

Walker migration: `walk_for_def_restricted_call` at `src/check.rs:3823` currently reads via `env.get_defined_value_restriction(head)`. Post-stone: reads from `binding_metadata[head]` looking up the `:restricted-to` key → extracts Vec<String> from the Vector<Keyword> WatAST value.

```rust
// post-stone shape
if let Some(meta) = env.get_binding_metadata(head) {
    if let Some(WatAST::List(prefix_items, _)) = meta.get(":restricted-to") {
        // prefix_items[0] is :wat::core::Vector head; rest are prefix keywords
        let prefixes: Vec<String> = prefix_items[1..].iter()
            .filter_map(|n| if let WatAST::Keyword(k, _) = n { Some(k.clone()) } else { None })
            .collect();
        if !caller_matches_prefix_list(enclosing_fn, &prefixes) {
            errors.push(CheckError::DefRestrictedCallerNotAllowed { ... });
        }
    }
}
```

(Walker renamed to drop "def_restricted" — proposed: `walk_for_restricted_call`. Error variant name stays `DefRestrictedCallerNotAllowed` per `feedback_inscription_immutable` — historical naming preserved.)

### S2 — DELETE `defined_value_restrictions` storage entirely

- `SymbolTable.defined_value_restrictions` field DELETED (runtime.rs:1724)
- `CheckEnv.defined_value_restrictions` field DELETED (check.rs:2050)
- `set_defined_value_restriction` / `get_defined_value_restriction` methods DELETED
- All populate-paths in `register_runtime_defs_form` (runtime.rs:2676, 2996, 4430, 4504) DELETED
- All read paths consolidated through `binding_metadata`
- Mirror copy in `CheckEnv::from_symbols` (check.rs:2095) DELETED

### S3 — HARD CUT `:wat::core::def-restricted` form

Mirror Stone 241.11/241.12/241.13 pattern. Add HARD-CUT-rejection arm at `src/check.rs`:

```rust
":wat::core::def-restricted" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.14); use ':wat::core::def' with metadata-map: `(def :name {{:restricted-to [<prefix-kw>...]}} expr)`", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}
```

Substrate parser code that PARSED `:wat::core::def-restricted` (runtime.rs:4306+, related check.rs:9978-10072 + 10245 + 5615+ arm) — DELETED.

### S4 — HARD CUT `:wat::core::defn-restricted` macro

The wat-source macro at `wat/core.wat:202-209` is DELETED. Add HARD-CUT-rejection arm at `src/check.rs` mirror to S3:

```rust
":wat::core::defn-restricted" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.14); use ':wat::core::defn' with metadata-map: `(defn :name {{:restricted-to [<prefix-kw>...]}} [<args>] -> :<Ret> body)`", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}
```

### S5 — Append 8th + 9th RETIREMENT_TABLE entries

`src/remedy/retirement.rs`:

```rust
// Stone 241.14 — def-restricted absorbed into def + metadata-map.
(":wat::core::def-restricted",    ":wat::core::def"),
(":wat::core::defn-restricted",   ":wat::core::defn"),
```

### S6 — Migrate `RestrictionEntry` inventory channel to populate `binding_metadata`

Current state: `src/restriction_entry.rs` defines `RestrictionEntry { wat_name, prefixes }`. Rust crates `inventory::submit!` entries. `freeze.rs:835` iterates entries + inserts into `defined_value_restrictions`.

Target state:
- `RestrictionEntry` struct STAYS (same shape; `wat_name` + `prefixes`)
- `inventory::collect!(RestrictionEntry)` STAYS
- `freeze.rs:835`-equivalent iterates + populates `binding_metadata[wat_name]` with `:restricted-to` → Vector of prefix-keywords WatAST
- `#[restricted_to(...)]` proc-macro attribute STAYS (no surface change for Rust-side declarations)

This preserves arc 170 Stone B's restriction on `Thread/join-result` + `Process/join-result` (the original RestrictionEntry consumers) without disruption.

### S7 — Cascade migration of user-surface forms

**Tests (5 sites in `tests/wat_arc198_def_restricted.rs`):**

```scheme
;; OLD (Test 1-4):
(:wat::core::def-restricted :my::kernel::restricted-fn
  :restricted-to [:wat::kernel::]
  (:wat::core::fn [] -> :wat::core::i64 42))

;; NEW:
(:wat::core::def :my::kernel::restricted-fn
  {:restricted-to [:wat::kernel::]}
  (:wat::core::fn [] -> :wat::core::i64 42))
```

```scheme
;; OLD (Test 5 defn-restricted):
(:wat::core::defn-restricted :my::kernel::restricted-fn
  :restricted-to [:wat::kernel::]
  [] -> :wat::core::i64 42)

;; NEW:
(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [:wat::kernel::]}
  [] -> :wat::core::i64 42)
```

**Other test sites:**
- `tests/wat_arc170_stone_b_walker_collapse.rs` — 1 reference; migrate or repurpose for HARD-CUT acceptance

**Docs:**
- `docs/USER-GUIDE.md:734-795` — `:wat::core::def-restricted` / `defn-restricted` section: rewrite to document the metadata-map approach; old form names referenced as RETIRED with arc 241.14 citation
- `docs/CONVENTIONS.md:33-34` — replace "Wat (sugar) | fn binding | `(:wat::core::defn-restricted ...)`" entry with the new metadata-map shape

### S8 — wat/core.wat macro DELETION

`wat/core.wat:202-209` — the `:wat::core::defn-restricted` macro definition. DELETED. The HARD-CUT arm at check.rs catches any residual callers.

`wat/core.wat:187-201` — the macro's documentation comment block. UPDATE to historical (e.g., "Arc 198 defined defn-restricted; Stone 241.14 retired the macro; restrictions now via metadata-map on defn").

### S9 — Reflection emitter audit

Per Stone 241.12/13 trap-door precedent: grep `Keyword.*def-restricted\|Keyword.*defn-restricted` in src/. Any AST-construction emitting these forms migrates to emit `:wat::core::def` / `:wat::core::defn` with metadata-map.

### S10 — Probe verification

`tests/probe_arc241_stone14_restricted_absorbed.rs` (NEW). FM 2-bis disconfirming contracts:
- C01: `(def :name {:restricted-to [:allowed::]} expr)` registers restriction; allowed caller passes
- C02: same form; non-allowed caller fails with `DefRestrictedCallerNotAllowed`
- C03: `(defn :name {:restricted-to [:allowed::]} [args] -> :Ret body)` registers restriction
- C04: `:wat::core::def-restricted` HARD-CUT-rejected
- C05: `:wat::core::defn-restricted` HARD-CUT-rejected
- C06: rejection remedies name `:wat::core::def` / `:wat::core::defn` respectively

### S11 — Author SCORE doc

Per `feedback_score_present_check_before_closure`. `SCORE-STONE-241.14.md` at strike-end.

## Locked decisions

### D1 — Per user direction: def and defn are the ONLY definers

Both `def-restricted` (substrate primitive) AND `defn-restricted` (wat macro) die. The form-family collapses to def + defn. Restrictions move to metadata-map (`:restricted-to` key).

### D2 — `binding_metadata` is the sole restriction store

`defined_value_restrictions` storage DELETED entirely. ONE store; ONE walker; ONE source of truth.

### D3 — Walker renames; error variant name preserved

`walk_for_def_restricted_call` → `walk_for_restricted_call`. But `CheckError::DefRestrictedCallerNotAllowed` keeps its name per `feedback_inscription_immutable` — historical variant names stay even when underlying mechanism changes.

### D4 — `RestrictionEntry` inventory channel migrates path; surface unchanged

`RestrictionEntry` struct + `inventory::collect!` + `#[restricted_to(...)]` attribute all STAY. Only the populate-target changes: `binding_metadata` instead of `defined_value_restrictions`. Rust-side substrate primitives (Thread/join-result; Process/join-result) keep their declared restrictions; the migration is transparent.

### D5 — RETIREMENT_TABLE grows to 9 entries

Stone 241.13 made it 7; Stone 241.14 adds 8 + 9 (def-restricted, defn-restricted).

### D6 — Vigilia NOT required (no namespaced home)

### D7 — INTERSTITIAL orchestrator-exclusive (`feedback_sonnet_never_drafts_interstitial`)

### D8 — SCORE-write at end (`feedback_score_present_check_before_closure`)

### D9 — Stone 241.15 scope OFF-LIMITS

Sonnet does NOT touch `is_mutation_head`, `parse_define_form`, `register_define`, `is_define_form` — those are Enemy 3 (Stone 241.15) scope. **`:wat::core::define` is already HARD-CUT** at startup (Stone 241.11); Stone 241.14 doesn't touch it.

## Trap-door audit

### T1 — Walker reads from binding_metadata; need keyword-value extraction

The walker needs to extract `Vec<String>` (prefix list) from `binding_metadata[name].get(":restricted-to")` which returns `Option<&WatAST>`. The value is a `WatAST::List` with `:wat::core::Vector` head; subsequent items are prefix keywords. Helper function to extract prefix list from Vector-AST may be needed.

Resolution: write small `extract_prefix_list_from_metadata` helper; place at check.rs adjacent to walker.

### T2 — RestrictionEntry migration breaks existing arc 170 Stone B restrictions if not handled

Thread/join-result + Process/join-result + any other `#[restricted_to(...)]`-decorated Rust fns must continue to be restricted post-stone. The inventory channel migration MUST populate `binding_metadata` correctly.

Resolution: freeze.rs RestrictionEntry iterator post-stone inserts WatAST::List of WatAST::Keyword prefixes. Verify with existing arc 170 acceptance tests.

### T3 — `defined_value_restrictions` reads from older arcs may still exist

Older code that read from `defined_value_restrictions` directly (rather than via the walker) gets compile errors when the field is deleted. Cascade per substrate-as-teacher.

Resolution: let the compiler drive site-finding.

### T4 — Reflection emitters producing def-restricted/defn-restricted AST

If reflection code constructs these forms, they break. Grep + migrate.

### T5 — Doc cascade may surface stale references

USER-GUIDE.md + CONVENTIONS.md + maybe SERVICE-PROGRAMS.md reference def-restricted/defn-restricted. Per Stone 241.12's S5 doc-fold-in pattern.

### T6 — Test `wat_arc198_def_restricted.rs` is the canonical acceptance test

Per Stone 241.13 precedent (wat_arc146_dispatch_mechanism.rs DELETED) — option to delete wat_arc198_def_restricted.rs entirely OR migrate its 5 tests to use the new metadata-map syntax (preserve as regression coverage for the new mechanism).

Recommend: MIGRATE — the tests test the access-control SEMANTICS which the new mechanism preserves. Delete only if the tests test the OLD FORM SYNTAX (which they do; per the SCORE doc the tests are about "def-restricted defmacro" semantics). Per-test judgment.

### T7 — Empty whitelist edge case (`:restricted-to []`)

Current semantics: empty whitelist matches nothing; every caller fails. Per arc 198 SCORE: "honest substrate-internal-only reading." Preserve this semantic post-migration — `:restricted-to []` in metadata-map still means "no caller allowed."

### T8 — Sonnet "infrastructure stays empty" temptation (Stone 241.13's #6 trap-door class)

Per D1 + `feedback_hard_cut_admits_no_bypasses`. STOP if surfaces.

## STOP triggers — REJECTION

1. Compile errors not traced to restriction migration cascade
2. Lib < 890 (post-241.13 baseline) — note: test migration count may shift; document
3. **180 min elapsed**
4. holon-rs touched (STOP-5)
5. `:wat::core::def-restricted` or `:wat::core::defn-restricted` survives as ACTIVE substrate use post-stone (outside HARD-CUT arms + retirement entries + historical comments + probe source)
6. `defined_value_restrictions` field/method PRESERVED (D2 violation — the field MUST be DELETED)
7. Files outside permitted scope (`src/check.rs` / `src/freeze.rs` / `src/runtime.rs` / `src/resolve.rs` / `src/remedy/retirement.rs` / `src/restriction_entry.rs` / `src/closure_extract.rs` if reflection emitters touched / `wat/core.wat` macro deletion + comment update / `tests/wat_arc198_def_restricted.rs` migration / `tests/wat_arc170_stone_b_walker_collapse.rs` / `tests/probe_arc241_stone14_*` / `docs/USER-GUIDE.md` / `docs/CONVENTIONS.md` / SCORE doc)
8. Stone 241.14 probe < 6/6
9. Stone 241.x or arc 237/238/242 probes regress (except wat_arc198 if delete-decision lands there)
10. Clippy > 925 (looser gate; substrate refactor causes line-shift; arc 109 sweeps to zero)
11. Auto-fixer crate survives commit
12. Sonnet writes to INTERSTITIAL → D7 + `feedback_sonnet_never_drafts_interstitial` violation
13. SCORE-STONE-241.14.md NOT authored at end → D8 + `feedback_score_present_check_before_closure` violation
14. Stone 241.15 scope touched (`is_mutation_head`, `parse_define_form`, etc.) → D9 violation
15. Arc 170 Stone B Thread/join-result + Process/join-result restrictions silently broken (T2 violation)

## FM 2-bis evidence

`tests/probe_arc241_stone14_restricted_absorbed.rs` (NEW; 6 contracts; verified disconfirms at HEAD before BRIEF spawns).

## Calibration

**Target band: 90-180 min Mode A.**

Stone 241.14's scope decomposition:
- Walker migration (read from binding_metadata) — **~20-30 min**
- `defined_value_restrictions` storage DELETION cascade — **~30-45 min** (similar to Stone 241.13's DispatchRegistry cascade scope; per substrate-as-teacher)
- RestrictionEntry inventory channel migration — **~10-15 min**
- HARD-CUT arms (2) + RETIREMENT_TABLE entries (2) — **~10 min**
- `wat/core.wat` macro deletion — **~5 min**
- Test migration (`wat_arc198_def_restricted.rs` 5 tests + `wat_arc170_stone_b_walker_collapse.rs`) — **~20-30 min**
- Doc migration (USER-GUIDE.md + CONVENTIONS.md) — **~10-15 min**
- Pre-INSCRIPTION grep + final verification — **~10 min**
- SCORE doc authoring — **~10-15 min**

Per `feedback_stone_briefs_cite_prior_score`: BRIEF cites SCORE-STONE-241.13.md (substrate scaffolding deletion + per-test judgment + clippy-down-with-deletion pattern); SCORE-STONE-241.12.md (cascade discipline + trap-door absorption); SCORE-STONE-241.10.md (substrate-mint shape for the walker enhancement).

## What this unblocks

**Stone 241.15** — Enemy 3 (`:wat::core::define` eval-time residue completion; closes Stone 241.11's partial HARD CUT)

**Stone 241.16** — INSCRIPTION closes arc 241. Explicitly acknowledges Stone 241.6 orphaned commitment + Stone 241.14 closes the work 25 days late + `feedback_defer_by_naming` lesson inscribed.

**Arc 237.8b** — reopens after Stone 241.16

**The def\*-prefix family** — def / defn / defclause / defmacro / defstruct / defenum / defalias all live native. Restrictions are metadata on def/defn — NO special form for restriction. The form family is COMPLETE and access-control semantics live as binding metadata (`feedback_wat_llm_first_design`: one canonical path; def + defn are the only ways).

**Future binding-level metadata** — `:doc`, `:deprecated`, `:since`, `:see-also`, etc. — all consume the same metadata-map mechanism. The pattern is foundational.
