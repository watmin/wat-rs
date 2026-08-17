# DESIGN STONE — 294.j · `edn_shim` forgets the algebra

**Ruled by the builder, 2026-08-16.** Verbatim:

> *"edn shim needs to forget HolonAST entirely.... that's likely the better killshot"*

and, on what survives:

> *"we need to preserve /some/ holon markers..... thermometer is one of them"* … *"yes, slotmarker
> is the same class"*

and the model, stated as an example:

> *"`#holon {:a "b"}` represents two atoms, bound together...
> `(Bundle (Bind (Bind (Atom "keyword") (Atom "a")) (Bind (Atom "string") (Atom "b"))))`"*

---

## The model, and why the previous framing was wrong

`#holon {:a "b"}` **is** the wire form. `Bundle`/`Bind`/`Atom` is what the encoder **derives** from
it — deterministically, so a receiver derives the same structure from the same data. The algebra
never crosses a wire.

The framing this stone replaces asked *"how do I faithfully serialize a HolonAST?"* and produced a
rule — *a tag survives iff deleting it loses information* — which correctly killed six leaf tags and
then **preserved the algebra under a renamed namespace**. That was the wrong question. The right one
is *why is the algebra on a wire at all*, and the answer is that it should not be.

**This is not a new ruling.** It is 294 R1, already on disk, quoted in
`tests/comms/probe_arc294_holon_wire_is_plain_edn.rs`:

> *"the `#wat-edn.holon/*` tags — scar tissue from a hologram-canonical wire"* … cure: *"the wire is
> plain EDN."*

…and already executed twice:

| flank | what landed | where |
|---|---|---|
| holon **records** | 294.g — discriminator moved from BODY SHAPE to the REGISTRY; `reconstruct_holon_record` **derives** the hologram via `build_holon_hologram`, never reads it | `edn_shim.rs:2950-2968` |
| **WatAST** | the old `watast_to_holon` path encoded every node under `#wat-edn.holon/*`; the bridge replaced it, output carries NO holon tags | `wat_edn_bridge.rs:23`, `:836` |

The `#wat-edn.holon/*` AST tags are the last room in a house already cleared.

## ★ The cure is TWO ARMS AWAY, in the same match block

```rust
// edn_shim.rs:3728 — the ruling, already written, already applied:
// "A WatAST is a parsed form — by definition an EDN value (watast_to_edn/edn_to_watast are a
//  total bijection). Render it faithfully as its form (legible + recoverable); opaque-nil was a lie."
Value::wat__WatAST(a) => crate::wat_edn_bridge::watast_to_edn(a.as_ref()),

// edn_shim.rs:3731 — the arm that never received it:
Value::holon__HolonAST(h) => holon_ast_to_edn(h),   // ← 16 hand-rolled tag arms
```

**The lowering already exists and is total.** `holon_to_watast` (`runtime.rs:20625-20737`, 112
lines) returns a bare `WatAST` — not a `Result` — and an audit of its body finds **no `panic!`,
`unwrap()`, `expect(`, `unreachable!` or `todo!`**. It handles **every** variant, lowering each to
the wat source form that constructs it, Thermometer and SlotMarker included:

```
HolonAST::Bind(a,b)                  → (:wat::holon::bind  <a> <b>)
HolonAST::Thermometer{value,min,max}  → (:wat::holon::Thermometer value min max)
HolonAST::SlotMarker{min,max}         → (:wat::holon::SlotMarker min max)
```

It is live at **8 call sites in `runtime.rs`** — and at **zero** in `edn_shim.rs`, which
reimplemented it as a 16-arm tag serializer. **Sixth instance today of *capability built, never
adopted*** (`insert-all`, the `into` mirror clause, the May linter sweep, `error_ns`, `inventory`,
and now this — the worst of the six, because this one is adopted *everywhere except* the module
that rebuilt it).

## The classification — 14 die, 2 survive, and the survivors survive as VERBS

| class | tags | disposition |
|---|---|---|
| **the algebra** (derived; a receiver rebuilds it) | `Atom` `Bind` `Bundle` `Permute` `Blend` `Vector` | **DIE** |
| **the leaves** (already the data) | `String` `I64` `F64` `Bool` `Char` `Keyword` `Symbol` `Tag` | **DIE** |
| **encoding directives** (the data cannot say "encode me this way") | `Thermometer` `SlotMarker` | **SURVIVE** |

