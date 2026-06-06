# Arc 251 — The Great Migration: SCOUT LIFT MAP

**Status:** scouting in progress (2026-06-06). Six parallel scouts dispatched to map
what lifts from the flat monoliths (`runtime.rs` 31,328 lines, `check.rs` 19,126) +
the mid/small flat files into warded homes. Each scout's raw findings are recorded
below as it returns (compaction-safe); the SYNTHESIS section at the end is written
once the fleet is home.

**Classification legend** (per the clojure-ination — arc 251):
- **LIFTS** — stable; ward into a home as-is.
- **TRANSFORMS** — changes shape when types become forms / keyword→symbol; warding now
  risks ward-then-rewrite. Lift with an inscribed `// TRANSFORMS` marker, or sequence
  after the clojure-ination touches it.
- **DIES** — retired by the clojure-ination (the TypeScheme / scheme machinery).

---

## Scout 1 — runtime.rs CORE DATA MODEL → proposed home `src/value/`

**Proposed home:** a single `src/value/` home (cohesion too tight to split into
value/ + error/ — `RuntimeErrorKind` carries `ValueSnapshot`, `EvalBreak` wraps
`RuntimeError`). Submodules:
- `value.rs` — `Value` enum + impls (PartialEq/Eq/Hash/`impl Value`) + payloads
  (`StructValue`, `EnumValue`, `SpawnOutcome`, `ProgramHandleInner`, `Clause`/`ClauseSet`/
  `ClauseAttempt`/`ClauseFailureReason`) + `sequence_eq`/`hash_sequence`.
- `environment.rs` — `Environment`, `EnvBuilder`, `EnvCell`, `BoundEntry` (+ co-locate
  `Function` here: it carries `closed_env: Option<Environment>`).
- `encoding_ctx.rs` — `EncodingCtx`.
- `symbol_table.rs` — `SymbolTable` + impl.
- `tracked.rs` — `TrackedValue`, `ValueSnapshot`, `Provenance`.
- `signal.rs` — `EvalSignal`, `EvalBreak`, `RuntimeError`, `RuntimeErrorKind` + impls.
- `frame.rs` — `FrameInfo`, `FrameGuard`, `CALL_STACK`, `snapshot_call_stack` (tail ~20105).

**TRANSFORMS (clojure-ination targets):**
- `Value::type_name()` / `declared_type_name()` (impl Value block ~1260–1402) — return
  keyword-encoded strings `"wat::core::i64"`; flip to symbol-form `"wat.type/i64"`.
- `EnumValue.type_path`, `Clause`/`ClauseSet` (carry `crate::types::TypeExpr`),
  `Function` (carries `Vec<TypeExpr>`), `SymbolTable` (unit_variants/defined_values/
  binding_metadata keyed by keyword-encoded paths), `RuntimeErrorKind` (Display surface).
- Everything else (Environment, TrackedValue, EvalSignal/Break, frame, encoding_ctx,
  the PartialEq/Eq/Hash impls) = **LIFTS** (stable).

**Blast radius (external refs):** RuntimeError 567 · SymbolTable 280 · Environment 213 ·
ValueSnapshot 121 · EvalBreak 73 · Function 51 · TrackedValue 30. `lib.rs` re-exports
EncodingCtx/EnvBuilder/Environment/Function/RuntimeError/RuntimeErrorKind/StructValue/
SymbolTable/Value (re-export paths must update).

**Key entanglement:** `ValueSnapshot::of()` calls `render_value` (runtime.rs:18628) —
~80 lines of `match Value` deep in the evaluator interior. Either `render_value` moves
with `tracked.rs` or `ValueSnapshot::of` stays a shim in runtime.rs. **This is the single
most dangerous entanglement** — resolve before `ValueSnapshot` fully lives in the home.
Also: `register_defines` (2662) / `register_struct_methods` (3029) sit near the data model
but are eval-bootstrap, not types — they go with the registration concern (Scout 4), not here.

**Lift difficulty (do in this order):** frame.rs EASY · encoding_ctx.rs EASY ·
signal.rs MEDIUM · tracked.rs MEDIUM (render_value friction) · environment.rs MEDIUM
(co-locate Function) · symbol_table.rs MEDIUM (god-struct, wide one-way imports) ·
value.rs HARD (foundational; do last; tag the `impl Value` block TRANSFORMS in-source).

