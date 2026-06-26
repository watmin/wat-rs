# BRIEF — arc 293.2-rename: `Record::def` → `defrecord` (the aggregate trio reaches final names)

**You are a LEAF executor. Model: sonnet. Work ONLY in `/home/watmin/work/holon/wat-rs/`. Do NOT spawn
subagents, no git worktrees, do NOT commit.** If the work exceeds these rooms or hits a STOP trigger, STOP and
report — do not improvise. **TRUST ONLY FORCED CLEAN BUILDS** (`cargo clean -p wat && cargo build --release -p wat`)
before claiming green — incremental builds + rust-analyzer lag emit stale `E0xxx`. Read the disk, not the cache.
**After editing any `wat/*.wat`, `touch tests/test.rs`** (wat-tests re-scan on `.rs` recompile).

## The work, in one paragraph

The aggregate trio is `defstruct` / `defrecord` / `holon::defrecord` — all thin macros over the
`structtype` / `recordtype` primitives (DESIGN §, R2). `defstruct` already wears its final name (293.2-parity).
The two record macros still wear OLD heads; this strike renames **only the macro head**:
- `:wat::Record::def` → **`:wat::core::defrecord`** (95 files)
- `:wat::holon::Record::def` → **`:wat::holon::defrecord`** (15 files; a RECLAIMED name — it was hard-cut at
  Stone 234.6, reclaimed here per "we reserve names for ourselves before we need them")

**SURGICAL — only `::def` moves.** The sibling names MUST survive untouched: `:wat::Record::of` (the ctor
primitive, 5 files), `:wat::Record/field-at` (the accessor, 6 files), and **`:wat::Record` (the holder TYPE /
lattice root, everywhere)**. The `fix::rename-keyword-prefix` verb is boundary-aware and nothing else begins with
`:wat::Record::def`, so the prefix match is exact — but you must NOT use `:wat::Record` as a prefix (that would
eat the type). Use the FULL old name `:wat::Record::def` as the prefix.

## THE GATE = the committed RED probe goes GREEN (un-ignore it)

