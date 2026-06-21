# BRIEF — seq-container registry strike 4: depth fix (exhaustive `match container`)

**Model:** sonnet. **cwd:** `/home/watmin/work/holon/wat-rs/` (verify `pwd` first; reject any
`.claude/worktrees/` path, re-cd, use `git -C /home/watmin/work/holon/wat-rs`). **Read the DESIGN first:**
`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-seq-container-registry-strike4-depth.md`.

## The work, in one paragraph

The seq-container registry's per-op runtime dispatch currently classifies via `SeqContainer::of_value(&v)` and
then does `match v { Value::Vec(..) => …, _ => unreachable!() }`. That inner match is over **`Value`** with a
**catch-all `_`** — so adding a new container variant compiles clean and panics at runtime (a partial impl is
representable). Fix: replace each inner `match` so it dispatches over the **closed `SeqContainer` enum,
exhaustively, with NO `_` arm**. Then adding a `SeqContainer` variant breaks every dispatch site at compile time.
**Behavior must not change** — same helpers, same error messages, same accepted sets. This is a pure
structural/compile-forcing refactor (the Phoenix rising; behavior-preserving).

## The pattern (apply uniformly to every site)

BEFORE (example — `eval_conj`, `runtime.rs:12442`):
```rust
match SeqContainer::of_value(&arg0_val) {
    Some(container) if container.has_append() => {
        match &arg0_val {
            Value::Vec(_)                        => vector_conj_inner(&arg0_val, &arg1_val),
            Value::wat__std__HashSet(_)          => hashset_conj_inner(&arg0_val, &arg1_val),
            Value::wat__core__PersistentVector(_)=> persistentvector_conj_inner(&arg0_val, &arg1_val),
            Value::wat__core__List(_)            => list_conj_inner(&arg0_val, &arg1_val),
            _ => unreachable!("…has_append"),     // ← CATCH-ALL: the bug
        }
    }
    Some(_) | None => Err(type_mismatch(OP, &arg0_val)),
}
```
AFTER (exhaustive over the closed enum, keep the capability gate as the single source of truth):
```rust
match SeqContainer::of_value(&arg0_val) {
    None => Err(type_mismatch(OP, &arg0_val)),                       // not a seq container
    Some(container) if container.has_append() => match container {   // exhaustive over the 6 — NO `_`
        SeqContainer::Vector           => vector_conj_inner(&arg0_val, &arg1_val),
        SeqContainer::PersistentVector => persistentvector_conj_inner(&arg0_val, &arg1_val),
        SeqContainer::List             => list_conj_inner(&arg0_val, &arg1_val),   // PREPEND — keep distinct
        SeqContainer::HashSet          => hashset_conj_inner(&arg0_val, &arg1_val),
        // capability gate excludes these — NAMED arm (not `_`), genuinely dead, still compiler-forced:
        SeqContainer::Tuple | SeqContainer::WatAstList =>
            unreachable!("has_append() gate excludes Tuple/WatAstList"),
    },
    Some(_) => Err(type_mismatch(OP, &arg0_val)),                    // has_append()==false
}
```

**Rules for every site:**
1. The inner match is `match container { … }` over `SeqContainer` — **exhaustive, no `_` / no wildcard.**
2. Keep the existing capability gate (`if container.X()`) — it stays the single source of truth (the checker
   reads the same `X()`; do NOT introduce a second accepted-set the checker can drift from).
3. Containers the gate **excludes** get a **named** arm `unreachable!("X() gate excludes …")` — genuinely dead
   (the gate already filtered them), but a named arm (not `_`) so a future variant still breaks the match.
4. For supported containers, call the **exact same helper** the old arm called (or inline the same body). For
   the inlined positional accessor (no helper), extract with `let Value::Vec(items) = &v else {
   unreachable!("of_value⇒Vector") };` then the identical body.
5. Nested `WatAstList` match (`match &*ast { WatAST::List(..) => .., _ => unreachable!() }`) stays as-is — that
   `_` is over `WatAST`, not `Value`, and is the right shape.
6. **Zero behavior change** — identical error variants, messages, helper calls.

## Rooms (read in order; all 11 sites, same transform)

- `src/runtime.rs:10961` — positional accessor (`first`/`second`/`third`, `indexable`). Inlined; use let-else.
  Catch-all at `:11015` (`:11012` is the WatAST nested `_`, leave it).
- `src/runtime.rs:12442` — `eval_conj` (`has_append`). Helpers exist. Catch-all `:12454`.
- `src/collection/eval.rs:1435` — `eval_rest` (`has_tail`). Catch-all `:1497` (`:1478` WatAST nested, leave).
- `src/collection/eval.rs:761` — `vector_concat_inner` (`mappable`; TWO `of_value` — left `:761`, right `:764`).
  Catch-all `:783` ("matching container kinds"). Convert the dispatch to `match container`; keep the
  left/right-kind-match logic behavior-identical.
- `src/collection/transform.rs` — 7 HOF sites (`mappable`): `:44, :112, :155, :312, :370, :426, :481`;
  catch-alls `:58, :127, :170, :328, :384, :441, :517`. (map / filter / foldl / foldr / reverse / take / drop.)

## Verify (in order)

1. `cargo build --release` — green.
2. **Compile-forcing proof (the recon differential):** temporarily add `ProbeDummy,` to the `SeqContainer` enum
   (`seq_container.rs`), `cargo build` — confirm it now errors at **every one of the 11 dispatch sites** (plus
   the 4 capability methods). BEFORE this strike it errored only at the 4 capability methods. Record the site
   count in the SCORE. **Then remove `ProbeDummy`** and confirm green.
3. Floors: `cargo test --release` (lib + tests) — the collection suite + `probe_seq_container_parity` +
   `probe_seq_container_registry` + `probe_first_bare_accessors` all green, **no count regression** vs HEAD.
4. `cargo clippy --release` — no new warnings.

## STOP triggers (reject + report; do NOT improvise)

- **STOP-1:** if any site can't be made exhaustive without a `_` (e.g. an op matches `Value` variants that
  aren't in `SeqContainer`) — STOP and report the site. Do not leave a `_` "just for that one."
- **STOP-2:** if making a site exhaustive would **change behavior** (different error, different helper, a
  container newly accepted/rejected) — STOP and report. This strike is behavior-preserving only.
- **STOP-3:** if a helper's signature or a capability method would need to change to do this — STOP; that's out
  of scope (this strike only rewrites the dispatch shape).

## Out of scope (affirmative cut)

- Routing `get`/`contains?`/`length`/`empty?` — strike 5. MapContainer — strike 6. The checker side
  (`of_type` + capability methods, `check.rs`) — unchanged this strike. No new types, no helper rewrites.
