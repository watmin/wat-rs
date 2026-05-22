# BRIEF — Arc 225 Stone 225.1 — Substrate rename + wat-side caller sweep (atomize / materialize)

**Stone scope (sonnet portion):** Rename the boundary-crossing verb pair substrate-wide. Both verbs land in `:wat::holon::*` namespace. Old names DELETED entirely (hard-cut per user direction: *"dishonesty is illegal in our code; arcs are a fractal of correctness; if names break, users are broken and they need their own fixes"*). Substrate-as-teacher cascade methodology per FM 15.
**Type:** Sonnet Mode A.
**Time budget:** 90-180 min target; 240 min STOP.
**Depends on:** Arc 224 audit findings (FINDINGS-INTUERI-RUNTIME.md L1-1 + family pattern; AGGREGATE-FINDINGS.md Group B). Per `feedback_spawn_block_winding`, this is arc 224's spawn child.
**Calibration:** Per `feedback_stone_briefs_cite_prior_score` — closest precedents: Stone 221.4b (~100 min for dispatcher + cascade in wat-rs) and arc 159 (substrate-wide sweep precedent at ~951 sites). This sweep is smaller (~95 sites estimated); pattern locked.

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/wat-rs/`** (NOT holon-rs!)
- Branch: `arc-170-gap-j-v5-deadlock-state` (already current)
- Linux only; no `--no-verify`.
- DO NOT commit. Orchestrator commits after independent scoring.
- DO NOT touch holon-rs (the algebra primitive `HolonAST::Atom` STAYS; intueri 224.1 confirmed it's honest)
- DO NOT touch wat-edn (wire format doesn't reference these verbs)
- **HARD CUT — no deprecation aliases.** Old names DELETED. Per user direction + `feedback_no_known_defect_left_unfixed`.

## BASH DISCIPLINE (per `feedback_sonnet_bash_firewall` + Stone 221.4b lessons)

- ONE cargo/git command at a time, foreground
- NO piping cargo output through `| grep` / `| tail` (pipe buffers until process exit; fools you into thinking commands hang)
- NO concurrent background cargo runs
- `cargo test --release --lib -p wat` has ~5 known signal-handler test hangs (task #413). Use the targeted-skip command from BRIEF section "Verification" below; do NOT run the full --lib sweep.

## Pre-flight verified (orchestrator-grep'd 2026-05-23 morning)

### Rust function rename targets

| Current name | New name | Location |
|---|---|---|
| `eval_algebra_atom` | `eval_holon_atomize` | `src/runtime.rs:13820` |
| `value_to_atom` | `atomize_value` | `src/runtime.rs:13838` |
| `eval_atom_value` | `eval_holon_materialize` | `src/runtime.rs:13633` |
| `holon_item_to_value` | `materialize_holon_item` | `src/runtime.rs:13504` |

### Verb-name rename (in dispatch tables + TypeScheme registrations)

| Current verb | New verb |
|---|---|
| `:wat::holon::Atom` | `:wat::holon::atomize` |
| `:wat::core::atom-value` | `:wat::holon::materialize` (note namespace move!) |

### Scope estimates (orchestrator grep)

- **31** Rust call sites for `:wat::holon::Atom` literal
- **10** Rust call sites for `:wat::core::atom-value` literal
- **54** wat-side caller sites (`wat/`, `wat-tests/`) using either verb
- Plus dispatch table entries + TypeScheme registrations + adjacent doc comments
- **Total estimated: ~95-110 touch points**

### Known dispatch table sites (just-grep'd)

- `src/runtime.rs:13820` `eval_algebra_atom` registration
- `src/runtime.rs:13633` `eval_atom_value` registration
- `src/check.rs:13558` `:wat::holon::Atom` TypeScheme
- `src/check.rs:13591` `:wat::core::atom-value` TypeScheme
- `src/check.rs:5326` `:wat::holon::Atom | :wat::holon::leaf` special-case handler
- `src/check.rs:5362` `:wat::core::atom-value` special-case handler

Per arc 224 Stone 224.3 finding: TypeScheme registrations are HONEST (∀T. T → HolonAST + ∀T. HolonAST → T). The verb-name rename does NOT change the type signature — just the verb-name string + the registration key.

## Your scope (sonnet)

### Phase 1 — Substrate Rust rename (enumerate precisely; mechanical)

1. **Rename `eval_algebra_atom` → `eval_holon_atomize`** at `src/runtime.rs:13820`. Update dispatch table entry (search for `"eval_algebra_atom"` references; there may be wrapper/handler indirection).
2. **Rename `value_to_atom` → `atomize_value`** at `src/runtime.rs:13838`. Update all callers in `src/runtime.rs` + any sibling .rs files. Doc comment refresh.
3. **Rename `eval_atom_value` → `eval_holon_materialize`** at `src/runtime.rs:13633`. Update dispatch table entry.
4. **Rename `holon_item_to_value` → `materialize_holon_item`** at `src/runtime.rs:13504`. Update all callers + signature documentation. (Per arc 224 L1-runtime-3 finding: also thread `op: &str` parameter while we're touching the signature, mirroring `require_holon` / `require_vec` pattern. This closes the latent op-name lie identified in arc 224.)
5. **Rename dispatch-table verb entries** in `src/runtime.rs`:
   - `":wat::holon::Atom"` → `":wat::holon::atomize"`
   - `":wat::core::atom-value"` → `":wat::holon::materialize"` (namespace move)
6. **Rename TypeScheme registrations** in `src/check.rs`:
   - At `:13558`: `":wat::holon::Atom"` → `":wat::holon::atomize"`
   - At `:13591`: `":wat::core::atom-value"` → `":wat::holon::materialize"`
7. **Rename special-case handlers in `infer_list`** in `src/check.rs`:
   - At `:5326`: `":wat::holon::Atom" | ":wat::holon::leaf"` → `":wat::holon::atomize" | ":wat::holon::leaf"`
   - At `:5362`: `":wat::core::atom-value"` → `":wat::holon::materialize"`

### Phase 2 — Substrate-as-teacher cascade

After Phase 1 lands, `cargo build --release -p wat` will fail with many E0xxx errors from:
- Rust call sites that still use the old function names
- Rust integration tests that reference the old verb strings
- wat source files (loaded into the substrate) that use the old verb names

**Iterate per FM 15:** read the failures; apply the rename rule; rerun cargo; the fail-count drops; repeat until green.

Methodology: ONE cargo command at a time, no pipes, no concurrent runs. Don't pre-enumerate; trust the cascade.

### Phase 3 — Wat-side caller sweep

The `wat/` and `wat-tests/` sources are loaded at startup; they emit cascade errors when the substrate verb-name registry changes. Per the pre-flight grep, ~54 caller sites use either retired verb. Sweep:
- `wat/**/*.wat` — substrate-bundled wat files
- `wat-tests/**/*.wat` — test fixtures
- `tests/*.rs` — Rust integration tests with embedded wat strings

Replace `:wat::holon::Atom` → `:wat::holon::atomize` and `:wat::core::atom-value` → `:wat::holon::materialize`. Update any adjacent doc comments naming the retired verbs.

### Phase 4 — Doc-comment refresh as discovered

While sweeping, refresh adjacent doc comments that name the retired verbs. Don't go on a global doc-comment hunt — fix what you touch.

### Verification (after all green)

Run each command DIRECTLY (no pipes, foreground, one at a time):

```
cargo build --release -p wat
cargo test --release --lib -p wat -- --skip reset_sighup --skip reset_sigusr1 --skip sigusr1_query --skip sigusr2_and_sighup --skip user_signal_predicates --skip reset_sigusr2
cargo test --release --test wat_arc220_char
cargo test --release --test wat_arc221_char_atomization
cargo test --release --test wat_arc221_keyword_nil_tag_atomization
cargo test --release --test wat_arc221b_keyword_dispatcher_completeness
cargo test --release --test wat_arc221b_macro_support_keyword_shape
cargo test --release --test wat_arc143_manipulation
cargo test --release -p wat-edn
cargo clippy --release --all-targets -p wat-edn -- -D warnings
```

All must complete cleanly (signal-handler hangs explicitly skipped per task #413).

**Holon-rs untouched** — `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` must be empty.

**Write `wat-rs/docs/arc/2026/05/225-atomize-materialize-rename/SCORE-STONE-225.1.md`** mirroring SCORE-STONE-221.4b.md shape.

## STOP triggers

- **STOP-1 (substrate compile error after Phase 1 sites — UNEXPECTED):** if cargo emits errors that DON'T match the rename pattern (errors from sites that shouldn't depend on the renamed verbs/fns), STOP and report. The cascade is the brief, but unrelated breakage is a distinct surface.
- **STOP-2 (test failure beyond cascade-rename consequences):** if a test fails after the rename sweep is complete (cargo build green) for reasons OTHER than verb-name change, STOP + diagnose + frame per Stone 221.3 Delta 1a discipline (broken-by-this-stone honest framing).
- **STOP-3 (240 min elapsed):** wall-clock STOP.
- **STOP-4 (holon-rs touched accidentally):** STOP and report.
- **STOP-5 (additional substrate verbs found polymorphically named):** if you encounter other `:wat::holon::*` verbs that show the same Atom-style polymorphic-dispatch pattern (NOT in the audit's known L1 list), DO NOT auto-extend this stone's scope. Surface as a finding for the orchestrator to spawn additional fix-arcs. Arc 225 is scoped to Atom + atom-value only.
- **STOP-7 (bash discipline — cargo hang):** if a `cargo` command runs >5 min with no streaming output, do NOT panic. Check whether you accidentally piped through `| tail` / `| grep`. The targeted-skip command should complete in seconds-to-minutes after compile.

## Out-of-scope

- holon-rs changes (algebra primitive `HolonAST::Atom` stays)
- wat-edn changes (wire format unaffected)
- Arc 224 Group A fixes (Stone 224.5's scope — those are separate L1 lies)
- Other arc 224 L2 mumbles
- USER-GUIDE / BOOK / 058 spec updates (Stone 225.3's scope)
- INSCRIPTION (Stone 225.4)
- Deprecation aliases for backwards compatibility (HARD CUT per user direction)

## Notes on the "fractal of correctness" principle

User direction inscribed in arc 225 DESIGN.md: *"we break what we break - dishonesty is illegal in our code - if the names break - then the users are broken and they need their own fixes - the arcs are a fractal of correctness."*

This stone HARD-CUTS the retired names. Any consumer code that depended on the old names will see a clean compile error and must adapt. That's not a bug — that's the substrate refusing to lie. The substrate-as-teacher cascade is the substrate enforcing the new doctrine through every call site.
