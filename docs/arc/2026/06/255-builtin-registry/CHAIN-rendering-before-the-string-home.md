# THE CHAIN — A → E: the rendering layer, then the string namespace, then home #4

> Ruled 2026-08-14 against HEAD `2278b350`. **This is an ORDER, and every arrow in it is a
> derivation, not a preference.** Each stone must precede the next because shipping them the other
> way round either migrates the same names twice or deletes a check. Where a "why not the other
> order" was actually argued, it is written down here so the far side does not re-litigate it.

## How this chain got found

It started as one question — *"can `:wat::core::string` become `:wat::string`?"* — and every layer
under it turned out to be load-bearing:

```
rename the string namespace      → but is `concat`/`join` even in it?
  join returns a String, so yes  → but what does join ACCEPT?
    a Seqable                    → and what does it do with the ELEMENTS?
      renders them via `str`     → but `str` is PARTIAL (5 scalars, raises on the 6th)
        so make `str` total      → the total renderer already exists: the EDN encoder
          adopt it               → which broadcasts `#wat-edn.*` — the CRATE NAME — into every output
            rename the tags      → but the `opaque` namespace is ALSO the decoder's refusal key
              so the type must declare portability  ← A, the bottom
```

**The bottom of the stack is A.** Nothing above it is safe to ship first.

---

## ⊘ TRUED UP 2026-08-19 — **THREE OF A's FOUR DEFECTS ARE ALREADY CLOSED, AND A's OWN AMENDMENT IS STALE**

Measured against the disk this session. **Read this before striking anything below** — a reader
working from the un-amended text would hunt three defects that no longer exist and honour a blocker
that has evaporated.

| the chain says | measured 2026-08-19 |
|---|---|
| **A** — tags hand-typed as `wat-edn.opaque` | ✅ **CLOSED.** Every arm carries its own namespace: `("wat.io","IOReader")`, `("wat.kernel","Sender")`, `("wat.holon","OnlineSubspace")`. `#wat-edn.` is down to **2** occurrences tree-wide, both in a **stale test doc comment**; `#wat.` is at 418. |
| **A** — the `opaque` bucket | ✅ **DISSOLVED.** Per-type namespaces, exactly the `#wat.io/Reader nil` shape the builder specifies: *"a resource, a rust struct… anything that's not edn gets mapped to `#some.ns/SomeThing nil` … nil is the honest value."* |
| **A** — silent degrade to `#wat-edn.opaque/unnamed` | ✅ **KILLED AND GATED** (`edn_shim.rs:4703`): *"tag_from_type_path must RAISE, not fabricate the opaque/unnamed placeholder."* |
| **A** — "eighteen arms, each hand-typing its own tag" | ⬜ **16 remain.** THIS IS ALL THAT IS LEFT OF A. |
| **B** — `#wat-edn.*` → `#wat.*/*`, opaque bucket dissolves | ✅ **DONE** (arc 294). |
| **C** — `str` goes TOTAL | ✅ SHIPPED `25d9d015` (unchanged). |
| **D** — `Seqable` as a nameable type | ✅ **SHIPPED. Arc 118 is INSCRIBED** (`ba3bd70c`, 2026-08-19). The *"← TOMORROW"* below is four months of tomorrows and it arrived. |

### ★ A REDUCES TO ONE STRUCTURAL MOVE

**The type declares its own tag, instead of a `match` arm declaring it on the type's behalf.**
Sixteen arms → a `const TAG`. The naming is already right; the degrade is already fatal; the bucket
is already gone. What is NOT yet enforced: the match is exhaustive, so a **new** `Value` variant fails
to compile — but a **misspelled namespace still does not.** Nothing checks that `Value::io__IOReader`
says `"wat.io"` and not `"wat.core"`.

### ★★ A's OWN AMENDMENT (2026-08-16) IS NOW STALE — the blocker evaporated

That amendment says A *"cannot be struck as written"* because `EdnRepresentable` is taken by the live
comms wire trait, and notes the builder's intent: *"EdnRepresentable was meant to replace all
HolonRepresentable."* **That replacement is COMPLETE:**