---

## Scout 3 — runtime.rs VALUE & COLLECTION OPS → `src/collection/` (extend) + NEW `src/scalar/` + NEW `src/algebra/`

**The collection/ home is COMPLETE & clean — no duplication.** Two-tier design:
- Tier 1 (already in `collection/eval.rs`, 1082 lines): 23 `*_inner` value-level helpers
  (operate on pre-evaluated `&Value`) for Vector/HashMap/HashSet/List length/empty?/
  contains/get/conj/assoc/dissoc/keys/values/concat; `infer.rs` (4 intrinsic inferences);
  `transform.rs` (15 Vector ops: map/filter/foldl/foldr/sort/reverse/range/take/drop/zip/window/…).
- Tier 2 (still in runtime.rs, ~295 lines "thin fanout"): `eval_length`/`eval_empty`/
  `eval_contains`/`eval_conj`/`eval_get`/`eval_assoc` (~13907–14088), `dispatch_substrate_impl`
  (10184), `eval_tuple_ctor`, `eval_positional_accessor`, `require_vec`/`require_i64` (pub(crate)).
  These route AST→`*_inner`. They can't be IN collection/ today only because they call
  `eval_inner` (runtime.rs:5213) → cycle. **Handoff is clean** (the seam is `dispatch_substrate_impl`,
  which takes pre-evaluated `&[Value]`). MEDIUM lift — move to `collection/surface.rs` once
  `eval_inner` is co-located/accessible.

**NEW `src/scalar/` home (~850 lines, EASY)** — arithmetic + conversions + bool + comparison:
- `eval_i64_arith`/`eval_f64_arith` (8289–8401), `arith_i64_i64_inner`/`arith_f64_f64_inner`
  (10322–10366); scalar conversions `eval_*_to_*` (8441–8860, ~15 fns); `eval_f64_round`/
  `unary`/`clamp` (8538–8669); `eval_not`/`eval_and`/`eval_or` (9514–9583); `eval_compare`/
  `eval_f64_compare` (9432–9497); `eval_math_unary`/`eval_math_pi`/`eval_stat_*` (20766–20928).
- **LIFTS** (zero TypeScheme dependency — runtime dispatch is pure keyword-string match).
- `eval_keyword_to_string`/`eval_keyword_from_string` (8780–8860) = **TRANSFORMS** (keyword→symbol).
- `values_equal` (9128–9325, ~198 lines) + `values_compare` (9354–9430) = the relational
  intrinsics; **MEDIUM** — called by `collection/eval.rs` sort + `eval_eq`/`eval_not_eq`; need a
  shared home (a `src/eq/` candidate, or co-locate in scalar/). `eval_type` (13873) LIFTS (delegates
  to `Value::declared_type_name()` — that method itself is the TRANSFORMS site, per Scout 1).

**NEW `src/algebra/` home (~3,800 lines, HARD)** — the VSA/holon `:wat::holon::*` layer, the
largest un-lifted block (15098–19876). Proposed internal split: construct.rs (Bind/Bundle/
Permute/Thermometer/Blend + from-holon/to-holon/leaf), classifier.rs (classifier-wrapped
ctors + is-X? predicates), measure.rs (cosine/presence/coincident/dot/simhash), encode.rs
(holon_encode/vector_bytes/bytes_vector/vector-algebra), memory.rs (subspace/engram/reckoner/
library), support.rs (require_holon/coerce_to_holon_ast/pair_values_to_vectors/require_encoding_ctx).
**HARD because:** `eval_form_*_coincident_q` calls `run_constrained`/`parse_and_run` (runtime
eval-loop internals — a hard inbound coupling); subspace/reckoner depend on `sigma.rs`;
`eval_algebra_bundle` needs `require_encoding_ctx` (private). The classifier ctors + is-Keyword?/
is-Symbol? = **TRANSFORMS** (keyword→symbol string topology; logic survives).

**string_ops.rs** — already lifted (flat file, 612 lines); could be PROMOTED to a home. Residual:
one inline `String/empty?` arm at runtime.rs:5864–5882 to extract. EASY.

