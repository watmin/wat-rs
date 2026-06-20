# BRIEF — Stone P12a: `fire-rules-explain` + the `Explained {session, support}` substrate

**Executor:** one **sonnet** Shadowdancer. **No sub-agents. No `git`. No worktrees.** Do NOT run
`./target/release/wat` (orchestrator-only). `cargo test` is yours. EMBED: this brief + the DESIGN
(`DESIGN-STONE-P12a-explain-substrate.md`) — read both fully before touching code.

## The work (one paragraph)
Add an OPT-IN diagnostic fire `(:wat::rete::fire-rules-explain <session>) -> :wat::rete::Explained` that runs
the EXACT same delta fixpoint as the public `fire-rules'` but additionally records, for each derived fact, the
token that produced it (and its rule). It returns a NEW ephemeral record `Explained {session, support}` —
`session` is the same frozen Session the fast path produces, `support` is a `PersistentMap<derived-fact,
Support>` where `Support {rule, token}` carries the producing rule name and the producing `Token` (whose
`matches` IS the support chain). The fast `fire-rules'` / `fire-rules-spec` must stay **byte-for-byte
behaviorally identical** — this is purely additive. Then un-ignore the P12a probe; all 3 of its tests go green.

## Read in order (the rooms)
1. `src/rete/kernel.rs` ~**:1440–:1577** — `fire_fixpoint_delta`, the delta engine `fire-rules'` runs. The
   **production-delta loop** (~:1545–:1562) fires production nodes on new tokens: builds
   `derived = build_insert_fact(form, &tok.bindings)`, then **`if !seen.contains(&derived)`** pushes it. THIS
   `if !seen` branch is the recording seam — and it gives first-producer-wins for free. `rule_name` (~:1537)
   and `tok` are in scope. The final **beta-clear (~:1577) STAYS** — the index clones the token, so nothing
   reads beta later.
2. `src/rete/kernel.rs:1596` — `eval_fire_rules_native` (the `fire-rules'` entry; calls
   `fire_fixpoint_delta(&session, sym)`). Add `eval_fire_rules_explain` beside it.
3. `src/rete/kernel.rs` ~**:503–:550** — `token_to_value` (native `Token` → wat `:wat::rete::Token` Value) and
   the Token round-trip. The support index's native `Token`s must become wat `Token` Values via this so the wat
   walk (P12b) and the probe can read `Token/matches`.
4. `src/runtime.rs:4012` — the dispatch arm `":wat::rete::fire-rules'" => …`. Add
   `":wat::rete::fire-rules-explain" => crate::rete::kernel::eval_fire_rules_explain(...)` beside it.
5. `src/check.rs` — grep the `:wat::rete::fire-rules'` TypeScheme registration; mirror it for
   `fire-rules-explain` with return type `:wat::rete::Explained`.
6. `wat/rete.wat:124` — the `:wat::rete::Session` Record def (the shape to sit beside). `wat/rete.wat:1028` —
   `fire-rules` (the one-line wrapper pattern over the native). Add the `Explained` + `Support` Record defs near
   the other rete records, and a `fire-rules-explain` public wrapper if a wat-level wrapper is wanted (the
   native verb may suffice — match how `fire-rules'` is surfaced).

## Implementation sketch (fill it; do not invent the shape)
- **One engine, two modes** — add an optional param to `fire_fixpoint_delta`:
  ```rust
  fn fire_fixpoint_delta(session: &Value, sym: &SymbolTable,
                         mut support: Option<&mut HashMap<Value, (String, Token)>>) -> Result<Value, EvalBreak>
  ```
  At the `if !seen` branch: `if let Some(idx) = support.as_deref_mut() {
      idx.entry(derived.clone()).or_insert_with(|| (rule_name.to_string(), tok.clone())); }`.
  `eval_fire_rules_native` passes `None` (zero behavior change — verify the differential).
- **The explain entry:**
  ```rust
  pub(crate) fn eval_fire_rules_explain(args, list_span, env, sym) -> ... {
      // eval the one Session arg (mirror eval_fire_rules_native's arg handling)
      let mut idx: HashMap<Value, (String, Token)> = HashMap::new();
      let session_out = fire_fixpoint_delta(&session, sym, Some(&mut idx))?;
      // build support: PersistentMap<fact, Support{rule, token_to_value(token)}>
      // build Explained{ session: session_out, support: <pm value> }
  }
  ```
- **`Explained` + `Support`** as wat Records (rete.wat), constructed positionally in Rust the way other rete
  records are built (grep how `Element`/`Token` Values are constructed — `token_to_value` shows the pattern).

## Blast radius (bounded)
- `src/rete/kernel.rs` — the `support` param on `fire_fixpoint_delta` + the `if !seen` recording + the new
  `eval_fire_rules_explain` + the `Explained`/`Support` Value construction. The fast path's behavior unchanged.
- `src/runtime.rs` — one dispatch arm. `src/check.rs` — one TypeScheme (mirror fire-rules').
- `wat/rete.wat` — `Explained` + `Support` Record defs (+ optional `fire-rules-explain` wrapper). **Additive
  only** — NO change to `Session`, `fire-rules'`, `fire-rules-spec`, `fire-once'`, or any existing path.
- `tests/probe_arc278_P12a_explain_substrate.rs` — remove the 3 `#[ignore]` lines (only those).
- **NOT** the wat `explain` walk (P12b), **NOT** `Why`/`WhyVia` (P12b), **NOT** `:met` (P12c), **NOT** the base
  `Session` type, **NOT** EDN round-trip / `from-edn` for `Explained`.

## STOP triggers (halt and surface — do not improvise)
1. **STOP if `Session` would need a new field.** Contract is the `Explained` return type; `Session` is
   untouched. If something forces a Session field, surface it.
2. **STOP if the fast `fire-rules'` / `fire-rules-spec` differential goes RED** (deep-cascade +
   `probe_arc278_P4a`/`P4c` + `P2`). This stone is additive; the `None`-param path must be identical. The
   differential is the guard.
3. **STOP if recording the index needs anything beyond cloning data already in hand** (`rule_name`, `tok`) at
   the `if !seen` branch — no new RHS eval, no second production pass.
4. **STOP if `fire_fixpoint_delta` cannot take the param without a near-total rewrite.** It should be a
   one-param + one-branch change. A copy of the 380-line engine is a differential-drift hazard — surface it.
5. **STOP if making the probe green needs EDN round-trip for `Explained`** — it's ephemeral; the probe never
   serializes it.

## Acceptance (the probe — un-ignore all 3)
`cargo test --release -p wat --test probe_arc278_P12a_explain_substrate` → **3 passed**:
- `fire_rules_explain_preserves_the_closure` (ColdAndWindy closure via `Explained/session` == 1),
- `support_index_has_an_entry_per_derived_fact` (`PersistentMap/length` of `Explained/support` == 2),
- `support_tokens_carry_their_full_chains` (sum of `Token/matches` lengths over `Explained/support` values == 3).

## Prior comparable (copy the shape)
- `eval_fire_rules_native` (kernel.rs:1596) + its dispatch (runtime.rs:4012) — the verb-registration shape.
- `token_to_value` (kernel.rs:503) — native→wat Value record construction.
- `fire-rules` / `retract` (rete.wat:1028/:1040) — the wat record-construction + wrapper shapes.