```
HolonRepresentable ......  0 sites          (arc 294.h deleted it)
EdnRepresentable ........ 46 sites          (was "28 production bounds" when the amendment was written)
```

**Builder, 2026-08-19:** *"all holon-rep should have moved to edn-rep — we built holon first and
seeded wat on holon's tooling (holon data == edn data, just holographically; same with holon-ast =>
wat-ast). **holon must be only used for vsa/hdc things now.**"*

So the question is no longer *"pick a different name."* It is: **do `TAG`/`PORTABLE` join
`comms::EdnRepresentable` (`to_wire`/`from_wire`, the trait the builder wants everything on), or sit
on `ToEdn` (`crates/wat-edn/src/lib.rs:125`, 74 impls)?** That is A's one real design decision and it
is now answerable. `[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`

### ⚠ WHAT IS STILL UNSCOPED — `PORTABLE`

The `const PORTABLE: bool` half — *"the decoder refuses, by TYPE, not by prefix"* — is **not
measured**. The decode side refuses via capability gating (arc 272 6a-i, `edn_shim.rs:1748`), but
whether it keys on the tag's **prefix** or on the **type** was not established this session and must
not be assumed from either. **Read it before drawing A.**

### ⚠ AND THE HOLON→WAT MIGRATION IS NOT FINISHED, one layer over

`HolonRepresentable` is 0, but **`HolonAST` is at 1014 mentions across `src/`** — still doing AST duty
where the builder's ruling says holon is for VSA/HDC only. That is **task #91**'s population, it is
large, and it is NOT this chain's work. Named here only so nobody reads "holon-rep is done" as
"the holon/wat separation is done."

## A — `EdnRepresentable`: the type declares its tag AND its portability

**The defect.** `edn_shim.rs:3726-3764` — eighteen arms, each hand-typing its own tag:

```rust
Value::wat__core__fn(_)       => opaque_nil("wat-edn.opaque", "fn"),
Value::wat__kernel__Sender(_) => opaque_nil("wat-edn.opaque", "Sender"),
Value::io__IOWriter(_)        => opaque_nil("wat-edn.opaque", "IOWriter"),
```

Eighteen names and eighteen copies of the namespace literal. The match is exhaustive — a new `Value`
variant fails to compile, which is the good part — but **misspelling a tag does not**, and at
`:3900`/`:3905` an unnameable type degrades silently:

```rust
Tag::try_ns(&ns, name).unwrap_or_else(|_| Tag::ns("wat-edn.opaque", "unnamed"))
```

An opaque whose name is not a legal tag renders as `#wat-edn.opaque/unnamed` — identity replaced by
a word, no error. Same class as the `str` hole one level up, and the same class as the enum
hand-lists killed in `aa33c0e7`.

**The fix.** The type supplies both facts:

```rust
trait EdnRepresentable {
    const TAG: &'static str;   // ":wat::io::IOWriter"  →  #wat.io/IOWriter
    const PORTABLE: bool;      // false → the decoder refuses, by TYPE, not by prefix
}
```

The trait may well be the existing `ToEdn` (`src/to_edn.rs` + the `wat-to-edn-derive` crate) rather
than a new one — **ground that before drawing**; minting a second trait beside a live one is the
mistake this arc has made in other places.

> ### ⊘ AMENDED 2026-08-16 — **the name `EdnRepresentable` is ALREADY TAKEN. A cannot be struck as written.**
>
> This section said "ground that before drawing" about `ToEdn`, and was right to. It did not know the
> *name* was occupied too. Grounded this session:
>
> | name | where | what it is |
> |---|---|---|
> | `EdnRepresentable` | **`src/comms/mod.rs:102`** | the **live comms wire trait** — `to_wire()` / `from_wire()`. 28 production bounds across `comms/process.rs` + `kernel/peer.rs`. |
> | `ToEdn` | `crates/wat-edn/src/lib.rs:125` | live, **74 impls** |
>
> And the collision is worse than a clash, because `comms::EdnRepresentable` is **the trait the
> builder wants everything on**: *"EdnRepresentable was meant to replace all HolonRepresentable…
> HolonAST and co tooling must only be used for VSA/HDC things."* Arc **294.h** acts on exactly that —
> it deletes `HolonRepresentable` and leaves `comms::EdnRepresentable` as the one wire trait
> (`DESIGN-STONE-294.h-holon-representable-is-deleted.md`, drawn `683eaab8`).
>
> So A's `{ const TAG; const PORTABLE; }` — a **tag+portability declaration**, a different thing from
> a wire encoder — **must pick a different name.** Two live traits, two real jobs, one word between
> them: whoever draws A picks the third name, and does not resolve it by widening either incumbent.
>
> ★ **A also just got smaller.** 294.h removes the comms-side producer of the `#wat-edn.holon/*`
> family — the largest of the five buckets in B's table below. Re-measure B's five families **after**
> 294.h lands; this section's counts predate it.

