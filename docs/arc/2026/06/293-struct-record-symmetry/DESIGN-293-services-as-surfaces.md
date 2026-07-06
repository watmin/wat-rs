# DESIGN — 293: services-as-surfaces (a `defservice` satisfies a surface — the AWS service model, decomplected)

> **Status: CONTRACT SETTLED (2026-07-05 — the operation model: `RequestRecord → ResponseRecord`, errors as `Reply`
> variants, both width-evolvable + checker-walled); the CODEGEN re-pointing + intueri names + a feasibility probe are
> what remain to draw (see § The mechanism + § Open).** This is 293's
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

## The mechanism — the OPERATION model (SETTLED 2026-07-05, the T1b co-design)

The whole contract, symmetric and grounded — reasoned to over the T1b design conversation.

**Every operation is `RequestRecord → ResponseRecord`. Both are NAMED records — structured units, not loose args.**
The surface declares them all:

```clojure
(:wat::core::defsurface :wat::query::Store :holder :wat::core::Struct
  :features [(put  [self PutRequest]  -> PutResponse)      ;; PutRequest [rows <- Vector<StoredRow>]  — a unit
             (scan [self ScanRequest] -> ScanResponse) …]) ;; ScanRequest already a record; ScanResponse now too
```

- This is the **AWS operation**, exactly: an Input shape → an Output shape, both records, nothing loose.
- It fixes a real inconsistency the current `Store` surface *already* shows — the smell that proves the point: `scan`
  takes a `ScanRequest` **record**, but `put` takes a bare `Vector<StoredRow>` **collection**. Same surface, two shapes
  for "the input" — the loose-args model leaking. Uniform: `put` takes `PutRequest`.

**Errors are `Reply` variants.** If everything on the wire is a named shape, an error is *also* a named shape. The
`Op` enum carries the request shapes; the `Reply` enum carries **success (the response record) + one variant per
modeled error** (each an error record). AWS models errors as first-class shapes; `Store` already models `Error` as a
recovery-axis enum — this is that, on the wire. (Supersedes the doc's earlier `-> Result<T,Error>` sketch: the Result
becomes the `Reply` enum's success-vs-error split.)

**Both records evolve by WIDTH and are CHECKER-WALLED — on BOTH ends:**
- *Evolvable:* add an optional field to `PutRequest` (a `:consistency` mode) or to `ScanResponse` (a `:scanned-count`)
  and every existing client keeps compiling — a *wider* record still satisfies the surface (293's structural
  width-subtyping). A loose arg list can grow *neither* direction without a breaking change; a record grows *both*.
  **This is the capability the AWS model existed for — evolve a service without versioned breakage — delivered by the
  subtyping 293 already built.**
- *Walled:* the client constructs a `PutRequest` (named — cannot misshape it); the server returns a `PutResponse`
  (named — checked against the surface, exactly the `extend-type` honesty strike `fa8bbcb9` / R29). The wrong shape is
  **uncompilable on both ends of the wire**, not just the response. Safety by wall, not author-discipline.

**The two selves.** A surface method `(put [self PutRequest] -> PutResponse)` — `self` is the *client's* view (the store
you call `Store/put` **on**, the dial target). Inside the service, that same position is the *server's* `s <- :State`.
One method, two ends: the client dials `self`, the server binds `s`. The macro owns the mapping. (This is the
client/server duality of a service wearing a surface.)

**The clause shape.** `:satisfies <one surface>` replaces `:ops` with `:impls` — **bodies only**; the sigs ARE the
surface, and re-listing them is the duplication `derive-is-the-wall` forbids (like a body-only `extend-type`). Each
impl constructs the named response record; the wire (`Op`/`Reply`, the request/response *wrapping*) is DERIVED from the
surface, invisible to the author:

```clojure
(:wat::service::defservice :wat::query::mem-store'  :satisfies :wat::query::Store
  :durable [rows <- …] :ephemeral [] :init …
  :impls   ;; (<surface-method> [s req] body). s = State (the server's self); req = the named request record.
  [(put  [s req] (:let [merged …] (:wat::service::Outcome::Reply (State merged) (:wat::query::PutResponse …))))
   (scan [s req] (:let [page …]   (:wat::service::Outcome::Reply s          (:wat::query::ScanResponse …))))])
```

**One consistent story.** Everything on the wire is a named shape. A service is a set of operations; each operation is
`RequestRecord → ResponseRecord` (errors are `Reply` variants); the surface declares every shape; the server
implements them (`:impls`) and the client is generated from them (`:calls [surface]`); both records evolve by width and
are enforced by the checker. The AWS service model, with the two things AWS never had — **structural evolvability
instead of versioned breakage, and the type system instead of codegen.**

## Open — what's left to scout + draw (the CONTRACT above is settled)

1. **Re-point the codegen.** `defservice`'s existing request/response/variant/`Op`/`Reply` codegen (grounded:
   `wat/service.wat` §C.3, `client-forms-def`) re-sourced FROM the surface's methods, emitted ONCE under the surface
   namespace (`Store::Op`, `Store::PutRequest`, …) instead of per-service `{fqdn}::Op`. Lifecycle
   (`Admin`/`Status`/`Handle`/`serve`/`start`) stays per-service.
