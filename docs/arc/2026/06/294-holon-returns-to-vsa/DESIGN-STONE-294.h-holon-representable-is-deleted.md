# Arc 294 · DESIGN STONE 294.h — `HolonRepresentable` is DELETED; the wire trait is `EdnRepresentable`, and it always was

> **STATUS: DRAWN 2026-08-16 against HEAD `6b5c8232`. STRIKE-READY.**
>
> Builder's ruling: *"EdnRepresentable was meant to replace all HolonRepresentable or whatever… the
> HolonAST and co tooling must only be used for VSA/HDC things… HolonAST was a bridge for WatAST to
> get built."* Then, on the eight container impls: **B has been reasoned.**
>
> This is **294.g's sibling**, not a new arc. `294.g` (`21b7079f`) annihilated the holon *record's*
> wire. This closes the other half of the same flaw — DESIGN.md flaw 3, verbatim: *"The
> `#wat-edn.holon/*` tags are scar tissue. They exist only to serialize a HolonAST losslessly to EDN
> for the wire… With EDN canonical, the wire ships plain native EDN."*

## THE FINDING — the trait has no production consumers, and never did

Seven measurements, each one command, all run 2026-08-16 against `6b5c8232`:

| # | question | instrument | answer |
|---|---|---|---|
| 1 | Does anything in `src/` **bound** on `HolonRepresentable`? | grep, all of `src/` | **0** |
| 2 | Does anything in `src/` **call** `.to_holon_ast()` / `::from_holon_ast(`? | grep, `src/` minus `comms/mod.rs` | **0** |
| 3 | What do the process-tier types bound on? | `src/comms/process.rs` + `src/kernel/peer.rs` | **28 sites, all `EdnRepresentable`** |
| 4 | What concrete `T` ever crosses a wire in `src/`? | grep, closed set | **`Value` · `String` · `i64`** |
| 5 | Do `Value` / `String` use the holon path? | read the impls | **NO** — `value_to_edn_string_with(self, None)` and `self.clone()` |
| 6 | Do `defservice` / `defsurface` emit a Rust generic wire type? | grep `expand.rs`, `macros.rs` | **0** — wat message types ride inside `Value` |
| 7 | Which sites put a **container** on a real wire? | grep, whole repo | **3**, all inside the arc-216 probes that test the container impls |

`i64` is explained and is not a counterexample: `comms::thread::pair<T: Send + 'static>`
(`src/comms/thread.rs:434`) — the **thread tier requires no `EdnRepresentable` at all**, because
crossbeam passes `T` directly with no serialization. `kernel/peer.rs:647-648` is that path.

**Therefore:** `HolonRepresentable` is a *private implementation strategy* for eight container types'
`to_wire`, wearing a public trait's clothes. `impl HolonRepresentable for String`
(`src/comms/mod.rs:172`) exists for exactly one reason — to satisfy the `T: HolonRepresentable` bound
so `Vec<String>` and friends compile **in the arc-216 test probes**. The capability's entire consumer
set is its own test suite.

`Value`'s doc comment (`src/comms/mod.rs:788`) already states the rule this stone enforces:

> *"`Value` does NOT impl `HolonRepresentable` — it is a plain wat value that serializes as plain
> EDN, not a holographic value with a HolonAST IR."*

## THE FOUR QUESTIONS — the grid the builder ruled from

Four options were enumerated. **Option B (delete) is the only one that clears Obvious + Simple +
Honest.** Recorded so the far side does not re-litigate it.

| option | Obvious | Simple | Honest | UX | |
|---|---|---|---|---|---|
| **A** — rewrite the 8 container impls to plain EDN | YES | **NO** | **NO** | — | 8 hand-written encoders beside the one canonical `value_to_edn_string_with`; preserves a zero-consumer capability, which is what makes it read as load-bearing |
| **B** — delete them; `comms` keeps `String` + `Value` | YES | YES | YES | YES | **4/4 — RULED** |
| **C** — census first, then decide | YES | YES | **NO** | — | the census had already been run; offering it again is a deferral wearing diligence's clothes |
| **D** — keep the trait, relocate it to the VSA module | **NO** | **NO** | **NO** | — | "Representable" is a wire word; and the capability D rescues **already exists as free functions** (`write_holon_ast_tagged` / `read_holon_ast_tagged`) |

A and D fail Honest for the same underlying reason: **they keep a shape whose only proof of life is
the test written to prove it.**

## WHAT IS DELETED — `src/comms/mod.rs`

