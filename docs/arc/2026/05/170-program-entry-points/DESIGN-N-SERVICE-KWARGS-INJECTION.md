# Arc 170 — the bracket worker's injected context is KWARGS (the N-service stone)

> **Status: DESIGN (drawn, not yet struck).** The shape is reasoned + ratified with the builder over a long
> design thread; two groundings gate the strike (see *Study the enemy*). Single-service (Stone A) is LANDED
> (`bc472c7c`); this is the N-service generalization.

## The problem

A bracket process-pool worker must reach **N heterogeneous services** it dials (a store, a cache, an echo, …).
They are separate processes (the firm boundary — no shared memory), so a worker cannot *hold* a service; it must
**dial** it over the wire. N services means N **differently-typed** peers (`Peer'<S1,R1>` … `Peer'<Sn,Rn>`) — they
cannot live in a homogeneous collection, and a baked generic runner cannot carry variadic type params.

## Where we are (Stone A, landed `bc472c7c`)

The capability surface `Grantable` → **`Capability`** gained a **`coordinate [self] -> Address'`** hook, so one
`Vector<Capability>` of handles carries grant + revoke + dial. `process/dials [gs][ds]` collapsed to
**`process/uses [handles]`**; map-worker derives each dial address via `(Capability/coordinate g)`. Single-service
is proven end-to-end (`process/uses [eh]` → `["echo:a" "echo:b" "echo:c"]`; M1-teeth still bites). The `lit-check`
stone (`fbc60b94`) made `[eh]` write clean (expected-type-directed vector literals).

## The ratified shape — the worker's injected context is a KWARGS bundle

The work-fn takes the mapped **item** positionally and the injected context as **kwargs** (`& [...]` — the real
wat kwargs syntax, `core.wat:644`, `service.wat:1114`):

```clojure
;; work-fn: item POSITIONAL, the dialed services (and anything else) as KWARGS
(:wat::core::defn :probe::work
  [item <- :wat::core::String
   & [kv   <- :wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>
      echo <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>]]
  -> :wat::core::String
  (… (:probe::Kv/get kv …) … (:probe::Echo/echo echo …) …))     ;; services bound DIRECTLY: kv, echo

;; :user::main — provide the handles BY NAME, matched to the work-fn's kwargs
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [kvh   (:probe::kv'/start   :locus (:wat::spawn::process) :record (:probe::kv'::Record))
     eh    (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     locus (:wat::spawn::process/uses :kv kvh :echo eh)          ;; ← BY NAME, not positional
     out   (:wat::bracket::map locus ["a" "b" "c"] :probe::work)]
    (:wat::kernel::println out)))
```

Three properties, all won at once:
- **Order-free** — kwargs don't care about order; `:echo eh :kv kvh` is identical.
- **Compile-checked** — the bracket matches `uses.kv` → the `kv` kwarg **by name** and type-checks that `kvh`'s
  coordinate fits `Peer'<Kv…>`. A wrong-service handle (`:kv eh`) is a **type error**; a missing/extra name is a
  **name error**.
- **Unambiguous** — names are unique, so duplicate surface types (two Kv services) are fine (`:kv1 … :kv2 …`).

It is the **kwargs mechanism, symmetric on both ends**: the work-fn *declares* the context it needs as kwargs; the
locus *provides* it as kwargs; the compiler reconciles. The kwargs bundle is a `<work-fn>::Kwargs` record —
extensible by construction (`kwargs-lower`, `core.wat:503`).

## Why NOT positional (the soundness gap — the reason name-matching is mandatory, not cosmetic)

