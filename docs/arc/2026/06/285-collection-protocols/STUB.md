# Arc 285 — collection protocols (`Map` / `Seq`): built-in + persistent collections satisfy ONE interface

> ## ⛔ CORRECTED 2026-08-20 — HALF OF THIS ARC IS ALREADY BUILT, AND THE STUB BELOW SAYS IT IS NOT.
>
> Everything under "STATUS" is the 2026-06-17 snapshot and is preserved as written. Read THIS banner
> first; it is what the disk says today (HEAD `85174fc3f`, floor 4818/4818).
>
> **The crux is ANSWERED — the probe the stub prescribes would come back GREEN.** The stub calls this
> the one unknown: *"can a built-in `Value` type satisfy a wat-defined `defprotocol` whose methods
> route to those Rust intrinsics?"* It can, and does, in the shipped corpus — `wat/seq.wat:75–91`:
>
> ```wat
> (defsurface :wat::core::Seqable<T> :nature :wat::core::Struct
>   :features [(seq [self <- :wat::core::Seqable<T>] -> :wat::stream::Stream<T>)])
> (extend-type :wat::core::Vector            :wat::core::Seqable<T> …)   ; built-in Value variant
> (extend-type :wat::core::PersistentVector  :wat::core::Seqable<T> …)   ; the persistent family
> (extend-type :wat::core::List              :wat::core::Seqable<T> …)   ; built-in Value variant
> (extend-type :wat::stream::Stream          :wat::core::Seqable<T> …)   ; lazy
> ```
>
> **That IS this arc's Layer 2 for the Seq half**, shipped inside arc 278 under the name `Seqable<T>`
> rather than `Seq`. Layer 1 shipped too: 19 `:wat::core::Persistent*` dispatch arms in `runtime.rs`.
>
> **The other two "is it difficult?" questions are answered as well**, and neither is in the stub:
>
> | question | answer on disk |
> |---|---|
> | multi-FEATURE surface (Map wants ~7; `Seqable` has 1) | **3 features** — `TypedCapability`, `wat/capability.wat:65` |
> | TWO type params (`Map<K,V>`, not `Seq<T>`) | **`<S,R>`** — `Dialable`, `capability.wat:44` |
>
> All of it loads and type-checks inside the green floor.
>
> ### What is ACTUALLY left
>
> 1. **The Map half only** — a `:wat::core::Map` surface (`get`/`assoc`/`dissoc`/`keys`/`vals`/
>    `contains?`/`count`) that `HashMap` and `PersistentMap` both `extend-type`. There is no
>    Map-side `defsurface` in `wat/` today (`grep defprotocol wat/*.wat` → only deporder's
>    def-head list). The Seq half is a worked four-impl precedent in the same file.
> 2. **A naming call** — does the Seq half get renamed `Seq` to match this arc's vocabulary, or does
>    `Seqable<T>` keep the name it shipped under? A rename is a corpus codemod; the name is not wrong.
>
> ### ⚠ There is no BLOCKED consumer for the Map half, and that matters
>
> The stub names arc 278 (rete) as the forcing consumer. **278 is parked**, and rete did not wait:
> it typed its Token bindings against the concrete impl (`bindings <- :wat::core::PersistentMap`,
> `wat/rete.wat:30,37`) and works. So the Map half is an **honesty** case — the stub's own
> *"it is dishonest to fracture 'a map is a map'"* — not a demand case. That is legitimate ground
> here (`[[feedback_an_honesty_defect_is_not_gated_on_demand]]`), but it should be chosen knowingly.
>
> Built with no consumer, this arc ships an UNARMED mechanism — R59's dead protocol, a green floor
> certifying something nothing exercises. `wat/rete.wat:668` says exactly this about `total?`:
> *"an unarmed mechanism is R59's dead protocol … the `where` fence is its FIRST REAL CONSUMER, and
> proving it here is what earns the right to lean on it elsewhere ([[300 ALIVS ARGVIT]] — the
> consumer is the crucible)."*
>
> ### ★ THE CRUCIBLE ARRIVED FROM 255 — 2026-08-20, builder-ruled
>
> Home #12 (`:wat::io::`) surfaced a live, measured demand for exactly this mechanism. **Six of the
> 29 io verbs only work on ONE backing** while wearing the shared interface's name — the substrate
> says so in its own error text, *"writer does not support snapshot (only StringIoWriter does)"*
> (`src/io.rs:1404`):
>
> ```
> IOWriter/new · IOReader/from-bytes · IOReader/from-string   always construct a StringIo*
> IOWriter/to-bytes · IOWriter/to-string                      raise on any other backing
> IOReader/rewind                                             raises on Pipe; ruled to raise on stdin
> ```
>
> Making `IOReader`/`IOWriter` SURFACES with a concrete `StringIo` extending them puts
> `(rewind stdin)` beyond representation instead of faulting at runtime — the no-form rung instead of
> the check rung. **That is the same shape as `Seqable<T>` over three backings.** The io split is
> therefore this arc's real forcing consumer, and 285 built for it is armed on arrival.
>
> Builder, 2026-08-20: *"do we pivot and work on 285? … using 285's loot to help us with the 255
> dilemma?"* and *"255 has been a forcing function for us to do this corrective work — that's been
> one of its largest utilities beyond its actual utility of being a source of truth for the lang."*
>
> ⚠ **285 is NOT a technical prerequisite for the io split** — the mechanism is already proven, so
> the io work could be done first. The argument for collections-first is STEPPING-STONE, not
> dependency: *"a map is a map"* has an obvious right answer, while the io split still has open
> design questions (does `StringIo` extend both reader and writer? what does `new`'s `@Category`
> become?). Establish the recipe where the answer is not in doubt.
>
> ### Blocked on this arc, deliberately NOT patched first
>
> - The `StringIo` **rename** — renaming those six would migrate 25 corpus call sites that the type
>   split then moves AGAIN. `[[feedback_do_not_defer_content_on_mechanisms_difficulty]]`
> - The `rewind` **fault change** — builder-ruled that every non-string backing must fault, but if
>   `rewind` only ever takes a `StringIo` the fault branch is never written at all. Do not construct
>   the situation that needs the patch.
> - ⚠ **LIVE COST, stated so it is not implicit:** `RealStdin::rewind` (`src/io.rs:179`) returns
>   `Ok(())` today — silently succeeding while doing nothing, so read-all → rewind → read-all on real
>   stdin yields the content, then `""`, with no error. That lie stays in the tree until the split
>   lands. It has a named closer rather than an unnamed one; that is the whole reason to record it.
>
> ### Two open classification strains this arc would dissolve
>
> `255`'s io homes produced two rows that would not classify, and both are Category-shaped gaps that
> a type split makes moot rather than a taxonomy ruling having to fill:
>
> | row | strain |
> |---|---|
> | `IOReader/rewind` | a handle op that moves no bytes — `:Io` cannot hold it (Purity since ruled Pure) |
> | `IOWriter/new` | a nullary mint with no OS resource — `:Transform` cannot hold it |
>
> ---