```
:134   trait HolonRepresentable                          DELETE
:172   impl HolonRepresentable for String                DELETE
:230   impl HolonRepresentable for HashSet<T>            DELETE
:301   impl HolonRepresentable for Vec<T>                DELETE
:426   impl HolonRepresentable for HashMap<K,V>          DELETE
:525   impl HolonRepresentable for (T1,T2)               DELETE
:569   impl HolonRepresentable for (T1,T2,T3)            DELETE
:617   impl HolonRepresentable for (T1,T2,T3,T4)         DELETE
:669   impl HolonRepresentable for (T1,T2,T3,T4,T5)      DELETE

:196   impl EdnRepresentable for HashSet<T>              DELETE  (the delegating shim)
:270   impl EdnRepresentable for Vec<T>                  DELETE
:390   impl EdnRepresentable for HashMap<K,V>            DELETE
:506   impl EdnRepresentable for (T1,T2)                 DELETE
:549   impl EdnRepresentable for (T1,T2,T3)              DELETE
:596   impl EdnRepresentable for (T1,T2,T3,T4)           DELETE
:647   impl EdnRepresentable for (T1,T2,T3,T4,T5)        DELETE
:701   the shared tuple from_holon_ast helper            DELETE (its only callers go)
```

## WHAT SURVIVES, AND WHY EACH

- **`trait EdnRepresentable`** (`:102`) — loses its `Send + 'static` supertrait status change? **No.**
  It is unchanged. It was always the wire trait.
- **`impl EdnRepresentable for String`** (`:148`) — `self.clone()`. Already plain. **Untouched.**
- **`impl EdnRepresentable for Value`** (`:794`) — already plain EDN. **Untouched.** It is the
  exemplar this stone generalizes to "the only shape."
- **`write_holon_ast_tagged` / `read_holon_ast_tagged`** (`src/edn_shim.rs:4265`, `:4274`; public via
  `src/lib.rs:138`) — **KEEP.** They are the `HolonAST ↔ EDN` round-trip, and after this stone their
  callers are purely VSA, which is the builder's rule satisfied rather than violated. Losing them
  would delete a real capability; losing the trait deletes only a wrapper around them.
- **`WireError`** (`:1030`) — keep the type; its doc names `HolonRepresentable::from_holon_ast` as its
  producer and that prose must be corrected, not the type.
- **`coerce_to_holon_ast`** (`src/runtime.rs:18681`) and `edn_to_holon_ast*` (`src/edn_shim.rs:4050`,
  `:4067`) — **untouched.** Different functions entirely; these are the VSA Bind/Bundle path. Do not
  confuse a name containing `holon_ast` with the trait.

## ⛔ THE TEST DISPOSITIONS — read this before touching a probe file

Eight test files reference the trait. **Do NOT delete a whole file.** The arc-216 probe files carry
**both** wat-surface probes and a Rust-cascade probe; the wat probes are the majority and they are
untouched by this stone. Deleting a file destroys live wat-side coverage to remove one Rust test.

### ★ The rule — classify by the BODY, never by the name or the file's own header

Verified 2026-08-16 by reading `probe_arc216_stone1`: its probes 1–9 drive **`.wat` fixtures** via
`call_beside_value(file!(), ":t::p1-forward-rt-len")` and `startup_from_file(…)`. They exercise the
**wat-side VSA surface** — `:wat::holon::to-holon` / `from-holon`, the atomizable check, an EDN
golden. Only probe 10 touches Rust. **The wat probes are the majority of every one of these files and
this stone has no business touching them.**

So the disposition is mechanical, and it reads bodies:

> **REMOVE** a probe iff its body references `HolonRepresentable`, `.to_holon_ast()`,
> `::from_holon_ast(`, or instantiates `pair::<C>()` for a container `C`.
> **KEEP** every probe whose body drives a `.wat` fixture (`call_beside_value` / `startup_from_file`)
> or asserts against an EDN golden.

⚠ **Do NOT trust the files' own `//! The N probes` headers to enumerate the removals.** Measured:
`stone2`'s header names only probe 11 as "HolonRepresentable Rust-side", but **probe 12** also calls
`<Vec<String> as HolonRepresentable>::from_holon_ast`. The header undercounts. Grep the bodies.