`tests/types/probe_arc293_defrecord_rename.rs` (committed `2bd5f07f`, both `#[ignore]`'d, verified RED):
- `core_defrecord_is_the_record_decl_head` — `:wat::core::defrecord` registers a core record → is_ok
- `holon_defrecord_is_the_holon_record_decl_head` — `:wat::holon::defrecord` registers a holon record → is_ok
**REMOVE the two `#[ignore]` lines** when they pass.

## Decisions pinned (do NOT re-litigate)
- **Pure rename, behavior-preserving.** No emission/ctor/accessor logic changes — only the macro head keyword.
- **Reclaim `:wat::holon::defrecord`.** It was retired (234.6); it is now the canonical holon record head. No
  conflicting retirement-table entry exists today (verified — `retirement.rs` has none for it).
- **Old heads become RETIRED.** Add two `RETIREMENT_TABLE` entries so `:wat::Record::def` / `:wat::holon::Record::def`
  throw the teaching remedy pointing at the new heads (substrate-forces-idealized-state; nothing drifts silently).

## Rooms — read in order (re-ground before editing)

### A. The macro DEFINITIONS (the heads themselves)
1. **`wat/Record.wat:91`** — `(:wat::core::defmacro :wat::Record::def …)` → rename head to `:wat::core::defrecord`.
2. **`wat/Record.wat:166`** — `(:wat::core::defmacro :wat::holon::Record::def …)` → `:wat::holon::defrecord`.

### B. The `.wat` call sites + macro-body EMISSIONS (fix-wat — use the tool, do NOT hand-edit)
3. **`wat/core.wat:387-388`** — the KWARGS macro EMITS `:wat::Record::def` in its body (`record-def
   ` + "`(:wat::Record::def ~kwargs-ty ~kw-argvec)`"). This is an emission site, not a call — fix-wat's
   text rename catches it (it is the literal keyword in source). VERIFY it became `:wat::core::defrecord`.
4. **All other `.wat` sites** (~88 files across `wat/`, `tests/`, `wat-tests/`): drive with a **fix-wat codemod**.
   Write `wat-scripts/fixes/rename-record-def-to-defrecord.wat` modeled EXACTLY on the existing
   `wat-scripts/fixes/rename-kernel-to-spawn.wat` (read it — same `:user::migrate` / `:user::apply-each` /
   `:user::main` shape). Two `rename-keyword-prefix` calls, full-old-name prefixes:
   ```clojure
   (:wat::fix::rename-keyword-prefix ":wat::holon::Record::def" ":wat::holon::defrecord"
     (:wat::fix::rename-keyword-prefix ":wat::Record::def" ":wat::core::defrecord"
       src))
   ```
   ⚠ ORDER MATTERS: do `:wat::holon::Record::def` FIRST. `:wat::Record::def` is NOT a prefix of
   `:wat::holon::Record::def` (different namespace), so they are actually disjoint — but run holon-first anyway to
   be safe. Run it: `printf '[<every .wat path with Record::def>]' | ./target/release/wat ./wat-scripts/fixes/rename-record-def-to-defrecord.wat`
   (get the path list from `grep -rln ':wat::Record::def\|:wat::holon::Record::def' wat/ tests/ wat-tests/ --include='*.wat'`).
   The codemod is idempotent (re-run = 0 changes). Delete neither the codemod nor leave it dangling — it stays in
   `wat-scripts/fixes/` as a recorded migration (like the others there).

### C. The `.rs` wat-in-string fixtures (fix-wat CANNOT reach these — hand-substitute)
5. **`src/rete/kernel.rs`** (~10 sites, e.g. `:2016-2018, :2256, :2345-2346, :2430-2431, :2503-2504`) — wat source
   embedded in Rust string literals (`"(:wat::Record::def :weather::Temperature …)\n\"`). fix-wat operates on
   `.wat` files only, so these need a literal text substitution `:wat::Record::def` → `:wat::core::defrecord`
   (and any `:wat::holon::Record::def` → `:wat::holon::defrecord`). `grep -rln ':wat::Record::def\|:wat::holon::Record::def' src/ --include='*.rs'` for the full `.rs` set (~81 files — most are test fixtures with embedded wat). Use an
   unambiguous full-string substitution; the old name is fully-qualified so there is no false-match risk.

### D. Retirement table + stale doc comments
6. **`src/remedy/retirement.rs:69`** — append two entries to `RETIREMENT_TABLE`:
   ```rust
   // Arc 293.2-rename — defrecord replaces Record::def (the aggregate trio's final names).
   RetirementEntry { retired: ":wat::Record::def",        replacement: ":wat::core::defrecord",  note: None },
   RetirementEntry { retired: ":wat::holon::Record::def", replacement: ":wat::holon::defrecord", note: None },
   ```
   Add matching rows to the module's doc-comment table (`:40-48` region) for the human reader.
7. **`src/stdlib.rs:37,94,107,114,331`** — comments saying "uses `:wat::Record::def`" — update to the new head
   (stale-doc; the loads themselves are unaffected — `Record.wat` still loads at the same point).

## STOP triggers (halt + report; do NOT improvise)
1. **STOP** if fix-wat's rename touches `:wat::Record::of`, `:wat::Record/field-at`, or the bare `:wat::Record`
   type (it must NOT — the prefix is the full `::def` name). Grep after the codemod to PROVE the siblings survive:
   `grep -rn ':wat::Record::of\|:wat::Record/field-at' wat/` must still show them present.
2. **STOP** if the kwargs macro emission (`core.wat:387`) does NOT get renamed by the codemod — report; that
   emission MUST become `:wat::core::defrecord` or kwargs records break.
3. **STOP** if a `.rs` fixture site is NOT a wat-in-string literal (e.g. an actual Rust identifier referencing the
   head) — report it; do not blind-substitute Rust code.
4. **STOP** if reclaiming `:wat::holon::defrecord` collides with a live registration (a `DuplicateDefine` or an
   existing recognition) — report; recon says clean, but verify.
5. You are a LEAF. No subagents. If the cascade exceeds these rooms, STOP and report.

## Gate (the orchestrator re-runs every line AFTER a forced clean build)
| what | command | expected |
|---|---|---|
| forced clean build | `cargo clean -p wat && cargo build --release -p wat` | clean (no `error[E…]`) |
| **the rename probe goes green** | `cargo nextest run --release -p wat -E 'binary(types) & test(defrecord_rename)'` (after removing the 2 `#[ignore]`) | **2 passed** |
| siblings survive (ctor + accessor + type) | `cargo nextest run --release -p wat -E 'binary(types) & test(holder_substitution)' -E 'binary(types) & test(record_surface)' -E 'binary(types) & test(holder_bound)'` | all green |
| records still construct/accessor/EDN/holon | `cargo nextest run --release -p wat -E 'test(core_record_def)'` | green |
| kwargs records still work | `cargo nextest run --release -p wat -E 'test(kwargs)'` | green |
| retirement remedy fires | `cargo nextest run --release -p wat -E 'binary(remedy) + package(wat) & test(retire)'` (or the retirement.rs unit tests) | green |
| no new regressions | `cargo nextest run --release -p wat`, failing-test SET vs HEAD (`2bd5f07f`; floor = 0 deterministic) | **∅ new** |

## Report back
Full `git diff --stat`; the fix-wat codemod path + the path-list you fed it; verbatim gate output (forced-clean
run); the grep PROOF that `:wat::Record::of` / `field-at` / the `:wat::Record` type survive; the failing SET if any;
whether any STOP fired. Do NOT commit.

Runtime: 60–120 min (wide mechanical cascade — the fail-count is the progress meter). Trap-doors: (a) the
type/ctor/accessor siblings (STOP-1 — prove they survive); (b) the kwargs macro-body emission (STOP-2); (c) the
`.rs` wat-in-string fixtures fix-wat can't reach (room C, hand-substitute); (d) cache whipsaw (forced clean build
only). Pure rename — the SET-diff ∅ + the probe green are the truth oracle.
