# HALT — STONE-a-keyword-is-a-keyword is WITHDRAWN. Do not re-strike it with a cleverer separator.

**Stop work on `[[BRIEF-STONE-a-keyword-is-a-keyword]]`.** The strike landed and its encode half is
correct. The stone is withdrawn because its **decode half cannot be written at all** — not by the
casing test it shipped, and not by any other rule. The distinction it tried to reconstruct must not
exist.

> The builder's ruling, mid-session: *"we must never key any tooling on character case - we are not go."*

That ruling is why the stone was flagged. **This document is why it is withdrawn**, which is a
different and stronger reason: the heuristic is refuted on the disk, and its replacement is
impossible under a rule the builder had already given.

## WHAT THE STONE DID

```
encode  keyword_from_wat_path   ":wat::rete::Explained/support"  →  :wat.rete.Explained/support
        splits on "/" instead of the last "::" — CORRECT, and it is what killed
        7 #wat.ast/Keyword carriers in one rendered #wat.doc/Row
decode  ns_to_wat_path          ("wat.rete.Explained", "support") →  ":wat::rete::Explained/support"
        rejoins with "/" or "::" BY WHETHER THE LAST ns SEGMENT IS CAPITALISED   ⛔
```

## ⛔ REFUTATION 1 — the heuristic is wrong on the live corpus

Census instrument: `grep -rhoE '":wat::…/…"'` over `src/` + `crates/*/src/` — a **string-literal**
census, so it is a pattern over spellings, not a site census of registered entities. It finds **177**
distinct slash-bearing FQDN literals. Of those, **18 have a receiver that is not capitalised**,
across **7 receiver segments**:

```
:wat::core::__internal/primitive      :wat::core::keyword/from-string
:wat::core::__internal/registered     :wat::core::keyword/to-string
:wat::core::__internal/special-form   :wat::core::keyword/to-symbol
:wat::core::__internal/type-decl      :wat::core::keyword/to-type-form
:wat::core::and/or                    :wat::core::keyword/to-type-form-colon
:wat::core::char/of                   :wat::core::keyword/*          (table wildcard)
:wat::core::i64/to-f64                :wat::core::rational/numerator
:wat::core::i64/to-string             :wat::core::rational/denominator
:wat::spawn::process/grants           :wat::core::rational/*         (table wildcard)
```

Every one of them takes the `::` branch and decodes to a name that is not the live one.
`:wat::core::i64/to-f64` is a **live dispatch arm at `src/runtime.rs:2665`**; the heuristic rebuilds
it as `:wat::core::i64::to-f64`, which `src/remedy/retirement.rs:204` lists as **RETIRED**.

★ **The stone's own re-captured goldens carry the counterexample.** It rewrote
`tests/reflection/wat_arc144_uniform_reflection__*.edn` to render
`:wat.core.__internal/primitive`, `/special-form` and `/type-decl` — three lowercase receivers the
decode cannot take back. The floor stayed green because **nothing exercises that direction**.

## ⛔ REFUTATION 2 — the guess is already on the resolution path

The SEAM recorded this as a risk `#95` *would* introduce. It is present today. `ns_to_wat_path` is
not a codec-private helper:

```
src/resolve/normalize.rs:413    resolve_namespaced_symbol — the PRIMARY resolution candidate
src/types.rs:5060, 5177         "the canonical mapping" (its own comment)
src/macros/expand.rs:587        macro expansion's primary
src/declare/parse.rs:378, 679   declaration parsing
```

A wrong answer here binds the wrong function, silently — not an ugly keyword.

## ⛔ REFUTATION 3 — no rule can replace it, because the distinction must not survive

A clojure keyword carries **at most one slash**: `(ns, name)`. Both wat spellings collapse onto the
same tuple:

```
:wat::core::i64::to-f64      →  (wat.core.i64, to-f64)
:wat::core::i64/to-f64       →  (wat.core.i64, to-f64)
```

There is no legal one-slash EDN keyword that keeps them apart, so **no encoder can preserve the
boundary and no decoder can restore it.** The casing test was not a bad implementation of a
reconstructible fact; it was standing in for a distinction the builder has already ruled out of
existence:

> *"all symbols must be a tuple of (ns, name) … `wat.core/+ :=> [wat.core +]`"* — at most one slash;
> a three-member tuple is illegal.

This is why the stone must not be re-drawn with a different separator, a sigil, or a metadata side
channel. **The corpus has two spellings for one symbol. That is the defect.**

## THE REAL OWNER — already named on disk, before this session

`src/resolve/normalize.rs:420–433`, carrying a `rune:exigere(attested-arc)`:

> *"LATENT GAP, named not buried: a type-member SYMBOL head (`wat.core.HashMap/length`) normalizes to
> `:wat::core::HashMap::length`, which passes resolve but is NOT the runtime op
> (`:wat::core::HashMap/length`), so it would not dispatch. … correct `Type/member` symbol
> normalization lands when symbol-head type-members first appear, at **arc 251 stone 251.5**."*

The same comment names why a registry LOOKUP cannot discriminate today either: `is_resolvable_call_head`
**accepts a reserved `:wat::`/`:rust::` namespace without validating the leaf**, so it answers YES to
both spellings. That shortcut is arc 255's to close; 251.5 is what consumes a registry that answers.

```
255   the registry answers precisely (no reserved-prefix shortcut)
251.5 ONE spelling — the corpus migration, by the wat-fix codemod (R21)
codec then has nothing to reconstruct, because there is nothing to choose between
```

## WHAT SURVIVES, so the successor does not re-derive it

- **The encode split is right** and is recoverable verbatim from this commit's parent
  (`src/edn/render.rs`, `keyword_from_wat_path`'s `/` arm). It becomes correct the moment the corpus
  has one spelling.
- **7 `#wat.ast/Keyword` carriers in one rendered doc row, and 89 corpus `@examples`** would have
  carried the verbatim tag into arc 255's sweep. That pressure is real and unrelieved — the
  verbatim carrier is restored by this halt, deliberately.
- **The round-trip direction is untested.** No test in the floor decodes a rendered keyword back to
  its wat FQDN; that absence is what let a 18-of-177 error ship green.
- **`ns_to_wat_path` is a resolution primitive, not a codec helper.** Its six non-codec callers are
  listed above; any future edit to it is a change to symbol resolution.

Nothing here is a criticism of the strike. The encode was asked for and delivered; the decode was
impossible before the brief was written, and I did not know it when I wrote it.

---

## ⚠ THE SAME HEURISTIC IS ALREADY ON DISK, AND IT PRE-DATES THIS STONE

`crates/wat-macros/src/edn_doc.rs:255` `fqdn_of` — the function the strike copied its inverse from,
and it carries the identical defect:

```rust
let type_is_type   = type_seg.starts_with(|c: char| c.is_uppercase());
let name_is_method = !name.is_empty() && !name.starts_with(|c: char| c.is_uppercase());
if type_is_type && name_is_method { return format!(":{head}::{type_seg}/{name}"); }
```

Reverting `ns_to_wat_path` does **not** clear the ruling. `fqdn_of` runs at proc-macro time over the
`@`-form doc corpus and is wrong on the same 18 spellings — including the four
`:wat::core::__internal/*` heads that every reflection golden carries. It is in scope for the
successor, not for this halt, and it is named here so nobody reads a clean
`grep -c is_uppercase src/edn/render.rs` as the whole answer.

Two further structural case-tests were surfaced this session and are unexamined:
`src/types/subsume.rs:60` and `:95`, and `src/declare/parse.rs:1013`.
