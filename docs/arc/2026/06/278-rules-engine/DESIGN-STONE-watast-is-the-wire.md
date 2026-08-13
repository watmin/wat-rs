# DESIGN STONE — `WatAST` is the universal top of the WIRE, and the decode guard is its last unclosed edge

> **Drawn 2026-08-12.** Nothing built. Found by a probe that set out to prove rules ship over a
> pipe and instead found the boundary refusing the one type that cannot fail it.

---

## The law, stated first because everything follows from it

**Only a `WatAST` can cross the wire. That is the whole of it.**

An `i64` crosses because an i64 *is* a WatAST. A String, a keyword, a record's tagged map — each is
EDN, and EDN *is* WatAST. What cannot cross — a `Peer'`, a live fd, a handle — is exactly what
cannot be written as a form, which is the line 293.W already draws.

So the two boundaries are one boundary:

```
what may cross the wire  ==  EDN  ==  WatAST  ==  what holds no fd and no peer
```

A declared field type is therefore **not** a gate on *whether* a value may cross. It is a
**refinement applied after decode** — a narrowing of the WatAST that has already arrived. For every
type that narrowing is a real predicate. For `:wat::WatAST` itself the narrowing is the **identity**,
and identity is the one case the guard does not implement.

## ★ WHAT THIS STONE ACTUALLY IS — the tail of an unfinished migration

Not "the guard is wrong about one type." **The wire was built for the PREDECESSOR AST, and one
edge of the successor's migration was never walked.**

The builder's history, and it explains everything above: *"all of wat started on holon-ast … we
slowly built out wat-ast and … never fully killed holon-ast."*

So `src/edn_shim.rs:42`'s table is chronology, not design:

```
| HolonAST | Tagged per variant (Symbol/String/I64/F64/Bool/Atom/Bind/Bundle/Permute/Thermometer/Blend) |
```

HolonAST got the wire treatment because it **was** the AST when the wire was built. WatAST grew up
underneath it and the encoder never caught up — it serializes faithfully (`:604-607`) and the decode
side has no arm for it. `types.rs:915` shows the seam in one sentence: *"wat::WatAST, the terminal
value as wat::holon::HolonAST."* Two AST types, a bridge between them, and one of them older.

**⛔ RULED 2026-08-12 (the builder): HolonAST is not going anywhere — it is for VSA ops now.**

That is a ruling on what the thing IS, which is the only honest tiebreaker
(`[[feedback_no_consumers_does_not_mean_dead]]`). It also explains why the two fixes are DUALS
rather than the same fix:

| | identified by | why |
|---|---|---|
| **HolonAST** | **its tag** | its composites (`Bind`/`Bundle`/`Permute`/`Thermometer`/`Blend`) are VSA operations with NO bare EDN spelling — the tag is what makes them EDN at all. It must stay tagged. |
| **WatAST** | **"it parsed"** | its variants ARE the EDN shapes — a List *is* `(…)`, a Keyword *is* `:foo`. There is nothing to tag, and tagging would break `write-forms` output being `read-string`-able as plain text. |

So WatAST is not catching up to HolonAST. Once the identity arm lands it crosses by the **cheaper**
mechanism: HolonAST pays a tag per node; WatAST pays nothing, because the bare list on the wire IS
the form.

**⚠ NOT MEASURED, and the distinction cost real time today:** the table documents the ENCODER. No
HolonAST has been observed crossing a live service op in this session — "HolonAST was already
permitted" is well-evidenced, not proven. Worth a one-op positive control before any realization
claims a gap closed; a sentence resting on a table is not resting on a run.

**Filed separately:** where HolonAST still does AST duty rather than VSA duty is a CENSUS question,
not a guess — and per the ruling above it produces an inventory needing dispositions, never a kill
list.

## The symptom, measured

`wat-scripts/scratch-pad/probe-arc278-rules-cross-the-wire.wat` — a real `defservice`, process
locus, one op taking `[defs <- :wat::core::Vector<wat::WatAST>]`:

```
SUBJECT (helper IN payload) => LOST disconnected
CONTROL (helper OMITTED)    => LOST disconnected
```

Decomposed in `probe-arc278-watast-on-the-wire-decomposed.wat` — same service, two ops differing
ONLY in the request field's type:

```
CONTROL echo(i64)          => Ok n=7                                              (process)
SUBJECT count(Vec<WatAST>) => LOST disconnected                                   (process)
ISOLATOR count THREAD      => REQUEST-MALFORMED expected=:wat::WatAST got=List
                              path ["defs" "[0]"]                                 (thread)
```

The frame **arrived** and the guard walked into element 0 — the value crossed. It then compared the
decoded shape against the declared type and refused a **List**. But a List *is* a `WatAST`.

And `println` shows exactly why there is nothing for it to match on:

```
[(:wat.core/defrecord :usr/A [c <- :wat.core/i64]) (:wat.core/defrecord :usr/B …) …]
```

