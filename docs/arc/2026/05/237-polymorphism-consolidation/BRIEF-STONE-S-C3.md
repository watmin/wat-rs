# BRIEF — Stone S-C.3 — macro split: base `:wat::Record::def` / holonic `:wat::holon::Record::def`

**Status:** READY TO SPAWN. `model: "sonnet"`.
**Anchor cwd:** `/home/watmin/work/holon/wat-rs/` (`pwd` first; reject `.claude/worktrees/`; `git -C` if needed).
**Sub-DESIGN:** `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-C3.md` — read it; it has the full mechanism + the 15-contract coverage + the cascade rule.
**Mirror:** `eval_record_of` (runtime.rs:16540), the `:wat::Record::def` macro (wat/Record.wat), `parse_recordtype` (types.rs:2345).

## What to do — flip the unmarked name to BASE, holonic becomes opt-in

This is a coordinated cascade: substrate constructors + macros + recordtype parents, then migrate
the breaking callers. **Atomic green commit** (no broken disk).

### 1. Constructors (`src/runtime.rs`)
- **Rename** the existing `:wat::Record::of` (3-arg holonic, `eval_record_of`) → **`:wat::holon::Record::of`**
  (dispatch arm + `const OP` + the fn name; body UNCHANGED — still builds `wat__holon__Record`).
- **Mint** **`:wat::Record::of`** (NEW, 2-arg: class keyword + struct Vec → `Value::wat__Record`). A
  stripped `eval_record_of` minus the 3rd arg + the `holon_val` branch. Register the dispatch arm + a
  checker scheme if `:wat::Record::of` had one (mirror).

### 2. Macros (`wat/Record.wat`)
- **Rename** the current macro `:wat::Record::def` → **`:wat::holon::Record::def`**. Change two things in
  its body: the recordtype parent `:wat::Record` → **`:wat::holon::Record`**, and the constructor verb
  `:wat::Record::of` → **`:wat::holon::Record::of`**. Constructor return type `-> :wat::holon::Record`.
- **Mint** **`:wat::Record::def`** (NEW, BASE) — same structure MINUS the holon_form construction:
  emit `(recordtype ~fqdn :wat::Record [names])` + a constructor calling the 2-arg **`:wat::Record::of`**
  with `(class, [struct syms])` (NO Bind/Bundle/to-holon block) + the per-field accessors (identical) +
  return type `-> :wat::Record`. is-X? is auto-minted (arc 237.6) — don't emit it.

### 3. The migration cascade (substrate-as-teacher)
Flipping `:wat::Record::def`→base breaks callers that use holon-ops on the instance. Run
`cargo test --release`; for each break, apply the rule:

> **Stays BASE** (`:wat::Record::def`, no edit) unless the record instance is fed to holon-ops
> (`:wat::holon::to-holon` / `:wat::holon::` extraction / holon auto-dispatch) → migrate that file's
> defs to **`:wat::holon::Record::def`**. Default base; holonic only by demonstrated holon-op use.

Most of the ~23 caller files silently become base and KEEP PASSING (field-access/predicate/`=`/
`same-data?`/`assoc`/`record->map` all work on base). Expected migrations: `probe_arc234_stone5_holon_auto_dispatch`
and whatever else the cascade surfaces. **The fail-count is the worklist; ride it to zero.**

## Coverage — the probe is your spec (per logic-coverage mandate)

`tests/probe_arc237_sC3_macro_split.rs` is on disk (RED now). 18 test fns covering: base ops
(construct/field/accessor/predicate/`=`/`same-data?`/`assoc`/**to-holon→error**), holonic preserved
(**to-holon→ok**), the **Liskov accept×3 / reject×1** (base-defined rejected at a `:wat::holon::Record`
param — the static proof), cross-flavor (`same-data?`→true / `=`→false). Make ALL green. Do NOT weaken
a contract to pass; if a contract can't pass, STOP and surface why.

## STOP triggers (REJECTION)
1. `:wat::Record::def` building holonic (it must build BASE — the unmarked name is the cheap case).
2. Base macro emitting holon_form / Bind / Bundle (base has no holon flavor).
3. Wrong recordtype parent per flavor (breaks the Liskov contract 13).
4. Migrating a caller to holonic that doesn't use holon-ops (default is base).
5. Weakening a probe contract to force green.
6. holon-rs touched; non-obvious error (→ STOP + surface verbatim).
7. 120 min (STOP-3); 150 (STOP-4).

## Regression suite
```
cargo build --release -p wat                                       # 0 errors
cargo test --release --test probe_arc237_sC3_macro_split           # 18/18 (was RED)
cargo test --release --lib -p wat                                  # >= 834, 0 failed
cargo test --release --test probe_arc238_eq_completeness           # 8/8
cargo test --release --test probe_arc237_sC2d_same_data            # 6/6
cargo test --release --test probe_arc227_stone2_defrecord          # 35/35 (after any holon-op migration)
cargo test --release --no-fail-fast                                # workspace clean after the cascade
```

## SCORE doc
`SCORE-STONE-S-C3.md` (NEW). Scorecard + the constructor rename/mint + the two macros + recordtype
parents + the list of caller files migrated to holonic (with the holon-op that forced each) + the
files that stayed base + cascade rounds + honest deltas + `git status --short`. DO NOT commit.

## Calibration
2 constructor changes + 2 macro changes + recordtype parents + migration cascade + 18-contract probe.
**Target band: 60–100 min Mode A; 120 STOP-3; 150 STOP-4.** Macro authorship + the cascade is the cost.
