# NOTE — a namespace question must not be asked with a string prefix (2026-09-06)

> **Builder, watching a codemod rewrite a prefix literal:**
> *"we need a proper intrinsic api for `ns-of some-bound symbol` or whatever…. all symbols are
> namespaced… even let binders… and arg-spec… they belong to `$bound` ns….. this string prefix
> matching needs to go away in favor of namespace checking."*

## THE TELL

The three-spellings stone's codemod produced this diff, and the builder stopped on it:

```diff
- (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::rete::core::f64::"))
+ (:wat::rete::where (:wat::rete::string::starts-with? ?n "wat.rete.core.f64/"))
```

**Both lines ask the same question — *is this symbol in the `f64` namespace?* — and neither one asks
it.** They ask *"does this string begin with these characters"*, and the answer happens to coincide.
So a spelling change is not a rename: it is a rewrite of every literal that was standing in for a
namespace test. `[[NOTE-operator-namespaces-dotted]]`

★ **This is a stem-patch and the reland that ordered it was one too.** Rewriting the literals makes
the current corpus pass and leaves the mechanism that will need rewriting again at the next spelling
change, the next namespace move, the next `::`→`/` decision.

## ★★ THE ANSWER IS ALREADY HALF-BUILT — arc 251 Stone 251.8a

This is not new design. The substrate already holds exactly the concept the builder named:

```rust
// crates/wat-reader/src/identifier.rs:145
/// STONE 251.8a: the namespace is DERIVED from the spelling (split on the last `/`),
/// not stored — `Identifier` still holds one `name` string. 251.8b is where derived
/// swaps for stored behind this same signature.
pub fn namespace(&self) -> &str {
    match self.name.rfind('/') {
        Some(slash) => &self.name[..slash],
        None => BOUND_NAMESPACE,
    }
}

pub fn is_reference(&self) -> bool { self.namespace() != BOUND_NAMESPACE }
```

And `$bound` is **already a reserved namespace**, for the reason the builder gave:

```rust
// src/resolve/reserved.rs:28 — Arc 251 Stone 251.8a
// $bound is the reserved namespace every non-namespaced (binder) symbol carries.
// Reserved so user source cannot define into it — a user-defined `$bound/x` would be
// indistinguishable from a real local binder.
":$bound::",
```

**So "all symbols are namespaced, even let binders and arg-spec, and they belong to `$bound`" is
already the substrate's design.** The gap is not the concept. The gap is:

```
1  namespace() is RUST-ONLY. Nothing exposes it to wat, so every rule that needs the
   question hand-rolls starts-with? on a string literal.
2  the namespace is DERIVED FROM THE SPELLING, so it is exactly as spelling-fragile as
   the prefix match it should replace — 251.8b is the named stone where derived becomes
   stored, and until then `namespace()` on ":wat::core::+" has no `/` and therefore
   answers `$bound`, which is WRONG for a reference.
```

⚠ **Point 2 is the one that decides scope.** Exposing `namespace()` to wat today would make
FQDN-spelled references answer `$bound` — a confidently wrong answer replacing an honestly fragile
one. **The intrinsic is worth minting only alongside, or after, 251.8b.**

## THE CENSUS — small, and that is the point

```
8  string::starts-with? against a namespace-shaped literal
2  string::contains "::"      — the same question, spelled differently
   in 5 wat-scripts/fixes/ recorded migrations, 1 grep rule, 1 scratch-pad probe
```

Eight sites is not a crisis; it is a **class caught early**. Every one of them is a namespace
question wearing a string's clothes, and each is a site that must be touched by every future
spelling change.

## WHAT A PROPER API WOULD LOOK LIKE

```
(:wat::core::ns-of <symbol-or-keyword>)   ->  the namespace, or `$bound` for a binder
(:wat::core::in-ns? <sym> <ns>)           ->  membership, spelling-independent
```

with the answers coming from `Identifier`'s door — never from a second parser, which
`one_name_grammar` already forbids in Rust and which this NOTE extends in spirit to wat.

★ **The prize is that a namespace question stops caring which of the three spellings it is looking
at.** `:wat::rete::core::f64::sqrt`, `:wat.rete.core.f64/sqrt` and `wat.rete.core.f64/sqrt` are one
symbol in one namespace; only the string form differs, and only a string test can be confused by
that.

## STATUS

**A NOTE, not a stone.** It is blocked on 251.8b (derived → stored) for the reason in point 2, and
it is recorded here so that:

- the eight prefix sites are known to be **stem-patches**, not correct code, whatever the
  three-spellings codemod leaves them looking like;
- the next person who reaches for `starts-with?` on a namespace finds this first;
- and 251.8b's value is visible from outside arc 251 — it is not only about `Identifier`'s
  internals, it is what makes a namespace askable at all.

`[[NOTE-scoped-name-reaches-identifier-bare]]` · `[[NOTE-root-namespace-user-code]]` ·
`[[NOTE-operator-namespaces-dotted]]`
