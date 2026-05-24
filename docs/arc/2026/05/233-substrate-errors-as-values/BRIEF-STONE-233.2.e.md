# BRIEF — Arc 233 Stone 233.2.e — AST-derived provenance (Literal + SymbolBound)

## What we're doing

Populate the two latent `Provenance` variants — `Literal { span }` and `SymbolBound { binding_span, head_span }` — that currently exist in the enum with ZERO populate sites. After this stone, every Value flowing through eval_inner carries meaningful provenance; errors gain source-coordinates context; let-bound values name their binding site. This is the **diagnostic-richness payoff** that arc 233 was opened for.

**The cascade (7 phases per sub-DESIGN):**

1. **Literal{span} at eval_inner literal arms** — `src/runtime.rs:4496+`:
   ```rust
   // Before
   WatAST::IntLit(n, _) => Ok(TrackedValue::from(Value::i64(*n))),
   // After
   WatAST::IntLit(n, span) => Ok(TrackedValue::new(
       Value::i64(*n),
       Provenance::Literal { span: span.clone() },
   )),
   ```
   Same shape for FloatLit, BoolLit, StringLit, Vector, Keyword (the `:wat::core::nil` and `:None` special cases).

2. **`BoundEntry` struct + EnvCell shape flip** — `src/runtime.rs:~1267`:
   ```rust
   pub struct BoundEntry {
       pub value: TrackedValue,
       pub binding_span: Span,
   }
   struct EnvCell {
       bindings: HashMap<String, BoundEntry>,  // was HashMap<String, TrackedValue>
       parent: Option<Environment>,
   }
   ```

3. **env.lookup signature flip** — constructs SymbolBound at boundary:
   ```rust
   pub fn lookup(&self, name: &str, head_span: &Span) -> Option<TrackedValue> {
       if let Some(entry) = self.inner.bindings.get(name) {
           Some(TrackedValue::new(
               entry.value.value().clone(),
               Provenance::SymbolBound {
                   binding_span: entry.binding_span.clone(),
                   head_span: head_span.clone(),
               },
           ))
       } else {
           self.inner.parent.as_ref().and_then(|p| p.lookup(name, head_span))
       }
   }
   ```
   The 4 known lookup call sites (runtime.rs:4197, 4588, 4655, plus parent recursion) add `head_span` argument.

4. **LetBinding shape change** — `src/runtime.rs:~6090`:
   ```rust
   enum LetBinding<'a> {
       Single { name: String, name_span: Span, rhs: &'a WatAST },
       Destructure { names: Vec<(String, Span)>, rhs: &'a WatAST },
       StructDestructure { field_names: Vec<(String, Span)>, rhs: &'a WatAST },
   }
   ```
   `parse_let_binding` extracts the span from `WatAST::Symbol(ident, span)` it currently discards.

5. **bind_let_binding propagates binding_span**:
   ```rust
   LetBinding::Single { name, name_span, rhs } => {
       let tv = eval_inner(rhs, scope, sym)?;
       Ok(scope.child().bind(name, name_span, tv).build())
   }
   ```
   `EnvironmentBuilder.bind(name, binding_span, tv)` — accepts binding_span; constructs BoundEntry.

6. **eval_let_tail flip** — `src/runtime.rs:4315`:
   ```rust
   fn eval_let_tail(...) -> Result<TrackedValue, RuntimeError>  // was Result<Value, ...>
   ```
   Closes the 233.2.k honest delta; mirrors 233.2.j eval_let pattern. Callers update via `.value_owned()` if they want bare Value.

7. **ValueSnapshot::Display smoke verify** — the impl already renders Literal + SymbolBound per existing code at runtime.rs:1780-1810. Probe 5 verifies the rendering works end-to-end; sonnet smoke-checks no regression.

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.e.md`** (commit `12bb8b1`) — sub-DESIGN; six decisions inscribed (Environment storage shape, env.lookup signature, LetBinding shape change, literal-arm scope, eval_let_tail flip, recv/try-recv honest delta). **Authoritative for shape decisions.**

2. **`tests/probe_stone_233_2_e_ast_derived_provenance.rs`** (commit `97fa595`) — FM 2-bis probe. 5 contracts. Pre-stone state: 1/5 PASS (probe 5 Display smoke; probes 1-4 FAIL with "got Unknown"). **The probe IS the success criterion** — sonnet flips to 5/5.

3. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.k.md`** — Option A Environment storage pattern this stone extends (BoundEntry is the natural extension of the post-233.2.k HashMap<String, TrackedValue>).

4. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.l.md`** — the sealed substrate this stone builds on (#[wat_value] proc-macro forbids future wrapping variants; arc 233.2.e safe to populate new provenance shapes).

5. **`src/runtime.rs:1780-1810`** — existing `ValueSnapshot::Display` impl already renders all 4 Provenance variants. Verify post-stone the rendering surfaces Literal + SymbolBound spans correctly.

6. **`docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15** — substrate-as-teacher pattern (small BRIEF; cargo enumerates).

## Implementation surface

- 6-7 literal-arm constructor swaps (Phase 1)
- BoundEntry struct mint + EnvCell shape flip (Phase 2)
- env.lookup signature flip + 4 call sites updated with head_span (Phase 3)
- LetBinding enum 3-variant shape change + parse_let_binding span-extraction (Phase 4)
- bind_let_binding span propagation (Phase 5)
- eval_let_tail Result<TrackedValue> flip + callers (Phase 6)
- ValueSnapshot Display smoke (Phase 7 — likely no code change)

