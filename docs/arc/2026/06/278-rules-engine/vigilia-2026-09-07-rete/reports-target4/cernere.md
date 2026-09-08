# CERNERE — Cast Report (TARGET 4 — the test corpus) — **FINDINGS**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> ⛔ **ORCHESTRATOR NOTE, ADDED AT RECORDING TIME:** this report's FORM-LEVEL claims all verify.
> Its LINE NUMBERS for the finding do not — the cited file is **40 lines** and the report cites
> `:542-543`, `:527-535`, `:537-545`. Corrected coordinates are in `FINDINGS.md`.

**Target:** `tests/rete/` (144 `.wat`, 19 `.wat.bad`, 23 `.edn`, 100 `.rs`) + `src/rete/kernel/tests/` (20 `.rs`). Re-derived: 144/19/23/100/20, 38,058 total lines, 0 `rune:cernere` in target — all match exactly.

**Spec sources consulted:** `wat/Record.wat` (defrecord), `wat/rete/syntax.wat` (defrule/defquery/query/cond), `wat/rete.wat` (Rule/Query/Session/DerivationNode/DerivationStep/explain), `wat/rete/compile.wat` (compile/compile-all), `wat/rete/oracle/{insert,fire,explain}.wat` (InsertOutcome/FireOutcome/insert-all/fire-once/fire-rules-explain), `wat/core.wat` (defclause), `src/rete/vocabulary.rs` (`RETE_OPS` table), `src/rete/clause.rs:376-382` (`where`/`not`/`exists`), `src/types.rs` (defenum/defsurface), `src/check.rs` (collect-rules, assertion-failed!, edn::write, aggregate-new), `src/runtime.rs`, `src/assertion.rs`, `src/edn_shim.rs`, `src/rete/export.rs`, `wat-scripts/fixes/rete-oracle-sigil.wat` and `type-query-to-defquery.wat` (retirement records for the `$oracle` sigil migration and the `query-by-type-string`→`defquery` migration).

**Method:** For `datamancer.src.wat` and all 19 `.wat.bad` I read every line and traced every `:wat::`-headed form individually against a definition site (`defmacro`/`defn`/`defrecord` in `wat/*.wat`, or a literal-string dispatch arm in `src/*.rs`). For the wider 143 driven `.wat` I did not re-trace every form but ran a targeted sweep for the specific retired spellings named in the two retirement codemods (`insert-spec`, `insert-all-spec`, primed `fire-once'`/`fire-rules'`, `query-by-type-string`) across all 144+19 files — code position only.

### `datamancer.src.wat` — traced in full, 0 phantoms

Every one of its ~30 distinct forms traces cleanly:
- `defrecord`, `defn`, `PersistentVector`, `match`, `let`, `length`, `string::length`, `string::concat`, `None` — core forms.
- `defrule`, `defquery`, `where`, `not`, `exists` — `wat/rete/syntax.wat:202,251`; `src/rete/clause.rs:376-382`.
- `Rule`, `Query`, `Session` — `wat/rete.wat:48,55,199`.
- `insert` (variadic) — `src/rete/purity.rs:2429` + dispatch at `src/runtime.rs:5381`.
- `compile-all`, `CompileOutcome::{Compiled,MayNotTerminate}` — `wat/rete/compile.wat:1138,1127-1141`.
- `fire-rules`, `FireOutcome::{Fired,MemoryCeilingExceeded,RoundCapExceeded}` — `wat/rete/oracle/fire.wat:587,170,278,282`.
- `query` (macro), `InsertOutcome::{Inserted,MemoryCeilingExceeded}` — `wat/rete/syntax.wat:88`; `wat/rete/oracle/insert.wat:26,54`.
- `export`, `edn::write`, `edn::write-pretty` — `src/rete/export.rs:142`; `src/edn_shim.rs:70,83`.
- `kernel::assertion-failed!` — `src/assertion.rs:116`. `io::write-file` — `wat/io.wat`.

No rune present, none needed. This is the file "nothing checks" (unreached by any probe, per its own line 135), and it holds up.

### The `.wat.bad` question — FINDING: two retired forms in one fixture

`tests/rete/probe_constructor_meta_surface_total_enum.wat.bad` calls:
- **`(:wat::rete::fire-rules-spec session)`**
- **`(:wat::rete::query-by-type-string fired "cg::Wrap")`**

Both are **retired**, not merely undocumented:
- `:wat::rete::fire-rules-spec` is the explicit LHS of a rename in `wat-scripts/fixes/rete-oracle-sigil.wat:4` (`:wat::rete::fire-rules-spec -> :wat::rete::fire-rules$oracle`), whose own body carries `;; rune:lint(rete-name-unminted) :wat::rete::fire-rules-spec — pre-$oracle spelling of the wat reference impl, retired by the very rewrite recorded below` (line 45).
- `:wat::rete::query-by-type-string` is named "retired" explicitly in `tests/rete/probe_arc278_query_type_safe.rs:6`, and is the exact target of the `wat-scripts/fixes/type-query-to-defquery.wat` codemod (its own `rune:lint(rete-name-unminted)` at line 74).
- Neither string appears anywhere in `src/*.rs` as a dispatch arm — they are not aliases still wired to something; they are dead.

I checked reachability against the driving test, `tests/rete/probe_constructor_meta_surface_audit.rs::nested_surface_enum_variant_wrong_arity_is_now_a_check_time_rejection`: it does not use bare `is_err()` — it asserts the error message contains `"RhsArityMismatch"`, the rule name `cg::gather`, the variant `cg::Status::Active`, and the arity numbers. `:user::run` (containing the two phantom forms) is never called by this test — only `startup_from_file` is invoked. This matches the file's own design idiom: a body is written but structured to be unreachable *by the check that actually governs the fixture*, so the retired forms sit inert rather than proven — the exact "unreached, unvalidated" shape cernere flags.

Grepped both occurrences across the full 144+19 corpus — isolated to this one file, both lines. No other `.wat`/`.wat.bad` in the target uses either retired spelling.

None of the other 18 `.wat.bad` fixtures use a phantom or retired form — I traced every non-boilerplate form they introduce (`defenum` → `src/types.rs:3996,4128`; `defsurface` → `src/types.rs:4006,4138`; `defclause` → `wat/core.wat:58`; `collect-rules` → `src/check.rs:21414`; `aggregate-new` → `src/check.rs:4696`; `enum::not=` → `src/rete/vocabulary.rs:1183`; `i64::>`/`i64::-` rete aliases → `src/rete/vocabulary.rs:314,435`) — all real, all trace.

### What I looked for and did NOT find

- Bare `:wat::rete::fire-once` (7 uses) — I initially suspected this was the retired pre-migration oracle spelling per the codemod header. Checked `src/runtime.rs:5664` (`":wat::rete::fire-once$native" | ":wat::rete::fire-once" =>`) and `wat/rete/oracle/fire.wat:189` — bare `fire-once` is the CURRENT native spelling post-migration (the codemod reassigned the bare name's meaning, it did not retire the spelling). **Not a phantom.**
- The `:undefined` keyword literal used as an extra argument in two `.wat.bad` fixtures — a keyword VALUE in argument position, not a head; not a form at all, so out of cernere's scope.
- Swept all 144 `.wat` + 19 `.wat.bad` for every other retired dual-impl spelling named in `rete-oracle-sigil.wat`'s rename table (`insert-spec`, `insert-all-spec`, primed `'` forms) — zero hits anywhere in the target.
- `:wat::rete::compile` (bare, not `-all`) — real, `wat/rete/compile.wat:1128`.

**FINDINGS**