2. **`:satisfies`/`:impls` validation** — the `:impls` must cover exactly the surface's methods, and each impl's
   returned response record is checked against the surface's declared response type (mirror the extend-type body-check,
   the honesty strike).
3. **`:calls [surface]`** — extend `:calls` (today: concrete service keywords) to accept a surface + ship the
   surface-sourced client-forms; the consumer dials `Address'<Store::Op, Store::Reply>` (uniform, given as a pure
   operating-input, dialed in `:init`).
4. **The `Reply`-as-error-union shape** — how modeled errors become `Reply` variants (success + per-error records) and
   how the client method surfaces them to the caller.
5. **Naming (intueri)** — `:satisfies` / `:impls` clause names; where the shared protocol types live (the surface
   namespace); the Request/Response record naming convention.
6. **A disconfirming probe** — can `defservice`'s request/response codegen be re-pointed at a surface's methods and
   emit ONE shared protocol two services reference? (S1's feasibility, before a shadowdancer.)

## S1 — DRAWN (2026-07-05: intueri weighed, feasibility grounded, A/derived-purity decided, reference verified)

**Names (intueri-cast + weighed):** `:satisfies` (the defservice clause; `:serves` rejected — collides with the
`{fqdn}::serve` loop fn) · `:impls` (bodies-only, over keep-`:ops` — the grammar genuinely changes) · `<Op>Request` /
`<Op>Response` (the existing wat/gRPC convention; not AWS `Input`/`Output`) · protocol under the **surface** namespace.

**Gating decision (four-questions; the builder corrected my marker-framing):** NOT a declared marker (that's the footgun —
build threaded, forget the flag, can't network in prod). **A service is loci-agnostic by nature** — thread-hosting is a
novelty of location — so it is *always* dialable. The gate is **DERIVED PURITY**, self-applying: a surface emits its
protocol **iff its method sigs are pure (EDN-crossable)**. Impure sigs (a method taking/returning a live `Peer'`/
`Connection`) can't cross → 293.W already rejects → not a service, honestly. No new branch; the wire wall applied once more.

**Locus (grounded):** `defsurface` is **Rust-handled** (`src/types/surface.rs` `parse_defsurface`, routed
`src/types.rs:1941`) — it registers a `TypeDef::Surface`, it is NOT a wat macro. So S1 is a **Rust strike**: in the
surface-registration path, when the method sigs are pure, **synthesize + register `Surface::Op` and `Surface::Reply`
`EnumDef`s** from the surface's `SurfaceMember::Method` list (each method → one variant per enum). The request/response
records are **user-declared** (the method members reference them); S1 emits only the two enums.

**The map (per method):** `(put [self <- :S  req <- :S::PutRequest] -> :S::PutResponse)` ⟶
`:S::Op::Put [req <- :S::PutRequest]` + `:S::Reply::Put [resp <- :S::PutResponse]` (PascalCase variant per method).
Error variants in `Reply` are a downstream concern (the `Reply`-as-error-union item below).

**Reference target (VERIFIED — `cargo wat` prints "S1 reference target type-checks"; the codegen is graded against it):**

```clojure
;; user-declared request/response records + surface (methods reference the records; pure sigs → serviceable):
(:wat::core::defrecord :probe::Kv::PutRequest  [key <- :wat::core::String  val <- :wat::core::String])
(:wat::core::defrecord :probe::Kv::PutResponse [ok  <- :wat::core::bool])
(:wat::core::defrecord :probe::Kv::GetRequest  [key <- :wat::core::String])
(:wat::core::defrecord :probe::Kv::GetResponse [val <- (:wat::core::Option :wat::core::String)])
(:wat::core::defsurface :probe::Kv :holder :wat::core::Struct
  :features [(put [self <- :probe::Kv  req <- :probe::Kv::PutRequest] -> :probe::Kv::PutResponse)
             (get [self <- :probe::Kv  req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse)])
;; S1 SYNTHESIZES (from the method members) — the target shape:
(:wat::core::defenum :probe::Kv::Op :wat::enum::Pure
  :Put [req <- :probe::Kv::PutRequest]  :Get [req <- :probe::Kv::GetRequest])
(:wat::core::defenum :probe::Kv::Reply :wat::enum::Pure
  :Put [resp <- :probe::Kv::PutResponse]  :Get [resp <- :probe::Kv::GetResponse])
```
(full file: `scratchpad/s1-reference-target.wat`.)

**S1 gate:** a pure-sig surface's synthesized `::Op`/`::Reply` exist + are constructible/matchable (the reference target,
now produced by the codegen rather than hand-written); an impure-sig surface synthesizes nothing (no regression); whole
floor 0-new-failures.

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

## S3 — DRAWN (2026-07-06): the client side, the Nature model, and "a service is a surface at a coordinate"

**The crystallization (278 R32 `QVANTVMVIS PROCVL, IDEM NEXVS`, the builder's synthesis):**
**every service is a *surface* whose *nature* is `:Peer`, living at a *coordinate*.** From the caller's side there is
nothing else — two things: the **surface** (*what* to say — build the request, handle the reply) and the **coordinate**
(its `Address'` — *where* to dial). The Locus (transport) is orthogonal + unbounded. Because a service is *nothing but*
`(surface, coordinate)`, the relationship is **distance-invariant** — dial → peer → speak the surface, identical
in-thread (`ThreadSelfPeer'`) or across the world (`Process'`/mTLS). Distance became a coordinate VALUE, not a wall.

### The client-side names (intueri-cast + weighed + ratified)
- **peer-as-satisfier — NO NAME.** A dialed `Peer'<S::Op,S::Reply>` **directly satisfies** the surface `:S` (a generated
  `extend-type` whose bodies `send'`/`recv'` the Op/Reply). A wrapper type re-complects R31 — `Client`/`Remote`/`Proxy`/
  `Stub` all rejected (role-name / locus-lie / codegen-jargon).
- **client entry — `:S/<op>`, UNIFORM.** The surface method itself, dispatched through whatever satisfies `:S` (a local
  struct OR a dialed peer). The wire is invisible in the name; RPC-vs-local is the decomplection, not a distinction.
  (`:S/<op>!` lies — a bang asserts the wire; `:S::call/<op>` mumbles the mechanism.)
- **dial — `connect'`.** No surface-specific dial; the `Address'` is already protocol-typed to `:S`.
- **return — `Result<Response, OpError>`** at the face, **uniform across every satisfier** (the surface method's return
  becomes this too — else local returns bare `Response`, dialed returns `Result`, and the wire LEAKS through the return
  type). `Reply` stays the **pure per-op demux** (which op's reply is this); the **outcome** (success vs error) is the
  `Result`. **This supersedes the earlier "errors are `Reply` variants" sketch** (§ The mechanism) — one axis per enum.

### The Nature model (`Holder` → `Nature`; ratified)
A `Peer'` **cannot satisfy a holder-bound surface**: the holder (`:Struct`/`:Record`/`:HolonRecord`) is an **aggregate**
floor, and a peer is no aggregate (the foreign-satisfaction path requires `holder.is_none()`, but `:holder` is mandatory
— arc 293 K0a). Grounding *why* (`AGGREGATE-AUDIT.md` — the holder is the **capability trit**; `DESIGN-293.W` — mandatory
because **a default masks intent**) flipped the naive "just relax it" lean: the capability must be **declared, not
defaulted**. So intueri was cast: **`Holder` LIES** — a peer *holds nothing* (a category pun, the same 293.W cut for
enums) → **`Nature`**, the satisfier's intrinsic character, from which the boundary trit is *derived*. ONE axis, four
mutually-exclusive variants (a satisfier is a backed-value **or** a peer, never both):

```clojure
(:wat::core::defenum :wat::types::Nature :wat::enum::Portable
  :Struct         ;; stays home — may hold live resources, cannot cross
  :Record         ;; travels by copy — pure data, crosses as EDN
  :HolonRecord    ;; travels with VSA — pure data + a hologram
  :Peer)          ;; IS the door everything else travels through — a live channel endpoint,
                  ;;   reached only by its coordinate (Address'); never moves. (:Remote LIES — may be same-process.)
(:wat::core::defsurface :wat::query::Store :nature :Peer   ;; declared, never defaulted (293.W)
  :features [(put [self <- :Store  req <- :Store::PutRequest] -> :Store::PutResponse) …])
```
A `:nature :Peer` surface is **peer-satisfied only** — local = an in-thread peer; there is no coexisting bare-struct
satisfier (R31's one act). `:Value`/"Any" was explicitly REJECTED (a non-assertion; the builder: *"i'm very hesitant to
accept an Any"*) in favor of the positive nature `:Peer`.

### The decomposition (updated)
- **S3a — DONE (`93e936b3`).** A parametric `extend-type` self decomposes to `Parametric` (not a flat `Path`), so
  `send'`/`recv'` accept a `Peer'` extend-type self (`runtime.rs:709` → `parse_type_expr`). Closed `project_peer_io`;
  0-new-failures. (Exposed Gap B below.)
- **S3-Nature — the substrate stone (NEXT).** Rename `Holder` → `Nature` (the `Holder` enum + `:holder`→`:nature` clause
  + the ~99 `Holder::…` branches + `AGGREGATE-AUDIT.md`); add the `:Peer` nature; **teach the checker that a
  `Peer'<S::Op,S::Reply>` satisfies a `:nature :Peer` surface** — which *is* the fix for S3's **Gap B** (the call-site
  `:S/<op>` receiver check rejecting the `Peer'` today).
- **S3b — the feature (assembly after S3-Nature).** `:calls [surface]` (**surface-only** — ratified; zero current
  consumers, born surface-only, no retirement sweep; the client mirror of the server-side `:ops`→`:impls` endpoint).
  Generate the peer-as-satisfier `extend-type` + the surface-sourced `:S/<op>` client-forms returning `Result`; the
  caller dials `Address'<S::Op,S::Reply>` **blind**. → 278/T1b (the blind sink) is then assembly.
