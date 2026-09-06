# DESIGN — STONE 251.8b: the tuple is STORED, not re-derived

> **Builder:** *"all symbols must be a tuple of (ns, name) ... right?.... that `Identifier` is this
> tuple?...  `wat.core/+ :=> [wat.core +]` · `foo :=> [$bound foo]`"*
> and, on what keeps going wrong: *"we must never key any tooling on character case."*

**The model is right and the code already claims it. It does not hold it.**

```rust
pub struct Identifier {
    name: String,                 // "wat.core/+"  — ONE FLAT STRING
    scopes: BTreeSet<ScopeId>,
}
pub fn namespace(&self) -> &str {
    match self.name.rfind('/') { Some(i) => &self.name[..i], None => BOUND_NAMESPACE }
}
```

`namespace()` already returns `$bound` for a bare name and `is_reference()` is literally
`namespace() != BOUND_NAMESPACE`, so **`foo → [$bound, foo]` is the stated semantics today.** It is
recomputed from a `rfind('/')` on every read.

251.8a said this out loud and named its own successor:

> *"STONE 251.8a: the namespace is **DERIVED from the spelling** (split on the last `/`), not stored
> — `Identifier` still holds one `name` string. **251.8b is where derived swaps for stored** behind
> this same signature."*

## ★★★ WHY THIS IS THE STONE EVERYTHING ELSE IS QUEUED BEHIND

Every guess found this session is the same defect wearing a different hat — **structure being
re-derived from a flat string, and the derivation having to choose:**

```
keyword_from_wat_path   splits ":wat::rete::Explained/support" on the last "::",
                        strands the "/" inside the name, and falls back to
                        #wat.ast/Keyword {:path "…"}          — 7 in one doc row, 89 corpus examples

ns_to_wat_path          rejoining, must decide "::" or "/" — and does it by CAPITALISATION:
                          type_seg.starts_with(uppercase) && !name.starts_with(uppercase)
                        ⛔ RULED OUT: "we must never key any tooling on character case"

#95                     a dotted call head cannot be looked up without that same rejoin, so
                        widening infer_list's gate would put the casing guess on the type
                        checker's RESOLUTION path — wrong binding, silently

receiver / method       split on the LAST "/", so `wat.core//` reads as ["wat.core/", ""]
                        rather than [wat.core, /]                     — 5 live division operators
```

**None of these questions exist once the boundary is stored.** There is nothing to find, nothing to
rejoin, nothing to infer from case. `wat.core/+` is `[wat.core, +]` because that is what it *is*.

## THE SURFACE — 13 pub fns, and the name-shaped ones all return `&str`

```
bare(name)      as_str()      namespace()    is_reference()
leaf()          path()        receiver()     method()
prime()         deprimed()    add_scope()    scopes()
```

⚠ **`as_str(&self) -> &str` is the design's one real problem.** A borrow cannot be returned from a
string assembled on demand, so a stored `(ns, name)` pair leaves `as_str()` with nothing to point at.

## THE CONTRACT DECISION

**Store the tuple AND the flat spelling; every accessor keeps its `&str` signature.** 251.8a's own
promise is *"behind this same signature"*, and the redundancy is safe here because construction has a
**single chokepoint** — `Identifier::bare`, which the module doc already calls *"the single
chokepoint"* and which is debug-guarded. Derived once at construction, never re-derived after.

Alternatives, recorded so a later self does not re-litigate them:

- **`as_str() -> String` / `Cow`.** Honest, no redundancy — and it changes a signature 251.8a
  promised would not change, rippling through 147 construction sites and every caller of the flat
  form. **Rejected on the promise, not on difficulty.**
- **Flat string + a memoised boundary index.** Removes the re-derivation without the tuple. Cheaper,
  and it keeps the model implicit — the thing the builder asked to make explicit. **Rejected on the
  model.**

## ★ WHAT MAKES THIS UNUSUALLY CONTAINED

```
name is a PRIVATE field, read 11 times, ALL inside identifier.rs's own accessors.
`pub name` : 0.    External `.name` reads on an Identifier : 1.
```

**The representation is fully encapsulated.** The swap is invisible outside the crate — which is
exactly what 251.8a set up.

## ⚠ AND `scopes` IS NOT PART OF THE TUPLE

`scopes: BTreeSet<ScopeId>` is Racket's sets-of-scopes macro hygiene (Flatt 2016). It rides
*alongside* the name and is orthogonal to `(ns, name)`. **A stone that folds it into the tuple has
made a three-member name, which is the illegal shape the builder described.**

## WHAT THIS STONE IS NOT

- **NOT `#95`.** Dotted call heads stay unchecked; this removes the reason that fix needs a guess.
- **NOT the codec.** `keyword_from_wat_path` / `ns_to_wat_path` stay as they are; this makes their
  honest fix possible.
- **NOT killing keywords as heads.** That is the interim state's far side.
- **NOT the split rule.** Whether the boundary is the first `/` or the last, and whether
  `wat.core//` is `[wat.core, /]`, is a READER question. **This stone stores whatever the reader
  decides; it does not decide it.** ⚠ If the reader's current answer disagrees with the builder's
  model, that is a finding to surface, not to fix here.

## FILES

```
crates/wat-reader/src/identifier.rs    the representation + the accessors
```