**Why A is FIRST and not merely nice.** See B.

## B — `#wat-edn.*` → `#wat.*/*`, and the `opaque` bucket dissolves

Builder, 2026-08-14: *"we need to kill off #wat-edn .... everything need to be #wat.\*/\*"*.

`wat-edn` is **the crate name leaking into the wire format**. A wat program reading a pipe is being
told about our Cargo layout. Five families carry it:

| family | examples |
|---|---|
| `wat-edn.opaque` | `RustOpaque` `WatAST` `fn` `Sender` `IOWriter` … (18 arms) |
| `wat-edn.holon` | `Bind` `Blend` `Permute` `Keyword` `Char` `SlotMarker` |
| `wat-edn.cap` | `address` `test-token` |
| `wat-edn.float` | `nan` `inf` `neg-inf` |
| `wat-edn.local` | `edn_shim.rs:3904` |

~30 code sites, 22 test files, 286 doc files (docs are mostly historical prose — leave them).

**Opaques stop having a bucket at all.** The tag is just the type's FQDN under the rule already in
use (`#wat.core.Option/Some`, `#user/Pt` — last segment is the name, the rest joined by `.`):

```
#wat.core/fn nil        #wat.kernel/Sender nil        #wat.io/IOWriter nil
```

### ⛔ CORRECTION 2026-08-14, BY MEASUREMENT — the claim below is OVERSTATED

**Written first, measured after — the wrong order, and the run refuted me.** The section that follows
argues that renaming the namespace *deletes a capability check*. **It does not.** Measured live
against the built binary:

```
(:wat::edn::read "#wat-edn.opaque/IOWriter nil")  → REFUSED  (edn_shim.rs:2860, the ns check)
(:wat::edn::read "#wat.io/IOWriter nil")          → REFUSED  (edn_shim.rs:2957, unknown substrate tag)
(:wat::edn::read "#wat.io/IOWriter []")           → REFUSED
(:wat::edn::read "#wat.io/IOWriter {}")           → REFUSED
(:wat::edn::read "#wat.io/IOWriter [1 2]")        → REFUSED
```

There is a **second, general refusal** behind the namespace one: an unrecognized substrate tag is
rejected whatever its body. The system **fails closed**. Renaming the namespace degrades a specific,
well-worded refusal into a generic one — a **diagnostics** regression, not a soundness hole.

**What survives, and it is still the stone:** the twelve hand-typed tags bypass `tag_from_type_path`,
which already exists and already does the FQDN→tag rule; they DISCARD the namespace (`Sender`, not
`wat.kernel/Sender`) where leaf collisions are abundant in the type space (`Atom` ×17, `Bundle` ×16,
`Bind` ×15); and `:3900`/`:3905` silently degrade an unnameable type to `unnamed`. **What dies:** the
ordering argument as stated. A-before-B is now moot for opaques rather than load-bearing — deriving an
opaque's tag from its FQDN *is* renaming it, so they are one edit, not two.

The failure is kept visible rather than edited away: this is
[[feedback_measure_the_decomposition_never_read_it]] on a *security* claim, and the tell was that I
asserted a consequence from reading two code paths without running either.

### The original argument, kept for the record — the namespace IS doing two jobs

`edn_shim.rs:2858`:

```rust
if ns == "wat-edn.opaque" {
    return Err(UnsupportedTag(…));   // "no serializable identity"
}
```