## What does NOT change

- **Provenance enum** — variants exist; only populate sites change
- **TrackedValue struct + methods** — unchanged
- **ValueSnapshot::of_tracked / of** — unchanged
- **eval / eval_inner signatures** — unchanged (both already TrackedValue-typed)
- **5 producers** — RuntimeBuilt provenance still attached; no regression
- **Value enum** — no variants added or removed (sealed by 233.2.l)
- **Other arc 233 probes** — all stay GREEN
- **holon-rs** — NOT touched
- **HARD CUT** — no parallel API; no deprecation aliases

## Out of scope (affirmative scope-bounding per sub-DESIGN)

- **Chained provenance** (RuntimeBuilt → SymbolBound when let-bound producer result) — Provenance enum is flat; chain needs new variant; not load-bearing today. SymbolBound REPLACES stored RuntimeBuilt per Decision 2.
- **Carrier-level recv/try-recv provenance restoration** — permanently lost per 233.2.j Phase 6; indirect coverage via let-binding SymbolBound (Decision 6 honest delta)
- **ValueSnapshot::of(&Value) sweep to of_tracked** — incremental migration per 233.2.k; out of scope
- **Destructure-source per-element provenance** (e.g., tracing slot `a` back to the producing tuple's element-span) — slot gets binding_span pointing at LHS pattern; deeper tracing out of scope
- **List call-form provenance** (whole-call-span attached to dispatch result) — dispatch fn determines result provenance (RuntimeBuilt or SymbolBound or composed). Not a "literal" — Decision 4.
- **holon-rs** — STOP-4
- **HARD CUT** — no aliases

## Verification flow

```bash
cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 | tail -5    # 5/5 PASS post-stone
cargo build --release -p wat 2>&1 | tail -5                                              # 0 errors
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                          # ≥ 827 passed; 0 failed
cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 | tail -3            # 3/3 PASS
cargo test --release -p wat-macros 2>&1 | tail -3                                        # all pass (incl. trybuild)
cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 | tail -3           # 5/5 PASS
cargo test --release --test probe_stone_233_2_j_producer_migration 2>&1 | tail -3        # 5/5 PASS
cargo test --release --test probe_eval_signature_returns_tracked_value 2>&1 | tail -3    # 3/3 PASS
cargo test --release --test probe_tracked_value_mint_contract 2>&1 | tail -3             # 6/6 PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3     # 8/8 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"              # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                                  # empty
```

## STOP triggers (REJECTION criteria)

- **STOP-1:** unexpected compile errors NOT tracing to the cascade
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** **180 min elapsed** (per sub-DESIGN calibration: 90-150 Mode A; 180 STOP)
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning above 54
- **STOP-6:** scope creep — chained provenance; recv/try-recv carrier restoration; ValueSnapshot::of sweep; deeper destructure tracing; List-call-form provenance
- **STOP-7:** probe still has failures post-stone (any of 5 contracts not PASS)
- **STOP-8:** existing arc 233 probes regress
- **STOP-9:** cascade exceeds time-box — apply partial-state-grading per `feedback_partial_state_grading`

Per FM 2-bis: STOP triggers are REJECTION criteria; never permission-to-defer.

## Trap-door audit

- **LetBinding shape change cascades through parser** (parse_let_binding) — verify all WatAST::Symbol(ident, span) sites in let-binding-context extract the span; don't drop it
- **env.lookup head_span recursion** — parent chain lookup must pass head_span through; verify the recursive call has the arg
- **BoundEntry value clone** — env.lookup returns owned TrackedValue (constructed with SymbolBound); the stored entry's value is cloned (existing TrackedValue is cheap-clone via Arc internals — verify)
- **SymbolBound REPLACES stored RuntimeBuilt** — per Decision 2, this is intentional. Document any case where chained provenance would have been valuable but is structurally absent (out of scope; future work)
- **eval_let_tail flip ripple** — its callers (likely 2-5 sites) update via `.value_owned()` — mechanical
- **The Literal{span} rendering already exists** at runtime.rs:1780-1810 — verify Phase 7 by smoke check; sonnet shouldn't need to change Display impl
- **Test runtime `Span` literal syntax** — probe 5 uses `wat::span::Span { file: ..., line: 7, col: 13 }` struct literal. Verify Span has those public fields (it does, per the Display impl reading `binding_span.file/line/col`)

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no deprecation aliases for env.lookup or LetBinding shapes
- Per `feedback_inscription_immutable`: SCORE is a NEW file (`SCORE-STONE-233.2.e.md`)
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification.
- This is the **DIAGNOSTIC-RICHNESS PAYOFF** stone — closes arc 233's original thesis ("errors are remarkable"). After this stone: every Value carries meaningful provenance.
- The probe at `tests/probe_stone_233_2_e_ast_derived_provenance.rs` IS the success criterion. Flip 1/5 → 5/5.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.e.md` — sub-DESIGN (commit `12bb8b1`)
- `tests/probe_stone_233_2_e_ast_derived_provenance.rs` — FM 2-bis probe (commit `97fa595`)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.k.md` — Environment storage precedent
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.l.md` — sealed substrate this builds on
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15 — substrate-as-teacher
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis — probe-before-BRIEF
- `feedback_partial_state_grading` — discipline if STOP-3 fires
- `scratch/FAILURE-ENGINEERING.md` — the doctrine driving the chain