`coordinate` returns a **bare** `Address'` (the `Capability` surface erased the service type). The bracket ships
that bare address over the wire; the child runner types it by *its own* ascription (`Address'<Kv…>`). The wire is
untyped bytes. So with a **positional** erased `uses`, writing `[eh kvh]` (echo first) makes echo's address get
received-and-typed as a `Kv` address, connect'd into the `kv` kwarg, and the worker sends `Kv::GetReq` to the
**echo** service → a RUNTIME protocol failure, **silent at compile time**. Positional order over an erased vector
is a hole where a mis-order silently talks to the wrong service. Name-matching (with the handle types preserved to
the reconciliation site) closes it: the wrong service is a compile error. *(The builder: "a compiler forced order
feels objectively better" — it is not just tidier, it is sound.)*

## The crossing discipline — each kwarg's TYPE picks how it crosses (the ocap law as the context API)

"Whatever we add later" is any kwarg — but each kwarg's **type** governs how it reaches the worker (this is the
firm-boundary/ocap doctrine, not a kwargs limit):

| kwarg type | you provide | how it crosses |
|---|---|---|
| `Peer'<S,R>` (a service) | a **Handle** (`:kv kvh`) | grant its pid + dial its `coordinate` → a live **peer** (ocap: the address crosses, becomes a peer) |
| pure data (`i64`, a record, `String`) | the value (`:idx 3`) | **copied** as EDN (no dialing) |
| a live resource | — | **forbidden** (293.W — a resource stays home; the compiler refuses it) |

Note the type asymmetry that *is* the point: for a service you PROVIDE a `Handle` and DECLARE a `Peer'` — the
bracket is the bridge (Handle → grant+dial → Peer'); for data, provide-type == declare-type. `defservice :peers`/
`:init` is the same shape (typed injected context, crossing per field nature).

## The build — what changes (measured, not guessed)

The user-facing UX is the **base name** — `(:wat::bracket::map locus items :probe::work)` — **never `$impl`**. The
`$impl`/companion split is an internal artifact of the kwargs `defn` (`core.wat:834`): `<name>` is the companion
*macro*, `<name>$impl` the callable *fn*, `<name>::Kwargs` the bundle *struct*. The bracket resolves this internally;
the user never writes `$impl`. (Measured: passing the base name `:probe::work` errors `fn-forms: expected fn value,
got keyword` because it's the companion macro; passing `:probe::work$impl` hits **Gap A** below. So the bracket must
*recognize + handle* a kwargs work-fn from the base name — it cannot be shipped as a plain value.)

The pieces:
1. **`process/uses` → named handles.** `(process/uses :name handle …)` carries the handles as a **typed named
   bundle** (each keeps its concrete type so the reconciliation type-checks; grant/revoke up-cast to `Capability`).
2. **Recognize the kwargs work-fn** — via `metadata-of` (`runtime-meta.wat`, which we HAVE): the base name resolves
   `:Kind :Macro` (the companion), with a sibling `<name>$impl` `:Fn` + a `<name>::Kwargs` struct.
3. **Read the `::Kwargs` fields — needs Gap B (struct-field reflection).** The struct's fields (name + `Peer'<S,R>`)
   ARE the dial targets. The data lives in the `TypeEnv`; expose a wat-level "type → fields" primitive.
4. **Reconcile + assemble.** Match each `::Kwargs` field to a `process/uses :name handle` **by name**; type-check the
   handle's service against the field's `Peer'<S,R>` (this closes grounding-1's gap); per kwarg: `Peer'` → grant+dial
   the named handle's coordinate, data → copy. Construct the `::Kwargs` struct in the child.
