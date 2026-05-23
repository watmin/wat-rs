# BRIEF — Arc 225 Stone 225.1 (v3) — Bridge naming family substrate-wide rename + mint

**Stone scope:** Resolve the bridge naming honesty findings. Substrate-wide rename + mint + narrow, all under the symmetric layer-name + direction discipline. **Substrate-as-teacher cascade methodology** per FM 15 — cargo fail-count is the progress meter.

**Type:** Sonnet Mode A.
**Time budget:** 180-300 min target; 360 min STOP.
**Depends on:** Arc 224 audit complete; arc 225 reshape committed; FINDINGS-INTUERI-BRIDGE-OPS.md resolved the naming.
**Calibration:** Closest precedent — Stone 221.4b Phase 1+2 (~100 min for dispatcher rename + cascade). This stone is larger (~150-200 touch points vs ~95) + adds two new minted verbs + two cosmetic renames. Pattern locked.

## v3 supersedes earlier drafts

The v1 BRIEF (commit 898f2ed) proposed `:wat::holon::Atom` → `:wat::holon::atomize` rename. That was REJECTED twice: (a) `atomize` doesn't always produce HolonAST::Atom, so it's still polymorphic (b) the `materialize` framing was honest but broke the from-watast/to-watast family pattern. Intueri bridge-naming cast (FINDINGS-INTUERI-BRIDGE-OPS.md) verdict: ship `to-holon` / `from-holon` (symmetric directional family). User confirmed + observed the watast/holon asymmetry: drop "ast" suffix everywhere → `to-wat` / `from-wat` as well. v3 captures the final shape.

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/wat-rs/`** (NOT holon-rs!)
- Branch: `arc-170-gap-j-v5-deadlock-state` (already current)
- Linux only; no `--no-verify`
- DO NOT commit. Orchestrator commits after independent scoring.
- DO NOT touch holon-rs (the algebra primitives stay; arc 230 retires variants later)
- DO NOT touch wat-edn (wire format unaffected)
- **HARD CUT — no deprecation aliases.** Per user direction + `feedback_no_known_defect_left_unfixed`. Old verb names DELETED entirely.

## BASH DISCIPLINE (per `feedback_sonnet_bash_firewall` + Stone 221.4b + Stone 225.1-v1 lessons)

- ONE cargo/git command at a time, foreground
- NO piping cargo output through `| grep` / `| tail` (pipe buffers; fools you into thinking commands hang)
- NO concurrent background cargo runs
- `cargo test --release --lib -p wat` has 5 known signal-handler test hangs (task #413). Use targeted-skip command in Verification section.

## Pre-flight grep verified (orchestrator-grep'd 2026-05-22 late)

### Rust function rename targets

| Current name | New name | Location |
|---|---|---|
| `eval_algebra_atom` | `eval_holon_atom_constructor` | `src/runtime.rs:13820` (verb dispatcher; narrow) |
| `value_to_atom` | `wrap_holon_as_atom` | `src/runtime.rs:13838` (Rust helper; narrow) |
| `eval_atom_value` | `eval_holon_from_holon` | `src/runtime.rs:13633` (rename + namespace) |
| `holon_item_to_value` | `from_holon_item` + thread `op: &str` | `src/runtime.rs:13504` |
| `eval_holon_from_watast` | `eval_holon_from_wat` | `src/runtime.rs:14101` |
| `eval_holon_to_watast` | `eval_holon_to_wat` | `src/runtime.rs:14144` |

NEW Rust function to add: `eval_holon_to_holon` (the lift Value → HolonAST polymorphic op; absorbs the retired UP arms of `value_to_atom`).

### Verb-name rename (in dispatch tables + TypeScheme registrations)

| Current verb | New verb |
|---|---|
| `:wat::holon::Atom` (polymorphic) | `:wat::holon::Atom` (NARROW — only HolonAST input) |
| (NEW) | `:wat::holon::to-holon` (polymorphic UP from any Value) |
| `:wat::core::atom-value` | `:wat::holon::from-holon` (rename + namespace) |
| `:wat::holon::from-watast` | `:wat::holon::from-wat` |
| `:wat::holon::to-watast` | `:wat::holon::to-wat` |

### Scope estimates (orchestrator grep)

- **31** Rust call sites for `:wat::holon::Atom` literal
- **10** Rust call sites for `:wat::core::atom-value` literal
- **54** wat-side caller sites for `:wat::holon::Atom` + `:wat::core::atom-value`
- Plus `:wat::holon::from-watast` / `to-watast` callers (fresh grep at start of stone)
- **Total estimated: ~150-200 touch points**

### Known dispatch table sites + TypeScheme entries

- `src/runtime.rs:13820` — `eval_algebra_atom` registration
- `src/runtime.rs:13633` — `eval_atom_value` registration
- `src/runtime.rs:14101` — `eval_holon_from_watast` registration (at line 4941 in dispatcher arm too)
- `src/runtime.rs:14144` — `eval_holon_to_watast` registration (at line 4942 in dispatcher arm too)
- `src/check.rs:13558` — `:wat::holon::Atom` TypeScheme
- `src/check.rs:13591` — `:wat::core::atom-value` TypeScheme
- `src/check.rs:5326` — `:wat::holon::Atom | :wat::holon::leaf` special-case handler
- `src/check.rs:5362` — `:wat::core::atom-value` special-case handler

## Your scope (sonnet)

### Phase 1 — Substrate Rust rename + mint (enumerate precisely; mechanical)

**A. Narrow `:wat::holon::Atom`**:
- Rename Rust fn `eval_algebra_atom` → `eval_holon_atom_constructor` (the verb dispatcher).
- Rename Rust fn `value_to_atom` → `wrap_holon_as_atom`. Body: accept ONLY `Value::holon__HolonAST(inner)` input; return `Value::holon__HolonAST(HolonAST::Atom(inner))`. DELETE all other input-arm branches.
- TypeScheme `check.rs:13558` change from `∀T. T → HolonAST` to `HolonAST → HolonAST` (narrow).
- Update special-case at `check.rs:5326`.

**B. Mint `:wat::holon::to-holon`** (absorbs the retired UP arms):
- NEW Rust fn `eval_holon_to_holon` — accepts `Value` of any type; produces appropriate HolonAST. Body: same polymorphic dispatch logic that USED to live in `value_to_atom` (primitives → leaves; WatAST → structural lower via `watast_to_holon`; collections → bare Bundle composition; Uuid → `Bind(Tag, String)`; HolonAST → Atom-wrap).
- NEW dispatch table entry: `":wat::holon::to-holon" => eval_holon_to_holon(args, list_span, env, sym)`.
- NEW TypeScheme registration in `check.rs` — `∀T. T → HolonAST` (polymorphic; operation-name honest).
- New `infer_list` special-case if needed.

**C. Mint `:wat::holon::from-holon`** + retire `:wat::core::atom-value`:
- Rename verb registration: `":wat::core::atom-value"` → `":wat::holon::from-holon"` (in `runtime.rs` dispatch table + `check.rs` TypeScheme).
- Rename Rust fn `eval_atom_value` → `eval_holon_from_holon`. Body unchanged (still polymorphic decode).
- **Doc comment refresh:** `runtime.rs:13619-13629` — fix L1-3 finding. Current doc says "Composite (Bundle/...) → error" but body handles three-way Bundle dispatch (Vec/HashMap/HashSet) since arc 216. Refresh to honestly describe the polymorphic decode.
- Rename Rust helper `holon_item_to_value` → `from_holon_item`. **Thread `op: &str` parameter** through the signature (closes arc 224 L1-runtime-3 latent lie); update all callers to pass their own op name.
- Update special-case at `check.rs:5362`.

**D. Rename `from-watast` → `from-wat`**:
- Verb registration in dispatch table: `":wat::holon::from-watast"` → `":wat::holon::from-wat"`.
- Rust fn: `eval_holon_from_watast` → `eval_holon_from_wat`.
- TypeScheme + special-case handler updates.
- No semantic change.

**E. Rename `to-watast` → `to-wat`**:
- Verb registration: `":wat::holon::to-watast"` → `":wat::holon::to-wat"`.
- Rust fn: `eval_holon_to_watast` → `eval_holon_to_wat`.
- Same as D — cosmetic.

### Phase 2 — Substrate-as-teacher cascade

After Phase 1 lands, `cargo build --release -p wat` will fail with many E0xxx errors from:
- Rust call sites that still use the old function names
- Rust integration tests that reference the old verb strings
- wat source files (loaded into the substrate) that use the old verb names

**Iterate per FM 15:** read the failures; apply the rename rule; rerun cargo; the fail-count drops; repeat until green.

Methodology: ONE cargo command at a time, no pipes, no concurrent runs. Don't pre-enumerate; trust the cascade.

### Phase 3 — Wat-side caller sweep

The `wat/` and `wat-tests/` sources are loaded at startup; they emit cascade errors when the substrate verb-name registry changes. Per pre-flight grep, ~54+ caller sites use one of the renamed/retired verbs. Sweep:
- `wat/**/*.wat` — substrate-bundled wat files
- `wat-tests/**/*.wat` — test fixtures
- `tests/*.rs` — Rust integration tests with embedded wat strings

**For `:wat::holon::Atom` callers** (this is the tricky one):
- If the input was a HolonAST (Atom-wrap case) → keep `:wat::holon::Atom` (now narrow)
- If the input was ANY other type → change to `:wat::holon::to-holon` (the new polymorphic UP verb)
- Use the type at the call site to decide; if uncertain, prefer `to-holon` (more general)

**For `:wat::core::atom-value` callers**:
- Replace with `:wat::holon::from-holon` (verb + namespace change)

**For `:wat::holon::from-watast` / `to-watast` callers**:
- Replace with `:wat::holon::from-wat` / `:wat::holon::to-wat` respectively (cosmetic; same semantics)

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
- **STOP-2 (test failure beyond cascade-rename consequences):** if a test fails after the rename sweep is complete (cargo build green) for reasons OTHER than verb-name change, STOP + diagnose + frame per Stone 221.3 Delta 1a discipline (broken-by-this-stone honest framing; do NOT call it "pre-existing").
- **STOP-3 (360 min elapsed):** wall-clock STOP.
- **STOP-4 (holon-rs touched accidentally):** STOP and report.
- **STOP-5 (additional substrate verbs found polymorphically named):** if you encounter other `:wat::holon::*` verbs that show the same Atom-style polymorphic-dispatch pattern (NOT in the audit's known L1 list), DO NOT auto-extend this stone's scope. Surface as a finding for the orchestrator to spawn additional fix-arcs. Arc 225 is scoped to the 5 bridge ops only.
- **STOP-7 (bash discipline — cargo hang):** if a `cargo` command runs >5 min with no streaming output, do NOT panic. Check whether you accidentally piped through `| tail` / `| grep`. The targeted-skip command should complete in seconds-to-minutes after compile.

## Out-of-scope

- holon-rs changes (algebra primitives stay; arc 230 retires variants later)
- wat-edn changes (wire format unaffected)
- Collection classifier-wrap (arc 228's scope; `to-holon`'s collection arm ships bare Bundle for now)
- Type predicates `(is-X? value)` (arc 226's scope)
- INSCRIPTION (Stone 225.2; blocked on arc 228 closing per spawn-block)
- Deprecation aliases for backwards compatibility (HARD CUT per user direction)
- Quasiquote evaluator changes (arc 229; deferred)
- EDN-form named constructors (arc 222)
- WatAST primitive-layer honesty (arc 223)

## Notes on the "fractal of correctness" principle

User direction inscribed in arc 225 history (2026-05-22): *"we break what we break - dishonesty is illegal in our code - if the names break - then the users are broken and they need their own fixes - the arcs are a fractal of correctness."*

This stone HARD-CUTS the retired names. Any consumer code that depended on the old names will see a clean compile error and must adapt. That's not a bug — that's the substrate refusing to lie. The substrate-as-teacher cascade is the substrate enforcing the new doctrine through every call site.