That string is **the decoder's refusal key**, and the refusal is a security property, not
formatting — `registry.rs:2830` states it: *"REFUSED exactly like an opaque: an object-capability is
obtained by being handed it over a [wire], never by parsing it."* Once a handle renders as
`#wat.io/IOWriter` it is structurally identical to a legitimate tagged record, and **the
discriminator is gone.**

**Do B without A and you delete a check.** Do A first and the check gets *stronger*: portability
becomes a declared type fact, and a type that never declares it cannot compile — where today an
unknown one silently renders `unnamed`.

A also makes B cheap: once the namespace lives in one constant, the rename is a one-line change for
the opaque family instead of eighteen.

## C — 279.2: `str` goes TOTAL — ✅ **SHIPPED `25d9d015`, 2026-08-14**

> ⊘ **AMENDED 2026-08-17.** This section said "RED" for three days after it went green, and that
> staleness was quoted forward into the seam and three later stones. C is DONE.

**Drawn, probe committed and RED at `2278b350`.** See `279-format/DESIGN-STONE-279.2-str-totality.md`,
its BRIEF and EXPECTATIONS. `tests/value/probe_arc279_str_totality` is 3 controls green / 5 rows red.

279's own `DESIGN.md:67` specified `str` as rendering *"ANY value unquoted (…)"*; a five-arm match
shipped. `str` and `show` both become the EDN encoder, differing at exactly one place: a **top-level**
`Value::String` renders bare under `str`, quoted under `show`. Nested strings stay quoted in both.

**Why C comes after B.** C routes `str` through the encoder, so every `str` and `format` call site
starts emitting the encoder's tags. Ship C first and `#wat-edn` spreads across the whole surface
before it is renamed — the same rename-before-carve logic that reordered home #4 on 2026-08-13.

**Ruled in passing, do not re-open:** map key ORDER is not normalized — *"maps are unordered.... we
don't do string equality here, we do data equality."* And `render_value` SURVIVES: `ValueSnapshot::of`
still needs it for diagnostics, which has its own depth cap and golden blast radius.

## D — `Seqable`, and `join` renders its elements — ✅ join half SHIPPED · Seqable is TOMORROW

> ⊘ **AMENDED 2026-08-17 — D IS TWO STONES, and the signature below was written against a type that
> does not exist.**
>
> | | status |
> |---|---|
> | `join` renders its elements | ✅ **279.3 `8b2cdbf2`** — and it stayed a Rust INTRINSIC: `core.wat` ↔ `string.wat` is a real bootstrap cycle. It ships over **`Vector<T>`, NOT `Seqable<T>`** — narrower than this section specifies. |
> | (sibling) `interpolate` goes total | ✅ **279.4 `94142919`** — the partial renderer DELETED; one rendering fn, three callers |
> | `Seqable` as a nameable type | ← **TOMORROW.** 118.3-B (`a15f4ea9`) landed the mechanism |
>
> ⛔ **`wat.type/Seq` DOES NOT EXIST and never did.** The `(seq [self] :- (wat.type/Seq [T]))`
> signature below names a type nobody built. The real contract — intermediate vs terminal, what
> `seq` returns, and why `length`/`empty?`/`get` REFUSE a Stream — is ruled in
> **`docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN-118.4-the-seqable-contract-terminal-vs-intermediate.md`**.
> Read that, not the sketch below.
>
> ★ And the three blockers this section's neighbours cite from `infer.rs:638` were **STALE** — refuted
> by a test that was already green a month before they were written.
> `118/NOTE-the-blockers-were-stale-seqable-is-spellable.md`



```
(defsurface  wat.type/Seqable [T]  (seq [self] :- (wat.type/Seq [T])))

(wat.string/join [T] :- wat.type/String
  [sep :- wat.type/String
   xs  :- (wat.type/Seqable [T])])
```

`T` is **unconstrained**, and that is the payoff of C: with a total `str` there is nothing left to
constrain it by. Ruby's `join` needs no bound for the same reason.

