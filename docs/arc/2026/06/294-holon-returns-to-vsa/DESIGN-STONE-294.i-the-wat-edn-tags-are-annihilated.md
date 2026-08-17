# Arc 294 · DESIGN STONE 294.i — `#wat-edn.*` is ANNIHILATED. Two prefixes survive: `#wat.*` and `#holon`.

> **STATUS: DRAWN 2026-08-16. `.opaque` is STRIKE-READY; the other four families await one ruling each.**
>
> Builder's ruling, verbatim: *"255 has been in a state of partial work for months… we do this now….
> we are killing #wat-edn tags…. only tags remain are #wat.\* and #holon … that's it…. this is our
> next target… #wat-edn/opaque has a death warrant."*
>
> Ownership was already settled: `f333bf43` — *"STOP-0 — the #wat-edn tags are arc 294's, not this
> stone's."* This is 294, alongside `294.g` (the holon record's wire is plain EDN) and `294.h`
> (`HolonRepresentable` deleted).

## THE DEFECT, in one line

**`wat-edn` is our Cargo layout leaking into the wire format.** A wat program reading a pipe is being
told the name of one of our crates. Every other tag in the tree already reads `#wat.<ns>/<Name>` —
`#wat.core/Option`, `#wat.lint/Finding`, `#wat.kernel/ProcessPanics`, `#wat.macro/MainSignatureError`.
`#wat-edn.*` is the lone outlier.

## MEASURED 2026-08-16 (post-`294.h`, on `1eaf83ce`)

```
73 tag lines · 22 files · 68 .rs / 5 .wat        59 quoted "wat-edn.*" constants in Rust
 3 golden .edn files                             ~90 docs (HISTORICAL — do not touch)

families:  .holon 54   ·  .float 10  ·  .cap 10  ·  .opaque 4(+16 via helpers)  ·  .result 2
```

⚠ **No cross-language hazard.** `#wat-edn.` appears **ZERO** times in `crates/wat-edn/wat-edn-clj/`
and in `interop-tests/` — the Clojure sibling passes tags through generically and never names one.
(Its `wat-edn.core` / `wat-edn.scanner` hits are its own **library namespaces**, a different thing;
this was nearly reported as a blocker before the `#`-prefix discriminated it.)

---

# PART 1 — `.opaque`: THE DEATH WARRANT (strike-ready)

## ★ THE MODEL, corrected by the builder — `nil` is the RIGHT body

> *"i expect these rust things to just decorate nil….. they contain no edn…. `#wat.io/Sender nil` is
> the data literal for a Sender instance….. a hologram is full of holonic data.. but it cannot be
> represented as edn… we can transmit these as edn but the receiver can gain no knowledge…. there's
> no edn to represent a resource."*

The apparatus had this inverted — it read `opaque_nil` as a **missing encoder** and proposed writing
encoders for the VSA types. That would have meant inventing EDN for things that have none. **The `nil`
body is correct and final.** The tag says *what it was*; the receiver learns nothing more, and that is
the honest report for a resource.

**So the defect is NOT the word `opaque`. It is that `opaque` occupies the NAMESPACE slot** — a
catch-all bucket — where the type's **home** belongs. `#wat.io/Sender nil` names a home and a type;
`#wat-edn.opaque/Sender nil` names a bucket and a type, and throws the home away.

## The destinations

The home is already in the `Value` variant for eleven of them:

| variant | becomes |
|---|---|
| `Value::wat__kernel__Sender` | `#wat.kernel/Sender nil` |
| `Value::wat__kernel__Receiver` | `#wat.kernel/Receiver nil` |
| `Value::wat__kernel__ChildHandle` | `#wat.kernel/ChildHandle nil` |
| `Value::wat__kernel__HandlePool` | `#wat.kernel/HandlePool "<name>"` ⚠ **body is NOT nil** — it carries the pool name today. Preserve it; do not flatten to nil. |
| `Value::io__IOReader` | `#wat.io/IOReader nil` |
| `Value::io__IOWriter` | `#wat.io/IOWriter nil` |
| `Value::wat__core__fn` | `#wat.core/fn nil` |
| `Value::wat__core__clauses` | `#wat.core/clauses nil` |
| `Value::wat__core__extend_def` | `#wat.core/extend-def nil` |
| `Value::wat__stream__Stream` | `#wat.stream/Stream nil` |
| `lazy-seq` | `#wat.stream/lazy-seq nil` — **not a variant.** It is `Stream::Thunk(_) \| Stream::NativeThunk(_)`, a sub-state of the Stream arm, so it shares Stream's home. |

### ★ AND THE VSA FIVE NAME THEIR HOME TOO — through the INNER Rust type

Not derivable from the `Value` variant (bare `Hologram`, `Engram`, …), but the inner type says it:

```rust
OnlineSubspace(Arc<ThreadOwnedCell<holon::OnlineSubspace>>),
Reckoner(Arc<ThreadOwnedCell<holon::Reckoner>>),
Engram(Arc<ThreadOwnedCell<holon::Engram>>),
EngramLibrary(Arc<ThreadOwnedCell<holon::EngramLibrary>>),
```

All five are `holon::X`. **So no namespace has to be invented for them** — which narrows the open
question from *"where do these live?"* to a single naming call the builder owns:

> `#wat.holon/Hologram nil`, or does the bare `#holon` prefix cover them?

The builder's target names `#holon` as a surviving prefix and `#holon {:key1 val1}` as the
holonic-MAP literal — a tag with no name segment. Whether the resource types take
`#wat.holon/<Name>` or sit under `#holon` is the ruling; the HOME is measured, not guessed.

## ★ `RustOpaque` IS ILLEGAL AS A TAG NAME

Builder: *"RustOpaque … feels illegal… that's like an 'abstract class'… a thing that cannot be used,
but is like a holder for other things to use?"* — correct, and the code says so:

```rust
Value::RustOpaque(Arc<RustOpaqueInner>)          // a CARRIER; the inner holds a `type_path`

// today, when the capability path declines:
Tag::ns("wat-edn.opaque", "RustOpaque"),
Box::new(OwnedValue::String(inner.type_path.to_string()))
```

`#wat-edn.opaque/RustOpaque "trading.cache.L1"` — **the tag names the box and the identity is demoted
to a string in the body.** Against the model: the tag must BE the thing, the body must be nil. This is
the same defect class as `wat-edn` itself: an implementation noun of ours standing where the user's
type belongs. `RustOpaque` is a Rust word a wat program must never see.

**The fix already exists, nine lines below the offending arm.** `tag_from_type_path` turns
`:trading::cache::L1` into `#trading.cache/L1`, and is ALREADY used at five other sites in the same
file (structs, enums, records). The `RustOpaque` arm is the one that doesn't call it.

> `Value::RustOpaque(inner)` → `Tagged(tag_from_type_path(&inner.type_path), Nil)`

## ★ AND THE `None` DOOR DIES

```rust
if let Some(t) = types {
    if let Some(cap_tag) = encode_capability(inner, t) { return cap_tag; }
}
// else → fall through to the opaque tag
```

**The same value renders two different ways depending on whether the caller happened to pass a type
registry — and 8 call sites pass `None`.** A portable capability emits `#wat-edn.cap/…` on one path
and a carrier-tagged string on another. The comment calls this "appropriate"; it is a default nobody
audits at the call site. `[[feedback_a_totality_claim_is_only_as_good_as_its_sampling]]` records the
identical shape: one function hardcoded `None` for the type registry and 7 callers silently rendered
records positionally for twelve hours under a green suite — and **deleting the door beat fixing the
callers.**

Same disposition here: **delete the door.** Portability is a property of the VALUE, not of whether the
caller had a registry to hand.

---

# PART 2 — THE OTHER FOUR FAMILIES: facts, and ONE RULING EACH

Builder: *"the holon, cap, float, local…. we need to discuss those… i do not trust your judgement on
them."* **No recommendation is offered below. Facts only.**

### `.holon` — 54 sites, and it is NOT the opaque kind
`Atom · Bind · Blend · Bool · Bundle · Char · Keyword · Permute · SlotMarker · String · Symbol · Tag ·
Thermometer · Vector` — HolonAST **node** types, and they carry **real bodies**:
```rust
HolonAST::Bind(role, filler) => Tagged(ns("wat-edn.holon","Bind"),
                                       Vector([holon_ast_to_edn(role), holon_ast_to_edn(filler)]))
```
This family round-trips actual data. **RULING NEEDED:** the builder's target names `#holon` as a
surviving prefix and `#holon {:key1 val1}` as the holonic-map literal. Do the AST **nodes** become
`#wat.holon/Bind […]`, or does `#holon` cover them too? The container (`Hologram`) is a resource; its
nodes are data — the split is real and the naming is the builder's.

### `.float` — 10 sites, values EDN has no literal for
`#wat-edn.float/nan nil` · `/inf` · `/neg-inf`, written by `crates/wat-edn/src/writer.rs` and read by
`parser.rs:361`'s hardcoded `if ns == "wat-edn.float"`. Decorates nil like a resource, for the
opposite reason: not a handle, a **value with no EDN literal**. **The only family that lives in the
crate rather than the substrate** — it is `wat-edn`'s own spec surface. **RULING NEEDED:** does the
crate's spec namespace move with the substrate's?

