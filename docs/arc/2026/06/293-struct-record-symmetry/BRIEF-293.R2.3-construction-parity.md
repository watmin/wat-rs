# BRIEF — 293.R2.3: construction-form parity — every type-name is its own constructor (bare `:T`), `/new` annihilated

**The work, in one paragraph.** Records construct via the bare type name (`(:Pt 1 2)` — the `defrecord` macro emits a
bare-name `defn` ctor). Structs and newtypes are the holdouts: they construct only via `:T/new`
(`register_struct_methods` mints the ctor at `format!("{}/new", struct_def.name)`, runtime.rs:953 + 999;
`register_newtype_methods` at runtime.rs:1418). This breaks "the holder is the only variance" — construction FORM
differs by holder for no reason. **Mint the struct + newtype constructor at the BARE type name** (parity with
records), **annihilate `/new`** (the builder's decided call — NOTE-base-struct-horizon: *"the name is the ctor just
like records … every type-name is its own constructor"*), and **ride the `:T/new` call-site cascade to zero**
(hard-cut; the fail-count is the meter). Accessors are untouched (struct fields via `register_aggregate_methods`;
newtype unwrap stays `:T/0`). ONLY the constructor key moves.

## The one contract decision (pinned — already decided, do NOT re-litigate)
The constructor for a struct / newtype is registered at the **bare type name** (`agg.name` / `nt_def.name`), exactly
like a record's macro-emitted ctor. `:T/new` is **dropped entirely** (not kept as an alias — hard-cut). A bare `:T`
ctor is a `Function` in `sym.functions`; the type `:T` lives in `sym.types` — different namespaces, no collision
(records already prove this: a `defn :Pt` ctor coexists with the type `:Pt`).

## Read in order (the rooms — grounded 2026-06-28)
1. **`src/runtime.rs:948-982` (`register_struct_methods`, the ctor loop)** — `constructor_path = format!("{}/new",
   struct_def.name)` (`:953`) → `let constructor_path = struct_def.name.clone();` (bare). The body
   (`struct-new` + the type keyword + params) is UNCHANGED. Also `:999` references `format!("{}/new", …)` (a second
   ctor-path use — likely the `from-map` companion or a re-register; change it to bare consistently).
2. **`src/runtime.rs:1415-1440ish (`register_newtype_methods`, the ctor)** — `constructor_path = format!("{}/new",
   nt_def.name)` (`:1418`) → bare `nt_def.name.clone()`. The newtype unwrap accessor (`:T/0`) is UNCHANGED.
3. **`src/types.rs` / docs** — `///` doc-comments saying "constructor at `:T/new`" (runtime.rs:861, 949, 1389, 2220)
   → update to "bare `:T`" (stale-doc; amend-with-recognition).
4. **The `:T/new` CALL-SITE cascade** — hard-cut the minting (steps 1-2), then `cargo build --release -p wat` +
   `cargo nextest run --release` surface every broken `:T/new` (resolve/UnknownFunction). Fix each `(:X/new args…)` →
   `(:X args…)`. The corpus: `.wat` in `wat/` (universe — e.g. `:wat::spawn::Launched/new` at spawn.wat:251/272,
   plus services/edn/etc.) + `.wat` test fixtures + `.rs` wat-in-string fixtures. **Prefer a fix-wat codemod for the
   `.wat` bulk** (a `wat-scripts/fixes/` form-aware transform: a call head `:X/new` whose `X` is a struct/newtype
   → drop `/new`; model on the existing `wat-scripts/fixes/rename-*` codemods + `wat-grep`), and hand-substitute the
   `.rs` fixtures (fix-wat can't reach them). Judge per the toolkit (a guarded codemod the headless caller can't
   misuse > hand-editing 40 sites; but a codemod only if the `:X/new`→`:X` pattern is cleanly form-detectable).

## STOP triggers (halt + surface — do NOT improvise)
- **STOP-1 (a non-ctor `/new`):** if `/new` appears as something OTHER than a struct/newtype constructor call —
  a method literally named `new`, a record field `new`, an enum variant — do NOT blanket-rename it. Migrate ONLY
  struct/newtype constructor call heads. If you can't form-distinguish them, STOP and report the ambiguous sites.
- **STOP-2 (a bare `:T` ctor collides):** if minting the ctor at bare `:T` hits a `DuplicateDefine` (something else
  already holds `:T` in `sym.functions`) — STOP and report; records don't collide, so a struct that does is a real
  finding.
- **STOP-3 (the cascade is unbounded):** if the `:T/new` call set is far larger than ~the spawn/services sites and
  spreads incoherently, STOP, report the count + which files are migrated, so the orchestrator can split it.

## The gate (orchestrator re-runs forced-clean)
- `cargo build --release -p wat` → clean.
- **R2.3 probe GREEN (un-ignore):** `cargo nextest run --release -E 'test(construction_form_parity_bare_ctor_for_struct_and_newtype)'` → PASS, `(:b::probe)` = **41**.
- `grep -rn '/new' src/runtime.rs` shows no `format!("{}/new"` ctor minting (the docs may still mention it until step 3).
- Whole workspace: `cargo nextest run --release` → floor 0, SET-diff ∅ vs HEAD (the un-ignored probe = +1 pass / −1
  skip → `4099 passed / 0 failed / 92 skipped`). Oracle: `-E 'test(defstruct) + test(newtype) + test(core_record_def) + binary(types) + binary(services)'` green (services constructs `Launched`/`Bound` via the migrated bare ctors).

## You are a LEAF
Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject `.claude/worktrees/`. Do NOT spawn subagents. Do NOT
commit. Build incrementally; ride the cascade to zero (the fail-count is the meter). Read every diff. Trust only
forced-clean builds. STOP + report if a STOP fires.