A form crosses as a **bare, untagged EDN list**. Every other typed value crosses tagged
(`#wat.core/Foo {…}`) and the tag is what the guard identifies it by. A form has no tag because it
**is** the EDN. There is nothing to mark.

## Why a tag is the wrong fix — the chapter that already ruled this

`algebraic-intelligence.dev` **Chapter 59 — "42 IS an AST"** is this exact recognition one substrate
over. `HolonAST::Atom` was `Arc<dyn Any>`: an escape hatch that let anything be an atom payload, at
the price of three workarounds (no structural hashing, an `AtomTypeRegistry` living beside the
algebra, a wat-lru shim that panicked on non-primitive keys). The fix was not a better wrapper. It
was recognising that **42 is already a leaf**:

> *the algebra was never open — we'd just been carrying an escape hatch that pretended otherwise…
> nothing was added to the math; the code caught up.*

Our guard is doing what `dyn Any` + `AtomTypeRegistry` did: demanding a **nominal marker** for
something **structurally already the thing**. Tagging `WatAST` on the wire would re-mint, at the
boundary, precisely the escape hatch that chapter deleted.

The chapter also called this shot, under what closing the algebra unblocks:

> **Cross-process AST handoff becomes straightforward.**

That is the thing this stone is for. The guard is the last edge where the algebra is not closed.

**Honest bound:** `HolonAST` (the VSA algebra) and `WatAST` (wat forms) are **distinct types**. This
is the same *shape* of lesson, not the same object, and the fix lands in a different file.

## The precedent, in our own source

R7 solved the identical shape for the **type** lattice in one branch (`src/types.rs:5212`):

```rust
// :wat::core::Value is the universal subtype-top: every type <: Value.
if sup == ":wat::core::Value" { return true; }
```

`Value` is the universal top of the type lattice; `WatAST` is the universal top of the wire. Same
move, one domain over, one branch. R7's own words: *the universal top is a fixed point you point at,
not a feature you build.*

And the substrate already asserts the premise at `types.rs:1007` — *"`:wat::WatAST` holds no fd and
no peer (a form is a tree of keywords…)"* — declared wire-pure, then refused at the wire.

## THE ONE CONTRACT DECISION

**In the decode walker, a declared `:wat::WatAST` accepts any well-formed EDN value. The refinement
is the identity; it can never fail.**

Not a tag on write. Not a special case in `defservice`. One arm in the walker every op already
routes through, so the law holds for every service in the substrate at once.

## Where it lands

`wat/service.wat:1437` generates the guard into every op arm unconditionally:

```clojure
shape-guarded `(:wat::core::match (:wat::edn::validate ~req-binder ~req-ty-kw) …)
```

`:wat::edn::validate` (`runtime.rs:15213`, dispatched at `:4726`) is by its own doc *"a THIN WRAPPER
over the deep walker … `edn_shim::edn_to_typed_value` walks the declared `TypeExpr` per-field /
per-element and yields the offending path"*. That walker's `TypeExpr::Path` arm is the site.

**⚠ THE ADJACENT-IMPLEMENTATION TRAP.** `conforms_check` (`runtime.rs:15307`) has a near-identical
`TypeExpr::Path` arm and is **NOT** the subject — it returns a bare `bool` and yields no path, and
`validate`'s doc says outright that `conforms?` *"cannot serve here"*. The subject is the walker that
produced `path ["defs" "[0]"]`. `[[feedback_an_adjacent_implementation_is_not_the_subject]]` — four
instances in one session on the record. Confirm by the path, not by proximity.

## What this does NOT weaken

A `WatAST` field is an **untyped hole in a typed wire** — `[n <- i64]` says one thing, `[defs <-
Vector<WatAST>]` says "any EDN whatsoever." For anything but shipping code that would be a hole to
refuse. For code it is the point, and the typing does not vanish — it **relocates**. Measured, in the
declared-payload probe's own control arm:

```
CONTROL (helper OMITTED) => CHECK-FAILED
    "the accumulated definition set no longer freezes on its own"
      → 1 unresolved reference :usr::big?