Extended to Vector · PersistentVector · HashSet · Seq · Stream. **Terminal ops consume** — a Stream
handed to `join` is consumed, which is not a problem to solve (builder: *"if the user passes it a
stream, its consumed - why is this confusing?"*). Single-pass is a property of the value, never of
the surface.

`join` today is `(sep: String, pieces: Value::Vec)` with every element required to already be a
String — `string_ops.rs:455`. That signature is what D deletes.

**Mechanism note, measured:** this is `defsurface`, **not** `defprotocol`. Arc 293.4e annihilates
`defprotocol`; `AGGREGATE-MODEL.md` — *"methods-only [surface] = the old `defprotocol`."* Parametric
surfaces already ship (`:wat::cache::Cache<K,V>`, `:wat::capability::Dialable<S,R>`, 123 sites) and
the three 293.4e-pre probes are green in the floor, including generic surface methods with type
params. Related arc: **285-collection-protocols**.

## E — `wat.string/*`, then home #4

**1,617 code sites** across 22 verbs (wat 567 · wat-scripts 622 · wat-tests 71 · src 139 · tests 218).
By **wat-fix codemod**, never by hand — prior art is `wat-scripts/fixes/rename-kernel-to-spawn.wat`,
which re-parented a whole namespace the same way.

- **Already ruled, twice, and never executed:** `109-kill-std/NOTE-stdlib-namespace-homing.md` names
  `wat.string/` as the home; `278/SEAM.md:82` — *"`wat.string/join` ; string RELOCATES, Clojure-style"*.
- **The file already moved and the namespace did not** — `wat/string.wat` is top-level, not
  `wat/core/string.wat`. That divergence is on disk right now.
- **No reserved-prefix work.** `RESERVED_PREFIXES` reserves `":wat::"` at the root, covering every
  sub-namespace; `:wat::string::` is language-owned the instant it exists.
- **The type is untouched.** It is `:wat::core::String` (capital S); the rename's prefix is
  `:wat::core::string::` with the trailing `::`, which cannot match it. `wat.type/String` and
  `wat.string/join` never collide. Same discipline `rename-kernel-to-spawn.wat` documents in its own
  header: use the FULL name as the prefix, because the parent segment is shared.
- **The rete mirror moves too.** Every string op has a paired `:wat::rete::core::string::*` row in
  `rete/vocabulary.rs`, and admission is `RETE_MODULES` — a module SET. `vocabulary.rs:1565` already
  asserts every row is admitted, so a half-done move SCREAMS rather than silently dropping rows.
  **OPEN, and it is the builder's call:** does the mirror follow to `:wat::rete::string::*`, or keep
  `rete::core::` as its own module identity?
- **`concat` is NOT in this stone.** `string::concat` is String→String and `Vector/concat` is
  Vector→Vector — same-kind-in-same-kind-out, which is genuine receiver dispatch and a separate
  question. `join` is not: it always returns a String, which is why it stays in `wat.string/`.

**Then home #4** — the `core::string` carve into the intrinsic registry, landing on final names with
final signatures, once instead of twice. This is why home #4 moved from "first strike" to last: it
registers exactly the names and shapes A–E change.

---

## The order, and what breaks if you take it out of order

| | stone | ship it early and… |
|---|---|---|
| **A** | `EdnRepresentable` | — (it is the bottom) |
| **B** | `#wat-edn.*` → `#wat.*` | **you delete the decoder's refusal check** |
| **C** | `str` total | `#wat-edn` spreads to every `str`/`format` site first |
| **D** | `Seqable` + `join` | `join` needs a type-variable bound wat has no form for |
| **E** | `wat.string/*` + home #4 | 22 names migrate twice; the registry registers stale ones |

## Standing rulings this chain rests on — do not re-derive

1. **`join` renders its elements** (builder: *"A is clearly the only answer"*). Not
   `Seqable [String]` with the caller mapping `str` first.
2. **`str` is total; `show` is `str` with top-level strings quoted.** One rendering, one difference.
3. **Maps are unordered** — no key-order normalization, anywhere in this chain.
4. **The `nil` body on an opaque is contract, not laziness** — it means "no transferable content",
   and the decode side refuses it. A carries that meaning to the type; it does not delete it.
5. **`defsurface`, never `defprotocol`.** 293.4e annihilates the latter.
