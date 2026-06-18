# Arc 285 — collection protocols (`Map` / `Seq`): built-in + persistent collections satisfy ONE interface

> **STATUS: STUB — banked 2026-06-17, future work (maybe tomorrow). Surfaced in arc 278 (rete) as "Layer 2"
> of the persistent-collections decision.** Builder: "layer 2 is a named arc … that's a future us problem."

## Why

Arc 278 (the rete engine) adds an **opt-in persistent collection family** (`im::*`-backed, structural
sharing, O(log n) immutable updates) alongside the existing std `:wat::core::HashMap` / `:wat::core::Vector`
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