```

Untyped at the wire; **fully typed at the freeze**, where the forms become a world and meet the whole
checker — unresolved references, law A, purity, all of it. This is
`DESIGN-STONE-connection-lifecycle-ops.md`'s *"the parse is the check"*, arriving from the other
direction: it wrote that about chunked text, and it is the general property of shipping code in a
homoiconic substrate.

## The four questions

- **Obvious?** YES — a form is EDN; a guard that refuses EDN where a form is declared is asking a
  question with one possible answer.
- **Simple?** YES — one arm in one walker; no new machinery, no tag, no per-service change.
- **Honest?** YES — it closes the algebra rather than papering the boundary, and it names the cost
  (an untyped field) rather than hiding it.
- **Good UX?** YES — cross-process form handoff works the way homoiconicity already promised.

## ⛔ SEPARATE FINDING, tracked not bundled — the LOCUS ASYMMETRY

The same condition is **faced** on one locus and **lost** on the other:

| locus | outcome |
|---|---|
| thread | `RequestMalformed` carrying `path` / `expected` / `got` |
| process | `LOST disconnected` — the cause gone |

One locus surfaces the failure as a value; the other destroys it. That is R53/R57's
no-hidden-failures class surviving at a **locus** boundary, and it is why this took three runs to
diagnose instead of one. It is a real defect and it is **not** this stone — fixing the guard makes
this path stop firing, which would hide the asymmetry rather than close it. Its own strike.

## NOT in this stone — affirmatively cut

- **`walk.rs`'s `skip(4)` (#90)** and **`validate.rs:453`'s boundary blindness.** Same family as the
  expander drift (`f1a811cb`), unrelated to the wire.
- **The `install-rules` macro and the rete service.** Downstream consumers; they are unblocked by
  this, not part of it.
- **Payload size / chunking.** `write-forms` + `read-string` remain the right answer for
  transmission AT SIZE (the lifecycle stone's own scope). That is a different question from whether
  a form can be a typed field at all, and this stone must not be read as settling it.

## ⛔ STRUCK 2026-08-12 — the arm LANDED and the gate was DRAWN WRONG. Three defects, not one.

The identity arm is on the disk and correct, weighed by the orchestrator's own `--release` re-run:

```
ISOLATOR count THREAD      => Ok n=3        (was REQUEST-MALFORMED — the walker fix, proven)
GATE-3 bare WatAST field   => VALID
GATE-4 i64 handed a String => INVALID at ["n"] expected=:wat::core::i64 got=String
floor 4391/4391 passed, 0 failed · clippy 0
```

**Gate rows 1 and 2 are NOT met, and that is this stone's error, not the strike's.** It was drawn
believing there was ONE failure. A rider that kept digging when the gate disagreed with it — and
found the root with `strace -f` on the child rather than by theory — established there are **three**:

| # | defect | state |
|---|---|---|
| 1 | thread-tier post-decode validate refuses a form (`expected=:wat::WatAST got=List`) | **FIXED — this stone** |
| 2 | process-tier GENERIC UNTYPED decode refuses any `Edn::Symbol` | **OPEN** |
| 3 | the child's `Reply::Failed` never reaches the client, which sees `LOST disconnected` | **OPEN** |

**Defect 2, verbatim from the child's own write, caught by strace:**

```
poll (process tier): client message decode failed: src/edn_shim.rs:1773:52:
EDN Symbol — wat has no symbol value type
```

`edn_to_value_caps` (via `decode_trusted_wire`, `runtime.rs:28719`) runs FIRST, untyped, to
determine WHICH op a frame is for — before any type-directed walk is reachable. It refuses
`Edn::Symbol` unconditionally, and a real form always contains symbols (`<-`, bare identifiers).
So no WatAST reaches a process-locus op at all, and this stone's arm is downstream of a door that
never opens. It predates this work and is unaffected by it.

**★ THE ARCHITECTURAL POINT, which is this stone's law arriving one level down.** The wire currently
decodes **generic-then-validate**: EDN → `Value` (LOSSY — `Value` has no symbol variant, by design)
→ refine against the declared type. If the law holds — *only a WatAST crosses* — then the honest
order is inverted: **EDN → WatAST (TOTAL, lossless, every EDN value is one) → refine to the declared
type.** The lossy generic step is where defect 2 lives, and `edn_to_watast`
(`wat_edn_bridge.rs:412`) already exists to do it.

**Defect 3 sharpens the locus asymmetry recorded below, and worsens it.** The child does NOT fail
silently — the strace shows it constructing and writing a proper `#…Reply/Failed [#wat.kernel/Failure
{…}]` frame carrying the full cause. The CLIENT loses it and reports `LOST disconnected`, where
`wat/service.wat:727`'s own comment documents an unignorable raise. A failure is faced, written to
the pipe, and destroyed in transit.

## The gate (as originally drawn — rows 1 and 2 superseded by the box above)

1. **★ `probe-arc278-rules-cross-the-wire.wat` goes green** — `SUBJECT DERIVED n=1` and
   `CONTROL REJECTED check-failed`. It is red-by-measurement today and its header says so; rewrite
   that verdict when it turns.
2. **★ `probe-arc278-watast-on-the-wire-decomposed.wat`** — all three arms `Ok`, the thread arm
   included (`count THREAD => Ok n=3`).
3. **A bare `WatAST` field**, not only `Vector<WatAST>`, crosses — the parametric and the bare case
   are different code paths and only one has been measured.
4. **Nothing else loosens.** A genuinely wrong field (`[n <- i64]` handed a String) must still come
   back `RequestMalformed`. The identity arm applies to `WatAST` alone.
5. Floor unmoved but for the new probes; clippy clean.