`Thermometer` and `SlotMarker` earn their marker because `{:value :min :max}` cannot express *"build
a thermometer encoding, not a 3-key-map holon."* Under this stone they survive as the **call form**
`(:wat::holon::Thermometer v min max)` — which `holon_to_watast` already emits — rather than as a
reader tag. That is strictly better: a call form is constructible wat, and it is plain EDN.

⚠ `runtime.rs:20727` records SlotMarker as *"a substrate-internal sentinel. Non-round-trippable."*
That is a pre-existing property of the value, not something this stone introduces, and the gate below
pins it rather than pretending otherwise.

## ⛔ THE ONE OPEN BOUNDARY — a STOP, not a decision

Two surfaces are both EDN and this stone only settles the first:

1. **rendering a `HolonAST` value** (`str`, `to-edn`, a diagnostic) → the source form, settled here.
2. **a directive appearing INSIDE `#holon`-shaped data** → the builder said *"`#wat.holon/Thermometer`
   is probably the correct name."* A directive nested in data may want the reader-tag spelling rather
   than the call-form spelling.

These do not conflict — they are different layers — but **(2) is NOT this stone's to settle.** If the
rider finds a site where a directive must be read back out of `#holon` data, that is **STOP-2**.

## Blast radius — MEASURED 2026-08-16, so nothing is hunted

```
src/edn_shim.rs ............................ 40 sites   ← the strike zone
tests/ (8 files) ........................... 26 sites
src/wat_edn_bridge.rs ....................... 2 sites   ← COMMENTS only, no code
golden .edn ................................. 3 files, ALL regenerate:
    wat_arc221b_..._keyword_foo.edn   #wat-edn.holon/Keyword :foo
    wat_arc221b_..._keyword_bar.edn   #wat-edn.holon/Keyword :bar
    wat_arc221b_..._symbol_nil.edn    #wat-edn.holon/Symbol "nil"
                                             ─────────
                                              72 total
```

**Already dead, delete on sight:**

| symbol | status |
|---|---|
| `write_holon_ast_tagged` (`edn_shim.rs:4277`) | **ZERO callers anywhere** — not called, not exported |
| `read_holon_ast_tagged` (`:4286`) | exported `lib.rs:138`, **zero in-tree callers** |
| `read_holon_ast_natural` (`:4298`) | exported `lib.rs:138`, **zero in-tree callers** |
| `("Nil", OwnedValue::Nil)` arm (`:4139`) | the encoder never writes `Nil` — a decoder arm for a tag that cannot arrive |

**The four live couplings — the whole surface:**

```
edn_shim.rs:3731   Value::holon__HolonAST(h) => holon_ast_to_edn(h)     encode
edn_shim.rs:2870   if ns == "wat-edn.holon" { … }                       decode dispatch
edn_shim.rs:2094   ":wat::holon::HolonAST" from-edn coercion arm        typed slot
edn_shim.rs:2098   tag.namespace() == "wat-edn.holon"                   tagged-vs-natural selector
```

## ★ MEASURED: the strict/natural reader fork is vestigial and dies with the tags

A disconfirming probe (`tests/value/probe_arc294_holon_bare_leaf_read.rs`, run 2026-08-16) measured
the read side. Verbatim result:

```
strict  · bare leaf inside a composite    FAIL   ← edn_shim.rs:4068
strict  · tagged leaf inside a composite  PASS   ← control: the harness works
natural · bare leaf inside a composite    FAIL   ← edn_shim.rs:4068, the SAME line
natural · bare leaf at top level          PASS   ← natural tolerance does exist
```

Both failures land on **one line**. The finding: `edn_holon_tag_to_ast`'s composite arms recurse
through `edn_to_holon_ast` — the **strict** reader — unconditionally (`:4145`, `:4149-4150`, `:4156`,
`:4161`, `:4176-4177`). So **"natural" describes only the top-level entry point**; one level in,
tagless tolerance stops.

The fork exists *solely* to compensate for leaf-wrapping. Remove the wrapping and there is no
distinction left to draw — `edn_to_holon_ast` and `edn_to_holon_ast_natural` collapse to one reader,
and the mode selector at `:2097-2109` becomes two arms calling the same function.

## The four questions — flat, each answered

**Obvious? YES.** *The shim renders a HolonAST as its source form, the way it already renders a
WatAST.* One sentence, and the precedent is three lines above the site.

**Simple? YES.** One arm replaces sixteen; two readers collapse to one; a mode selector disappears.
The strike deletes more than it adds, and adds no new concept — `holon_to_watast` and `watast_to_edn`
both already exist and are both already total.

