# DESIGN — seq-container registry strike 4: the depth fix (inner dispatch compiler-forced)

**Status:** STRIKE-READY draft (contract decision pinned below; awaiting go before sonnet).
**Arc:** 278 · the narrow-waist registry · strike 4 of {4 depth, 5 seq-coverage, 6 MapContainer}.
**Prereq for:** strike 5 + 6 inherit this strike's dispatch pattern. It lands first, on known-green ops.

## Why (the guarantee, not a bug)

The goal (R14 / builder): *the next primitive we introduce isn't allowed a partial/wrong impl* — drift
**unrepresentable**, not merely caught. Strikes 1-3 routed the seq ops through `SeqContainer`, but the registry
forces only **"what can it do"** (the capability methods `match self` over the enum). It does **not** force
**"do the thing."**

### Recon evidence (2026-06-20, re-runnable)

Add a throwaway `ProbeDummy` variant to `SeqContainer` and `cargo build`:

```
errors ONLY at:  indexable (seq_container.rs:147) · has_tail (:161) · has_append (:177) · mappable (:196)
errors NOWHERE at: of_value/of_type (have `_ => None`)  ·  any inner dispatch in runtime.rs
```

So a new container variant compiles clean through every op's actual behavior. The inner dispatch sites
(`runtime.rs:11012, 11015, 12454`, and the HOF/rest sites) are `match &arg0_val { Value::Vec => …, _ =>
unreachable!() }` — they match **`Value`**, so a new **enum** variant never touches them. A `has_append`-true
container with no inner arm compiles and panics at runtime. **`conj` itself can be given a partial impl today.**

## What ships

Every seq-op inner dispatch becomes **exhaustive over the closed `SeqContainer` set** (no `_`, no
`unreachable!`). Adding a variant then breaks **every** dispatch site at compile time → the behavior arm cannot
be forgotten. Retrofit the already-green routed ops: positional accessors (`first`/`second`/`third`), `rest`,
`conj`, and the HOF family (`map`/`filter`/`foldl`/`foldr`/`reverse`/`take`/`drop`/`concat`).

No observable behavior changes — the accepted sets and error messages stay byte-identical (this is the
behavior-preserving Phoenix rising, R14). The floors + `probe_seq_container_parity` + the full collection suite
are the net.

## THE ONE CONTRACT DECISION — `match container` over the closed enum (Form 1)

**Decided Form 1** after an architecture audit (2026-06-20; see *Audit weigh* below). Each op's inner dispatch
becomes exhaustive over the **closed `SeqContainer` enum** (no `_`, no catch-all), reusing the existing,
**semantically-named** helpers:
```rust
match container {                               // container: SeqContainer — exhaustive over the 6
    SeqContainer::Vector           => vector_conj_inner(&arg0_val, &arg1_val),    // append
    SeqContainer::PersistentVector => persistentvector_conj_inner(&arg0_val, &arg1_val),
    SeqContainer::List             => list_conj_inner(&arg0_val, &arg1_val),      // PREPEND (divergent — explicit on purpose)
    SeqContainer::HashSet          => hashset_conj_inner(&arg0_val, &arg1_val),   // insert
    SeqContainer::Tuple | SeqContainer::WatAstList => Err(type_mismatch(OP, &arg0_val)), // ∅ N/A
}
```
Adding a container variant breaks **every** such `match container` at compile time → the behavior arm cannot be
forgotten. (The `let Value::X = &v else { unreachable!() }` extraction inside each helper is genuinely dead —
`of_value` paired them — *unlike* the current `_ => unreachable!()` catch-all, which silently swallows new
variants. That catch-all is the bug this strike removes.)

**Why not `match v` (over `Value`)?** `Value` has hundreds of variants; an op can never enumerate them, so
`match v` *must* carry a `_` — which defeats exhaustiveness-forcing. The forcing can only come from matching the
**closed** `SeqContainer` enum. This is the heart of the fix.

**Why not Form 2 (data-carrying `SeqRef`)?** Rejected after the audit:
- `WatAstList` is `Value::wat__WatAST(Arc<WatAST>)` wrapping an AST node; its `first` returns a *wrapped AST
  node*, not a plain `Value` (`runtime.rs:11004`). A `SeqRef::WatAstList(&[Value])` would misrepresent it.
