# BRIEF — Arc 232 Stone 232.0a — typed-entities reflection layer

## What we're doing

Mint THREE wat-callable reflection verbs that the typed-entities doctrine demands but the substrate hasn't yet exposed:

1. **`:wat::holon::extract-classifier <h>` → `:Option<String>`** — lift the existing Rust `fn extract_classifier(holon: &HolonAST) -> Option<String>` (at `src/runtime.rs:13986`) to a wat verb. Returns `Some(class-name)` for canonical-wrap shape `(Bind (Atom <s>) <right>)`; `None` otherwise. defprotocol's dispatcher consumes this.

2. **`:wat::holon::Bind/left <h>` → `:Option<HolonAST>`** — NEW Rust fn + wat verb. Returns `Some(left)` for literal `(Bind left _)`; `None` otherwise. The LEFT position of a Bind primitive. In classifier-wrap shape, holds the `(Atom <ClassName>)`. In field-Bind shape, holds the `(Atom <field-name>)`. Symmetric peer of Bind/right.

3. **`:wat::holon::Bind/right <h>` → `:Option<HolonAST>`** — NEW Rust fn + wat verb. Returns `Some(right)` for literal `(Bind _ right)`; `None` otherwise. The RIGHT position of a Bind primitive. In classifier-wrap shape, holds the data (typically a Bundle of field-Binds). Mirrors the `Bundle/children` precedent — naming the STRUCTURAL fact, not the doctrine-conventional reading. defrecord accessor synthesis consumes this composed with `Bundle/children`.

**NAMING DECISION (per intueri cast 2026-05-23 night late):** original proposal was the asymmetric `Bind/inner`. Intueri verdict: Level 2 (mumbles) — borrows meaning from one specific use case (classifier-wrap doctrine) rather than from Bind's general primitive shape. `Bind/left` + `Bind/right` are positional, symmetric, honest about Bind's structural shape. Convention-based semantic verbs (extract-classifier) compose on top. User confirmed: ship symmetric pair; arc 232 closure depends on this delivery.

After this stone: defrecord instances can be FULLY INSPECTED — both halves (classifier-Atom on the left, data-Bundle on the right). defprotocol macro work (Stone 232.1) unblocks; defrecord accessor synthesis (later stone) has the tools to walk field-Binds via Bind/left → classifier-Atom name, Bind/right → field value.

## The rank-up — arc 233 tools active

**Arc 233 just closed (`69e0ada`).** Sonnet is now equipped with diagnostic-richness substrate the prior arc-232 sessions didn't have:

- **ValueSnapshot in errors** (Stone 233.1) — NotCallable / TypeMismatch / BadCondition errors render the actual value + carry provenance. If a probe fails because wrong type passed in, error names WHAT not just type-string. Iteration is faster.

- **Provenance tracking** (Stones 233.2.a/b/c/d/h/i/j/k/l/e) — every Value carries `Provenance::Literal { span }` for literals, `Provenance::SymbolBound { binding_span, head_span }` for let-bound symbols (env.lookup boundary), `Provenance::RuntimeBuilt { producer, call_span }` for the 5 producers. Probe 1 defines `v = (:myapp::Voltage 5.0)` then calls `(extract-classifier v)` — `v` carries SymbolBound provenance. If extract-classifier fails internally, the error names WHERE v was bound + WHERE the verb was called.

- **`#[wat_value]` structural seal** (Stone 233.2.l) — sonnet writing Rust in src/runtime.rs CANNOT accidentally add a wrapping variant to Value. Compile error. Confidence to extend the substrate.

- **Errors-as-EDN at IPC boundary** (Stone 233.3) — if any panic surfaces during cargo test, stderr is parseable EDN. Pattern-match on `#wat.kernel/<Variant>` tag.

**Use these tools.** When a probe fails, READ the error's ValueSnapshot.rendered + ValueSnapshot.provenance before guessing. The substrate teaches WHAT and WHERE; the BRIEF doesn't need to enumerate every failure mode because the errors do.