### `.cap` — 10 sites, and it is a SECURITY BOUNDARY, not a label
The general decode path **REFUSES** `wat-edn.cap` (`edn_shim.rs:2844`); only the audited trusted-peer
door may reconstruct a live capability. **The namespace string participates in a refusal predicate.**
Renaming it moves a security check. **RULING NEEDED:** confirm the new name, and that the refusal
predicate moves atomically with the emitter.

### `.local` — 3 sites, a fabrication
```rust
// No namespace separator — fabricate a "wat-edn.local" namespace
// so wat-edn's spec-required namespace constraint is met.
Tag::try_ns("wat-edn.local", stripped)
    .unwrap_or_else(|_| Tag::ns("wat-edn.opaque", "unnamed"))
```
A user type path with no `::` gets an **invented** namespace purely to satisfy the spec, and if even
that fails it falls to the SILENT `unnamed` — identity replaced by a word, no error raised.
**RULING NEEDED:** does a namespace-less user type get a fabricated home, or is it a **raise**?

---

# PART 3 — A COLLISION HOLE FOUND WHILE GROUNDING THIS (recorded, NOT designed)

Builder: *"we need every opaque to declare their name….. at compile time we must know there are no
collisions."* **Measured: that guarantee does not exist, at compile time or run time.**

```rust
pub fn register_type(&mut self, decl: RustTypeDecl) {
    self.types.insert(decl.path.to_string());     // ← HashSet::insert's bool is DISCARDED
}
```
A second opaque claiming an existing path **silently wins**. `RustDepsRegistry` is a **runtime**
`OnceLock`, not `inventory`. And the decode side has the same shadow: `decode_capability` is a linear
`caps.iter().find(|c| c.type_path == …)` — a duplicate `type_path` makes the second codec
**unreachable**, on the security-critical door. Its doc promises *"an unregistered name is refused"*,
which is the easy half; the **duplicately-registered** name is what nothing checks.

⚠ `inventory` is already the house style at **13 sites** (intrinsics, special forms, freeze
validators, restriction entries, rete validation, `wat-to-edn-derive`). The opaque registry is a
population that never joined a mechanism twelve others use — the third instance of that shape today,
after `error_ns` and the unrun linter sweep.

**DELIBERATELY NOT DESIGNED HERE.** `NOTE-arc-255-IS-HALF-BUILT` reserves the entry-shape as 255's
DAY ONE decision; minting an opaque-registry shape inside this stone would pre-empt it and add a
FOURTH registry to the arc whose thesis is ONE. Recorded so the hole is not re-derived.

---

## THE GATE (Part 1 only)

| # | assertion |
|---|---|
| 1 | `grep -rn '#wat-edn\.opaque\|"wat-edn\.opaque"' src/ crates/ tests/ wat/` → **0** |
| 2 | `RustOpaque` appears in **no tag name** anywhere; the arm routes through `tag_from_type_path` |
| 3 | the `if let Some(t) = types` door is GONE from the `RustOpaque` arm |
| 4 | every ex-`.opaque` value emits `#wat.<home>/<Name> nil` — **except `HandlePool`, whose body stays its name** |
| 5 | the 3 golden `.edn` files are REGENERATED, and the diff is inspected hunk-by-hunk |
| 6 | floor GREEN via `scripts/floor.sh` — read the **Summary line** |
| 7 | clippy **0** |
| 8 | the run/skip arithmetic accounted for |

**Row 4's exception is the trap.** `HandlePool` carries a name today; "everything decorates nil" is
true of the model and FALSE of that one arm. Flattening it silently drops data.

## STOP TRIGGERS

- **STOP-1 — a `.opaque` member has no derivable home.** RESOLVED before the strike: `lazy-seq` is a
  Stream sub-state, and the VSA five are all `holon::X` inside. If any OTHER member turns out to have
  no home in either its variant or its inner type, report it; do NOT invent a namespace.
- **STOP-2 — killing the `None` door reddens a caller** that legitimately has no `TypeEnv`. Name the
  call site. That is a finding about the 8 `None` callers, not a licence to keep the door.
- **STOP-3 — a golden regenerates with a change beyond the tag prefix.** Capture the diff verbatim;
  a body that changed shape is not a rename.
- **STOP-4 — a red you did not intend. Do NOT re-run.** `scripts/floor.sh` keeps the untruncated log;
  copy the failing block **verbatim**, name the exact arm. There is no such thing as a known flake.

## Kin

- `294.g` (`21b7079f`) · `294.h` (`3656d1e4`) — the same arc, the same thesis, this is the third.
- `255/CHAIN-rendering-before-the-string-home.md` — its stone **B** is this rename. Its stone **A**
  (`const TAG`/`const PORTABLE`) is NOT a prerequisite: A bundles the namespace question with
  portability, and A's proposed name is already the live comms wire trait. Amend the CHAIN when this
  lands.
- `src/error_ns.rs` — twelve namespace constants, all `wat.*`. The door `#wat-edn.*` never walked
  through.
