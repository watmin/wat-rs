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

### Gap A — fn-forms cannot ship the kwargs `$impl` (THE DARK CORNER)
`fn-forms :probe::work$impl` fails with `free symbol 'kwargs' does not resolve` (`closure_extract`), where `kwargs`
is the `$impl`'s own 2nd param — it should be bound (`closure_extract.rs:209` collects `func.params`). Measured
**kwargs-specific** (rows 4–6: inline `let` ✓, named plain fn ✓, only the kwargs `$impl` fails ✗). **ROOT NOT YET
FOUND** — needs a closure_extract diagnosis (why the `$impl`'s param reads as free) to know small-fix vs structural.
This is the one thing to root before the wat wiring. Alternative if structural: ship the work-fn's *source* + invoke
via the companion — but Gap 8 shows per-defn source isn't cleanly wat-reachable, so that path *also* needs a seam.

### Gap B — struct-field reflection (the "what reflection don't we have" answer — small)
No wat-level "given a struct/record type, enumerate its fields (names + types)." The data is in the `TypeEnv`
(and `metadata-of` already reaches the registry for *callable* metadata); we simply never exposed the
*type-structure* side. Small + general (any macro wanting a type's shape at runtime); the bracket needs it to read
a work-fn's `::Kwargs` fields.

## The strike (decomposed — substrate first, then wiring)

- **Strike A — root + resolve Gap A** (fn-forms shipping the kwargs `$impl`). Diagnose the `kwargs` free-symbol in
  `closure_extract`; fix it (or, if structural, add a source-ship + companion-call seam). **Gate**: `fn-forms` a
  kwargs `$impl` round-trips. **THE PREREQUISITE** — invoking the work-fn in the child is blocked on it.
- **Strike B — Gap B** (struct-field reflection). Expose `TypeEnv` struct fields to wat, e.g. `(fields-of :T)` →
  `[(name, type) …]`. **Gate**: `(fields-of :probe::work::Kwargs)` → `[(kv, Peer'<Kv…>)]`.
- **Strike C — the wat wiring** (on A + B). `process/uses :name handle`; the spawn-runner AST-walk recognizes the
  kwargs `$impl`, reads `::Kwargs` via B, name-matches `uses`, per-kwarg grant+dial+assemble, invokes via A.
  Decompose by N: **C1** single-service via kwargs (N=1) → `["echo:a" "echo:b" "echo:c"]`; **C2** N heterogeneous +
  name+type-matched → a 2-service worker, a wrong-service handle a compile error.

Order: Strike A first (root the dark corner) — until fn-forms can ship the kwargs `$impl` (or a companion-call seam
exists), the rest is blocked. Then B (cheap), then C1 → C2.