5. **Ship + invoke — needs Gap A.** The child must run `work$impl [item bundle]`. Reifying the work-fn hits Gap A
   (fn-forms can't ship the kwargs `$impl`). Resolution TBD (Strike A) — fix fn-forms to ship the kwargs `$impl`,
   or ship the work-fn's source + invoke via the companion `(work item :name peer …)` (the existing call-site
   tooling; but Gap-8 shows per-defn source isn't cleanly wat-reachable, so that path also needs a new seam).

## Out of scope / affirmative cuts (`exigere`)

- **The data-kwarg path is not built speculatively.** The shape admits it for free (data crosses as EDN, no
  dialing), but this stone lands the **service-handle** kwargs (the dialing path — the hard part). Build the data
  path when a real consumer needs it.
- **fn-param brace-destructure sugar** (`[item {:keys [kv echo]}]`) — not this stone; `& [...]` kwargs is the shape.
- **Resource kwargs** — forbidden by the firm boundary; not a feature to add.

## The four-questions

| | verdict |
|---|---|
| **Obvious?** | YES — the work-fn signature tells the story (`[item & [kv echo]]` = "process this item, given these services"); provide-by-name mirrors declare-by-name. |
| **Simple?** | ~ — the UX is simple (kwargs both ends); the build is real (AST-walk reads kwargs, per-kwarg assembly). The complexity is inherent to N heterogeneous typed peers, not accidental, and it reuses the existing kwargs mechanism rather than inventing one. |
| **Honest?** | YES — name + type checked; a wrong-service handle is a compile error (closes the erased-positional soundness gap); the crossing discipline is the ocap law, structural. |
| **Good UX?** | YES — `(process/uses :kv kvh :echo eh)` + `[item & [kv echo]]`, order-free, extensible by one more kwarg. |

## Study the enemy — the kill, PROVEN (three groundings, all green)

1. **The gap is real — CONFIRMED** (`scratchpad/probe-gap-wrong-service.wat`). Feeding an **echo** handle to a
   worker typed for **Kv** *compiled* (no type error) and *crashed at runtime* (`recv' failed: peer closed`). The
   erased `Capability` (`coordinate -> bare Address'`, re-typed by the child across the untyped wire) makes a
   wrong-service / mis-order **silent at compile, fatal at runtime**. Name+type matching is mandatory, not cosmetic.
2. **The weapon holds — CONFIRMED** (`scratchpad/probe-kwargs-peer.wat`). A work-fn `[item & [kv <- Peer'<Kv>
   echo <- Peer'<Echo>]]` compiled: the `<name>::Kwargs` bundle is the **struct** R38 made impure-capable
   (`fa42a09f`), so it **holds the dialed `Peer'` services**, bound directly in the body.
3. **The lowered shape is legible — CONFIRMED** (`scratchpad/scout-kwargs-expand.wat`, macroexpand). A kwargs
   work-fn lowers to:
   ```clojure
   (do (defstruct :probe.work/Kwargs [kv <- Peer'<Kv::Op,Kv::Reply>])                       ;; bundle = a STRUCT of Peer' fields
       (def :probe/work$impl (fn [item <- String  kwargs <- :probe.work/Kwargs] -> String   ;; $impl = [item  Kwargs-struct]
         (let [kv (:probe.work/Kwargs/kv kwargs)] …)))
       (defmacro :probe/work [& call-args] … _kl-fvec (quote [kv]) _kl-np 1 …))              ;; companion bakes field-names + n-pos
   ```
   The `$impl` is `[item  Kwargs-struct]`; the `::Kwargs` defstruct carries each field's name + `Peer'<S,R>` type
   (= a dial target); the companion bakes the field-names. **CORRECTION (measured after this grounding):** the
   earlier optimism that the `$impl` is "fn-forms-readable like `[peer item]`" was WRONG — **fn-forms cannot ship
   the kwargs `$impl`** (Gap A, below). And `$impl` must NOT appear in the UX (the base name is the companion macro).

*Explorata caede* — the kill's cost is now mapped: two substrate gaps, one still-dark corner.

## The attack, measured — the two substrate gaps

Every assumption was probed on the disk. The map is complete except one dark corner (Gap A's root).

### Measurements ledger
| # | measured | result |
|---|---|---|
| 1 | mis-ordered / wrong-service handle over erased `Capability` | COMPILES + crashes at runtime (`probe-gap-wrong-service.wat`) — **the gap is real** |
| 2 | kwargs bundle holds `Peer'` (impure) | COMPILES (`probe-kwargs-peer.wat`) — the `::Kwargs` struct is impure-capable (R38 `fa42a09f`) |
| 3 | kwargs `defn` lowered shape | `::Kwargs` defstruct + `$impl [item Kwargs-struct]` + companion macro (`scout-kwargs-expand.wat`) |
| 4 | fn-forms ships a fn whose `let` references its params | YES (`probe-fnforms-let.wat`) — **not** a let/param issue |
| 5 | `bracket/map` accepts a named plain fn value | YES (`probe-named-plain-fn.wat`) — **not** a named-fn issue |
| 6 | fn-forms ships the kwargs `$impl` | **NO** — `free symbol 'kwargs'` (`probe-b1-kwargs-worker.wat`) — **kwargs-`$impl`-specific** |
| 7 | reflection surface | HAVE `metadata-of` (Kind/Purity/Layer/…); **LACK** struct-field ("type → fields") |
| 8 | per-defn retained source reachable from wat | **NO** — source is file-level (`wat/source.wat`), not per-defn |

### Gap A — fn-forms cannot ship the kwargs `$impl` — ROOTED + **FIXED** (Strike A, `eb7a2334`)
`fn-forms :probe::work$impl` fails `free symbol 'kwargs'` where `kwargs` is the `$impl`'s *own* param. Rooted to a
**key mismatch in `closure_extract`**:
- `FunctionDef.params` is stored by **`env_key`** (the scoped key) — `runtime.rs:733`
  (`…map(|(n,_)| crate::scope::env_key(n))…`).
- `walk_free_symbols` matches body symbols by **`as_str`** (the bare base name) — `closure_extract.rs:19`+`:30`
  (`let name = ident.as_str(); if !locals.contains(&name)`).

These agree only for **scope-less** symbols. The kwargs reshaping mints its param via
`(:wat::core::fresh-symbol "kwargs")` (`core.wat:761`) — a **scoped** symbol (env_key `kwargs\u{1}<scope>`). So
`func.params` holds `kwargs\u{1}<scope>`, the walk looks up bare `kwargs`, `contains` fails → the `$impl`'s own param
reads as free. **Confirmed** (`scratchpad/root-gapA.wat`): a hand-written same-shape fn with a **bare** struct-typed
2nd param + a `let` accessor ships fine (`hand: ok`); only the kwargs (`fresh-symbol`) `$impl` fails.

**Fix (small, targeted, NOT structural):** unify the keying — build `body_locals` and match body symbols by the
*same* key so a scoped `fresh-symbol` param reconciles with its (equally-scoped) body reference. closure_extract
already ships every bare-param fn; it simply never handled a hygienic param. Watch the blast radius: the walk's
`name` is reused for dep/type resolution (`closure_extract.rs:45,67`), so the locals key must be unified without
breaking those lookups (compute the locals key separately, or normalize scoped↔bare consistently).

### Gap B — struct-field reflection (the "what reflection don't we have" answer — small)
No wat-level "given a struct/record type, enumerate its fields (names + types)." The data is in the `TypeEnv`
(and `metadata-of` already reaches the registry for *callable* metadata); we simply never exposed the
*type-structure* side. Small + general (any macro wanting a type's shape at runtime); the bracket needs it to read
a work-fn's `::Kwargs` fields.

## The strike (decomposed — substrate first, then wiring)

- **Strike A — Gap A: the closure_extract keying fix — ✅ DONE (`eb7a2334`).** `walk_free_symbols`' Symbol arm now
  keys the locals-membership check by `crate::scope::env_key(ident)` (matching `func.params`), so a hygienic
  (`fresh-symbol`) param self-resolves. fn-forms now ships the kwargs `$impl` (`root-gapA.wat` → `both ok`); a general
  substrate fix (any hygienic-param fn now reifies). Weighed: capture/hygiene 52/52, floor 0-new.
- **Strike B — Gap B: struct-field reflection — ✅ DONE.** `field-names-of`/`field-types-of` expose the `TypeEnv`
  struct fields; absorbed into the reflection type-eviction (`76b25943`/`77e3db60`), now emitting canonical `wat.type/`
  forms (a DECOMPOSABLE list, not a mangled keyword). `(field-types-of :probe::work::Kwargs)` → the field `Peer'<S,R>` types.
- **The fn-forms name→Fn SEAM — ✅ DONE (`c8e3c7ff`).** `fn-forms` resolves a KEYWORD naming a registered fn
  (`sym.get`→`Function`, mirrors arc-009 at runtime.rs:3737); a miss → a LOCATED `TypeMismatch`, not a panic. Lets the
  walk ship a computed `<base>$impl` keyword by name.
- **Strike C — the wat wiring** (on A + B + the seam). `process/uses :name handle`; the spawn-runner AST-walk recognizes
  the kwargs work-fn via **defclause TYPE-dispatch on the work-fn VALUE** (kwargs base → bare `:keyword` via the companion
  MACRO; plain fn → `Value::fn`) — NEVER reflecting the anon value (it CRASHES); ships `<base>$impl` by name via the seam,
  reads `::Kwargs` via B, adapts onto `process-dial-runner`, invokes via the companion.
  - **C1 — single-service via kwargs (N=1) — ✅ DONE (`b0a1a211`).** Clean surface end to end:
    `(process/uses :echo eh)` + `(bracket/map locus ["a" "b" "c"] :probe::work)` → `["echo:a" "echo:b" "echo:c"]`.
    Binds the sole `::Kwargs` field POSITIONALLY (N==1 a real assertion, bracket.wat:260).
  - **C2 — N heterogeneous, name+type matched, a wrong-service handle a COMPILE error — the OPEN stone.** See below.

## Strike C2 — the design (scouted 2026-07-09; the runtime mechanism is composition, the compile-error is the stone)

**The runtime N-mechanism: every dependency is already built.** C2's dial-hold-assemble-invoke is pure composition:
- `PoolMsg<D,I>` (spawn.wat:275) is GENERIC over `D` — C1 uses `D = Address'<S,R>`; C2 uses `D = Tuple<Address'<S1,R1>, …>`,
  a HETEROGENEOUS Tuple of the N addresses (Tuple is heterogeneous by nature + up-casts at construction as of `d8d3e11e`).
- `::Kwargs` is already an N-field record (core.wat:645) — C1 only artificially asserts N==1.
- The runner holds `ctx = Option<::Kwargs>` (the `::Kwargs` IS the N-heterogeneous-`Peer'` bundle), assembled once Setup
  delivers all N addresses (`connect'` each Tuple component into its field). `process/uses :name handle …` already builds
  the named pairs (spawn.wat:196); grant/revoke fold over them.

**The compile-error is the real stone — the ProcessOpts ERASURE boundary.** C1's `process/uses` stores
`Vector<(keyword, Capability)>` (spawn.wat:145/168) — every handle ERASED to `Capability` at the `ProcessOpts` field
(a fixed homogeneous field can't hold an arbitrary-arity heterogeneous bundle; that erasure is why C1 works at all).
Once erased, the concrete service type is gone, so a swapped `:kv eh` up-casts to `Capability` fine → no static error →
the erased `Address'` dials the WRONG service at runtime (exactly `probe-gap-wrong-service.wat`, still un-closed).

The reject needs, at ONE static site, BOTH facts — and they live at two different calls with the erasure between them:
- `kvh : kv'::Handle` (concrete) — visible only at the `process/uses` call (which does NOT know the `::Kwargs`).
- `:echo` wants `Peer'<Echo…>` — known only from the work-fn's `::Kwargs`, at the `bracket/map` call (where `uses` is erased).

So C2's shape is NOT "hold N peers" (built) — it is: **make the `uses`↔`::Kwargs` name+type reconciliation a
compile-time check that lives where BOTH are un-erased** — a property of the `(process/uses …) + work-fn` PAIRING at the
`bracket/map` call site, *before* the `ProcessOpts` erasure. OPEN: whether `bracket/map` becomes a form that sees the
literal `process/uses` syntactically, or the concrete types are otherwise preserved to that check. **PROBE this before briefing.**

### The C2 user form (materialized — the gate)
```clojure
;; work-fn: item + TWO named service kwargs
(:wat::core::defn :probe::work
  [item <- :wat::core::String
   & [echo <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>
      kv   <- :wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>]]
  -> :wat::core::String  (… use both echo and kv …))

;; POSITIVE (proves the mechanism):
(:wat::spawn::process/uses :echo eh :kv kvh)     ;; → runs, both services hit
;; ORDER-FREE (proves name-binding, not position): :kv kvh :echo eh — identical result
;; NEGATIVE (THE proof): :echo kvh :kv eh — SWAPPED handles → MUST be a COMPILE error
;;   (located TypeMismatch: :echo expects Peer'<Echo…>, got kv'::Handle) — NOT a runtime `peer closed`.
```
**C2 is proven iff the swap fails `wat --check` with that located diagnostic.** Everything else is plumbing.

### C2 — the wrong-service compile error, MECHANISM PROVEN (2026-07-09, by measurement)

The erasure-boundary crux above was resolved not by a workaround but by building a general substrate capability. The full
mechanism is now proven end to end (green probes in `scratchpad/`, findings captured here):

1. **Parametric surfaces — BUILT (`7d8e3034` + `b2360c7a`).** A `defsurface :Name<T>` carries type params (bound per satisfier
   at `extend-type`, resolved at the call site — mirrors the `defenum` discipline). `SurfaceDef.type_params` was a dead field;
   now live. The receiver fix (`b2360c7a`) closed an embedded-placeholder gap (`-> Address'<S,R>`, not just whole-position `-> :T`).
   A general capability — surfaces are now generic.
2. **Typed coordinate — PROVEN (`probe-c2-typed-coordinate.wat`).** A parametric `Dialable<S,R>` surface with
   `(coord [self] -> Address'<S,R>)`, satisfied by each `<fqdn>::Handle` routing to its own typed `addr`, resolves
   `(coord eh) : Address'<Echo…>`, `(coord kvh) : Address'<Kv…>`. The flat `Capability`/`coordinate` (bare `Address'`) STAYS
   flat for uniform grant/revoke; the typed dial rides `Dialable`; a handle satisfies **both**. Sound both ways (a satisfier
   claiming `Dialable<Echo>` but returning `Address'<Kv>` is a `ReturnTypeMismatch`).
3. **Co-location — PROVEN (`probe-c2-colocation.wat`).** A `Tuple` of typed coords carries the field-ordered contract, SURVIVES
   a `let`, and a downstream consumer expecting the field-ordered `Address'` contract REJECTS a swapped handle at compile time.
   So **option C** (a typed locus that preserves the ratified `(process/uses …)` + `(bracket/map … :work)` surface) is real —
   no fused form, no `process/uses`-learns-the-work-fn divergence.

**Resume point: the C2-full WIRING** (each step on the proven mechanism — do NOT re-measure the mechanism):
(a) `process/uses` PRESERVES the typed handles (a heterogeneous typed carrier — a `Tuple` of typed coords, or a parametric
`ProcessOpts`) instead of erasing to `Vector<(keyword, Capability)>`; (b) the spawn-runner walk (it already reads `::Kwargs`
field types via `field-types-of`) generates the PARENT-SIDE contract check — a field-ordered `Tuple<Address'<Si,Ri>…>` from the
handles' typed coords, reconciled against the `::Kwargs` (the co-location reconciliation); (c) the child dials the typed
addresses + assembles the `::Kwargs` + invokes via the companion (C1's mechanism). GATE: `(process/uses :echo kvh :kv eh)`
SWAPPED → a located `TypeMismatch` at `wat --check`, not a runtime peer-closed. Every finding captured here; probes in `scratchpad/` (gitignored).