**Implementation surface:**

1. **`eval_extract_classifier`** — new Rust fn in `src/runtime.rs` that takes the args + list_span, expects 1 arg evaluating to `Value::holon__HolonAST(holon_ast)`, calls existing `extract_classifier(&holon_ast)`, wraps the `Option<String>` result as `Value::Option(Arc::new(...))` with `Value::String` payloads. Dispatch arm in `dispatch_keyword_head_value`: `":wat::holon::extract-classifier" => eval_extract_classifier(args, list_span, env, sym)`.

2. **`fn bind_left(holon: &HolonAST) -> Option<HolonAST>`** — new Rust helper alongside `extract_classifier`. Matches `HolonAST::Bind(left, _)` → `Some((*left).clone())`; other → None.

3. **`fn bind_right(holon: &HolonAST) -> Option<HolonAST>`** — new Rust helper. Matches `HolonAST::Bind(_, right)` → `Some((*right).clone())`; other → None.

4. **`eval_bind_left`** — new Rust fn that takes args + list_span, expects 1 arg evaluating to `Value::holon__HolonAST(holon_ast)`, calls `bind_left(&holon_ast)`, wraps as `Value::Option(Arc::new(...))` with `Value::holon__HolonAST` payloads. Dispatch arm: `":wat::holon::Bind/left" => eval_bind_left(args, list_span, env, sym)`.

5. **`eval_bind_right`** — symmetric to eval_bind_left. Dispatch arm: `":wat::holon::Bind/right" => eval_bind_right(args, list_span, env, sym)`.

6. **Type-checker integration** — all three verbs need entries in `src/check.rs` so the type checker accepts them. Look for how `Bundle/children` is registered (`src/check.rs` has parametric-call inference for `Bundle/children` returning `Vector<HolonAST>`); mirror that shape for the new verbs (return Option<String> for extract-classifier; Option<HolonAST> for Bind/left + Bind/right).

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/232-defprotocol-extend-type/DESIGN.md`** — arc 232 umbrella; Stone 232.0a row in the Work-items table; the typed-entities reflection layer rationale.

2. **`tests/probe_diagnostic_typed_entities_reflection.rs`** — FM 2-bis probe (7 contracts; updated post-intueri-cast to use Bind/left + Bind/right). Currently FAILS (verbs don't exist). **The probe IS the success criterion** — flip 0/7 → 7/7. Contracts: (1+2) extract-classifier on defrecord/bare-Atom; (3+4) Bind/right on defrecord/non-Bind; (5) composed walk extract-classifier + Bind/right + Bundle/children; (6+7) Bind/left on defrecord/non-Bind.

3. **`src/runtime.rs:13986`** — `fn extract_classifier(holon: &HolonAST) -> Option<String>` — the existing Rust fn `eval_extract_classifier` wraps. Pattern: match Bind → match key → match Atom → match String.

4. **`src/runtime.rs:12482`** — `fn eval_bundle_children(...)` — the precedent for wat-verb structure. Mirror the shape for `eval_extract_classifier` + `eval_bind_inner`.

5. **`src/runtime.rs:4892`** — `":wat::holon::Bundle/children" => eval_bundle_children(args, list_span, env, sym)` — the dispatch-table precedent.

6. **`docs/arc/2026/05/233-substrate-errors-as-values/INSCRIPTION.md`** — the arc that just closed; explains the new diagnostic-richness tools sonnet now has at its disposal.

## What does NOT change

- **HolonAST enum** — unchanged
- **Value enum** — unchanged (sealed by `#[wat_value]`; can't add wrapping variants anyway)
- **extract_classifier Rust fn** — unchanged (lifted, not modified)
- **defrecord / defprotocol macros** — unchanged (Stone 232.0a is substrate-only; macro work is Stones 232.1+)
- **arc 233 deliverables** — unchanged (regression guards for ValueSnapshot/Provenance/etc. all stay GREEN)
- **holon-rs** — NOT touched