| file | measured | what to do |
|---|---|---|
| `…stone1_hashset_roundtrip.rs` | 10 probes | remove probe 10 + the `assert_holon_representable` helper. **KEEP 1–9** (all `.wat`-driven) |
| `…stone2_vector_roundtrip.rs` | 12 probes | remove 11, 12 + helper. KEEP 1–10 |
| `…stone3_hashmap_roundtrip.rs` | 14 probes | remove 12 + helper; **check 13 and 14 by body** — the header's taxonomy is not reliable |
| `…stone7_tuple_roundtrip.rs` | 12 probes | remove 9, 10, 11, 12 + the `tuple_element_i64` + `assert_holon_representable` helpers. KEEP 1–8 |
| `…stone6_process_collection_roundtrip.rs` | 9 probes | **all nine** are `pair::<HashMap/HashSet/Vec>()` over the process tier — the file's whole subject is the deleted capability. The file goes. State that all 9 were checked, one by one |
| `tests/comms/foundation.rs` | `ToyType` | re-point `ToyType` at `EdnRepresentable` directly (plain-EDN `to_wire`). The round-trip + error-honesty properties it proves still matter and must still be proven |
| `tests/comms/process.rs` | `:5` | doc-comment reference only — prose fix |
| `tests/process/probe_arc254_process_ownership.rs` | `:16`, `:17` | doc-comment references only — prose fix |

## THE STALE PROSE — a second finding, fix it in the same motion

`src/comms/process.rs` carries six doc comments describing a wire chain **that is already not what
happens**: `:12`, `:13`, `:47`, `:53`, `:321`, `:710`, `:717`, `:1125` describe
`T::to_holon_ast → write_holon_ast_tagged` and `read_holon_ast_tagged → T::from_holon_ast`. No such
call exists in that file — measurement #2. The prose has been describing a dead path for as long as
`Value`/`String` have been the only wire types. Correct it to name `T::to_wire` / `T::from_wire`.

This is the `curare` rot class — *the map drifted from the territory* — and it is why the trait read
as load-bearing when nothing bore on it.

## THE GATE

| # | assertion |
|---|---|
| 1 | `grep -rn "HolonRepresentable" src/ crates/` → **0** |
| 2 | `grep -rn "to_holon_ast\|from_holon_ast" src/comms/` → **0** (the VSA `coerce_to_holon_ast` in `runtime.rs` is untouched and does NOT count) |
| 3 | `write_holon_ast_tagged` / `read_holon_ast_tagged` still exported from `src/lib.rs` and still compile |
| 4 | no `src/comms/process.rs` doc comment names `to_holon_ast`/`from_holon_ast` |
| 5 | floor GREEN via `scripts/floor.sh` — read the **Summary line** |
| 6 | clippy **0** |
| 7 | the run/skip arithmetic is **accounted for**: removed probes leave the count, they do not vanish silently. State the delta and which probes produced it |
| 8 | no wat-surface probe was deleted — for each touched file, name the probes KEPT |

**Row 8 is the trap.** The cheap way to satisfy rows 1–2 is `rm` on five files. That would pass every
other row and destroy the arc-216 wat coverage this stone has no business touching.

## STOP TRIGGERS

- **STOP-1 — a `HolonRepresentable` bound turns up in `src/` that measurement #1 missed.** Name the
  `file:line`. Do not work around it; the census was the ruling's basis and a miss invalidates it.
- **STOP-2 — a probe file's tests are NOT separable** (the wat probes share scaffolding with the Rust
  cascade). Report the file and the coupling. Do not delete the file, and do not rewrite the wat
  probes to make the removal tidy.
- **STOP-3 — deleting the container impls breaks a compile OUTSIDE `src/comms/` and `tests/`.** That
  would mean a consumer exists that all seven measurements missed. Capture the exact compiler error
  verbatim and report; it inverts the ruling.
- **STOP-4 — a red you did not intend. Do NOT re-run.** `scripts/floor.sh` keeps the untruncated log:
  copy the failing block **verbatim**, name the exact arm, report. There is no such thing as a known
  flake.

## Kin

- `DESIGN-STONE-294-holon-wire-is-plain-edn.md` + `21b7079f` — 294.g, this stone's sibling; same flaw,
  the record's half.
- `DESIGN.md` flaw 3 — *"the `#wat-edn.holon/*` tags are scar tissue."* This stone removes the
  comms-side producer of that tag family.
- `RULING-holonast-and-hologram-are-both-correctly-named.md` — the names stay; the **roles** shed.
- `feedback_no_consumers_does_not_mean_dead` — the memory that made this a ruling rather than my call.
  Zero consumers was **not** the argument; the argument was the four-questions grid, and the builder
  ruled it.
