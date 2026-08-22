# DESIGN — `is_subtype` answers TWO questions; give the second its own door

> **RULED S2, builder 2026-08-21.** `is_subtype` keeps EXACT-string semantics. A second, explicit
> query — `family_extends(sub_base, sup_base)` — answers the arg-agnostic question that
> `satisfies_bare_surface` was faking with a prefix match.
>
> **RULED B-3** — the identity work is three stones (lattice · `defservice`'s 53 sites · three
> one-offs). This is stone 1.

## ⚠ THIS DESIGN HAS BEEN WRONG TWICE, AND THE HONEST SCOPE SHRANK EACH TIME

| draft | claimed | refuted by |
|---|---|---|
| **A-ii** structured identity `(base, args)` | the args must be preserved, carefully | the corpus's own test header: the args in a key are a bound variable's NAME — `"<?454>" != "<T>", always` |
| **A-i** the key is the base; args never enter the lattice | *"the args were never doing anything"* | **two negative-control tests went red.** The args WERE doing something — in one consumer, by accident |
| **S2** *(this)* | two questions, two doors | — |

★ Each rewrite made the stone smaller. That is the tell that the first two were designed from a
reading rather than from the consumer. **The evidence for S2 was sitting in `check.rs`'s own comment
the whole time — in a file the earlier drafts never opened.**

## What the two reds proved

`check.rs`'s `assignable`, `(Parametric, Parametric)` arm:

```rust
// the FAST PATH — sound ONLY because is_subtype compares full strings
if ah != eh && transport_edge_keys(&a).any(|k| is_subtype(k, &format_type(&e), types)) {
    return nature_floor_ok(&a, &parametric_head_fqdn(eh), types);
}
// …the ELSE branch, which is ALREADY CORRECT:
else { … is_subtype(k, &bare) && aargs.zip(eargs).all(unify) … }
```

The exact-string compare was doing **two jobs**: *"does the edge exist"* and — accidentally, by
failing to match — *"and do the args agree."* Base-stripping made it succeed on head alone, so it
returned before the arg guard ran. Both reds are **negative controls** — arc 170's swap gate and
118.B1a's — and the `else` branch's own comment names them:

> ★ *SOUNDNESS LIVES IN THE GUARDS BELOW, NOT IN THE GATE … enforced by UNIFY on the args … Both are
> negative-control rows of 118.B1a's gate.*

**And the fast path cannot simply be deleted**: it serves 293.W.2f, where the arities DIFFER
(`Handle<K,V,T>` vs `TypedCapability<S,R>`), while the `else` requires `aargs.len() == eargs.len()`.
Both arms are load-bearing.

## ★ What this means for the original complaint — it is mostly ALREADY SOLVED

The `<T>` vs `<?454>` mismatch is real, and **Stone 118.3-B already fixed it where it mattered**: the
`else` branch queries by the BARE key and unifies args explicitly. So the lattice's "defect" is not
that types cannot match across spellings — it is narrower:

**`satisfies_bare_surface` fakes an arg-agnostic query with `format!("{surface}<")`, a prefix match.**
That is the one remaining fake, and it is what this stone replaces.

## The strike — honestly bounded

```
REVERT   register_subtype / is_subtype base-stripping        keys stay EXACT; the fast path stays sound
ADD      family_extends(sub_base, sup_base, env)             the arg-agnostic query, named and explicit
REPLACE  satisfies_bare_surface's prefix match               → family_extends. 4 callers unchanged in shape.
KEEP     extend-type's FORM-spelling acceptance              the rider's addition; ②-iii needs it
KEEP     transport_satisfier_heads' guess list               it guesses at EXACT keys, which remain — sound
```

## ⚠ WHAT THIS STONE DOES **NOT** DELIVER — stated because two drafts oversold it

- **It does not remove `transport_satisfier_heads`' three-key guess**, nor `transport_edge_keys`'
  hardcoded `["T","Xt"]` last-arg rewriting. Those guess at *exact* keys, and under S2 exact keys
  remain. They are ugly and sound. Removing them requires the fast path to stop needing exact keys —
  a different stone with its own ruling.
- **It does not fix the `<T>`/`<?454>` mismatch.** 118.3-B already did, in the `else` branch.
- **It does not touch `defservice`** — that is stone 2, 53 sites.

What it does deliver: **one fake becomes a named function**, and `extend-type` accepts the form
spelling, which is ②-iii blocker 3's lattice half.

## The four questions

*Shared premise, and it is what the two reds refuted: that ONE lattice query can serve both callers.*

| | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| **S1** A-i stands; the fast path gains an explicit arg guard | YES | **NO** | YES | — |
| **S2** two queries — `is_subtype` exact; add `family_extends` | YES | YES | YES | YES |
| **S3** revert entirely; keep the prefix match | YES | YES | **NO** | — |

**S1 fails Simple** — it duplicates the `else`'s guard in a second place under different arity rules,
so the swap-gate would be enforced twice, differently. Two enforcements of one invariant is how they
drift apart.

**S3 fails Honest** — `format!("{surface}<")` is a prefix match standing in for a relation it never
checks; leaving it means the code keeps claiming a question it is not asking.

## Acceptance

1. ★ Both negative controls PASS — `probe_arc170_parametric_surface_param` and
   `probe_stone_118_b1a_neg`. These are the two the A-i attempt broke; they are the gate.
2. `satisfies_bare_surface` no longer exists; `format!("{surface}<")` returns nothing under grep.
3. `family_extends` has exactly one implementation and the four ex-`satisfies_bare_surface` callers
   route through it.
4. `extend-type` accepts a FORM parent `(:Seqable :- [T])` — kept from the A-i attempt.
5. `is_subtype`'s 30 call sites and its signature are unchanged.
6. Floor **4854/4854**, clippy 0.

⚠ Row 1 is the whole stone. A green floor without those two tests specifically re-run proves nothing:
they were passing before the A-i attempt and must be passing after.