> **STATUS: STUB — banked 2026-06-17, future work (maybe tomorrow). Surfaced in arc 278 (rete) as "Layer 2"
> of the persistent-collections decision.** Builder: "layer 2 is a named arc … that's a future us problem."

## Why

Arc 278 (the rete engine) adds an **opt-in persistent collection family** (`rpds`-backed — MIT, safe-Rust;
NOT `im`/`imbl`; chosen by four-questions, arc 278 DESIGN — structural sharing, O(log n) immutable updates)
alongside the existing std `:wat::core::HashMap` / `:wat::core::Vector`
(`Arc<std>`, clone-on-write). Two layers of "a map is a map":

- **Layer 1 (ships IN arc 278, stone 0):** shared OP NAMES — `assoc`/`get`/`keys`/`vals`/etc. dispatch
  polymorphically on container type (already how wat collections work), so they work on std AND persistent
  maps with no caller change. Mandatory and nearly free.
- **Layer 2 (THIS arc):** a FORMAL `:wat::core::Map` / `:wat::core::Seq` **defprotocol** that BOTH families
  (and any future collection) satisfy — so generic code can be typed `[m <- :wat::core::Map]` and accept
  *either* impl: code against the interface, swap the implementation. Builder's stance: it is **dishonest to
  fracture "a map is a map"** — so this is mandatory *in spirit*, **unless it proves very difficult to
  build**, which must be GROUNDED, not guessed.

## The crux to ground (the "is it difficult?" question)

wat's built-in collections are **Rust intrinsics dispatched on the `Value` enum** (`collection/eval.rs`),
NOT protocol-based. The unknown: can a built-in `Value` type (`wat__std__HashMap`, the new persistent type)
**satisfy a wat-defined `defprotocol`** whose methods route to those Rust intrinsics? `extend-type` registers
a subtype edge on the receiver's `class_fqdn` (e.g. `"wat::core::HashMap"`); but a `defprotocol` method
normally needs a per-type impl (a wat method body), and here the "impl" is a Rust intrinsic.

**Probe (write first):** `(defprotocol :wat::core::Map ...)` + `(extend-type :wat::core::HashMap :wat::core::Map)`
+ a `(defn f [m <- :wat::core::Map] ...)` that calls `assoc`/`get` — does it type-check and dispatch for both
a std HashMap and a persistent map? RED at HEAD names exactly the gap.

- **Probe clean** → Layer 2 is mandatory; build it (no excuse to skip — fracturing the abstraction is dishonest).
- **Probe shows a real gap** (built-in-type-satisfies-wat-protocol needs new substrate) → that gap is its own
  sub-stone, named — not hand-waved.

## Mechanism that already exists

- `defprotocol` / `extend-type` (arc 232, shipped).
- Protocol-typed fn params accepting any extending type — host-parity used `[host <- :wat::kernel::Host]`
  accepting ThreadOpts/ProcessOpts via `extend-type` (arc 209/232/267).
- `derive`/typesub edges (arc 237/267) for the type hierarchy.

## Scope (when opened)

- `:wat::core::Map` protocol: `get`/`assoc`/`dissoc`/`keys`/`vals`/`contains?`/`count`.
- `:wat::core::Seq` protocol: `conj`/`first`/`rest`/`count` (Vector + List + persistent vector).
- Both std and persistent families `extend-type` them; the checker accepts protocol-typed collection params.
- Out of scope until needed: a full Clojure-style seq abstraction over lazy sequences.

## Pairs

- arc 278 (rete) — the forcing consumer (Layer 1 ships there; Layer 2 is this).
- arc 232 (defprotocol/extend-type) — the mechanism.
- arc 257 (edn-native-collections) — adjacent collection work; check for overlap when opened.
