# DESIGN — STONE 294.g: the holon record's wire is PLAIN EDN; the 16-tag hologram wire is ANNIHILATED

> Drawn 2026-08-14 against HEAD `25d9d015`. Probe committed and RED before this document was written.
> Closes 294 R1 **flaw #3**. Builder: *"annihilation is our greatest joy .... then that's our target."*

## The gap, measured — not read

Two records, same two fields, differing only in holder:

```
#t/Plain {:x 1 :y 2}

#t/Holo #wat-edn.holon/Bind [#wat-edn.holon/Atom #wat-edn.holon/String "t::Holo"
          #wat-edn.holon/Bundle [#wat-edn.holon/Bind [#wat-edn.holon/Atom
            #wat-edn.holon/String "x" #wat-edn.holon/Atom #wat-edn.holon/I64 1]
                                 #wat-edn.holon/Bind [… "y" … 2]]]
```

~22 bytes against ~250 — **and the data is IN the second one**, buried under the algebra it derives
(`"x"`→1, `"y"`→2). The wire ships the **index** instead of the **record**.

## Why it is wrong NOW (it was not always)

294.c.1 landed **identity-as-EDN-data** (`ed7ecd50` — `Eq`/`Hash` keyed on `(holder, class, fields)`).
Once the fields ARE the identity, the hologram is a **derived index**, and a derived index has no
business crossing a wire: the receiver knows `:t::Holo` is holon-held from the type registry and
derives its own. That is R1's flaw #3 (*"the `#wat-edn.holon/*` tags — scar tissue from a
hologram-canonical wire"*) with R1's own cure: ***"the wire is plain EDN."***

The doctrine is already written down one layer out — `comms/mod.rs:1130`, arc 214:

> *"the universe-boundary wire is plain EDN, never a holon-tagged envelope. **Holon-tagging is one
> representation of EDN, content INSIDE a holonic value — not the transport.**"*

with a live test (`string_wire_is_raw_edn_not_holon_tagged`) proving it for **scalars**. This stone
extends the same law from scalars to **composites**. It is not a new rule; it is the existing rule
reaching the case it never covered.

## The ONE contract decision

> **A holon record's wire form is IDENTICAL to a plain record's — the class tag and the fields.
> `#t/Holo {:x 1 :y 2}`. The holder is looked up, never transmitted; the hologram is derived on
> arrival, never serialized.**

The control proves the target is not invented: `#t/Plain {:x 1 :y 2}` is the sibling's **existing**
behaviour, green at HEAD, and the goal is that string modulo the class name.

## What annihilates, and what SURVIVES — the distinction flaw #3 does not make

The 16 `#wat-edn.holon/*` tags (`Atom Bind Blend Bundle Permute SlotMarker Thermometer Vector Bool
Char F64 I64 Keyword String Symbol Tag`) serve **two masters**:

| role | fate |
|---|---|
| **holon RECORD wire form** — a record crosses as its hologram (`edn_shim.rs:2126`, `:3163`) | **ANNIHILATED** — this stone |
| **HolonAST tag vocabulary** — the algebra's own wire form (`:2862`, arc 093 round-trip) | **SURVIVES** — see below |

**Why the vocabulary cannot simply be deleted:** the ruling of 2026-08-14 gives **EDN ⊆ HolonAST**
(*"whatever you can express with edn... you can build in holon-ast"*) and the containment is
**one-way**. `Bind` is elementwise multiplication; `Bundle` is sum + ternary threshold; `Permute`,
`Blend`, `Thermometer`, `SlotMarker` are VSA operations. **There is no plain EDN for "multiply these
two vectors."** Delete the vocabulary and a HolonAST carrying a `Permute` has no wire form at all.

**But this stone removes the only consumer that needed the algebra ON the wire.** Once a record
crosses as its fields, nothing algebra-shaped is transmitted — each side derives its own hologram.
Whether the residual vocabulary then has any live sender is a MEASUREMENT for the strike, not an
assumption here (see STOP-2).

