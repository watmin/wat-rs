# SCORE — every sweep names what it reads

**SCORED.** Executor: grok, 2026-09-18, branch `sns-sqs`, HEAD `693ff1865` (DRAWN). Did not commit.

⛔ **A is report-only. Nothing changed.** No behaviour, no perf, no fixes.

---

## Completeness method (row 2)

**Transitive callee closure** from each sweep's entry in `check_program` (`check.rs:648`).

Starting functions:

| sweep | entry (inside the `census::phase` that names it) |
|---|---|
| 8b `check:retired-syntax(ALL fns)` | `check.rs:691–699` — `for func in sym.function_values()`, five walkers on `FunctionBody::Wat` |
| 8c `check:restricted-call(ALL fns)` | `check.rs:751–755` — `for (name, func) in sym.functions_iter()`, `walk_for_restricted_call` |
| 8d `check:def-position(ALL fns)` | `check.rs:779–793` — `validate_def_positions_in_forms(forms)` **and** the ALL-fns loop |
| 8f `check:body-infer(ALL fns)` | `check.rs:852–882` — `env.get(path)` then `check_function_body` → `infer` |

**Stopping rule.** Expand every callee until the frontier is only:

1. `WatAST` field access / `children()` / list indexing;
2. `HashMap`/`HashSet` get whose **key is derived from the AST or the fn path**;
3. string prefix/equality against **compile-time constants**;
4. local `InferCtx` / `Subst` / `errors: &mut Vec` (not user-writable);
5. `Function` fields (`body`, `params`, `rest_param`, `synthesized_for`) of the row being walked.

**Not a function signature.** The eighth door (`get_binding_metadata`, `env.rs:418`, live read `check.rs:1679`) is reached through `walk_for_restricted_call`, which `check_program`'s signature (`&[WatAST], &SymbolTable, &TypeEnv`) does not name.

**Where the closure is unbounded (row 5):** there is no `dyn`, no `RefCell`/`Mutex` in `check.rs` (Tier B SCORE row 3, re-confirmed: `grep -cE 'RefCell|Mutex|…' src/check.rs` → 0). The remaining unbounded shape is **HashMap lookup keyed by a name taken from the program** (`env.get`, `env.types().get`, `get_binding_metadata`, `get_defined_value_type`, `get_defclause_clauses`, `unit_variant_type`, `has_registered_function`). That is a frozen map, runtime key — B must treat “keyed by any name” as open unless the key is forced `:wat::` / `:rust::` / `:$bound::`.

---

## 8b — `check:retired-syntax(ALL fns)`

Entry `check.rs:690–701`. Callees: `validate_bare_legacy_primitives` `:993` → `walk_for_bare_primitives` `:1138`; `walk_for_legacy_stream` `:1403`; `walk_for_legacy_lru_cache_service` `:1434` (+ `canonical_lru_leaf` `:1427`); `walk_for_legacy_kernel_queue` `:1480`; `walk_for_bare_legacy_console` `:1523`. Recursion: `node.children()` except console, which matches `Keyword`/`List`/`Vector` only (`:1524–1542`) and **does not enter Map/Set**.

Iterator: `sym.function_values()` (`symbol_table.rs:286`) — every registered `Function`, including user. Native bodies skipped (`FunctionBody::Wat` guard `:693`).

⚠ Adjacent, **outside** the census phase (`:702–719`): the same five walkers on user `forms` that are not fn-def forms. Not an ALL-fns input. Named so it is not silently folded in.