**Honest? YES**, and this is where it bites. Shipping a derived index while claiming to ship data is
the dishonesty 294 R1 named. It also kills a *dormancy*: if we stopped **writing** `#wat-edn.holon/*`
but kept **reading** it, the tag would be dormant rather than dead — and dormant is exactly how
`.opaque` survived long enough to need a death warrant. Both halves go.

**Good UX? YES.** `(:wat::holon::bind (:wat::holon::atom "a") …)` is legible, constructible wat that
a reader can paste back. `#wat-edn.holon/Bind [#wat-edn.holon/Atom …]` is a serialized index nobody
can act on. The 294 probe measured the record-side version of this at **~22 bytes against ~250**.

## The gate

| # | assertion |
|---|---|
| 1 | `grep -rn 'wat-edn\.holon' src/ crates/ tests/ wat/ wat-scripts/ wat-tests/` → **0** |
| 2 | `holon_ast_to_edn`, `edn_holon_tag_to_ast`, `edn_to_holon_ast`, `edn_to_holon_ast_natural`, `write_holon_ast_tagged`, `read_holon_ast_tagged`, `read_holon_ast_natural` are **GONE**, and `lib.rs:138`'s export list with them |
| 3 | `Value::holon__HolonAST(h)` renders via `holon_to_watast` + `watast_to_edn` — **one arm** |
| 4 | the `:wat::holon::HolonAST` coercion arm has **no mode selector** (one reader, not two) |
| 5 | `#wat-edn.holon/String "x"` is **REFUSED** on decode — the tag is dead, not dormant (negative control; it can be kept, so it must be — `[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`) |
| 6 | Thermometer + SlotMarker survive: each renders to its `(:wat::holon::<Name> …)` call form |
| 7 | 3 goldens regenerated; the diff is reviewed and each new value is plain EDN |
| 8 | `probe_arc294_holon_bare_leaf_read.rs` rewritten to the post-strike spec and **GREEN with ZERO `#[ignore]`** |
| 9 | floor GREEN via `scripts/floor.sh` — the **Summary line**, never a piped exit code |
| 10 | `cargo clippy --release --all-targets` → **0** |
| 11 | `#[ignore]` count is still **13** — the waterline does not move |

## ⛔ Gate 11 is not bookkeeping

The `#[ignore]` waterline went from 200+ to **13** over one day of deliberate work. While drawing
this stone I committed the probe `#[ignore]`d, per the house convention *"committed `#[ignore]`'d
(RED at HEAD), the strike un-ignores it"* — and the builder caught it: *"you are adding MORE
IGNORES?"*

**That convention is how the pile reached 200+.** It predates the campaign that killed the pile and
nobody reconciled the two. The reconciliation is recorded here: **a strike-ready RED probe is not
committed separately.** It stays in the working tree and lands GREEN in the same commit as the strike
that makes it pass. One commit, zero new ignores, and the probe still serves as the acceptance test.

## Relationship to task #91 — NOT a blocker

Task #91 (*HolonAST census — where is it still doing AST duty rather than VSA duty?*) is a **runtime**
question: which of `eval_walk` / the `step_*` family construct HolonAST for AST purposes. This stone
is a **serialization** question. An enclosing-function census of the 105 `Value::holon__HolonAST`
constructions in `runtime.rs` shows the mass under `to_holon_inner`, `eval_algebra_*`,
`eval_holon_is_*?`, `eval_hologram_*`, `eval_term_*`, `eval_bind_*`, `eval_bundle_*` — VSA duty —
plus a block of `#[cfg(test)]` fns. #91 stays open and stays independent; this stone does not wait
on it, and does not settle it.

⚠ An earlier attribution of those 105 by *"nearest preceding quoted `:wat::` string"* was **wrong**
and is not repeated here — it reported `:wat::core::quote` as a HolonAST producer when `eval_quote`
returns `Value::wat__WatAST` (`runtime.rs:5401`). Nearest-preceding-string is not the owning arm.
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

---

# ⛔ CORRECTION — 2026-08-16, after the first strike went RED. THE ENCODER MUST RENDER **DATA**, NOT SOURCE FORMS.

The strike above was built and reddened the floor on a real, deterministic defect. The builder's
correction, verbatim:

> *"`(:wat::holon::Thermometer 50.0 0.0 100.0)` → `"(:wat.holon/Thermometer 50.0 0.0 100.0)"` — this
> should be illegal.... it should be a record-ish thing... `#wat.holon/Thermometer {:value 50.0 :min
> 0.0 :max 0.0}`"*
>
> *"all things you can represent in Holon is EDN ... HolonAST is literally a vector builder... you
> give it edn (or edn-like data) and you get a vector out who represents that data.... both EDN and
> Holon are completely capable of holding the same data..... one looks like what rich hickey did with
> edn and the other looks like what kanerva did with ternary vectors."*
>
> *"**we do not transmit bind, bundle, atom, etc etc.... /they are data/**"*
>
> *"`#wat.holon {:key1 "val1"}` ;; this is both edn and holon … its also a bundle of typed binds …
> **the only things that need tags are stuff like thermometers as they convey constructor details
> about the data.**"*

## What the first strike actually shipped, measured

```
(:wat::holon::leaf :k2)                    →  ":k2"                                        ✅ data
(:wat::holon::Thermometer 50.0 0.0 100.0)  →  "(:wat.holon/Thermometer 50.0 0.0 100.0)"    ❌ SOURCE FORM
(:wat::holon::Bind a b)                    →  "(:wat.holon/Bind …)"                        ❌ SOURCE FORM
```

Decoding a source form structurally gives a **Bundle**, because `runtime.rs:19711` is unconditional:

```rust
WatAST::List(items, _) => HolonAST::bundle(items.iter().map(watast_to_holon).collect()),
```

A round-tripped Thermometer answers `Bundle/children` — which *raises* on a real Thermometer. That is
the far-side crash in `wat-tests/service-cache-hologram.wat:121`, reproduced on three independent runs.

## ★ THE DEFECT IN THIS STONE, NAMED

**I reached for `holon_to_watast` because it was total and adjacent — and it renders wat SOURCE.**
The job needs a renderer of **DATA**. `holon_to_watast`'s totality made it look correct the whole way
down; totality is not the property that was needed.

**The mechanism existed — SEVENTH instance today of *capability built, never adopted*.**
`from_holon_item` (`runtime.rs:16641`, `pub(crate)`) is the holon→data inverse. `edn_shim` never
called it. Its own error message already states the builder's rule:

> `"unclassified HolonAST (bare Bundle, non-classifier Bind, Permute, Thermometer, Blend, or other
> composite)"`

It converts data-shaped holons (`Map`/`Set`/`Vector`/`List`/`Tuple`/keywords/scalars) back to data and
**refuses everything that is not data** — Thermometer included, because a Thermometer is not data, it
is a constructor directive. The code drew this line before we did.

## THE CORRECTED DESIGN — three cases, no algebra on the wire

```
encode   from_holon_item(h) → Ok(data)  ⇒  #wat.holon <data>
         HolonAST::Thermometer          ⇒  #wat.holon/Thermometer {:value v :min lo :max hi}
         HolonAST::SlotMarker           ⇒  #wat.holon/SlotMarker  {:min lo :max hi}
         anything else                  ⇒  RAISE — neither data nor a directive

decode   #wat.holon <data>              ⇒  to_holon (derive the vector)
         #wat.holon/Thermometer {…}     ⇒  HolonAST::Thermometer
         #wat.holon/SlotMarker  {…}     ⇒  HolonAST::SlotMarker
```

`Bind`, `Bundle`, `Atom`, `Permute`, `Blend` **never appear on a wire in any form** — not as tags, not
as call forms. They are the derived vector structure. A receiver rebuilds them from the data.

The RAISE arm is the wall: a holon that is neither data nor a directive **cannot be silently
mis-transmitted**. Unrepresentable beats guarded.

## ★ BARE `#holon` IS NOT CONSTRUCTIBLE — the crate already ruled it

The builder asked whether to drop the `wat` prefix, then reconsidered on Clojure grounds. The disk
settles it — `crates/wat-edn/src/value.rs:382`:

> *"Build a namespaced tag. Per the EDN spec, user tags MUST be namespaced — **there is no
> `Tag::new(name)` because a no-namespace tag is invalid input.**"*

`Tag::namespace` is **required at the type level, not `Option`**. Census: `Tag::new` → **0 uses (the
constructor does not exist)**; `Tag::ns` → **33**. So `#wat.holon` and `#wat.holon/…` are the only
spellings available, and this is a wall, not a convention.

## What survives from the first strike

