# DESIGN — 293: services-as-surfaces (a `defservice` satisfies a surface — the AWS service model, decomplected)

> **Status: SHAPE AGREED (2026-07-05, the T1b circuit co-design); MECHANISM to scout + draw.** This is 293's
> **unfinished third face.** 293 fused struct+record into the aggregate and made `defsurface` a set-of-accessors
> satisfied by **attrs** (a struct's fields) and **methods** (an `extend-type`). The one citizen that never got folded
> in is the **service** — it predates surfaces (arc 209/291), still mints its own private `::Op`/`::Reply`, and
> satisfies nothing. This delivers the third way a surface is satisfied: **by a service, at the wire.** 293 was already
> open (closure-gated on the aggregate audit; closes with 294; 291 blocked on it) — this is a stone within it, not a
> new arc. **278/T1b (the blind telemetry sink) is blocked on this** and resumes by inheritance once it lands.

## How we got here (the arc-170 unfolding — "started with 'can we add argv to main'")

A small honest question, pulled on until it became the real thing. The thread, on the record:
1. **T1b:** how does the telemetry sink write to a store *without naming mem vs sqlite*?
2. **The wall (293.W, checker-taught):** you cannot hand a service a live satisfier — a `Store` struct is impure; a
   start operating-input lands in the pure `resume::Kwargs`. Only **addresses** cross (a peer is crossbeam tx/rx or a
   unix pipe pair — process-local; a process must *dial* its peer). 293.W was right; I handed it the wrong thing.
3. **The reframe:** the sink depends on the `Store` **surface**, not on a store. An enum is closed (a user brings their
   own store); a surface is open. The sink asks *"is this a store?"*, never *"which."*
4. **Remembering what we knew:** loci are already **unbounded** — `Locus` (a legacy `defprotocol`, doctrinally a
   surface) is open, `start [locus <- :Locus]` and `connect'` are locus-agnostic, *"a new transport joins as one
   `extend-type`, zero edit."* Transport-agnosticism is **inherited**, not built.
5. **The gap, grounded:** `defservice` mints `::Op`/`::Reply` **per-service** (`{fqdn}::Op`), and `:calls` names
   *concrete* services. So `mem-store'::Op ≠ sqlite-store'::Op` — the sink can't dial both by one address. *Not* assembly
   this time: a real weld is missing.
6. **The recognition (builder):** this is **AWS API-as-JSON** — one interface spec a service *implements*, and clients
   *generated from the same spec* to build requests + handle replies.

## The thesis

**A `defservice` can `:satisfies` a surface. The surface SOURCES the service's wire-protocol** — the shared
`Op`/`Reply` enums and per-op request/response records come *from* the surface's methods, so **every service that
satisfies the same surface speaks the identical protocol.** The service supplies only its state
(`:durable`/`:ephemeral`/`:init`) and the op **bodies**. From one surface fall out **two sides**: the **server** (the
service's wire-protocol + bodies) and the **client** (request constructors, reply accessors, the dial-and-RPC methods).
A consumer `:calls [<surface>]` (the *surface*, not a concrete service) and dials `Address'<Surface::Op,
Surface::Reply>` — a *uniform* type across every backend — so it talks to any satisfier **blind**.

## The design (the shape agreed)

```clojure
;; the surface is the ONE contract / interface spec (exists today):
(:wat::core::defsurface :wat::query::Store :holder :wat::core::Struct
  :features [(put  [self rows] -> :wat::core::Result<..nil,..Error>)
             (scan [self q]    -> :wat::core::Result<..Page,..Error>) …])

;; SERVER side — a service SATISFIES the surface (generative): its wire-protocol is SOURCED from Store's methods.
;; The shared types are :wat::query::Store::Op / ::Reply + per-op request/response records — ONE set, not per-service.
(:wat::service::defservice :wat::query::mem-store'  :satisfies :wat::query::Store
  :durable [rows <- …] :ephemeral [] :init …
  :impls [(put  [s rows] …pv body…)  (scan [s q] …pv body…) …])
(:wat::service::defservice :wat::query::sqlite-store'  :satisfies :wat::query::Store
  :durable […] :ephemeral [conn …] :init …
  :impls [(put  [s rows] …sql body…) (scan [s q] …sql body…) …])
;;   ↑ BOTH speak :wat::query::Store::Op — one protocol, two backends.

;; CLIENT side — a consumer :calls the SURFACE (not a concrete service) + dials any Store address, blind:
(:wat::service::defservice :wat::telemetry'::TelemetryService'  :calls [:wat::query::Store]
  :ephemeral [store <- (peer dialed from a given Address'<Store::Op, Store::Reply>)]  …)
```

### The load-bearing rulings (settled tonight)

1. **`:satisfies` is GENERATIVE, not a redundant label.** The builder's earlier 293 concern (*"explicit `:satisfies`
   feels wrong… ambient 'you do or you don't'"*) was aimed at **structs** — a struct satisfies structurally (it *has*
   the shape), so declaring is noise. A **service** is the opposite: `:satisfies` *sources* the protocol from the
   surface. It does work. It is honest here.
2. **Exactly ONE surface per service.** A struct's surface is **read** (structural match → many, passively). A
   service's surface is **generated-from** (the protocol *is* the surface → one source). Two surfaces = two `Op` enums
   unioned into one wire-face = two services fused = decomplect them (293's own thesis). Exactly-one is what makes the
   address type `Address'<Store::Op, Store::Reply>` **unambiguous** — and that clean type is the *entire reason* the
   client can dial blind. Open in the dimension that matters (any number of services satisfy `Store`), closed in the
   one that would rot (one surface per service).
3. **The honest asymmetry, named:** structs satisfy *many* surfaces; a service satisfies *one*. Not a spurious split —
   a genuine difference in *kind* of satisfaction: **data is matched (many); a protocol is generated (one).** Same
   surface, two relationships — read-from vs sourced-from.
4. **Decomplected from transport.** The **surface** is the API (the ops); the **`Locus`** is the wire (thread / process
   / UDS / mTLS / a wat-native L4 — unbounded, one `extend-type` each). *Orthogonal.* The same generated client dials a
   store over shared memory OR a socket OR remote mTLS with zero client edits. (AWS welds API to HTTP; wat does not.)
5. **Homoiconic + typed.** The surface *is* a wat form in the same language (not a separate JSON/Smithy IDL); the
   checker enforces **both** ends — the service must implement the surface (or it won't compile) and the client must
   use it correctly (or it won't compile). No external codegen, no spec↔impl drift, no "SDK a version behind."

## The recognition — the AWS service model, derived and decomplected

This *is* the AWS service model, which the builder ran for years (Kinesis KCL, API Gateway, Shield): one model
(`service-2.json` / Coral / Smithy) → the framework generates **the server skeleton AND every client SDK** from the
one spec (build request → serialize → send → parse response). `RATIONE NON MIRACVLO` (R19) / R15 — not invented here,
**derived to where the greats stand**, because it's the correct shape and he'd already found it once. What makes it
*ours*, not a copy — the three decomplections above: **API split from transport** (locus-agnostic), **spec brought
into the language** (homoiconic), **codegen replaced by the type system** (enforced). `300 R7 VIRTVTE PARES` — a
dialect that fields what they field (one-model-two-ends) *and* what they can't (transport-blind, drift-free).

**The scope this reveals:** `defservice :satisfies surface` is not a telemetry convenience — it is **wat growing its
own AWS-grade service framework**, which fell out of "how does the sink dial a store." That is the method (arc 170
began "can we add argv to main"); the size is the point, not an accident.

## Open — the MECHANISM to scout + draw (the shape is agreed; the how is not yet)

1. **Surface → wire-protocol generation.** How a surface's methods map to the shared `Op`/`Reply` enums + per-op
   request/response records. (Ground: `:calls`'s existing `client-forms-def` already emits request/response records +
   `Op`/`Reply` enums + op-methods per-service; the weld re-sources them from the surface, ONE set.)
2. **The `:satisfies` clause** on `defservice` (service.wat macro): validate the `:impls` cover exactly the surface's
   methods; emit the shared protocol types under the *surface's* namespace, not the service's.
3. **`:calls [surface]`** — extend `:calls` to accept a surface (today: concrete service keywords) and ship the
   surface-sourced client-forms.
4. **The address type** `Address'<Surface::Op, Surface::Reply>` as the uniform dial type; how a consumer is *given* one
   (a pure operating-input) and dials in `:init`.
5. **The checker weld** — a service `:satisfies`-ing a surface is checked like an `extend-type` (the `:impls` must
   match the surface's method signatures); the client use is checked against the same surface.
6. **Naming (intueri)** — the `:satisfies`/`:impls` clause names; where the shared protocol types live.

## Blast radius (for the eventual strike)

`wat/service.wat` (the `defservice` macro — the `:satisfies` clause + surface-sourced protocol emission) · the surface
machinery in `src/check.rs` (service-satisfies-surface checking, mirroring extend-type) · `:calls` accepting a surface
· the store services (`mem-store'`/`sqlite-store'` gain `:satisfies :wat::query::Store`, their per-service `::Op` retired
for the shared `Store::Op`) · then **278/T1b** (the sink `:calls [:wat::query::Store]`, dials blind — now assembly).

## Four-questions verdict (the shape)

**Obvious** YES (a service serves one protocol; the surface is its spec). **Simple** YES (one service = one surface =
one protocol, un-braided; the asymmetry with structs is named, not hidden). **Honest** YES (`:satisfies` is generative
here, not the redundant label it would be on a struct; the spec/server/client can't drift because they're one typed
object). **Good UX** YES (the client dials `Address'<Store::Op>` blind; a new backend is one `defservice :satisfies`,
a new transport is one `Locus` `extend-type`).
