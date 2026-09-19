## CERNERE — Cast Report

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> Status for every row lives in `FINDINGS.md`, and nowhere else.

## SCOPE

Swept, read in full this session:

**wat (6 files, all read completely):**
`wat/rete/oracle/{accum-pass,explain,fire,insert,pass,stratify}.wat`

**Rust (22 files, grepped for every `:wat::` token; contextually read where a hit needed disambiguation):**
`src/rete/kernel/{arm,census,insert,mod,node,outcome,session,stratify}.rs`, `src/rete/kernel/fire/{acc,delta,mod,rules}.rs`, `src/rete/kernel/fire/pass/{accumulate,alpha,filter,filter_after_join,hash_join,join_after_filter,mod,production,root_join,round_census}.rs`

**Method for the wat half:** extracted every distinct `:wat::rete::…` token appearing in the six files (152 raw matches, ~130 distinct names after dropping two artifacts of prose text with a trailing colon) and classified each against the three authorities named in `tests/lint/rete_names_in_wat_scripts_resolve.rs`'s header (read in full): zero of them are in the `:wat::rete::core::`/`:wat::rete::holon::` RETE_OPS namespace (confirmed by grep — none of the six files uses that prefix at all), so the whole set falls to authority 2 (attestation) or 3 (field declaration). Every self-defined name (rule-produces, stratify, fire-fixpoint, walk-*-ids, factbag-adjacent locals, etc.) is its own attestation. Every externally-referenced name was traced to a `defn`/`defrecord`/`defenum` in `wat/rete.wat`, `wat/rete/compile.wat`, `wat/rete/acc.wat`, `wat/rete/factbag.wat`, or a native-intercepted primitive wired in both `src/check.rs` (TypeScheme registration) and `src/runtime.rs` (dispatch arm) — `alpha-match`/`alpha-match-local`/`alpha-match-under`, `eval-insert`, `eval-test`, `cond-has-deferred-constraint?`, the five `$native` verbs, `arm-session`/`release-session`/`adopt-session-lease`, `return-type-of`. Every field accessor (`Session/*`, `Rule/*`, `Token/*`, `AlphaNode/*`, `AccumulateNode/*`, `TestNode/expr`, `NegationNode/negated-alpha-id`, `ExistsNode/exists-alpha-id`, `ProductionNode/rule-name`, `QueryNode/query-name`, `Element/*`, `FireOutcome`/`InsertOutcome`/`CompileOutcome` variants) was checked character-for-character against the `defrecord`/`defenum` field lists in `wat/rete.wat:31-400` — every one matches exactly.

**Method for the Rust half:** grepped all 22 files for every `:wat::` substring (143 raw hits, 44 distinct), then checked each name against `src/check.rs`/`src/runtime.rs` registration.

## THE LIVE QUESTION

Enumerated the `:wat::rete::` names in code position across the six oracle files: **all resolve** — none sit in the RETE_OPS namespace at all (so the registry/attestation split doesn't bite here the way it does in `wat-scripts/`), and every attestation-family name traces to a live `defn`/`defrecord`/native dispatch arm elsewhere in `wat/` or `src/`. The honest counter-argument in the cast holds: these six files are the reference engine's main entry paths (`fire-rules$oracle`, `insert$oracle`, `stratify`, `fire-rules-explain$oracle`) and every function in them is reached by ordinary execution — I found no `def` body naming a head that only that body ever mentions. `retain-supported` (already rowed dead by `purgare`) is the one truly-unforced-by-nothing-else function, but its names inside it are the same live vocabulary, not phantoms.

## FINDING

**F1 (L2, wat-rs — Rust half).** `src/rete/kernel/session.rs:754,981,1159` — the `OP` error-label constant on the three transient-decode helpers (`value_token_to_native`, `value_to_element`, `to_transient_inner`) is set to `":wat::rete::to_transient (beta decode)"`, `":wat::rete::to_transient (alpha decode)"`, and `":wat::rete::to_transient"`. This is styled exactly like the OP labels on real dispatchable primitives in the same file/area (`":wat::rete::arm-session"` at `arm.rs:1252`, `":wat::rete::release-session"` at `arm.rs:1362`, `":wat::rete::adopt-session-lease"` at `arm.rs:1426` — all three of which I confirmed ARE registered in `src/check.rs:21514/21526/21539` and dispatched in `src/runtime.rs:5677-5691`). `to_transient` has **no such registration anywhere** — not a RETE_OPS row, not a `check.rs` TypeScheme entry, not a `runtime.rs` dispatch arm, not a wat `defn`. It is a plain internal Rust function (only `to_transient_for_fire`/`to_transient` are ever called, from `src/rete/kernel/fire/delta.rs:279` and `src/rete/kernel/fire/rules.rs:382`, both on the live native-fire path — this is forced on every native fire, not dead code).

The `:wat::rete::` prefix on this label asserts, in the same idiom used for genuine callable forms elsewhere in this file, that `to_transient` is a language surface — it is not. A caller who hand-assembles a malformed `Session` (missing `matches`/`bindings`/wrong class) and calls `fire-rules` gets a `RuntimeErrorKind::TypeMismatch` citing `:wat::rete::to_transient` as the offending form; searching the language spec (RETE_OPS, check.rs registrations, or the wat corpus) for that name turns up nothing, because the actual form the caller invoked was `fire-rules`/`insert`, not this internal decode step.

**Authority that should have declared it:** none exists — it is Rust-internal-only, so no authority should ever have to. **What was probably meant:** the OP label should name the real user-facing verb whose call triggered the decode (e.g. `":wat::rete::fire-rules (session decode)"`), matching the convention its sibling functions in `arm.rs` follow correctly. **Recommendation:** rename the label so it doesn't read as a `:wat::rete::`-namespaced form (drop the `:wat::rete::` prefix, or attribute it to the actual entry verb). Severity L2 — it's a naming/labeling mumble in error-diagnostic text, not a source form that fails to run; nothing evaluates `:wat::rete::to_transient` as code.

## RUNES

No `rune:cernere(...)` annotations exist anywhere in the six target `.wat` files (checked by grep — zero hits). No exemption to evaluate.

## VERDICT

**DIVERGES** — one finding (F1, L2, `src/rete/kernel/session.rs:754,981,1159`) — everything else swept is clean:
- All ~130 distinct `:wat::rete::` names in the six oracle `.wat` files resolve against attestation or field-declaration, with zero use of the RETE_OPS `:wat::rete::core::`/`:wat::rete::holon::` namespace in this target at all.
- All 44 distinct `:wat::` tokens across the 22 in-scope Rust kernel files resolve to registered primitives or documented (non-invoking, prose) DSL examples, except F1.
- No `rune:cernere` exemptions present to adjudicate.
- Confirmed out-of-scope items were not re-reported: `retain-supported` dead code (fire.wat:231-254), the stale `fact_type_head` doc comment (stratify.rs:35-37), the `rule-negates` nested-`:not` gap (stratify.wat:138-160), and the `intueri` rune on `walk-filter-ids` (fire.wat:54) were all left untouched.
