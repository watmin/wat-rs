# DESIGN — arc 285: `:wat::core::Map`, and the end of "a map is a map" being a claim

Opened 2026-08-20 out of arc 255's home #12. The STUB (banked 2026-06-17) is corrected in place;
read its banner first. This DESIGN is what the arc actually is, measured.

## What is already built — do not rebuild it

| the stub's Layer | status |
|---|---|
| **Layer 1** — shared op names across both families | ✅ shipped. 19 `:wat::core::Persistent*` dispatch arms in `runtime.rs` |
| **Layer 2, Seq half** | ✅ shipped IN arc 278 as **`Seqable<T>`** (`wat/seq.wat:75–91`) — `Vector`, `List`, `PersistentVector`, `Stream` |
| **Layer 2, Map half** | ⛔ **this arc.** No Map-side surface exists in `wat/` |

## The crux — asked, probed, ANSWERED

The stub names one unknown: *can a built-in `Value` type satisfy a wat-defined protocol?* It can —
`Seqable<T>` has done it since 278. **But that is not the hard case, and the stub never names the
hard case:**

```
Seqable<T>        1 type param  ·  BUILT-IN types      wat/seq.wat:75
Dialable<S,R>     2 type params ·  user Struct         wat/capability.wat:44
Map<K,V>          2 type params ·  BUILT-IN types      ← neither precedent covers this
```

**Probe: `wat-scripts/scratch-pad/probe-285-map-surface-over-builtins.wat`** (loader-gated, so it
cannot rot). Measured at HEAD `9b360374f`:

```
(:user::lookup <HashMap>)        →  #wat.core.Option/Some [1]
(:user::lookup <PersistentMap>)  →  #wat.core.Option/Some [2]
```

One surface, two type params, both built-in families, one surface-typed fn param accepting either,
runtime dispatch reaching the right Rust intrinsic. **The arc is buildable today with ZERO new
substrate.** The stub's *"unless it proves very difficult to build, which must be GROUNDED, not
guessed"* is discharged: it is not difficult, and this is the grounding.

## ★ The feature set is measured, and the stub's names DO NOT EXIST

The stub proposes `get`/`assoc`/`dissoc`/`keys`/`vals`/`contains?`/`count` — Clojure's spelling.
The substrate's actual ops, taken from `runtime.rs` dispatch (both families, **set difference
EMPTY**):

```
assoc · contains-key? · dissoc · empty? · get · keys · length · values
```

`vals` → **`values`** · `contains?` → **`contains-key?`** · `count` → **`length`**. A surface written
from the stub would declare three methods that do not exist. The eight above are the feature set.

That the difference is empty is itself the finding that makes this arc cheap: **Layer 1 already
delivered name parity perfectly.** This arc adds the TYPE that lets a caller depend on it.

## ⚠ THE ASYMMETRY THE STUB NEVER SAW — and it is a live "a map is a map" violation

The two constructors do not agree, and both directions were tested:

```wat
(:wat::core::HashMap :wat::core::String :wat::core::i64 "a" 1)   ; REQUIRES leading K V type keywords
(:wat::core::HashMap "a" 1)                                       ; → "first two arguments must be type keywords"

(:wat::core::PersistentMap "a" 2)                                 ; INFERS K,V from the pairs
(:wat::core::PersistentMap :wat::core::String :wat::core::i64 …)  ; → "TYPE keyword, not a value"
```

The op names are identical; the CONSTRUCTORS are not. So today a caller cannot write one line that
builds "a map" of either kind — the fracture the stub calls dishonest survives at the one point the
stub did not look. `infer_hashmap_constructor` and `infer_persistentmap_constructor`
(`check.rs:3132`, `:3142`) are the two sites.

**This is in scope.** A `Map` surface that unifies the eight ops while leaving construction forked
would satisfy the letter of "a map is a map" and miss its point.

## Scope

1. **`:wat::core::Map<K,V>`** — a `defsurface` carrying the eight measured features.
2. **`extend-type` for `HashMap` and `PersistentMap`**, each method delegating to its existing Rust
   intrinsic — the `Seqable<T>` shape exactly.
