## CONFORMARE — Cast Report

> Written verbatim as returned. `&lt;`/`&gt;`/`&amp;` are HTML-entity artifacts of the agent's output.

**Scope check performed first, per the ward's own instruction to state scope rather than manufacture findings:**

- `grep -rn "enum.*Error\|struct.*Error" src/rete/kernel --include=*.rs` (excl. tests/) → **zero hits**. This target defines no error type of its own.
- `grep -rn "impl From&lt;" src/rete/kernel --include=*.rs` (excl. tests/) → **zero hits**. No `From`/conversion impls exist in the target at all.
- `wat/rete/oracle/*.wat`: only two `defrecord`s exist (`FireStratAcc`, `StratifyAcc`) — accumulator bookkeeping, not error types.

So: **this target defines no error types of its own, and has no `From` conversion impls to audit for span-drop.** The audit runs entirely on the ward's fourth surface — functions producing a `RuntimeError` while a caller-supplied span is in scope, checked against whether it is threaded or discarded for `crate::rust_caller_span!()` (the crate's documented sentinel for "no recoverable USER source location", per `fire/rules.rs:645-647`).

`outcome.rs` read in full (251 lines) — the ward flagged it as most likely to matter, but it is clean on this axis: `fire_result_to_outcome`, `insert_result_to_outcome`, `compile_result_to_outcome` never construct a new spanned error; they forward an existing `Err(EvalBreak::Diagnostic(e))` unchanged (span intact) or convert to an `Ok(Value)` outcome whose fields are plain data with no span by design — and that Value is never in an `Err` position, so it is out of scope per the ward's own "non-error types" exclusion.

### Findings

**Finding 1 — `fire_rules_on_session` holds a real user span but a sibling call path (`native_stratify`) discards it for a user-reachable diagnostic.**

- `src/rete/kernel/fire/rules.rs:648-652` — `fire_rules_on_session(session, span: &amp;Span, sym, support)`. The doc at `:638-642` states: *"`span` is the caller's WAT location — the `list_span` of the `(fire-rules …)` form the author actually wrote … a refusal a user can reach must name the line the user wrote, not a line in this file."*
- `span` is used for exactly one call: `refuse_export_without_arm(OP, span)` at `:677`.
- At `:707` the same function calls `native_stratify(&amp;pn_only)?` → `native_stratify_fix` (`stratify.rs:277`), which on a negation cycle raises `MalformedForm { reason: "stratify: negation cycle detected — rule set is not stratifiable" }` at `stratify.rs:288-294` using `crate::rust_caller_span!()` — **not** `span`, though `span` is in the same stack frame that called it.
- This is a genuine rule-authoring mistake (the user wrote a negation cycle), not an internal-corruption invariant like `driver_of` — exactly the class of error a wat author needs a source location for, and the location was available and discarded at the boundary.
- Same gap reachable a second way: `:673` calls `fire_rules_from_deps` (`:865`), whose signature has **no** span parameter at all, and which also calls `native_stratify` at `:880`.
- Level: **L2** (diagnostic-quality gap on a user-reachable path).
- Pattern proposal: **Pattern A-lite** — thread `span: &amp;Span` through `native_stratify`/`native_stratify_fix`/`fire_rules_from_deps`, as `insert.rs` already does uniformly. Four-questions: **Honest** fails (a constructor can produce this `MalformedForm` with zero relationship to the user's actual call site even though that site was one frame away).
- Cascade: 2 call sites into `native_stratify`, both downstream of the one span-bearing entry.

**Finding 2 — the `to_transient`/`pm_to_*` decode family is entirely span-less by signature, reached from the same span-bearing entry points.**

- `pm_to_production` (`session.rs:677`), `pm_to_beta` (`:895`), `pm_to_alpha` (`:1049`), `pm_to_query_memory` (`:1296`), `to_transient_inner`/`to_transient_for_fire` (`:1142-1157`) take `op: &amp;'static str` but **no `span`** — every `TypeMismatch` they raise (28 sites) uses `rust_caller_span!()`.
- `to_transient_for_fire` is called from `fire_fixpoint_delta_armed` (`fire/delta.rs:279`) and `harvest_stratified_queries` (`fire/rules.rs:382`) — both reachable from `fire_once_session` and `fire_rules_on_session`, which have `span` and do not forward it.
- Matches the ward's own worked example almost exactly.
- **Caveat I can only state, not resolve:** these fire only on a malformed Session-internal memory shape — data that never carried a span even at construction. Whether the outer `span` would be a *correct* attribution (vs. `driver_of`'s genuinely spanless class) is a domain judgment I cannot close from target files alone.
- Level: **L2**. Fix is either threading, or documenting with `rune:conformare(spanless-by-domain)` at each site — currently **absent**: none of the 28 carries a rune of any kind.

### Not findings (checked, ruled legitimate)

- `driver_of` (`fire/mod.rs:282-296`), `rematch_compiled` (`:388-400`), `exec_stashed_where` (`:2218-2233`) — explicitly documented as compile-time-invariant lookups; genuinely no user source location exists for a compiler bug.
- `fire/delta.rs:716,733` — converted to matchable `FireOutcome` values before reaching a user; span is moot.
- `fire/acc.rs` + `fire/pass/accumulate.rs:116` — per-fact hot-path helpers. A real gray area, but threading `&amp;Span` through the per-fact accumulate path is a perf call I cannot adjudicate without a benchmark I am not permitted to run. Reported as a **weaker instance of Finding 2's shape**, not a standalone finding.
- `insert.rs` — **clean comparator: zero `rust_caller_span!()` in the whole file**; all 5 `RuntimeError::new` sites thread `list_span.clone()`. This is the pattern Findings 1–2 should match.

### Runes encountered

41 runes found, **zero `rune:conformare(...)`**. `stratify.rs:350` — a record that a rune was *proposed and refused*, a different concern. `fire/rules.rs:645-647` — not a rune but names the structural gate (`span_substitution_justified`) partially covering this concern; its own text says it works "in this body", i.e. per-function, and does **not** reach across the call boundaries Findings 1–2 describe — real enforcement with a documented narrower reach, not a false claim.

### Substrate-wide manifest

| Error type | Owner | Conformance state |
|---|---|---|
| `RuntimeError` / `RuntimeErrorKind` / `EvalBreak` | outside target | not auditable here |
| `FireOutcome` / `InsertOutcome` / `CompileOutcome` | target | not an error type (never in `Err` position) — exempt |
| (no target-owned error enum exists) | — | N/A |

**Pattern recommendation:** the target owns no error type, so Patterns A/B/C don't apply structurally. The actionable fix is call-boundary span threading, matching what `insert.rs` already does everywhere.

**Retrofit ordering:** Finding 1 first — cheap (once per fire, not per-fact) and clearly user-reachable. Finding 2 next, gated on the domain question `driver_of`'s comment already answers for its siblings. The `acc.rs` family lowest, pending an actual benchmark.

**CONVERGED.**
