# DESIGN — `<K,V>` unexpressible ANYWHERE, minted names included

> *"make `<K,V>` unexpressable — `--check` should refuse any attempt to use this syntax — make it
> illegal. defservice must blow up with the appropriate error when someone causes the macro to expand
> into the illegal syntax — that's your test that you made the syntax illegal, yes?... unless those
> callers fail on illegal syntax, we've failed."* — the builder, 2026-08-23

## The hole this closes

The earlier strike walled the **reader**. A name that is never read has no reader to refuse it:

```
WRITTEN  :wat::core::Vector<wat::core::i64>          → REFUSED at the lexer
MINTED   string::concat + keyword/from-string        → ACCEPTED, silently
MINTED   "my::Map<K,V>"                              → ACCEPTED, comma and all
```

Demonstrated from ordinary user code — this was never a `defservice` quirk. The language had two
name-spaces: written (walled) and minted (unwalled), and only two stdlib verbs stood in the second.

## The wall

`keyword/from-string` and `keyword-node` now refuse a minted name carrying an angle type-head, using
**the same predicate `crates/wat-reader/src/lexer.rs` uses on source** — `<` preceded by an identifier
character. An operator `<` follows `::` or leads its token (`:wat::core::<`, `<-`, `<=`) and never
matches, so nothing that survived the lexer wall dies here either.

One shared message (`angle_minted_name_reason`), because two doors with two wordings is the shape this
whole arc has been killing.

## THE TEST IS THAT THE CALLERS BLOW UP

Measured, and this is the acceptance criterion the builder named — not a green floor, a **caller
failing**:

```
wat/cache.wat:195      ← a defservice CALL SITE
  macro :wat::service::defservice — program body eval failed
  wat/service.wat:942  ← the exact minting expression
    keyword/from-string refused "wat::cache::Cache::Op<K,V>"
```

`--check` refuses identically — the wall fires during macro expansion, which `--check` runs.

Cascade: **3034 of 4893** tests fail, all masked behind the first minting site. That is the
substrate-as-teacher waterfall, and it is the deliverable, not a crisis.

## ★ The measurement that decides the repair — the explicit type application is INERT

The obvious repair is "make `defservice` emit a type-application FORM instead of a name." **Measured:
it does not need to emit one at all.**

The checker's bespoke arm (`check.rs:5159`) parses the minted suffix and binds the surface method's
type params. Replacing every successfully-parsed argument with a fresh inference variable — i.e.
discarding the explicit application entirely — leaves the floor at **4893/4893 green.**

Three independent measurements agree:

```
source-level turbofish deleted        → --check clean, runs, returns "hello"
nested args (100,800 per floor run)   → ALREADY silently inferred since ③; green
ALL args discarded                    → 4893/4893 green
```

**Inference does the whole job.** The suffix has been decoration.

⚠ **And the honest bound: green proves nothing contradicted it.** A corpus that never exercises a
case where explicit application and inference DIVERGE will pass either way. That is
`[[feedback_a_green_test_can_prove_nothing]]`, and it is why the brief makes constructing a divergence
case the rider's first job — if one cannot be built, that IS the finding that licenses deletion.

## How ③ turned a defensive arm into the primary path, silently

Worth recording, because it is the reason none of this surfaced on its own.

`Err(_) => { mapping.insert(tp.clone(), fresh.fresh()); }` was written **2026-06-28** (arc 293.4e-pre.ii),
when `parse_type_expr` still accepted angle forms — a rare fallback for genuinely unparseable text.
**③ walled `parse_type_expr` on 2026-08-23** (`ab52b7188`; verified: no such wall in `ab52b7188^`).
From that commit, every NESTED minted argument — `Cache::Op<K,V>` inside
`Locus/launch<Cache::Op<K,V>,…>` — began failing that parse and taking the fallback.

Nothing went red, because the fallback fabricates a type variable rather than reporting. Making it
loud fails **2820 of 4893** tests: that is how much of the floor traverses a path whose type
application had silently become a no-op.

★ **A defensive arm that swallows an error and continues makes a wall's blast radius invisible.** ③
was measured, censused, and floored green, and still moved 2820 tests onto a degraded path without a
single signal.

## What ships

1. The wall at both minting doors (**already in the tree**, uncommitted — it cannot be committed alone
   because it is red by construction).
2. The minting sites stop building angle names — `wat/service.wat:942` and `:2375`,
   `wat/core.wat`'s `keyword/of`, and whatever the waterfall surfaces behind them.
3. A negative control proving the refusal, and its positive twin proving operator names survive.
4. The divergence probe, or the finding that none can be constructed.

## Out of scope, affirmatively cut

- **The 48 angle PARSERS** (`split_type_params`, `split_name_and_type_params`,
  `split_method_name_type_params`, `canonical_callable_name`, `check.rs:5159`'s arm). Once nothing
  mints, they are genuinely dead — but the floor must be green first to say so, and this stone's job
  is the wall and the repair. Tracked as `STONE-purge-the-angle-machinery`, briefed on green.
- **`:wat::core::keyword/of`'s retirement.** It is a stdlib macro whose entire purpose is minting the
  retired syntax; it has one caller. Killing it belongs with the purge, not the wall.

## The four questions

- **Obvious?** YES. One predicate, both doors a name can arrive through. The error names the macro,
  the minting line, the offending string, and the form to emit instead.
- **Simple?** YES. The wall is one predicate and one message, shared. The repair deletes rather than
  builds — the measurement says nothing needs to replace the suffix.
- **Honest?** YES, and this is the axis that failed before: the language claimed `<K,V>` was illegal
  while two stdlib verbs minted it 67,933 times per floor run. A rule the substrate itself breaks is
  not a rule.
- **Good UX?** YES. A macro author who concatenates a type name now learns at expand time, at their
  own call site, with the form spelled out — instead of shipping a name that no reader could have
  produced and that a second parser silently mis-binds.