3. ~~**Constructor parity**~~ — **MOVED OUT 2026-08-20, and it was never this arc's to decide.**
   The asymmetry measured here (`HashMap` requires leading `K V`; `PersistentMap` refuses them) is a
   filed, grammar-decided item: `109/NOTE-typed-literal-constructors.md`, builder-directed 2026-07-24
   and closed 2026-08-15. The destination is `(Head [type…] …values…)` for every parametric, and it
   ships in `109/DESIGN-STONE-all-parametrics-take-a-type-vector.md`. **This arc's honesty clause is
   therefore satisfied BY 109, not by a decision taken here** — and 109 must land first, or a `Map`
   surface would unify eight ops while construction stayed forked.
   ⚠ I wrote "the one real design decision in this arc, deliberately not pre-decided here." It was
   already decided, in a note I had not read. The builder pointed at it.
   `[[feedback_read_the_epitaph_before_you_build_on_prior_art]]`
4. **A consumer that proves it** — see below. Without one this arc ships unarmed.

### Out of scope — affirmatively cut

- **Renaming `Seqable<T>` → `Seq`.** The Seq half shipped and works; the name is not wrong, and a
  rename is a corpus codemod bought for symmetry alone. If it happens it is its own stone.
- **A Clojure-style lazy seq abstraction.** The stub already cut this; it stays cut.
- **`HashSet` / persistent set.** No demand measured; not enumerated here so it cannot be assumed.

## ★★ THE CONSUMER — arc 255's io split, and this is why the arc opens NOW

The stub names arc 278 (rete) as its forcing consumer. **278 is parked, and rete did not wait** — it
typed against the concrete impl (`bindings <- :wat::core::PersistentMap`, `wat/rete.wat:30`) and
works. Built for that consumer, this arc ships an UNARMED mechanism, which `wat/rete.wat:668` names
in its own words: *"an unarmed mechanism is R59's dead protocol — a green floor certifying something
that has never once run."*

Home #12 supplied a real one. **Six of the 29 `:wat::io::` verbs work on exactly one backing** while
wearing the shared interface's name — the substrate says so in its own error text, *"writer does not
support snapshot (only StringIoWriter does)"* (`src/io.rs:1404`):

```
IOWriter/new · IOReader/from-bytes · IOReader/from-string   always construct a StringIo*
IOWriter/to-bytes · IOWriter/to-string                      raise on any other backing
IOReader/rewind                                             raises on Pipe; ruled to raise on stdin
```

Making `IOReader`/`IOWriter` **surfaces** with a concrete `StringIo` extending them puts
`(rewind stdin)` beyond representation rather than faulting at runtime — the no-form rung instead of
the check rung. That is the same shape this arc builds for maps.

⚠ **The io split does NOT depend on this arc** — the mechanism is proven either way. The ordering is
STEPPING-STONE: *"a map is a map"* has an obvious right answer, while the io split still has open
design questions (does `StringIo` extend both reader and writer? what does `new`'s `@Category`
become?). Establish the recipe where the answer is not in doubt, then apply it where it is.

Builder, 2026-08-20: *"using 285's loot to help us with the 255 dilemma."*

## Held open by this arc, deliberately not patched first

- The **`StringIo` rename** — would migrate 25 corpus call sites that the type split then moves again.
- The **`rewind` fault change** — builder-ruled that every non-string backing must fault; but if
  `rewind` only ever takes a `StringIo`, the fault branch is never written. Do not construct the
  situation that needs the patch.
- ⚠ **LIVE COST, stated so it is not implicit:** `RealStdin::rewind` (`src/io.rs:179`) returns
  `Ok(())` today — silently succeeding while doing nothing.

## The four questions

- **Obvious?** YES — `[m <- :wat::core::Map<K,V>]` says what it accepts, and `Seqable<T>` is the
  worked precedent one file away.
- **Simple?** YES — one surface, eight features, two `extend-type`s delegating to intrinsics that
  already exist. The constructor-parity decision is the one place judgement is spent.
- **Honest?** YES, and only because of the constructor clause: a surface that unified the ops and
  left construction forked would claim "a map is a map" while the fracture stayed.
- **Good UX?** YES — generic code stops choosing an implementation in order to name a type.
