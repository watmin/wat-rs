# BRIEF — arc 293 unify-2b FIX: Holder rename + `parent` restored + nominal same-kind extension

**You are a LEAF executor. Model: sonnet. Work ONLY in `/home/watmin/work/holon/wat-rs/`. Do NOT spawn
subagents, no git worktrees, do NOT commit.** The broken unify-2b merge is ALREADY in the working tree
(uncommitted). You are PATCHING it — not starting over. **TRUST ONLY FORCED CLEAN BUILDS**
(`cargo clean -p wat && cargo build --release -p wat`) before claiming green — incremental builds + rust-analyzer
lag produce STALE `E0xxx` diagnostics; read the disk, not the cache.

## Context — what's in the tree + what's wrong

unify-2b merged `StructDef`+`RecordDef` → `AggregateDef { name, type_params, fields, kind: AggregateKind,
restrictions }` and collapsed `TypeDef::Struct`+`Record` → `TypeDef::Aggregate`. The kind-merge is RIGHT and STAYS.
**Two things are wrong, and they're the whole fix:**
1. `parse_recordtype` (`src/types.rs` ~`:2120-2135`) maps the parent arg to a kind via a hardcoded **2-element
   whitelist** (`:wat::Record`→`Record`, `:wat::holon::Record`→`HolonRecord`, **else → `Err`**). This REJECTS
   `:wat::program::Env` — a valid core-record base — and breaks record extension. (RED: `probe_arc258_program_env_record::c02_user_extends_program_env`.)
2. The merge **dropped `parent`** (derived it from kind). But `parent` is a real lattice edge that can point at an
   **extensible base** (`program::Env`), not just a root — so it must come back as a stored field.

The model (DESIGN § THE HOLDER × SURFACE MODEL): **extension is nominal, same-kind.** A record's `holder` = the
holder of its parent's **root** (`program::Env` roots at `:wat::Record` → `Record`). You never extend across the
trit; holon data enters as a *field*, not via inheritance. The holder-lattice Liskov rule is already PROVEN and
must STAY green (`tests/probe_arc293_holder_substitution.rs`).

## The fix (do exactly these; ride the compile cascade to zero)

1. **Rename `AggregateKind` → `Holder`** and the field/locals `kind` → `holder` (the aggregate's kind only —
   NOT `EnumVariant` kinds or unrelated `kind` fields). Mechanical sweep across the merge sites. `is_portable =
   holder != Holder::Struct` (unchanged logic, renamed).
2. **Add `parent: String` to `AggregateDef`** (`src/types.rs`). It is the base edge.
3. **`parse_recordtype`** (`src/types.rs` ~`:2120`): **DELETE the `else => Err` rejection.** Accept ANY parent.
   Set `holder = if parent == ":wat::holon::Record" { Holder::HolonRecord } else { Holder::Record }` (the common,
   correct case: `program::Env` is not the holon root → `Record`). Set `parent` = the parent arg. **STOP-1** if you
   find a test that extends a *holon* base (a non-`:wat::holon::Record` parent that should yield `HolonRecord`) —
   report it; the simple direct-parent check would mis-classify it and you'd need the root-walk instead.
4. **`parse_defstruct`** (`src/types/defstruct.rs`) + the ~15 built-in struct registrations + the root-type
   registrations (`:wat::Record` / `:wat::holon::Record` / etc.): set `holder: Holder::Struct` (already) **and
   `parent: ":wat::core::Value".to_string()`** (structs/roots sit directly under the Value top).
5. **Lattice registration** (`src/types.rs` ~`:424`, the `register_subtype` call): use the stored `parent`, and
   **skip when `parent == ":wat::core::Value"`** (Value-top is a rule, not a registered edge — registering it
   would be redundant/wrong). For records, `register_subtype(name, parent)`.
6. **`src/closure_extract.rs` ~`:2408`** (the old `r.parent` read): use the stored `parent` directly (it's a field again).

## STOP triggers
1. **STOP-1** (a holon-base extension — see step 3): report; do not guess the root-walk.
2. **STOP** if a `parent: ":wat::core::Value"` struct edge being skipped breaks a struct subtype test — report (the
   Value-top rule should already cover struct `<: Value`).
3. **STOP** if the cascade spreads past the merge sites + the 2 read sites into unrelated logic.
4. You are a LEAF. No subagents.

## Gate (orchestrator re-runs each AFTER a forced clean build)
| what | command | expected |
|---|---|---|
| forced clean build | `cargo clean -p wat && cargo build --release -p wat` | clean (no `error[E…]`) |
| **the regression fixed** | `cargo test --release -p wat --test probe_arc258_program_env_record -- c02_user_extends_program_env` | **GREEN** |
| **the proof stays** | `cargo test --release -p wat --test probe_arc293_holder_substitution` | 5 passed |
| surfaces + records + structs | `cargo test --release -p wat --test probe_arc293_record_surface --test probe_arc293_structural_surface --test probe_arc293_structtype_primitive` + `--test test -- core_record_def defstruct` | green |
| services (struct State + record wire) | `cargo test --release -p wat --test test -- counter_on seeded admin_stop hibernate_resume` | green |
| no new regressions | `cargo test -p wat --no-fail-fast`, failing-test SET vs HEAD (`15157c3d` code; floor ≈ 201-204 nondeterministic arc-170 leak class) | **∅ new** (weigh by SET; isolate + baseline-check any deterministic-named suspect) |

## Report back
Full `git diff --stat`; verbatim gate output (from the forced-clean-build run); the failing SET (sorted); whether
STOP-1 fired; any site where `holder`/`parent` handling was non-obvious. Do NOT commit.