## Out of scope (affirmative scope-bounding)

- **defprotocol macro** — Stone 232.1 (next stone)
- **extend-type macro** — Stone 232.2
- **defrecord accessor synthesis** — separate stone or future arc per DESIGN.md table row 232.4
- **HolonAST decomposers for other variants** (Atom/inner, Permute/decompose, etc.) — out of arc 232's scope; mint as classes surface
- **holon-rs** — STOP-4
- **HARD CUT** — no parallel API or aliases

## Verification flow

```bash
cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 | tail -5    # 5/5 PASS post-stone
cargo build --release -p wat 2>&1 | tail -5                                              # 0 errors
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                          # ≥ 827 passed; 0 failed
cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 | tail -3           # 5/5 PASS (regression guard)
cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 | tail -3    # 5/5 PASS
cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 | tail -3            # 3/3 PASS
cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 | tail -3           # 5/5 PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3     # 8/8 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"              # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                                  # empty
```

## STOP triggers (REJECTION criteria)

- **STOP-1:** unexpected compile errors NOT tracing to the 3 new verbs
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** **120 min elapsed** (predicted 30-60; symmetric pair adds minor scope)
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning above 54
- **STOP-6:** scope creep — defprotocol macro work, Atom/inner decomposer, defrecord accessors, OTHER positional decomposers (Permute/decompose etc.)
- **STOP-7:** probe still has failures (any of 7 contracts not PASS)
- **STOP-8:** existing arc 233 probes regress (the rank-up tools must STAY working)
- **STOP-9:** cascade exceeds time-box — apply partial-state-grading

Per FM 2-bis: STOP triggers are REJECTION criteria; never permission-to-defer.

## Trap-door audit (per arc 232 BRIEF discipline)

- **NO invented syntax** — use canonical inline `-> :T` per arc 108 + defrecord.wat verbatim
- **NO made-up primitive names** — verify `extract_classifier`, `Bundle/children`, `Value::holon__HolonAST` via grep before authoring
- **NO wrong arg orders** — verify against `eval_bundle_children` pattern: takes args + list_span + env + sym
- **`Bind/inner` clones the inner HolonAST** — the existing extract_classifier_inner_bundle returns `&Vec<HolonAST>`; the wat verb needs OWNED data (Option<HolonAST>). Clone the Arc<HolonAST> inside Bind to get HolonAST.
- **Probe's `(:wat::holon::Atom ...)` form** — for "bare Atom" tests (probes 2 + 4), verify this constructs `HolonAST::Atom(...)`. Per arc 225 Stone 225.1 v3: `(:wat::holon::Atom h)` is the narrow constructor accepting HolonAST input only.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no aliases or parallel verb names
- Per `feedback_inscription_immutable`: SCORE is a NEW file (`SCORE-STONE-232.0a.md`)
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification.
- The probe at `tests/probe_diagnostic_typed_entities_reflection.rs` IS the success criterion. Flip 0/5 → 5/5.
- **This is the RANK-UP DEMONSTRATION stone** — arc 233 just shipped; use the new diagnostic tools.

## Cross-references

- `docs/arc/2026/05/232-defprotocol-extend-type/DESIGN.md` — arc umbrella
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md` — predecessor (apply primitive)
- `tests/probe_diagnostic_typed_entities_reflection.rs` — FM 2-bis probe (already committed at `96bb6f4`)
- `docs/arc/2026/05/233-substrate-errors-as-values/INSCRIPTION.md` — the rank-up arc; the new tools
- `src/runtime.rs:13986` — existing extract_classifier fn (lift target)
- `src/runtime.rs:12482` — eval_bundle_children precedent
- `feedback_partial_state_grading` — discipline if STOP-3 fires