| input | lines | user-writable? | how closed (or not) |
|---|---|---|---|
| `Function.body` AST | `:693–698` | stdlib bodies **no** (frozen); user bodies **yes** (their own) | stdlib: bake-time bytes. User adding a fn adds a *new* verdict, does not rewrite a stdlib one. |
| `FunctionBody` tag (Wat vs Native) | `:693` | no for stdlib | set at registration; user cannot re-body a `:wat::` fn (`ReservedPrefix`). |
| hardcoded prefix/name tables | `LEGACY_STREAM_PREFIX` `:1400`, LRU `:1423–1425`, `LEGACY_KERNEL_QUEUE_NAMES` `:1474`, `LEGACY_CONSOLE_PREFIX` `:1521`, `BARE_PRIMITIVES` `:1213`, retired heads in `walk_for_bare_primitives` `:1155–1175` | **no** | compile-time constants. |
| `parse_type_expr_audit` on keywords | `:1197` | no | parse of the keyword spelling; no env. |
| `CheckEnv` / `TypeEnv` / `binding_metadata` | — | — | **not consulted.** |

**8b reads no door a user can write that would change a stdlib body's 8b verdict.** Structural gate sufficient.

---

## 8c — `check:restricted-call(ALL fns)`

Entry `check.rs:750–757`. Callees: `walk_for_restricted_call` `:1670`; `walk_restricted_quasiquote_template` `:1757`; `quote_boundary` `resolve/boundary.rs:79`; `is_unquote_escape` `boundary.rs:102`; `env.get_binding_metadata` `env.rs:418` live at `check.rs:1679`; `extract_prefix_list_from_metadata` `:1563`; `caller_matches_prefix_list` `:1798`; `WatAST::children` `:1732`.

Iterator: `sym.functions_iter()` (`:751`, `symbol_table.rs:282`) — path + `Function`.

| input | lines | user-writable? | how closed (or not) |
|---|---|---|---|
| `Function.body` AST | `:753` | as 8b | bake-time for stdlib. |
| `name` (enclosing FQDN) | `:751, :1672` | stdlib names **no** | `:wat::` prefix; `ReservedPrefix`. |
| `func.synthesized_for` (`owner_type`) | `:754, :1678` | no for stdlib | companions of `:wat::` types are `:wat::`. |
| `quote_boundary` / `is_unquote_escape` | `boundary.rs:79–104`; used `:1701, :1766` | **no** | static `match` on `:wat::core::quote` / `forms` / `literal` / `quasiquote` / `unquote` / `unquote-splicing`. |
| ⛔ **`CheckEnv.binding_metadata`** | field `env.rs:87`; get `env.rs:418`; **only live read in the tree** `check.rs:1679` | **YES — keyed by any name** | User writes it with `{:restricted-to […]}` on **their own** `defn`. Not a `:wat::`-keyed door. The reserved wall is a half-answer. |
| metadata AST (`:restricted-to` vector) | `:1563–1598` | yes, on user keys | contents of the map entry; only reached if the key lookup hits. |

**After `a-mention-in-a-quoted-form-is-not-a-call`:** AllData (`quote`/`forms`/`literal`) does not walk children (`:1706–1708`); quasiquote walks only unquote escapes (`:1713–1718`). The two user-declarable names in stdlib bodies (`:user::main`, `:user::spawn::service-locus`) sit in `(:wat::core::forms …)` — the walker no longer looks them up. ⚠ **Closed by corpus measurement, not by construction.** A third *unquoted* user-declarable name in a stdlib body reopens the door. That is what B must wall.

Spike: **does not instrument this sweep.** Agree with Tier B SCORE: 8c was the missing census.

---

## 8d — `check:def-position(ALL fns)`

Entry `check.rs:779–794`. Callees: `validate_def_positions_in_forms` `:9698` → `validate_def_position_with_wrapper` `:9721` (recursive). Match on hardcoded heads (`:wat::core::do`, `:wat::core::let`, … `:9745+`). `def` arm retired (Gap I-B, comment `:9746–9754`); leftover declarations still walked as non-top-level.

Two inputs inside one phase:

| input | lines | user-writable? | how closed (or not) |
|---|---|---|---|
| user `forms` | `:780` | **yes** | this is the user program's def-position check; not a stdlib verdict. Cannot be elided by B. |
| `Function.body` AST (ALL fns) | `:783–791` | as 8b | stdlib: bake-time bytes. Verdict = “does this AST contain a declaration in a non-splice position?” — no env. |
| `DefCtx` / `wrapper` strings | threaded `:9723–9724` | no | local walk state. |
| hardcoded splice heads | `:9755+` | no | compile-time. |
| `CheckEnv` | — | — | **not consulted.** |

**ALL-fns half: structural gate sufficient.** The `forms` half is user-program checking and must keep running.

---

## 8f — `check:body-infer(ALL fns)`

Entry `check.rs:852–884`. Per fn: `spike_probe::begin_body_sweep` (instrument only); `env.get(path)` `:878` (schemes, keyed by **the fn's own path**); `check_function_body` `:2389` → `infer` `:2560` → `infer_list` `:3191` → `unify` `:17093` / `assignable` `:17362` / many `infer_*`.

`infer` is `&CheckEnv` — **no writes** during 8f. Writes to `defined_values` / defclause / extend / `redef_allowed` happen in **8e** (`:807–837`) *before* 8f. 8f therefore *sees* user 8e state, but cannot mutate it.

Quoted data: `infer_list` does **not** recurse into `:wat::core::quote` (`:3965–3977`), `:wat::holon::literal` (`:4018–4027`), `:wat::core::forms` (`:4029–4038`). `:wat::core::quasiquote` (`:5657–5663`) returns `:wat::WatAST` **without inferring the template** (comment at `:5659–5661` claims unquotes infer; the body does not). So 8f does **not** look up `:user::main` inside `service-forms` templates — that is why the spike never attributed `:user::main` to a stdlib **file**.

| input | lines | user-writable? | how closed (or not) |
|---|---|---|---|
| `Function.body` + params / rest / scheme | `:2391–2432` | stdlib **no** | bake-time `Function` + `derive_scheme_from_function` in `from_symbols` (`env.rs:165–181`). |
| `env.get` **schemes** (spike `schemes`) | `env.rs:360`; 8f start `:878`; infer call-heads e.g. `:2198, :2291, :2659, :5681, :6350` | map contains user keys; lookup key = AST head or fn path | **Keyed by any name.** Stdlib path keys are `:wat::` → closed by construction for those lookups. A stdlib body calling a user name would be open. Spike: from stdlib **files**, no namespaced non-reserved scheme probe. |
| `env.types()` **TypeEnv** (spike `typeenv`) | `types.rs:644`; via `unify`/`assignable`/`reduce`/`is_subtype` | user may `deftype` under their prefix | **Keyed by any name.** `is_subtype` walks `TypeEnv::subtype_parents` (`types.rs:6501`). User `extend-type` writes **this** graph (not `CheckEnv.extend_registrations`). Spike: user-reachable typeenv probes are bare `:T`/`:K`/`:V`. |
| `unit_variant_type` (spike `unit_variant`) | `env.rs:298`; `infer` `:2620` | user enums add entries | keyed by variant keyword. User cannot mint `:wat::…::Closed`. |
| `get_defined_value_type` → `defined_values` then `corpus_values` (spike `defined_values`) | `env.rs:376–381`; `infer` `:2650` | `defined_values` is **filled by 8e from user `def`s**; `corpus_values` is bake-seeded (`env.rs:119–135, :210+`) | **Keyed by any name.** User can only insert namespaced non-reserved keys (gate). Stdlib bodies looking up those keys would be open; spike's 119 user-reachable hits are **unnamespaced** (`:Ok` `:Err` …) which `Privilege::User` cannot write. |
| `get_defclause_clauses` (spike `defclause_regs`) | `env.rs:461`; `infer_list` `:5681, :6179` | stdlib clauses seeded `env.rs:189–201`; user clauses added in 8e | keyed by call head. Same reserved-key rule. |
| `has_registered_function` (spike `registered_fns`) | `env.rs:291`; `collect_tail_user_fn_peer_calls` `:2288` (from `push_handle_creation_escape` in `check_function_body` `:2456`) | set copied from `sym.functions` at `from_symbols` `:166` | keyed by call head. Spike: user-reachable=0. |
| `get_extend_methods` / `extend_registrations` (spike `extend_regs`) | `env.rs:486` | — | **not in 8f's callee tree.** Zero call sites in `check.rs`. Spike `probed=0` is structural, not empirical. Live extend-type influence on 8f is `TypeEnv.subtype_edges` via `is_subtype`, above. |
| `get_binding_metadata` | `env.rs:418` | — | **not in 8f.** Only `check.rs:1679` (8c). |
| `env.iter` / `validate_aggregate_containment` | `check.rs:15725` | — | **not a four-sweep.** Freeze-time (`freeze/env.rs`). Named so it is not mistaken for 8f. |
| `InferCtx` / `Subst` / `errors` | `:2392, :2412, :2461` | no | local to `check_program`. |
| `spike_probe` atomics/mutex | `spike_probe.rs:15–29` | n/a | instrumentation; `WAT_SPIKE_WITNESS` off ⇒ one relaxed load. Not verdict-bearing (Tier B row 3). |

**8f vs instrument (row 6).** Seven spike doors are exactly the seven `CheckEnv`/`TypeEnv` getters that have `spike_probe::probe` (`env.rs:292, 299, 361, 377, 465, 492`; `types.rs:645`). Reading agrees: those are 8f's env doors. Divergence: (1) spike window is `P_CHECK_BODIES` only — 8b/8c/8d never appear; (2) `extend_regs` is instrumented but **unreachable** from 8f; (3) `binding_metadata` is a real door and **uninstrumented**; (4) 8f also reads `Function` fields and hardcoded infer arms, which are not “doors” in the spike's sense.

---

## Verdict for B and C (row 9) — recommendation, not a decision

| sweep | can a structural gate close every user-writable door? |
|---|---|
| **8b** | **Yes.** AST + const tables. Same stdlib bytes ⇒ same 8b verdict. |
| **8d ALL-fns** | **Yes**, same as 8b. **8d `forms` half is user-program checking — do not elide.** |
| **8f** | **Yes, with a bake-time snapshot**, on the spike induction: probes of stdlib **files** never hit a namespaced non-reserved key, and the user cannot write the keys they *do* hit. ⚠ Do **not** snapshot the **post-8e** env: `defined_values` / user defclauses are user-writable maps sitting in the same `CheckEnv`. Elide 8f against freeze-time `from_symbols` state (schemes, corpus_values, stdlib defclauses, TypeEnv-at-freeze, unit_variants, registered_fns). |
| **8c** | **Not by construction.** The door is keyed by any name. After the quoted-mention stone it is closed **by corpus** (two names, 19 occurrences, all `forms`). A third unquoted user-declarable name reopens it. **Leave 8c running, or elide it only behind a bake-time closure over live (unquoted) user-declarable leaves in stdlib bodies** — the surface test `the_user_declarable_surface_inside_stdlib_bodies_is_exactly_two` plus a live-position filter. The inverted Tier B witness is **no longer a control on 8c** (mention SCORE row 6); B must mint a fresh one. |

C (elision) on 8b+8d(ALL-fns)+8f keeps the 121.58 ms of 124.65 and is immune to the `{:restricted-to}` witness. 8c is 3.07 ms; running it is the cheap honesty. Fixing quoted-mention was the other cheap route to the whole 124 ms — **already taken**. What remains for a full 8c elision is a *construction* proof, which this table says 8c does not have.

---

## Floor / clippy / porcelain (rows 7–8)

Clippy `--all-targets -D warnings`: `Finished` in 19.76s, no `error`/`warning` lines in the log.

Floor, confirmed this strike (not inherited):

```
     Summary [ 272.278s] 5306 tests run: 5306 passed, 22 skipped
```

`.floor/2026-09-19T00-43-44Z/` — exit **0**, **no `ARM.txt`**. Count **5306**, unchanged from the mention stone. `src/` untouched. `git diff` of this stone is this SCORE only.

Open items from earlier stones (`check.rs:6068`, `env.rs:456`, `Existing::Equivalent`, top-level-mention hole, blame-inversion, rendezvous): **unread for effect, not fixed.**

---

## ⭑ REGRADED BY THE ORCHESTRATOR, 2026-09-19 — accepted, and the closure holds

**No fresh floor, deliberately and stated**: `git status` shows **one untracked file — this SCORE**.
`src/` and `wat/` are byte-identical to `da00bb3f4`, whose floor I ran myself
(`Summary [ 256.008s] 5306 tests run: 5306 passed`, no `ARM.txt`), and grok's own run on this tree agrees
(`272.278 s`, same 5306, green). A report-only stone that changes no code cannot move a floor.

| claim | my own check |
|---|---|
| ⛔ **`get_binding_metadata` is 8c's door alone** | ✅ **exactly one live call site** in the tree — `check.rs:1679`. Plus the definition (`env.rs:418`) and a doc mention (`check.rs:1607`). **This is why the spike never saw it: the spike instruments 8f, and the eighth door is not in 8f.** |
| **the closure frontier is bounded** | ✅ `dyn`/`RefCell`/`Mutex`/`RwLock` in `src/check.rs`: **0**. The transitive callee closure is completable rather than aspirational — which is what makes row 2's method credible. |
| report-only | ✅ nothing but the SCORE. |

### ⭐ NO NINTH DOOR — and the method is why

Row 2's **transitive callee closure** from each sweep's entry, carried until the frontier is only AST
field access, AST-keyed map gets, compile-time string comparison, local infer state, and the `Function`
row being walked. ★ **Not a function signature** — the eighth door is reached *through a helper*, and
`check_program`'s signature names none of it. That distinction is the whole difference between this
stone and the census that preceded it.

### The verdict, and it is sharper than "yes or no"

| sweep | ms | structural gate sufficient? |
|---|---:|---|
| **8b** retired-syntax | 11.13 | **Yes** — AST + const tables; same stdlib bytes ⇒ same verdict |
| **8d** def-position, ALL-fns half | 1.72 | **Yes**, same as 8b |
| **8f** body-infer | 108.73 | **Yes, with a bake-time snapshot**, on the spike induction |
| ⛔ **8c** restricted-call | 3.07 | **NOT by construction** — the door is keyed by **any** name; closed only **by corpus** after the quoted-mention stone |

⭐ **So C is 8b + 8d(ALL-fns) + 8f = 121.58 of 124.65 ms, immune to the `{:restricted-to}` witness, and
8c keeps running for 3.07 ms.** That is a *better* answer than the binary the stone was drawn to get:
the expensive sweeps are closable, the one that is not is the cheap one.

⚠ **And a second asymmetry nobody had noticed: 8d is two inputs in one phase.** Its `forms` half is the
**user program's own** def-position check and **must keep running** — only the ALL-fns half is elidable.
A merged table would have hidden that, which is exactly why row 1 demanded four separate tables.

### Instrument and reading agree, and the divergence is explained

8f's seven spike doors are **exactly** the seven `CheckEnv`/`TypeEnv` getters carrying a
`spike_probe::probe` call. `get_binding_metadata` has none — and is not in 8f's callee tree at all.
**The instrument was complete for the sweep it instrumented, and silent about the other three.**

### What B now has to do

Close, by construction: 8b and 8d(ALL-fns) trivially (stdlib bytes are bake-time); 8f via a bake-time
snapshot resting on the spike induction. **Leave 8c running.** The gate's shape is the one the namespace
stone already demonstrated — a symbol-table/structural test, not a grep.
