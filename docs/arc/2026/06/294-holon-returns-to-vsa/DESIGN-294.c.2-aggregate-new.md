# 294.c.2 — `aggregate-new`: the one holder-dispatched ctor; the hologram derived in Rust

> **Status: STRIKE DRAWN — lair studied + grounded against the disk 2026-06-28 (after 294.c.1 landed `ed7ecd50`).**
> This is REMAINING-PATH steps 2+3 fused: the hologram becomes derived (not stored-canonical) and construction
> collapses to ONE primitive. Splits cleanly into c.2a (mint + route) and c.2b (annihilate the of-funcs).

## The bug (grounded this session)
Three construction paths, the hologram derived in the WRONG place (a wat macro):
- **struct** — `register_struct_methods` (Rust codegen, `runtime.rs:961`) emits a `:T` ctor whose body is
  `(:wat::core::struct-new :T p1 p2 …)`.
- **base record** — `:wat::core::defrecord` macro (`wat/Record.wat:91`) emits a `:T` ctor calling
  `(:wat::Record::of (keyword :T) [fields])`.
- **holon record** — `:wat::holon::defrecord` macro (`wat/Record.wat:130`) emits a `:T` ctor calling
  `(:wat::holon::Record::of (keyword :T) [fields] <hologram>)`, where `<hologram>` is built **inline in the macro
  expansion** (the giant quasiquote `Record.wat:157-197`): `Bind(Atom(String(class)), Bundle([Bind(Atom(String(name_i)),
  Atom(to-holon(value_i)))…]))`, the Bundle wrapped in `Result/expect "… capacity exceeded"`.

So the holon hologram **is** already derived from the fields — but the derivation logic lives in a **wat macro**, and
`:wat::holon::Record::of` takes it as a **precomputed 3rd arg** (`eval_holon_record_of`, `runtime.rs:13597`, args[2]).
That precomputed-form arg is exactly the divergence source 294.c.1 made irrelevant to identity and 294.c.2 removes.
`record_assoc_inner` (`runtime.rs:13988-14031`) already rebuilds the hologram in Rust on `assoc` — but **incrementally**
(hoist old binds, replace one), so there is **no** reusable from-scratch `build_hologram(class, names, values)` helper.

## The contract (294 DESIGN:128 — pinned)
`(:wat::core::aggregate-new :T field…)` — **varargs**, **holder-dispatched**:
- look up `:T`'s `holder` from the TypeEnv (`TypeDef::Aggregate(a)` → `a.holder`; pattern at `runtime.rs:934/1046`),
  and the field NAMES from the same `AggregateDef` (`field_names()`, as `record_assoc_inner` does at `runtime.rs:13960`);
- **Struct** → `AggregateValue::struct_(class, fields)`;
- **Record** → `AggregateValue::record(class, Arc::new(fields))`;
- **HolonRecord** → `AggregateValue::holon_record(class, fields, build_holon_hologram(class, names, values)?)` — the
  hologram derived **internally**, no precomputed arg.

The hologram derivation extracts the macro's shape into ONE Rust helper:
```rust
fn build_holon_hologram(class: &str, field_names: &[String], field_values: &[Value], span: &Span)
    -> Result<Arc<HolonAST>, EvalBreak>
{
    // capacity: mirror the macro's Result/expect — field count must fit the Bundle width bound.
    //   (find the check the `:wat::holon::Bundle` verb does; reuse it — see CAPACITY below.)
    let field_binds: Vec<HolonAST> = field_names.iter().zip(field_values).map(|(name, val)| {
        let val_holon = match to_holon_inner(val.clone(), span)? {
            Value::holon__HolonAST(h) => (*h).clone(),
            _ => unreachable!("to_holon_inner always returns holon__HolonAST on Ok"),
        };
        Ok(HolonAST::Bind(
            Arc::new(HolonAST::Atom(Arc::new(HolonAST::String(Arc::from(name.as_str()))))),
            Arc::new(HolonAST::Atom(Arc::new(val_holon))),
        ))
    }).collect::<Result<_, EvalBreak>>()?;
    let class_atom = HolonAST::Atom(Arc::new(HolonAST::String(Arc::from(class))));
    Ok(Arc::new(HolonAST::Bind(Arc::new(class_atom), Arc::new(HolonAST::Bundle(Arc::new(field_binds))))))
}
```
This shape is verified against BOTH the macro (`Record.wat:157-191`) and the assoc-rebuild (`runtime.rs:14017-14031`):
field-bind = `Bind(Atom(String(name)), Atom(<holon-of-value>))`; outer = `Bind(Atom(String(class)), Bundle(binds))`.

