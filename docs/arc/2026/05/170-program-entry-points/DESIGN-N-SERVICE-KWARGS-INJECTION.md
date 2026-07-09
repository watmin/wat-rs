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

## The build (what changes)

1. **`process/uses` → kwargs.** `(process/uses :name handle …)` carries the handles as a **typed named bundle**
   (each handle keeps its concrete type so the reconciliation can type-check it; grant/revoke still work by
   up-casting each to `Capability`). This replaces the erased `Vector<Capability>` field/arg from Stone A.
2. **The spawn-runner AST-walk generalizes.** It already derives the single peer type off the work-fn's param; it
   now reads the work-fn's **`& [...]` kwargs** (the `<work-fn>::Kwargs` record fields) → the kwarg names + types.
3. **The bracket reconciles + assembles per kwarg.** At `bracket/map` (which sees both the locus's named handles
   and the work-fn's kwargs): match by name; type-check each provided handle against its kwarg's `Peer'<S,R>`;
   generate the per-kwarg bridge — a `Peer'` kwarg grants+dials the named handle's coordinate, a data kwarg crosses
   the value. Build the `<work-fn>::Kwargs` bundle in the child; invoke `work$impl [item kwargs-bundle]`.
4. **The runner** stays close to baked — it holds the assembled kwargs bundle (one value) and applies
   `(work$impl item bundle)`; the per-N work is confined to the assembly (the connect'-each step), not the loop.

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

## Study the enemy — the groundings that gate the strike (slow is smooth)

Before briefing the macro work, prove the two load-bearing assumptions with disconfirming probes:

1. **The gap is real.** A mis-ordered erased positional `uses` dials the WRONG service at runtime (echo's address
   received as `Kv` → `Kv::GetReq` to echo → failure). Prove it — this is *why* name-matching is mandatory. If it
   does NOT fail (the child validates somehow), the soundness argument is wrong; re-scope.
2. **The fix is reachable.** Can `process/uses` carry **typed named handles** reconciled against the work-fn's
   kwargs at `bracket/map` — matched by name, type-checked per handle? Scout the kwargs lowering
   (`defn` → `<name>::Kwargs` record → `$impl [pos… bundle]`, `core.wat:503-644`) and whether the AST-walk can read
   a work-fn's `& [...]` kwargs to derive the names+types. If the work-fn's kwargs aren't reachable from the
   AST-walk, or `process/uses` can't reconcile at `bracket/map`, STOP and re-scope.

Only when both are green does the shape lock and the macro strike get drawn (design → RED probe → brief → sonnet
shadowdancer → weigh by own re-run). *Explorata caede, non vincimur.*