The 14-tag deletion, the reader collapse, the mode-selector removal, the export deletions, and the
`#[ignore]`-count discipline were all **correct and stand**. Only the encode and decode arms change:
from `holon_to_watast`/`watast_to_holon` to `from_holon_item`/`to_holon` plus the two directive tags.

## Out of scope, tracked

**Thermometer as a real record** (kwargs `:value :min :max`, so `kwargs-construct` and
`positional-to-kwargs.wat` both reach it, and the directive tag collapses into the ordinary record
wire form) is its **own stone**, ruled by the builder to follow this one. Measured for it: **38 `.wat`
call sites**; `Thermometer` is a Rust intrinsic (`runtime.rs:6459`) so today's registry-driven kwargs
machinery does not reach it; **`SlotMarker` has ZERO `.wat` call sites and no runtime dispatch arm** —
it is not wat-constructible at all, so the kwargs question is moot for it and only its wire form matters.

---

# ⛔ CORRECTION 2 — 2026-08-16. THE DATA TAG IS `#wat/holon`. `#wat.holon <data>` IS UNPARSEABLE.

The RELAND went red on one row and the rider root-caused it as a model gap. It is not. **It is my
tag spelling, and the spelling cannot parse.**

## The wall, on disk

`crates/wat-edn/src/parser.rs:355` — a user tag with no `/` is a hard parse error:

```rust
Error::at(pos, ErrorKind::UserTagMissingNamespace(other.into()))
```

Only built-ins (`#inst`, `#uuid`, the `wat-edn.float` sentinels) may be slashless. So **`#holon {…}`
and `#wat.holon {…}` are BOTH unparseable** — `wat.holon` is a *namespace with no name*, which is the
same violation as a bare name.

**I wrote `#wat.holon <data>` into CORRECTION 1 immediately after quoting the rule it breaks.** I
applied *"user tags MUST be namespaced"* to `#holon`, then violated it one line later.
`[[feedback_a_claims_support_does_not_travel_with_the_claim]]` — the rule and its application were two
lines apart and I still did not carry it across.

## The consequence, and why the rider's STOP-3 is downstream of this

With no parseable tag available, the rider emitted **bare untagged data** (`edn_shim.rs:4013`,
`Ok(v) => value_to_edn_with(&v, types)`). That is the only move left when the specified tag will not
parse. And it is exactly what produced the failure: a `:wat::holon::HolonAST`-declared struct field
whose wire value is bare data is **wire-indistinguishable** from a plain field of the same shape, so
the general decoder cannot know to re-lift it via `to_holon_inner`.

The rider then proposed threading declared field types into `reconstruct_struct`. **That is not
needed**, and its premise was slightly off: the declared type is ALREADY in hand —
`for (fname, fty) in &def.fields` (`edn_shim.rs:3104`) passes `fty` to `rewrap_option_field`. Only the
*dispatch* is Option-only. But a tag is the better cure regardless, because a tag works in **every
position** — struct field, vector element, map value, service argument — while a declared-type lookup
only works where a declared type is in scope.

## THE SPELLING — measured free

```
#wat/holon {:key1 "val1"}        data — re-derived to a holon on read
#wat.holon/Thermometer {…}       directive
#wat.holon/SlotMarker  {…}       directive
```

`Tag::ns("wat", "holon")`. **The bare `wat` namespace is unused** — every existing tag is
`wat.<something>`: `wat.holon` 7, `wat.kernel` 5, `wat.core` 4, `wat.rete` 2, `wat.stream` 1,
`wat.parse` 1, and **zero** `Tag::ns("wat", …)`. No collision.

## Standing findings from the RELAND rider — both real, neither in scope here

1. **`#holon {:a "b"}` (arc 294.b's literal syntax) is BROKEN TODAY, pre-existing.** It lowers via
   `watast_to_holon`'s `WatAST::Map` arm, which builds `Bind(String("Map"), …)` — **missing the
   `Atom(…)` wrap** that its own doc comment specifies and that `to_holon_inner`'s sibling `HashMap`
   arm actually produces. So the builder's canonical example is wire-invisible to `from_holon_item`'s
   classifier match. Not this stone's to fix (`watast_to_holon` has 8 unrelated callers). **Tracked.**
2. **The rider deleted its own reproduction** of the field-decode gap after capturing the finding in
   prose. `[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]` — a reproduction that CAN be
   kept MUST be kept; prose is the medium that rots. If the gap survives the tag, the reproduction is
   rebuilt and kept this time.