## CAPACITY — ONE guard, two callers (EXTRACT, never copy — builder, 2026-06-28)
The macro wraps the Bundle in `(:wat::core::Result/expect (:wat::holon::Bundle […]) "… capacity exceeded")` — so the
`:wat::holon::Bundle` *verb* capacity-checks (Kanerva width bound). **Grounded against the disk:** that check is
**inline in `eval_algebra_bundle` (`runtime.rs:15791-15822`)** — `cost = children.len()`, `budget = floor(sqrt(d))`
(`d = ctx.dim_count`), over-budget → `CapacityMode::Error` returns `Err(CapacityExceeded{cost,budget})` / `Panic`
panics. It is NOT yet a shared function.

> **The builder's law (2026-06-28): do NOT "replicate" the check — that word is "duplicate," a SECOND copy free to
> drift, the flaw-#7 disease (equality-written-twice) in a new spot.** The one-canonical-path rule: *there is never
> 1+ ways to do a thing.* So **EXTRACT** the inline check from `eval_algebra_bundle` into ONE guard,
> `fn bundle_with_capacity(children: Vec<HolonAST>, ctx: &EncodingCtx, span: &Span) -> Result<HolonAST, EvalBreak>`
> (returns the checked `HolonAST::Bundle` or raises CapacityExceeded per mode); `eval_algebra_bundle` is rewritten to
> CALL it (pure extraction, behaviour-preserving, SET-diff ∅), and `build_holon_hologram` calls **the same one** for
> its field-binds Bundle. One guard, two callers — `aggregate-new` on a too-wide holon record fails loud at
> construction (294 Q-C) through the identical code path the verb uses. STOP-trigger if the inline check can't be
> cleanly lifted (e.g. it's entangled with the verb's arg-eval — then surface the seam, don't copy).

This extraction is a small prerequisite decomplection inside c.2a (or a clean c.2a.0 if the sonnet prefers): the
capacity guard becomes a single function the moment a second caller needs it — which is exactly now.

## Decomposition
### 294.c.2a — mint `aggregate-new` + route all three emitters through it (additive; of-funcs stay)
1. **New intrinsic** `:wat::core::aggregate-new` (varargs) in `runtime.rs` — dispatch arm near the of-funcs
   (`runtime.rs:4038-4244`), holder-dispatched per the contract above; extract `build_holon_hologram`.
2. **Check side** — `aggregate-new`'s return type is the constructed `:T` (mirror `infer_record_of` /
   `infer_struct_new`; `check.rs:5570/12548/12702`). Runtime-only dispatch like `struct-new` may suffice (no scheme) —
   ground which the macros' emitted ctor needs for its `-> :T` to check.
3. **Struct codegen** — `register_struct_methods` (`runtime.rs:961`): emit `(:wat::core::aggregate-new :T p…)` instead
   of `(:wat::core::struct-new :T p…)`.
4. **defrecord macro** (`Record.wat:91`) — body → `(:wat::core::aggregate-new :T ~@fields)` (drop the `Record::of`
   wrapping + the field-extraction `let`; the bare field syms suffice).
5. **defholon::defrecord macro** (`Record.wat:130`) — body → `(:wat::core::aggregate-new :T ~@fields)`; **the entire
   hologram quasiquote (`:157-197`) DIES** — the Rust helper now derives it.
6. **Gate:** RED probe `(:wat::core::aggregate-new :T …)` constructs all three holders + the holon one measures
   (cosine(r,r)=1.0, cosine(r,r')<1.0); existing record/holon/struct construction + measurement tests stay GREEN;
   workspace SET-diff ∅. (of-funcs still registered — uncalled by generated code, called nowhere else yet → c.2b.)

### 294.c.2b — annihilate `struct-new` / `Record::of` / `holon::Record::of`
grep-confirm no callers remain (generated code now emits `aggregate-new`); migrate any `.wat`/`.rs` fixtures still
calling the of-funcs directly (fix-wat the `.wat`, hand-sub the `.rs`); unregister the 3 dispatch arms + their check
handlers; retirement-table the heads. Gate: grep shows them gone (save the retirement table); SET-diff ∅.

## Out of scope (named)
- **assoc's incremental rebuild** (`record_assoc_inner`) — could be decomplected to call `build_holon_hologram`
  from-scratch too (true single-derivation), but it already keeps parity; leave it for a follow-up unless the helper
  extraction makes it free. NOT required for c.2's gate.
- **base-record lift in `to_holon_inner`** (the "has no holon flavor" reject) — that is **294.c.3** (step 4).
- The `HolonForm`-as-stored-field → the field stays (it caches the derived hologram); making it *recomputed-on-read*
  vs *stored-derived* is moot once construction derives it and assoc rebuilds it (eager parity holds either way).

## Pairs
`REMAINING-PATH.md` (steps 2+3) · `294/DESIGN.md:128` (the `aggregate-new` contract) · `Record.wat:91/130` (the two
macros) · `runtime.rs:961` (struct codegen) · `runtime.rs:13597` (`eval_holon_record_of`, the 3-arg of-func) ·
`runtime.rs:13988` (the assoc-rebuild shape) · `NOTE-base-struct-horizon.md` (`:T`/`/new` already done by R2.3).
