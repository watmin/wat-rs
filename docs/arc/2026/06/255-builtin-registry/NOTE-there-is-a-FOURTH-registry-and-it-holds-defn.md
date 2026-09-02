# NOTE — there is a FOURTH registry, it holds `defn`, and a prefix guess is all that keeps it disjoint

> Found while looking for the next hand-list to kill. The mutation pair
> (`is_mutation_head`/`is_mutation_form`) cannot become a registry query, and the reason is not the
> pair — it is that **the registry cannot answer for 41 names that exist.**

## The measurement

```
stdlib macros declared across wat/ ....................................... 41
  of those the intrinsic registry can answer for .......................... 0
  ⛔ absent — `lookup_entry` returns None, the SAME answer it gives for a
     name that does not exist at all .................................... 41
```

They are not obscure:

```
:wat::core::defn        :wat::core::defstruct    :wat::core::defrecord
:wat::core::cond        :wat::core::->           :wat::core::->>
:wat::core::format      :wat::service::defservice :wat::rete::defrule
:wat::rete::defquery    :wat::kernel::readln     :wat::holon::Ngram   …
```

★★★ **`:wat::core::defn` — the way every user in this language defines a function — is invisible to
the registry.** Ask it and you get `None`: identical to asking about `:wat::core::zorble`.

## The fourth registry

`src/macros/registry.rs`'s `MacroRegistry` is a name store with `contains` / `get` / `register`, and
`MacroDef` carries `name`, `params`, `rest_param`, `body`, `span`, and the retained declaration form.
It answers the RULING's **item 1** (every name), **item 4** (what they take) and **item 7**
(reflection — `src/reflect/lookup.rs:209` and `src/reflect/expand.rs` both consult it).

The census has now grown twice past what any document held:

```
RULING (original)                       ten authorities
+ solvere cast                          SPECIAL_FORMS          → eleven
+ NOTE-the-sloppy-registries            nine more, incl. five inconsistent hand-lists
+ THIS NOTE                             MacroRegistry — 41 names, 0 visible to the registry
```

⚠ **And unlike the others, this one is not obviously a duplicate to delete.** `registry()` is a
`&'static IntrinsicRegistry` folded at link time from `inventory`; a stdlib macro is wat source
parsed into a per-`FrozenWorld` `SymbolTable`. They differ in lifetime and in origin, not merely in
spelling. *"Fold it in"* is not a free move, and the RULING's own warning applies: **a campaign that
cannot tell a derivation from a duplicate deletes correct code.**

## ★★ Why this blocks the next hand-list

`is_mutation_head`/`is_mutation_form` — the pair that **disagree on disk today** — contain
`:wat::core::defstruct`. Their natural registry equality is *"this form is never evaluated"*, i.e.
`@Purity Unevaluated`. But `defstruct` is one of the 41: it has no row, so it can carry no purity,
so the equality cannot be proven and the flip cannot be made.

**The same shape that blocked 1a-β-ii until `defstruct` left the liftable domain now blocks the
mutation pair — except here `defstruct` legitimately belongs and cannot be removed** (measured
twice: `eval-ast!` and `eval_in_frozen` both see the literal, unexpanded head).

## ⛔⛔ AND THE PART THAT MAKES THIS URGENT

What stops a macro from claiming a name the registry already owns? Measured:

```
(:wat::core::defmacro :wat::core::if …)   →  #wat.macro/ReservedPrefix
(:wat::core::defmacro :wat::i64::+  …)   →  #wat.macro/ReservedPrefix
    "cannot declare macro … — reserved prefix (:wat::, :rust::, :$bound::)"
```

★★★ **The only thing keeping the two namespaces disjoint is `is_reserved_prefix` — the arc's
FOUNDING TARGET, and the exact authority the builder just ruled must die** (*"prefixes declaring
properties die when the registry matures… the prefix is nothing but a namespace"*).

Nothing else checks it. `no_two_submissions_claim_the_same_fqdn` guards the registry against itself;
**no gate compares the macro namespace against the registry's.** Today they are disjoint by
measurement (0 of 41 overlap) — never by construction.

So Phase 3a is not tidying. **When `resolve` stops guessing by prefix, the registry must be able to
answer *"is this name taken?"* for macros too, or the guard that prevents a user shadowing
`:wat::core::if` disappears with the prefix that currently enforces it.**

## ⬜ The fork — not decided here

Whether the registry gains macro rows, exposes one query door over two disjoint stores, or stays out
of the macro namespace entirely is a substrate ruling with real structural weight on both sides
(link-time static vs per-world state). It goes to the main chat against the four questions.

★ What is established and cheap either way: **a gate asserting the two namespaces are disjoint.**
It is true today, it is currently enforced only by the prefix the campaign is killing, and it is the
precondition for any of the three answers.
