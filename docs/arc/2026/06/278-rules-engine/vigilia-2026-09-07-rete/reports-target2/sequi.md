## SEQUI — Cast Report (TARGET 2 — the compile side and its spec) — **CLEAN**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## SCOPE

**Chains followed, end to end:**

1. **The compile chain (wat)** — `wat/rete/compile.wat`, the whole file (1163 lines, read in full): `CompileState`/`CondFoldAcc` threaded through `find-or-mint-alpha` → `find-or-mint-root-join` → `find-or-mint-hash-join` → `mint-leaf-alphas` → `compile-condition` (all six branches: `:or`, `:and`, `:where`, `:not`, `:exists`, accumulate, plain alpha+join) → `compile-rule` → `compile-query` → `compile-all` → `arm-session`. Verified: every mint/wire step returns an updated `CompileState`/`CondFoldAcc`, `:or`-arms correctly carry `state` forward across arms while resetting only `parent-ids` to `incoming`, `compile-rule`/`compile-query` both correctly thread `next-id`/`dedup`/`network` end to end (the duplication between them, rowed 2S3, does not break either thread).
2. **The wat accumulator library** — `wat/rete/acc.wat` (full) and `wat/rete/factbag.wat` (full): every fn is a textbook pure `foldl`, no `set!`/`Atom`/`defglobal` anywhere in the wat corpus in scope.
3. **The Rust lowering pipeline** — `expr_ir::lower` → `lower_expr`/`lower_list`/`lower_let`/`lower_match`/`lower_fn`/`lower_construct` (`expr_ir/mod.rs:255-1113`), all threading `&mut LowerCx` explicitly as a parameter (never a global); `lower_in_frame` (`mod.rs:292`) explicitly takes `&mut HashMap<String,u16>` + `&mut u16` and hands the counter/map back *"WHATEVER the outcome"* (`mod.rs:305-308`) — the honest shape even on the error path.
4. **`compiled_cond::compile_one` → `CompiledCond::from_parts`** (`compiled_cond.rs:257-428, 450-509`): `AlphaCompileCx`, `scope`, `field_slots`, `order`, `ops` all passed as explicit `&mut` parameters; `from_parts` (192-277) is a pure constructor that derives `has_seed_cmp` from `ops` rather than trusting a caller-supplied bool.
5. **`compiled_rhs::compile_rhs`** (`compiled_rhs.rs:137-210`): pure `Result<Option<CompiledRhs>>`, local `Vec<RhsOp>` accumulator, no shared state.
6. **`validate/mod.rs` walk** (`:108-189`): `validate_rete_rules` → `walk_for_make_rule`/`walk_for_make_query` → `validate_rule_when_and_reorder_then`/`validate_query_when`, threading `residue: &mut [WatAST]` (in-place rewrite, visible in signature) and `errors: &mut Vec<ReteCheckError>` explicitly at every level.
7. **The four-axis fence's classifiers** (`purity.rs:1610-1644`): `is_pure_expr`/`is_deterministic_expr`/`is_total_expr`/`is_rete_primitive_expr`/`find_axis_violation` each construct a **fresh** `&mut HashSet::new()` for `seen` per call (doc at `:1610`: *"fresh `seen` per call"*) — cycle-detection state scoped correctly, never leaked across calls.

**Hidden-state hunt (commands run against the 20 Rust + 5 wat files in scope):**
```
grep -n "set!"                        (5 wat files)   → 0 hits
grep -n "Atom"                        (5 wat files)   → 0 hits
grep -n "defglobal|:wat::core::var\b" (5 wat files)   → 0 hits
grep -n "static |lazy_static|OnceLock|OnceCell" (20 .rs) → 25 hits, all &'static str / OnceLock memo tables
grep -n "thread_local"                (20 .rs)        → 1 hit (eval.rs:87)
grep -n "Mutex|RwLock"                (20 .rs)        → 0 hits
grep -n "Atomic"                      (20 .rs)        → 0 hits
grep -n "RefCell|Cell<"               (20 .rs)        → 1 site (EXEC_ARENA)
grep -n "unsafe"                      (20 .rs)        → 0 hits
grep -rn "rune:sequi"                 (25 files)      → exactly the 2 handed-down sites
```

Every `OnceLock` found besides the two `rune:sequi` sites (`vocabulary.rs:1543 BY_NAME`, `compiled_cond.rs:1230/1255` — `#[cfg(test)]` only, `step_payload.rs:272/293`, `export.rs:186`, `purity.rs:159/1991`) is the same shape: a table **memoized once from a compile-time constant** (`RETE_OPS`, `DERIVATION_STEP_FIELDS`, `EXPORT_FIELDS`, `AXIS_VIOLATION_FIELDS`, `Axis::ALL`) — deterministic, read-only after first call, carrying no per-request/per-rule domain data. None coordinates a chain; they are interned constants, not state a caller relies on being threaded.

## Adjudicating the two handed `rune:sequi` sites

- **`expr_ir/eval.rs:85-89`, `EXEC_ARENA` thread-local `RefCell<ExecArena>`.** Read `with_exec_frame` in full (`:91-150`). The arena is zeroed (`*slot = None`) over exactly `[0,len)` on every entry before `f` runs, and a live nested borrow falls back to a fresh heap `Vec` (the documented `Err(_)` arm) rather than aliasing. No value survives from one `exec_value`/`exec_where` call into an unrelated one — it is a reused allocation, not accumulated domain data. **Not hidden domain state.** ⚠ The category label (`ambient-context`) undersells what it actually is — the doc's own words (*"reused frame buffer so … do not allocate per token"*) describe a `performance-counter`-shaped case, which the taxonomy asks to cite a measurement for and this rune doesn't — but that is a labeling/taxonomy nit, **not a composition break**: there is no state here the chain silently depends on across calls.
- **`expr_ir/eval.rs:896-900`, `KINDS` `OnceLock<Vec<OpExec>>`.** Purely `RETE_OPS.iter().map(...).collect()` — a static, compile-time-constant-derived interned table, indexed but never mutated after first init. Its own comment is accurate: *"opcode table interned once; not fire-domain state."* **Not hidden domain state; category and reason both correct.**

Neither site is domain state wearing a rune it shouldn't wear (unlike the `ARM_TABLE` finding on target 1). Both stand.

## Instrument noted, not flagged

`reachability.rs` (`:1-2180`, `#[cfg(test)]`-gated via `mod.rs:86-89`) holds ledger state as a committed disconfirming-probe instrument, exactly as its header states. Grepped it separately for `static`/`RefCell`/`Mutex`/`OnceLock` — every hit is a `&'static str` lifetime annotation, not a mutable global. Not a chain-state finding.

## Prior art — not re-reported

Confirmed still standing and left alone: `matcher.rs`'s `sym: Option<&SymbolTable>` (2P3, with the `eval_insert.rs:290` `None`-call correction verified), `CmpKind`'s dual dispatch table (2S1), `export.rs` pack/unpack asymmetry (2S2), `compile-rule`/`compile-query` duplication (2S3 — checked its state thread is honest in both copies), `where_tree.rs`'s `proven`-doc claim (2T1), and the `clause.rs`/`purity.rs:1513`/`AxisViolation`/`vocabulary.rs`/`compiled_cond.rs` rows.

## CLEAN

Zero thread breaks found. Zero hidden-domain-state sites found (0 globals/statics/atomics/`Arc<Mutex>` carrying domain data; the one `thread_local!` and the memoized `OnceLock` tables are all read-only derived-once caches). Two `rune:sequi` sites adjudicated and both stand.