**Load-bearing finding:** the TypeScheme "scheme" that DIES with clojure-ination lives ENTIRELY
in check.rs (consumed at check-time); the runtime eval ops are NOT entangled with it. Runtime
TRANSFORMS are limited to keyword→symbol string changes (eval_keyword_*, classifier predicates,
Value::type_name). So the runtime lift and the scheme demise are LARGELY DECOUPLED — runtime homes
can lift on the stable keyword-string dispatch; only the type-name-producing surfaces transform.

---

## Scout 5 — check.rs INFERENCE ENGINE + THE SCHEME DEMISE → check/ submodules

**Proposed check/ submodules** (extending the home's existing env.rs + error.rs):
- `check/result.rs` (~500, EASY) — `CheckResult<T>`, `InferCtx` (657–686), `Subst` alias (691),
  `UnifyError`. Pure types, no type-surface knowledge → **LIFTS**.
- `check/unify.rs` (~430, EASY) — `unify` (12471), `assignable` (12633), `walk` (12652),
  `occurs` (12860), `apply_subst`, `reduce`, `format_type`/`format_type_inner` (pub, used by
  runtime introspection). Algorithm stable → **TRANSFORMS** only in the TypeExpr variants it matches.
- `check/walk.rs` (~2,200, MEDIUM) — the AST-only validators/walkers (comm-positions, sandbox-leak,
  deadlock, restricted-call, legacy-rejection). Mostly **LIFTS** (AST-structure, no TypeExpr).
  EXCEPTION: `walk_for_bare_primitives` + `BARE_PRIMITIVES`(1637)/`BARE_CONTAINER_HEADS`(1652) →
  these go to the **resolve home (251.1c)**, NOT check/walk.rs — coordinate, don't double-lift.
- `check/scheme.rs` (~4,500, MEDIUM-to-lift but the DIES zone) — `TypeScheme` (79–87),
  `instantiate` (12875), `rename` (12899, the `:`-strip artifact), `derive_scheme_from_function`
  (13000), `register_builtins` (13020–17385, ~4,365 lines / ~215 entries).
- `check/infer.rs` (~8,800, HARD) — `infer` (3282), `infer_list` (3656–5683, the 2,027-line
  dispatch hub), all ~49 `infer_*`. C1 control-flow inferences (match/if/do/cond/let/try) =
  **LIFTS** (TypeExpr::Path string literals are sed-replaceable, not structural). C2 (infer_list
  keyword-dispatch + constructors) = **TRANSFORMS** (every `":wat::core::X"` arm string changes).
  `MatchShape` (6051) constructs `TypeExpr::Parametric{head:"wat::core::Option"}` = **TRANSFORMS**.

**★ THE SCHEME DEMISE MAP (load-bearing):** the dying core = `TypeScheme` struct +
`instantiate` + `rename` + `register_builtins`. `type_params: Vec<String>` (the ∀ quantifiers)
+ `rename`'s `Path(":T")`→Var by `:`-strip = pure keyword-encoding HM artifacts with NO analogue
in symbol-form types. **CLEAN EXCISION IS FEASIBLE:** `instantiate`/`rename` are called from
exactly TWO sites — `infer_list` generic fallback (5446) + keyword-as-value arm (3353).
`register_builtins` is one startup call populating CheckEnv → replaceable as a unit. ~80% of its
entries have `type_params: vec![]` (monotypes — survive as arity+concrete sig); the truly
polymorphic entries (Vector/list ctors, =/<, map/filter/fold, ∀I,O channels) become
form-matching intrinsics with custom `infer_*` arms post-clojure-ination.
**ORDERING VERDICT: LIFT FIRST, RETIRE SCHEME AFTER** — the ward maps the blast radius precisely;
the warding process IS the surgical excision plan; retire-then-lift edits the monolith during the
most invasive change (higher collateral risk). The `check/scheme.rs` stamp documents "warded
knowing this module is scheduled for retirement per clojure-ination."

**Blast reach into runtime:** `Function` (runtime.rs) carries `type_params`/`param_types`/
`ret_type`/`rest_param_type` — the runtime's copy of what TypeScheme reads; `derive_scheme_from_function`
bridges them. So the scheme demise touches `Function` (Scout 1's value home) too.

## Scout 4 — runtime.rs PROCESS/IO/CONCURRENCY + REGISTRATION → `src/comms/` (extend) + freeze-coupled

**Concurrency eval (~2,500 lines, 20409–22900) → extend EXISTING `src/comms/`** (add `comms/eval.rs`
or per-tier eval submodules) — comms/ already owns the mechanism (traits, channel impls); these
are the wat-surface wrappers that complete it. Families:
- A: channel/queue eval (send/recv/make/select/drop/handle-pool, ~550) → comms/eval.rs. MEDIUM.
- B: thread eval (spawn-thread/join/drain/peer-io, ~470) → comms thread tier. MEDIUM
  (`eval_kernel_spawn_thread` bridges comms+thread_io+RuntimeServices — most entangled).
- C: process-tier eval (~870; `eval_kernel_process_recv` ALONE is 432 lines) → comms process tier
  (alongside spawn_process.rs/fork.rs). HARD.
- D: DiedError value builders (~510, 22484–22990) → `comms/died_error.rs`. EASY (already pub(crate),
  consumed by fork.rs/spawn.rs).
- F: ambient/signal/config eval (~215, 20193–20408) → lift WITH the process-wide statics
  (KERNEL_STOPPED/ARGV/SHUTDOWN_RX, lines 67–233) into a kernel-state home, or leave in place. EASY.
- IO eval (println/readln/eprintln) = ALREADY extracted (thread_io.rs). No action.

**★ REGISTRATION PIPELINE — the dependency-ordering crux.** `register_defines` (2665),
`register_stdlib_defines` (2873), `register_struct/enum/newtype_methods` (3029/3174/3290),
`register_type_predicates` (3385), `register_runtime_defs` (3481), `register_defalias` (3743) —
ORCHESTRATED by `freeze.rs` (already a clean 2,079-line orchestrator) but DEFINED in runtime.rs
because they touch `SymbolTable`/`Function`/`Value`/`Environment`. The ORDER is load-bearing
(expand BEFORE register_defines; register_type_predicates AFTER newtype_methods; register_runtime_defs
AFTER check, phase-10). **HARD lift with a PREREQUISITE: cannot lift the register_* fns until the
core types (Value/Environment/SymbolTable) are in a stable home** → confirms `src/value/` lifts FIRST.

**comms/ reverse-dep:** `comms/mod.rs:22-28` references `crate::runtime::SHUTDOWN_RX` (shutdown
cascade) — extraction must keep the static accessible or move it first.

---

## ★ CROSS-SCOUT DEPENDENCY SPINE (emerging — confirm at synthesis)

Multiple scouts converge on one ordering truth: **`src/value/` (Value/Environment/SymbolTable/
RuntimeError) is the foundation everything imports.** The registration pipeline (Scout 4), the
concurrency eval wrappers (Scout 4), the collection fanout (Scout 3), and check's
`derive_scheme_from_function` (Scout 5) ALL depend on these core types. So the migration's first
move is the `value/` home; runtime sub-homes (scalar/algebra/comms-eval) and the registration lift
follow once value/ stabilizes. The scheme demise (check) is LARGELY DECOUPLED from the runtime lift
(Scout 3: TypeScheme is check-time only; runtime dispatch is keyword-string) — so check/ and
runtime/ homes can proceed on parallel tracks after value/ lands.

---

## Scout 2 — runtime.rs EVAL DISPATCH SPINE → NEW `src/eval/`

**Proposed `src/eval/` home** (the interpreter loop; clean seams to function/ + collection/):
- `dispatch.rs` (~1,750, HARD) — `eval_inner` (5213), `eval` (5377, pub), `eval_list` (5399),
  `dispatch_keyword_head` (5459), `dispatch_keyword_head_value` (5482–6638). **The seam.**
- `control.rs` (~4,300, MEDIUM) — `eval_let`/`eval_do`/`eval_if`/`eval_cond`/`eval_match`/
  `eval_quote`/`eval_quasiquote` + the defclause cluster (`parse_defclause_form` pub,
  `is_defclause_form` pub — both consumed by check.rs; `eval_call_to_defclause`,
  `value_matches_type_by_name`, `val_type_path`). Physically fragmented (7518, 10514, 13282) but
  semantically cohesive. `synthesize_fn_body` pub(crate) also used at defn-parse sites — re-export.
- `apply.rs` (~600, EASY — cleanest seam) — `eval_tail`/`emit_tail_call`/`*_tail` trampoline,
  `apply_function` (pub; callers: hologram/sigma/test_runner/freeze), `apply_value`,
  `FrameGuard`/`CALL_STACK`/`snapshot_call_stack` (pub). **LIFTS** — TCO machinery clojure-stable.
- `step.rs` (~1,750, MEDIUM) — the `:wat::eval-step!` small-step machine (`eval_form_ast`/
  `eval_form_step`/`eval_walk`/`step_*`/`substitute`). **LIFTS — NOT a DIES candidate**
  (actively live; Phase-3 symbol-head rules noted as future work; BOOK ch.59 cache depends on it).
  `eval_form_edn/file/digest/signed` are load-pipeline-adjacent → maybe `eval/load.rs` or stay.

**★ THE DISPATCH SEAM:** `dispatch_keyword_head_value` is a single hardcoded ~130-arm Rust
`match head { "...":wat::core::X" => ... }` (no runtime table). Three arm kinds: (a) ALREADY-clean
cross-home calls (`crate::collection::...`, `crate::function::eval_fn`, `crate::string_ops::...`);
(b) INLINE leaf-op calls (`eval_i64_arith`/`eval_not`/`eval_eq`/`eval_compare` — Scout 3's
scalar/ territory, still in runtime.rs); (c) the `other =>` user-fn fallback (sym.functions →
runtime_def_values → sandbox-leak → keyword-accessor). **HARD because of (b): the dispatch spine
cannot lift cleanly until the leaf ops (scalar/ + algebra/) lift or re-export first.** Lift order
within the runtime breakup: **leaf ops (scalar/algebra) → dispatch spine.**

**TRANSFORMS:** the dispatch arms for type-keywords (`:wat::core::i64`/`Vector`), `val_type_path`/
`value_matches_type_by_name` (return/compare keyword-encoded type strings vs `TypeExpr::Path`), and
the `-> :T` annotation checks in `eval_if`/`eval_match`/`*_tail` (validate `args[2]` as
`WatAST::Keyword` — under clojure-ination annotations may become `WatAST::Symbol`, so these must
accept both). Form-head keywords (`:wat::core::let`/`if`/`match`) are STABLE (don't change).
apply.rs + step.rs = **LIFTS** (clojure-stable).

---

## Scout 6 — FLAT-FILE TRIAGE (the ~35 mid/small src/*.rs, excl. the two monoliths)

**WARD-IN-PLACE (single coherent concern, NO move — just vigilia + stamp), ~25 files:**
sigma (116) · source (76) · restriction_entry (86) · sandbox (121) · span (144) ·
assertion (178) · compose (202) · harness (211) · diagnostic (218) · vm_registry (230) ·
stdlib (316) · special_forms (354) · form_match (375) · hologram (409) · panic_hook (427) ·
runtime_error_edn (441) · lower (516) · hash (814) · time (939) · config (948) ·
parser (1047) · test_runner (1084) · io (1496). **Fastest 5 (momentum):** sigma, source,
restriction_entry, sandbox, span — all <150 lines, zero ambiguity.

**CARVE → new multi-file homes:**
- `src/edn/` ← edn_shim.rs (2343): write.rs / read.rs / coerce.rs.
- `src/closure/` ← closure_extract.rs (2529): error/walk/topo/mod (4 sub-phases).
- `src/load/` ← load.rs (1741): resolve.rs / loaders.rs / error.rs.
- `src/freeze/` ← freeze.rs (2079): pipeline.rs + bootstrap.rs (section wall at freeze.rs:73).
- `src/thread_io/` ← thread_io.rs (953): ThreadIO / RuntimeServices / AmbientStdio.
- `src/string_ops/` (or `strings/`) ← string_ops.rs (612): string.rs / uuid.rs / regex.rs.

**LIFT-INTO / NEW concurrency homes:**
- NEW `src/spawn/` ← fork.rs (1808) + spawn.rs (340) + spawn_process.rs (497) — these are spawn
  *verbs* (USE comms, aren't comms); share `make_pipe` (fork.rs:346). Submodules fork/thread/process/pipe.
- `src/comms/channel.rs` ← typed_channel.rs (658); `src/comms/process_stdio.rs` ← process_stdio.rs (141).

**ALREADY-HOME / SKIP:** types.rs (Rust-2018 home root — needs a STAMP, no vigilatum yet) ·
check.rs (home root) · resolve.rs (251.1 in flight) · lib.rs (crate root).

**TRANSFORMS / DIES (clojure-ination casualties):**
- `lexer.rs` WARD-IN-PLACE but `lex_keyword`'s bracket machinery (605–730) **DIES within** at 251.3
  (parametrics-as-forms) — lex_keyword shrinks to "`:` until whitespace/`)`". Ward now, re-ward post-251.3.
- `ast.rs` WARD-IN-PLACE but `WatAST::Keyword` dual-role (operator-head + data) dissolves at 251.2/.3
  — a `Symbol` head representation is added; `keyword()` ctor transforms. Ward now, re-ward at 251.3.
- `edn_shim.rs` — `strip_keyword_colon` (1259) / `as_keyword` (1745) reshape when `:foo` becomes
  data-role. TRANSFORMS within the carve.
- No flat FILE dies entirely; only the `lex_keyword` bracket region dies.

---

# ★ SYNTHESIS — THE MASTER LIFT MAP + SEQUENCED MIGRATION

## The home federation (consolidated — all scouts deduped)

| Home | Source | What lifts in | Difficulty | clojure-ination |
|---|---|---|---|---|
| **`src/value/`** ⭐FOUNDATION | S1 | Value · Environment · SymbolTable · TrackedValue · ValueSnapshot · signal(EvalSignal/Break/RuntimeError) · frame · encoding_ctx | HARD | `Value::type_name`/`declared_type_name` + SymbolTable keyed maps = TRANSFORMS (in-source markers) |
| **`src/scalar/`** | S3 | arithmetic · conversions · bool · comparison(values_equal/compare) · math/stat | EASY | keyword_to/from_string = TRANSFORMS; rest LIFTS |
| **`src/algebra/`** | S3 | the VSA/holon `:wat::holon::*` layer (~3,800) — construct/classifier/measure/encode/memory | HARD | classifier ctors + is-Keyword?/is-Symbol? = TRANSFORMS; rest LIFTS |
| **`src/eval/`** | S2 | dispatch · control · apply · step | apply EASY · step/control MEDIUM · dispatch HARD | dispatch type-arms + val_type_path + `-> :T` checks = TRANSFORMS; apply/step LIFTS |
| **`src/comms/`** (extend) | S4 | eval.rs (concurrency wrappers) · died_error.rs · channel.rs(typed_channel) · process_stdio.rs | MEDIUM (process tier HARD) | LIFTS (clojure-stable) |
| **`src/spawn/`** NEW | S6 | fork · spawn · spawn_process · pipe | MEDIUM | LIFTS |
| **`src/collection/`** (extend) | S3 | surface.rs (Tier-2 fanout wrappers) | MEDIUM | LIFTS |
| **`src/check/`** (extend) | S5 | result · unify · walk · **scheme(DIES zone)** · infer | result/unify EASY · walk/scheme MEDIUM · infer HARD | scheme=DIES; infer dispatch=TRANSFORMS; control-flow infers=LIFTS |
| **`src/resolve/`** (in flight) | 251.1 | the surface→entity resolution (incl. BARE_* from check.rs) | — | the clojure-ination's own first stone |
| **`src/edn/` `src/closure/` `src/load/` `src/freeze/` `src/thread_io/` `src/string_ops/`** NEW | S6 | the carve files | MEDIUM each | edn = TRANSFORMS; rest LIFTS |
| **~25 ward-in-place** | S6 | sigma/source/.../io/lexer/ast | EASY | lexer + ast = TRANSFORMS (re-ward post-251.3) |
| registration pipeline | S4 | register_* fns → likely `src/freeze/` or co-located w/ SymbolTable in value/ | HARD (prereq: value/) | — |

## The dependency spine (the fire path)

```
resolve/ (251.1, in flight)
   │
   ▼
value/  ⭐ THE KEYSTONE — Value/Environment/SymbolTable; EVERYTHING imports it
   │     (registration pipeline + concurrency eval + collection fanout +
   │      derive_scheme_from_function ALL block on this)
   ├──────────────┬───────────────┬────────────────┬──────────────┐
   ▼              ▼               ▼                ▼              ▼
 scalar/      eval/apply      comms/+spawn/     check/         carve homes
 (leaf ops)   (EASY)          died_error        result/unify/  (edn/closure/
   │                          channel           walk           load/freeze/
   ▼                                              │            thread_io/strings)
 algebra/                                         ▼            [path-update only]
 (leaf ops)                                   check/scheme  ← lift-first
   │                                          check/infer     retire-after
   ▼
 eval/dispatch + control + step
 (needs leaf ops lifted FIRST so the match routes to crate::scalar/algebra)
   │
   ▼
 registration pipeline (needs value/ stable)
```

**Parallel, dependency-free tracks (start anytime):** the ~25 ward-in-place quick wins; the
carve homes (they only need a `crate::runtime::Value` → `crate::value::Value` path update once
value/ lands — internal reorg otherwise). The check/ track runs parallel to the runtime track
(Scout 3 + 5: TypeScheme is check-time only; runtime dispatch is keyword-string — DECOUPLED).

## ★ The strategic fork: lift-first vs clojureify-first vs interleaved (four-questions)

- **CLOJUREIFY-FIRST** (run 251's transforms on the flat monoliths, then lift): Obvious? **NO** —
  transforming a flat 31k runtime.rs + 19k check.rs is the exact pain we're escaping; Scout 5 warns
  retire-on-monolith = max collateral risk. Simple? **NO**. → **DISQUALIFIED.**
- **LIFT-FIRST** (ward the whole substrate, then transform within homes): Obvious? YES. Simple?
  partial NO — warding the TRANSFORMS surfaces (Value::type_name, dispatch type-arms, ast.Keyword,
  lexer brackets) means ward-then-reward when 251 changes them ("don't ward what you'll rewrite").
- **INTERLEAVED (tag-driven)** — LIFTS-tagged segments lift NOW (they shrink the monolith on the
  stable keyword surface); TRANSFORMS surfaces wait for their 251.x stone then ward; the DIES zone
  (check/scheme.rs) is lift-first-retire-after (the ward IS the excision plan, Scout 5):
  Obvious? **YES** (the scout tag IS the schedule). Simple? **YES** (each segment single-axis).
  Honest? **YES** (no ward-then-reward; no transform-on-monolith). Good UX? **YES** (floor rises
  continuously; monolith shrinks; 251 works in progressively-cleaner homes). → **FOUR YES.**

**VERDICT: INTERLEAVED, tag-driven.** The migration and the clojure-ination are ONE campaign,
scheduled per-segment by the scout tag: LIFTS now, TRANSFORMS at their 251 stone, DIES via
lift-then-retire. The phoenix burns by tag — the LIFTS rise from the ashes first, the TRANSFORMS
are reshaped in the fire, the scheme dies last.

## Recommended first strikes (the order of fire)

1. **resolve/** — continue 251.1 (in flight). The clojure-ination's first stone.
2. **value/** — THE KEYSTONE. Nothing else unblocks without it. Sequence its submodules per Scout 1:
   frame.rs + encoding_ctx.rs (EASY) → signal.rs + tracked.rs (resolve the `render_value`
   entanglement, runtime.rs:18628) → environment.rs (co-locate Function) → symbol_table.rs →
   value.rs (HARD, last; tag the `impl Value` block TRANSFORMS in-source).
3. **Quick-win ward-in-place wave** (parallel, momentum, zero dependency): sigma → source →
   restriction_entry → sandbox → span, then onward through the ~25. (Defer lexer + ast to post-251.3.)
4. After value/: **scalar/** (leaf ops) → then the eval **dispatch** spine can route cleanly;
   **comms/ + spawn/** ; the **check/ track** (result/unify/walk → scheme[lift-retire] → infer);
   the **carve homes** (path-update reorg). **algebra/** and **registration** are the HARD tail.

## What the scouts settled (load-bearing facts for every downstream stone)

- The TypeScheme "scheme" lives ENTIRELY in check.rs (check-time); runtime eval is NOT entangled
  with it (keyword-string dispatch). Runtime lift ∥ scheme demise.
- `instantiate`/`rename` called from exactly 2 sites (infer_list:5446, keyword-as-value:3353);
  `register_builtins` (4,365 lines) replaceable as a unit → clean excision.
- The dispatch spine (`dispatch_keyword_head_value`) needs leaf ops (scalar/algebra) lifted FIRST.
- `render_value` (runtime.rs:18628) is the value-home's key entanglement (ValueSnapshot reaches
  into eval-interior).
- comms/ already owns the mechanism; runtime concurrency eval is the wrapper layer that completes it.
- the eval-step! machine LIFTS (actively live, not DIES).

---

# ★ INTUERI VERDICT — the blessed names (weighed, not rubber-stamped)

A spawned intueri cast named the federation, grounded on the existing-home precedent
(domain-noun homes; `eval`/`infer`/`error`/`transform` concern-noun submodules; doctrine
in mod.rs prose NEVER in a filename). Orchestrator weighed each against precedent + the
four-questions. **Verdict: accept, with three carve-refinements flagged to their stones.**

**The blessed federation (final names):**

```
src/
  value/      — runtime value model    [submodules: value · environment · symbol_table ·
                                         observe (was tracked.rs) · signal · frame · encoding_ctx]
  scalar/     — arithmetic + numeric conversions + bool + comparison + math/stat
  algebra/    — VSA/holon vector algebra [construct · classifier · measure · encode ·
                                          memory · coerce (was support.rs)]
  eval/       — the interpreter loop    [dispatch · control · apply · step]
  spawn/      — process/thread birth    [fork · thread · process · pipe]
  edn/        — EDN/JSON ↔ Value bridge [write · read · coerce · error]
  closure/    — closure serialization   [error · walk · topo]
  load/       — source loading          [resolve · loader (was loaders.rs) · error]
  freeze/     — startup pipeline        [pipeline · bootstrap]
  stdio/      — per-thread stdio routing [was thread_io/; routing · services]
  text/       — string/uuid/regex       [was string_ops/; string · uuid · regex]
  resolve/    — surface→entity resolution (251.1 in flight)
  comms/ (extend) — eval · died_error · channel · process_stdio(interim; →process/stdio.rs later)
  collection/ (extend) — surface (the Tier-2 fanout wrappers)
  check/ (extend) — ctx (was result.rs) · unify · walk · builtin (was scheme.rs — the DIES
                    zone; doctrine in mod.rs prose) · infer   [home already has env · error]
```

**The L1 catch (credited to the cast — the [[feedback_intueri_names_all_things]] value):**
`check/scheme.rs` → **`check/builtin.rs`**. The module is dominantly `register_builtins`
(~4,365 of ~4,500 lines); `scheme.rs` would encode the retirement-DOCTRINE in the filename
(the file's about-to-die TypeScheme), violating "a filename shows the home's shape on `ls`,
not a doctrine." `builtin.rs` names what it IS now AND survives the demise (builtin
registration outlives TypeScheme). The DIES doctrine lives in `check/mod.rs` prose. The
orchestrator had proposed `scheme.rs` in this very map — the cast caught it.

**Three L2 mumbles fixed:** `thread_io/`→`stdio/`, `string_ops/`→`text/` (mechanism-compound /
`_ops` → domain noun); `tracked.rs`→`observe.rs`, `support.rs`→`coerce.rs`, `loaders.rs`→`loader.rs`.

**`eval/` collision — weighed, ACCEPTED:** a top-level `eval/` alongside `collection/eval.rs` +
`function/eval.rs` is unambiguous by module path (`crate::eval` = interpreter; `crate::collection::eval`
= the collection home's eval layer); `ls src/` reads `eval/` as the interpreter. Idiomatic over `interp/`.

**Three carve-refinements flagged to their stones (accept the name, refine the bundle there):**
1. `value/observe.rs` — revisit vs `provenance.rs` at the value/ stone (TrackedValue's essence IS
   provenance; ValueSnapshot is diagnostic-render — confirm "observe" unifies them when coding).
2. `stdio/` — the `RuntimeServices` orchestration is undersold by "stdio"; if services dominate,
   reconsider at its stone. (`routing.rs` + `services.rs` submodules already separate the concerns.)
3. `check/ctx.rs` — Scout 5's `result.rs` bundle (CheckResult/InferCtx/Subst/UnifyError) is itself
   mis-grouped: `Subst`→`unify.rs`, `CheckResult`→`error.rs`; `ctx.rs` then honestly holds InferCtx.
   A carve refinement for the check/ stone, not a blocker.

Names cited from the intueri cast 2026-06-06; per [[feedback_intueri_names_all_things]] every
surface name traces to the spawned cast, not a hand-proposal.