- The inner payloads are heterogeneous (`Arc<Vec>` / `Arc<LinkedList>` / `rpds::VectorSync` / `Arc<WatAST>`) and
  the semantics diverge (`List/conj`=prepend vs `Vector/conj`=append). **Explicit named-helper arms keep the
  divergence visible and greppable** — the honest shape for a heterogeneous family. A uniform data/ref
  abstraction would invite silently unifying distinct semantics.
- Smaller blast radius: reuses the tested `*_inner` helpers verbatim.

### Audit weigh (the reversal, recorded honestly)

A read-only architecture audit (general-purpose sonnet) graded Pattern A (exhaustive enum) vs Pattern B (uniform
capability traits / the `defprotocol`-flavored option). **Verdict: keep Pattern A** — and I weighed it against
the disk:
- **Right (kept):** Pattern B (uniform traits) is a correctness hazard here — heterogeneous payloads +
  divergent op semantics + `WatAstList` not being a real container. And `defprotocol` is our **runtime/wat-level**
  open-dispatch (symbol-table `extend:P:T`), the right idiom for *user* types — but these ops are **type-projective
  Rust intrinsics** (`get : Vector<T> → Option<T>`, `OP-PLACEMENT.md`), which can't be wat protocol methods. So
  the enum+capability registry is the correct Rust-internal shape. This **reverses an earlier trait lean.**
- **Wrong (corrected):** the audit claimed the `unreachable!()` was "defensive, never reachable; adding a `Value`
  variant already forces the inner arm." FALSE — `runtime.rs:11015` is `_ => unreachable!()`, a catch-all that
  swallows a new variant (compiles, panics at runtime). The depth hole is real; this strike closes it.

### Four-questions (hard constraint = *partial/wrong impl unrepresentable*; Pattern A assumed, Form 1 vs Form 2)

| | Form 1 (`match container`, named helpers) | Form 2 (`SeqRef` data-carrying) |
|---|---|---|
| **Obvious?** | YES — explicit arm per container, semantics named | YES, but `WatAstList` arm misrepresents the wrapped-AST return |
| **Simple?** | YES — reuse tested helpers, smallest change | NO — rewrites ~15-20 helper sigs; awkward for `WatAstList` |
| **Honest?** | YES — divergent semantics (prepend vs append) stay explicit/greppable | PARTIAL — uniform shape invites hiding divergence |
| **Good UX?** | YES — add a variant → every match breaks (compile-forced) | YES |
| **Hard constraint** | partial unrepresentable ✓ (closed-enum match); wrong = explicit named helper, review-visible | ✓ but at the cost of misrepresenting heterogeneous members |

**Form 1 wins.** For a *heterogeneous* family, explicit per-container dispatch over the closed enum is both the
forcing mechanism AND the honest one. Form 2's "forces the right arm" advantage is outweighed by its
misrepresentation of `WatAstList` and the larger rewrite.

## Disconfirming evidence + permanent guard

- **Disconfirming (recon differential, documented above):** BEFORE — a variant errors at 4 capability methods
  only. AFTER — it errors at 4 capability methods **+ every op dispatch site** (the count is the proof the
  dispatch is now compiler-forced). This is a structural strike; the proof is the compile-error delta, not a
  runtime RED test (R7-Value-top precedent).
- **Permanent guard:** (1) the compiler (exhaustive `match` over the closed enum, forever); (2) a new
  `tests/probe_seq_container_dispatch.rs` reachability test — every `SeqContainer` variant is produced by
  `of_value` for some `Value` and reaches its claimed-capability ops; (3) the existing
  `probe_seq_container_parity` (checker≡runtime) + the floors.

## Out of scope (strikes 5 + 6 — affirmatively cut, tracked in BACKLOG)

- **`get`/`contains?`/`length`/`empty?` seq arms** → strike 5 (they inherit this dispatch pattern).
- **MapContainer registry + the map arms** → strike 6.
- **`of_value`/`of_type` `_ => None` classification-forcing** — fundamentally limited (`Value` has hundreds of
  variants, so the classifier must carry a catch-all). The chain still holds: adding a container's *capabilities*
  requires the enum variant, which (post-strike) forces every dispatch. The reachability test (guard #2) covers
  "every variant is actually produced." Not a hole this strike can close further; noted, not deferred-vaguely.
