# DESIGN — arc 109, the ③ prerequisite: a type's IDENTITY is not its rendered string

> **Status: DRAFT for the builder's ruling. No rider flies on this until it is ruled.**
> Written 2026-08-21 from the ②-iii application-and-revert. The measurement is in
> `NOTE-2iii-is-blocked-the-angle-string-is-the-type-identity.md`; read it first.

## The finding, in one line

`②-iii` shipped correctly, floored **3030 red**, and reverted — because in three separate
subsystems the substrate uses the **rendered string `Head<A,B>` as a type's identity**, not as one
spelling of it. Migrate the corpus and every one of those comparisons silently stops matching.

Two of the three were narrow parse-slot gaps and are already closed (`wat_source_derive`,
`defsurface`). The third is not a gap; it is a representation choice, and it is what ③ actually
waits on.

## Where the identity lives today

```
src/types.rs   register_subtype(child, parent)   stores the string VERBATIM →
                                                 the key IS ":wat::core::Seqable<T>"
src/types.rs   transport_satisfier_heads         format!("{fq}<T>"), format!("{fq}<Xt>")
src/types.rs   satisfies_bare_surface            format!("{surface}<")  — a PREFIX match
wat/service.wat  fqdn-tp / proto-tp              "<K,V>" carried as a STRING, re-attached
                                                 as "{b}::Op{p}", "{b}::GetRequest{p}", …
wat/service.wat  the :peers check                builds "wat::kernel::Peer<{r},{o}>" and
                                                 COMPARES it to the declared :ephemeral type
```

The floor named the last one verbatim: *":peers declares surface :wat::query::Store but no
:ephemeral field is typed :wat::kernel::Peer<wat::query::Store::Op,…>"* — one side built by
interpolation in the angle form, the other read from the corpus. **The recurring class this arc has
now named four times.** `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

★ And `defservice` does not merely READ the angle form — it **EMITS** it, at every expansion. So
even a fully migrated corpus regrows the angle spelling on every service. That is why the ②-iii
dry-run's only code-position leftovers were nine `string::interpolate` sites in `wat/service.wat`,
two in `wat/bracket.wat`, one in `wat/fix.wat`.

## ⚠ The population is UNMEASURED, and the census must be the compiler

A grep for `format!("{}<{}>", …)` and for `split_once('<')` returns ~19 and ~21 hits in `src/`, and
~30 `<`/`>` string operations in `wat/service.wat`. **Those numbers are NOT in this design as a
scope**, because the pattern cannot tell a type-identity concatenation from
`format!("<{}>", inner.type_path)` rendering a RustOpaque, or from a `<{}>` placeholder in a doc
string. A list built from what the pattern happens to match is the same instrument that certified
the consumption wall would find zero violations and missed three.
`[[feedback_impose_the_check_and_read_the_screams]]`

**The honest census is the type system.** Give the identity a TYPE that a bare `String` cannot be
mistaken for, and every site that concatenates or splits one becomes a compile error naming itself.
That is the strike's method, not a preliminary survey.

## The shape, stated so the four questions have something to judge

A type reference has a **base name** and an ordered **argument list**. Today it is
`String("Head<A,B>")`. The strike replaces the identity used for registry keys, subtype edges and
satisfaction checks with the pair, and leaves the rendered string as a *rendering* — produced at the
boundary where a human or a diagnostic reads it, never compared.

`TypeExpr::Parametric` already carries exactly that pair; nothing new needs inventing. The work is
routing the three subsystems above through it and deleting the string surgery.

## The four questions, flat, on THIS shape

- **Obvious?** YES. "A name plus its arguments" is what the declaration already says, and
  `TypeExpr::Parametric` already holds it. The current design asks a reader to know that
  `":wat::core::Seqable<T>"` is a *key* in one map and a *prefix pattern* in one function.
- **Simple?** YES. One representation, one place to render it. Today the same fact is a registry
  key, a `format!` template in two Rust helpers, and a `<K,V>` substring in a wat macro.
- **Honest?** YES — and this is where the current design fails outright. A prefix match on
  `format!("{surface}<")` claims a relation between two types it never checked, and the `:peers`
  comparison reports a MISSING FIELD when the field is present and correctly typed.
- **Good UX?** YES. Every declaration in the corpus becomes free to wear either spelling, which is
  the precondition ② needs and ③ enforces.

⚠ The four questions above judge ONE shape. They do not choose between alternatives, and
`[[feedback_four_questions_cannot_see_a_shared_premise]]` says that is where they are weakest — so
the builder should be offered at least one rival (e.g. *normalize every identity to the base name at
the registry boundary and keep the string*, which is cheaper and loses the arity discrimination
`Handle<T>` vs `Handle<Xt>` relies on) and judge both flat before anything is briefed.

## What this stone does NOT do

- **No corpus migration.** ②-iii re-runs AFTER this lands, unchanged — the codemod is proven
  (idempotent, 36 files, the arrows unmoved) and needs nothing.
- **No ③.** Legality still does not change.
- **No `defservice` re-authoring beyond the identity.** Its `{b}::Op{p}` emission must stop
  producing the angle form, but its dispatch, its clause fold and its 18 de-suffixed names are
  β-ii's shipped work and stay.

## Sequencing

```
γ-i          `defn` / `fn` accept `:- [T …]`.  ✅ RULED FIRST (D1, builder 2026-08-21).
             DESIGN-STONE-gamma-i-defn-takes-the-binder.md — one fork open inside it
             (decision E: where the binder is consumed). Independent of this stone.
this stone   the identity is a base + args; the string is a rendering       ← ruling needed
②-iii        re-run the codemod on wat/. Unchanged. Floor. Commit.
②-iv         tests/ + wat-scripts/  (~2,070 sites)
③            the angle form becomes ILLEGAL; delete is_type_bracket_candidate
```

## Open, and belonging to the builder

1. Is this one stone or three (subtype edges · satisfaction checks · `defservice`'s emission)?
2. Does the rival shape (normalize to base name, keep the string) survive the four questions?
3. `Fn(…)->ret` and `:(a,b,c)` — the ②-iii dry-run migrates both, which the ② DESIGN scoped out.
   Its exclusion was written when the `Tuple` renderer was mode-blind; ②-i-b closed that, and the
   destinations are probe-verified legal. Ride along, or add the discriminator the DESIGN forbids?