## `#holon` is the anonymous case, and it is already built

`#holon <form>` (arc 294.b) desugars to `(:wat::holon::literal <form>)` — reader `parser.rs:392`,
lexer `:123`/`:338`, check `check.rs:3271`, runtime `runtime.rs:4856`. **It reaches the source
language and NOT the wire** (`edn_shim`: zero occurrences).

Two cases, two answers, no overlap:

- **Declared holon record** → `#t/Holo {…}`. The class tag names the type; the holder comes from the
  registry. No `#holon` needed.
- **Anonymous holonic literal** → `#holon {…}`. No class to look up, so the tag carries the intent —
  `identity` to Clojure, an AST to wat (294 R3's seam, and the asymmetry that makes it honest).

## Why this is 294 R3's fulfilment condition, not cleanup

R3's own clause: *"FULFILLED when the worlds collide in running code — … **a Clojure app drops into
wat for VSA over the wire**."* A Clojure app cannot do that today: it would have to emit
`#wat-edn.holon/Bind` trees — our crate name, our algebra, our internal representation. After this
stone it emits `#t/Holo {:x 1 :y 2}` (or `#holon {…}`), which is data any EDN reader already writes.

## The four questions

- **Obvious?** YES — a holon record and a plain record are the same thing under different policy, so
  they read the same on the wire. The current asymmetry is what a reader cannot predict.
- **Simple?** YES — it DELETES a serialization path rather than adding one; the receiving side's
  derivation already exists (`build_holon_hologram`, `f301a6fc`).
- **Honest?** YES — shipping a derived index as if it were the value is the inversion 294 exists to
  correct; and identity already moved to the fields, so the wire is currently contradicting the
  identity model.
- **Good UX?** YES — 22 bytes instead of 250, readable by any EDN consumer, and the clj↔wat seam
  becomes real instead of designed.

## Out of scope — affirmative cuts

- **The `#wat-edn.*` → `#wat.*` rename** (~118 sites, five families). Deliberately AFTER: renaming
  sites this stone deletes is wasted motion. Tracked, not deferred.
- **`HolonRepresentable`'s annihilation** (flaw #4, 11 impls) — a sibling, not this.
- **`HolonAST` doing code-AST duty** (flaw #5, task #91) — a scoping cleanup, unrelated to the wire.
- **`HolonAST`/`Hologram` renames** — VOID per the 2026-08-14 ruling; both names are correct.

## The probe (committed BEFORE this design, red at HEAD)

`tests/comms/probe_arc294_holon_wire_is_plain_edn.{rs,wat}` — 4 rows, **3 green / 1 red**:

```
PASS  control_plain_record_wire_is_the_class_tag_and_its_fields   → "#t/Plain {:x 1 :y 2}"
PASS  non_vacuity_the_hologram_still_exists_and_measures
PASS  non_vacuity_the_hologram_still_discriminates
FAIL  holon_record_wire_is_plain_edn_not_the_serialized_hologram
        left:  "#t/Holo #wat-edn.holon/Bind [… \"x\" … 1 … \"y\" … 2]"
        right: "#t/Holo {:x 1 :y 2}"
```

**The control IS the target** (R59 `NISI FRANGAS, NIHIL PROBAS`) — the plain record already produces
the goal shape, so a red row 2 cannot be waved off as "we just picked a different format."

**Rows 3-4 are load-bearing and they face the outcome.** `cosine` returns a `CosineOutcome` (arc 278's
totality wall — a measurement may not absorb its own undefined case), and the checker REFUSED the
first draft of this probe, which had assumed the happy path and declared `-> f64`. That refusal is the
wall working. A `Degenerate` arm would mean the index is a zero vector — deleted rather than derived —
which is the exact failure these rows exist to catch. Without them, a green row 2 could be achieved by
throwing the hologram away, and the probe would prove the opposite of the stone.
