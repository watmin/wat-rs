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

### W2 scout — the carrier

**The grounded flow (real files, not theorized).** `process/uses` (macro, `wat/spawn.wat:176-200`) builds
`Vector<(keyword,Capability)>` — each handle up-casts to `Capability` at the `process/uses-pairs` call boundary
(`spawn.wat:167-174`, param type `Vector<(keyword,Capability)>`), which is where the concrete handle type dies; the erased
vector lands in `ProcessOpts.uses` (`spawn.wat:66-71`). `bracket/map<I,O,W>` (`bracket.wat:537`) and `map-worker<I,O,W>`
(`bracket.wat:444`) are plain **fns**, not macros — `map-worker` reads `uses = (Locus/uses locus)` at `bracket.wat:460` via the
`:wat::spawn::uses` defclause (`spawn.wat:220-222`), a **runtime** accessor over the already-erased value; no macro ever sees
the literal `process/uses` call, only the `let`-bound `locus` symbol flowing in by reference. `spawn-runner` → `process-work-forms`
(`bracket.wat:210-304`) is ALSO not a macro — it's a `defclause` (runtime dispatch), invoked from *inside* `map-worker`'s `peers`
`mapv` closure (`bracket.wat:463-497`), i.e. once per worker **at pool spin-up, in the parent process, per spawn call** — not at
static-check time of the parent's own file. It reads `::Kwargs` field types via `field-types-of` (`bracket.wat:256`) but never
touches `uses`/the handles — it only has `work-fn` in scope. **Conclusion: concrete handle types and `::Kwargs` field types never
meet at any single site in the real flow** — `uses` is erased before `map-worker`; `process-work-forms` never sees `uses` at all.
This also means the strike's framing of `process-work-forms` as "a compile-time macro" is imprecise, grounded against the disk:
it is runtime code (a `defclause`) that *generates* the child's source — genuinely static-before-any-execution reconciliation must
happen through the ORDINARY type checker at an ordinary call site (exactly how `probe-c2-colocation.wat`'s `dial-all` catches the
swap), not by injecting a check into this walk.

**Candidate A — BLOCKED, structurally then empirically.** `process/uses` keeps a `Tuple`/struct of concrete handles (not erased).
Probe: `scratchpad/probe-w2-a-heterogeneous.wat` defines `UsesCarrier` with a FIXED 2-tuple field, then constructs it with a
3-element tuple at a second call site in the same file. `wat --check` → located `ArityMismatch` (`:wat::core::Tuple: expected 2
argument(s); got 3`, line 18) + `TypeMismatch` (`:probe::UsesCarrier: parameter #1 expects :(wat::core::i64,wat::core::i64); got
:(wat::core::i64,wat::core::i64,wat::core::i64)`, line 18). A `defstruct` field type is fixed **once**, canonically, at
authoring time (mirrors `ProcessOpts.uses`'s single declared field type) — one struct definition cannot carry an arbitrary-arity
heterogeneous contract across call sites of different N. A collapses into C (ProcessOpts itself would have to become generic).

**Candidate B — BLOCKED, on a deeper reason than policy (probed twice, live).** First pass: a macro (`process/uses`-shaped)
tries to resolve a let-bound handle's type via `field-names-of`/`field-types-of`. `scratchpad/probe-w2-b-macro-sees-symbol-not-type.wat`
→ `MalformedDefmacro` (`keyword head ':wat::runtime::field-names-of' refused at macro expand time — not on the pure-combinator
allow-list (default-deny F5 gate, arc 249 stone 249.2b-i)`, lines 26-33) — **both** `field-names-of`/`field-types-of` were
categorically refused inside ANY macro body, independent of what they're called with. Per live builder direction mid-scout, this
was fixed for real: `:wat::runtime::field-names-of` / `:wat::runtime::field-types-of` added to the `is_pure_total` allow-list in
`src/macros/eval.rs` (same category as the already-allowed `signature-of-fn`/`extract-arg-names`/`extract-arg-types` — confirmed
pure by reading `eval_field_names_of`/`eval_field_types_of`, `src/runtime.rs:11593-11662`: read-only lookups against the frozen
`sym.types` registry, no IO, no mutation). Rebuilt; `cargo test --release -p wat --test macros` → 105 passed / 0 failed (captured
`scratchpad/macro-suite.txt`). Two `tests/collection` failures seen in one interleaved run were a pre-existing parallel-test race,
confirmed unrelated by isolating the single file (4/4 clean re-runs, both `--test-threads=1` and default). **Re-ran probe B against
the rebuilt binary — the purity gate no longer fires, but the REAL blocker surfaces**: `MalformedForm: unknown type ':eh'` (line 32)
— `field-names-of`/`field-types-of` key off a **registered type name** in the frozen registry; a let-bound local's bare symbol
name (`eh`) is never one. Macros run on unevaluated syntax, before type inference — there is no primitive, purity-gated or not,
that maps "this local variable" to "its inferred type." **The purity fix was necessary but not sufficient for B as literally
probed** (a macro trying to read a handle's type off its call-site expression). The `src/macros/eval.rs` change is real, tested
green, and left **uncommitted** in the working tree (only this doc is committed by this scout, per the strike's scope) — the
builder should independently land or fold it.

**Candidate C — BLOCKED, empirically (no precedent, and the naive attempt silently no-ops).** `grep -n "extend-type" wat/*.wat`
shows every `extend-type` in the tree targets a CONCRETE struct (`ThreadOpts`, `ProcessOpts`) — never a generic one, even though
generic structs exist (`Bound<S,R>`, `Launched<S,R,Sh,Lu>`, both FIXED-arity type-param lists, never variadic). Probe:
`scratchpad/probe-w2-c-generic-extend-type.wat` defines `Carrier<U>` and `(extend-type :probe::Carrier<U> :probe::Foo (greet
[self] 42))`. The `extend-type` declaration itself raises **no error** (silently accepted) — but the resulting instantiation is
not wired: calling `(:probe::Foo/greet c)` where `c : Carrier<i64>` → located `TypeMismatch` (`:probe::Foo/greet: parameter #1
(receiver) expects :probe::Foo; got :probe::Carrier<wat::core::i64>`, line 22). There is no "blanket impl across all U" mechanism
in the language today — a parametric `ProcessOpts<U>` would NOT satisfy the plain (non-generic) `:wat::spawn::Locus` surface
(`spawn.wat:371`) the way concrete `ProcessOpts` does now.

**STOP-1 triggered, verbatim per the brief.** None of A/B/C, *as literally specified*, arrange a static reconciliation:
A collapses into C; B's macro-time reflection dead-ends on inferred-type-of-a-local (a real primitive gap, not a policy gap);
C needs a generic-struct-`extend-type` capability the substrate doesn't have and my probe shows silently no-ops rather than
erroring loud (worth its own bug ticket regardless of C2 — a `defclause`/`extend-type` target with a free type variable should
either work or refuse at the declaration, not silently accept and then fail every call site).

**A fourth direction survives grounding (not literally A/B/C — a synthesis, unbuilt, for the builder to ratify).** Re-reading
the ALREADY-PROVEN `probe-c2-colocation.wat` mechanism: `dial-all`'s swap-catch needs **no** surface/`extend-type`/`Locus`
machinery at all — it is an ordinary `defn` whose declared parameter type is a literal, field-ordered `Tuple<Address'<S,R>…>`,
called with a `Tuple` of `Dialable/coord` values built at the call site; the ORDINARY type checker does the reconciling, the
same way it already checks any other call. Generalizing: nothing stops a NEW macro (`bracket/map-uses`, since `process/uses`
itself never sees the work-fn and can't do this alone) that, given both the `:name handle` pairs and the work-fn keyword:
(1) reads `<base>::Kwargs`'s **field NAMES ONLY** via `field-names-of` (now macro-legal after the fix above, and unblocked by B's
own finding — the arg is `<base>::Kwargs`, a REGISTERED type name derived by string-concat off the literal work-fn keyword,
exactly like `process-work-forms` already does at `bracket.wat:230-231` — never a local symbol, so B's local-binding blocker
never applies here); (2) reorders the caller's `(kn vn)` pairs to match field order — pure AST/keyword-literal manipulation, no
type reflection; (3) splices each `vn` VERBATIM (exactly `process/uses`'s existing trick, `spawn.wat:191-196`) into
`(Dialable/coord ~vn)` calls, in field order, fed to a companion checker — a NEW auto-emitted declaration paired 1:1 with
`::Kwargs`, field-for-field `Peer'<S,R>` → `Address'<S,R>` (the SAME head-swap string-split/join `process-work-forms` already
proves at `bracket.wat:338`, since `::Kwargs`'s own ctor wants `Peer'` — the connected peer — not the bare `Address'` a parent
holds pre-dial, so `::Kwargs`'s own constructor cannot double as the checker). The macro never needs to know any handle's type;
it only ever touches registered names (`<base>::Kwargs`) and splices unevaluated syntax — the ordinary checker, at the ordinary
call it emits, does the catching. **Unbuilt, unprobed as a whole** — flagged for the builder, not this scout's deliverable.

**RECOMMENDED carrier: the fourth direction (companion-checker macro), NOT literally A/B/C.**

| | verdict |
|---|---|
| **Obvious?** | ~ — reuses two ALREADY-PROVEN moves (verbatim AST splice from `process/uses`; ordinary-call type-check from `dial-all`) rather than inventing a third; not obvious that C2's own mechanism generalizes this way until traced, which this scout did. |
| **Simple?** | NO, honestly — it needs a NEW macro (`bracket/map-uses`, since `process/uses` alone lacks the work-fn) AND a NEW auto-emitted companion declaration per `::Kwargs` (the `Peer'→Address'` head-swapped checker). Real, non-cosmetic build. |
| **Honest?** | YES — the actual reconciliation is done by the SAME type checker that already catches every other type error, at an ordinary call, not by a bespoke walk; a swap is caught the identical way `dial-all` already proves it, no new class of diagnostic. |
| **Good UX?** | Unclear pending the builder's call — `(bracket/map-uses locus items :work :echo eh :kv kvh)`-shaped is plausible but not designed; whether it subsumes or sits beside `process/uses` is an open sub-decision below. |

**Open sub-decisions for the builder:**
1. Does `bracket/map-uses` REPLACE `process/uses` + `bracket/map`, or compose beside them (keeping `process/uses`'s existing
   grant/revoke-only role and adding a SEPARATE check-emitting macro at the `bracket/map`/`each` call site, which already knows
   both `locus` and `work-fn`)? The latter touches less ratified surface.
2. Who emits the companion `Peer'→Address'` checker declaration, and when — the same machinery that mints `::Kwargs` (at
   *its* definition time, zero reflection needed there, the types are already literal source), or `process-work-forms` itself
   (at first spawn, runtime, reflection-based, mirroring how it already derives `Address'` from `Peer'` at `bracket.wat:338`)?
3. Land or drop the `src/macros/eval.rs` purity-allow-list change (uncommitted) — it is independently correct (proven pure,
   105/105 macro tests green) and is a prerequisite for step 1 of the fourth direction regardless of which sub-decision above
   is chosen; recommend landing it now, separately, rather than re-deriving it mid-W2-build.
4. File the `extend-type`-on-a-generic-struct silent-no-op as its own bug (STOP-1's C probe) — independent of whether C2
   ever needs generic-struct `extend-type`, a declaration that's accepted but never satisfiable at any call site is a footgun.

### W2 MECHANISM CORRECTED (2026-07-09b) — reflection at macro-expand is NOT viable; the check is minted at defn-time (Path B)

> **A disconfirming probe overturned the reflection assumption below.** The scout's `bracket/uses` macro was to
> reflect the work-fn's `::Kwargs` (via `field-*-of`) at MACRO-EXPAND time to build the per-field `Address'` check.
> **It cannot:** `field-types-of :probe::enrich::Kwargs` inside a macro body fails with `unknown type
> ':probe::enrich::Kwargs'` — the `::Kwargs` type is not registered when a macro expands (it registers later, at
> freeze). The SAME call works at runtime/check-time — which is why C1's `process-work-forms` (a *defclause*, not a
> macro) reflects `::Kwargs` fine. Probes (harness scratchpad, ephemeral): `probe-w2-reflect-at-macro-expand.wat`
> (fails at expand) vs `probe-w2-reflect-at-runtime.wat` (works at runtime).
>
> **Path B (ratified via the four-questions — Obvious+Simple+Honest+Good-UX all YES; the check-time-reflection-hook
> alternative died on Obvious+Simple):** don't reflect at expand — mint the check where the field types are LITERAL
> source (defn-time), as an ordinary typed fn, and let the ORDINARY type checker fire at freeze.
>
> - **Auto-mint `<fqdn>::kwargs-check`** at the site where the kwargs `defn` already mints `$impl` / the companion /
>   `::Kwargs`. It is a KWARGS fn whose params are the field-ordered `Address'<S,R>` types (head-swap `Peer'<S,R>` →
>   `Address'<S,R>`, exactly the transform C1's `process-work-forms` does at `bracket.wat:266-270`, but from the
>   literal defn source — zero reflection):
>   ```clojure
>   (:wat::core::defn :probe::enrich::kwargs-check
>     [& [echo <- :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>
>         kv   <- :wat::kernel::Address'<probe::Kv::Op,probe::Kv::Reply>
>         tag  <- :wat::core::String]]           ;; a data kwarg keeps its own type
>     -> :wat::core::nil nil)
>   ```
> - **`bracket/uses` degenerates to a THIN macro**: rewrap each service `:name val` → `:name (Dialable/coord val)`
>   (data kwargs pass through unwrapped), emit ONE call `(:probe::enrich::kwargs-check :echo (Dialable/coord eh) …)`,
>   then hand off to the runner (W3). NO reflection, NO field-name/type knowledge at expand — the generated kwargs
>   checker name-matches (its own kwargs) and the ordinary checker catches the swap.
> - **PROVEN (2026-07-09b, on the disk):** the hand-written checker shape catches it — `probe-w2b-ok.wat` (correct,
>   order-free) freezes CLEAN; `probe-w2b-swap.wat` (swapped) is a located `TypeMismatch` at `wat --check`:
>   `expects Address'<probe::Echo::Op,probe::Echo::Reply>; got Address'<probe::Kv::Op,probe::Kv::Reply>`. This is the
>   same compile-time mechanism the committed test `tests/services/probe_arc170_wrong_service_colocation.wat.bad`
>   proves — B just GENERATES the checker (kwargs-keyed → order-free) instead of hand-writing a positional `dial-all`.
>
> **The user-form UX and the compile-time guarantee are UNCHANGED** (the whole point — the reflection was an
> internal detail): the forms below stand verbatim; only the internal "how the check is emitted" changes from
> macro-reflection to defn-time-minted checker. **Build order (B):** (1) auto-mint `<fqdn>::kwargs-check` at the
> kwargs-defn codegen site; (2) `bracket/uses` (thin macro: rewrap + forward + hand to runner); (3) `bracket/uses'`
> runtime — generalize C1's `process-work-forms` N=1 → N. The GATE is unchanged (swap → located `TypeMismatch`;
> correct → `["run-7 echo:a·kv:a" …]`). The `field-*-of`-at-macro-expand reflection and the A/B/C carrier analysis
> below are SUPERSEDED by this; the user-forms + target-expansion sections are kept (UX unchanged; the target
> expansion's `(ann-form (Dialable/coord h) <field Address'>)` is now the `kwargs-check` call, semantically identical).

### W2/W3 — RATIFIED (2026-07-09): the carrier is `bracket/uses` (macro) + `bracket/uses'` (impl)  *(mechanism superseded by Path B above; UX + gate unchanged)*

The W2 scout's synthesis is RATIFIED with the builder, named per the companion/prime convention (same shape as
every `<name>` / `<name>$impl` kwargs companion in the language):

- **`:wat::bracket::uses`** (bare) = the **kwargs-surface MACRO** the user writes. It fuses the service provision
  INTO the dispatch — co-located with the work-fn, the ONLY site the compiler can catch a swap (the scout proved a
  standalone `process/uses` locus CANNOT: A blocked — a fixed `ProcessOpts` field can't hold the arbitrary-arity
  heterogeneous typed bundle; B blocked — a macro can't reflect a *local var's* type, only registered names; C
  blocked — `extend-type` on a generic struct never wires a concrete instantiation).
- **`:wat::bracket::uses'`** (prime) = the **expanded impl** the macro produces.

**User form:**
```clojure
(:wat::bracket::uses <locus> <items> <work-fn> :name val …)
;; e.g.
(:wat::bracket::uses (:wat::spawn::process) ["a" "b" "c"] :probe::enrich :echo eh :kv kvh)
```
- `<locus>` is CONFIG only (`(process)` / `(process/runner-count 8)` / `(process/env …)`) — composable as before;
  the SERVICES move to the `:name val` kwargs.
- **Order-free** (`:kv kvh :echo eh` identical); **swap → compile error** (`:echo kvh` where `:echo` wants
  `Peer'<Echo…>` → located TypeMismatch at `wat --check`, NEVER a runtime peer-closed).
- **`process/uses` (the standalone locus) RETIRES into this.**

**The `bracket/uses` macro, at expansion** (one site — items + work-fn + `:name val` all present):
1. reflects the work-fn's `::Kwargs` via `field-names-of`/`field-types-of` (now on the pure-total allow-list, `95460eb7`);
2. name-matches each `:name val` to the `::Kwargs` field by name (kwargs-lower's unification; missing/extra name = error);
3. per field, emits a compile-time type check that the value fits the field's declared type — for a `Peer'<S,R>` field,
   the handle's coordinate must be `Address'<S,R>` (a `Peer'→Address'` checker, reusing `dial-all`'s proven mechanism,
   `probe-c2-colocation.wat`) → a wrong-service handle is a compile error;
4. expands into `(:wat::bracket::uses' …)` with the reconciled bundle + the work-fn's `$impl`.

**`bracket/uses'` (the impl) — W3 runtime:** grant N pids + dial N typed addresses (via the typed `Dialable/coord`) +
assemble the one `::Kwargs` struct in the child + invoke via the companion (C1's mechanism, N=1 → N).

**Mixed kwargs — the `::Kwargs` struct holds ANY typed field; each field's TYPE routes its crossing** (ocap discipline):
`Peer'<S,R>` (a service) → a Handle → grant + dial → a live peer; pure data (`i64`/record/`String`) → the value →
copied as EDN across the wire (no dialing); a live resource → forbidden (293.W — compile error).

**Scope (`exigere`):** build the SERVICE-DIAL path first (the hard part); the data-copy field lands when a real consumer
passes one — the shape admits it for free (same reconcile-by-name-into-the-struct; the type routes copy-vs-dial).

**Build order (far side):** W2 = the `bracket/uses` MACRO (reflect `::Kwargs` + name-match + the per-field compile-check)
→ a `bracket/uses'` shell. W3 = the `bracket/uses'` runtime (dial N + assemble + invoke, generalizing C1) + retire
`process/uses`. **GATE:** `(bracket/uses (process) ["a" "b" "c"] :probe::enrich :echo kvh :kv eh)` SWAPPED → a located
TypeMismatch at `wat --check`; the correct wiring runs `["echo:a·kv:a" …]`.

**Open sub-decision (arg order):** `(bracket/uses locus items work-fn :name val …)` vs fn-first
`(bracket/uses work-fn locus items :name val …)` (to match the ratified core/map fn-first flip). Pin on the far side.

#### The exact forms (the gate program — write this verbatim)

```clojure
;; ── two services (surfaces + defservices) ─────────────────────────────────────
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])
(:wat::service::defservice :probe::echo'
  :satisfies :probe::Echo  :durable []  :ephemeral []
  :impls [(echo [s req]
            (:wat::service::Outcome::Reply s
              (:probe::Echo::EchoResponse
                (:wat::core::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Kv::GetRequest  [k <- :wat::core::String])
   (:wat::core::defrecord :probe::Kv::GetResponse [v <- :wat::core::String])]
  :features
  [(get [self <- :probe::Kv  req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse)])
(:wat::service::defservice :probe::kv'
  :satisfies :probe::Kv  :durable []  :ephemeral []
  :impls [(get [s req]
            (:wat::service::Outcome::Reply s
              (:probe::Kv::GetResponse
                (:wat::core::string::concat "kv:" (:probe::Kv::GetRequest/k req)))))])

;; ── the work-fn: item POSITIONAL; services + PURE DATA as KWARGS ───────────────
(:wat::core::defn :probe::enrich
  [item <- :wat::core::String
   & [echo <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>
      kv   <- :wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>
      tag  <- :wat::core::String]]                     ;; a DATA kwarg (copied, not dialed)
  -> :wat::core::String
  (:wat::core::string::concat tag
    (:wat::core::string::concat " "
      (:wat::core::string::concat
        (:probe::Echo::EchoResponse/reply (:probe::Echo/echo echo (:probe::Echo::EchoRequest item)))
        (:wat::core::string::concat "·"
          (:probe::Kv::GetResponse/v (:probe::Kv/get kv (:probe::Kv::GetRequest item))))))))

;; ── the dispatch: (bracket/uses locus items work-fn :name val …) ──────────────
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh  (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     kvh (:probe::kv'/start   :locus (:wat::spawn::process) :record (:probe::kv'::Record))
     out (:wat::bracket::uses (:wat::spawn::process) ["a" "b" "c"] :probe::enrich
           :echo eh :kv kvh :tag "run-7")]              ;; services + data, by name, order-free
    (:wat::kernel::println out)))
;; → ["run-7 echo:a·kv:a" "run-7 echo:b·kv:b" "run-7 echo:c·kv:c"]
```

**Order-free** — the last let-binding written any of these is IDENTICAL:
```clojure
(:wat::bracket::uses (:wat::spawn::process) ["a" "b" "c"] :probe::enrich :kv kvh :tag "run-7" :echo eh)
```

**The negative (the soundness gate) — a SWAPPED handle:**
```clojure
(:wat::bracket::uses (:wat::spawn::process) ["a" "b" "c"] :probe::enrich :echo kvh :kv eh :tag "run-7")
;; MUST fail `wat --check`:
;;   TypeMismatch: :echo expects Address'<probe::Echo::Op,probe::Echo::Reply>; got Address'<probe::Kv::Op,probe::Kv::Reply>
;; (via (Dialable/coord kvh) : Address'<Kv…> ascribed to the :echo field's Address'<Echo…>) — NOT a runtime peer-closed.
```

**A missing / extra name — a NAME error at expand:**
```clojure
(:wat::bracket::uses (:wat::spawn::process) ["a" "b" "c"] :probe::enrich :echo eh :tag "run-7")   ;; missing :kv
;; → macro-error: "bracket/uses: missing argument :kv" (kwargs-lower's unification)
```

#### The target expansion (what `bracket/uses` produces — pin the exact `uses'` arity on the far side)

```clojure
;; (bracket/uses (process) ["a" "b" "c"] :probe::enrich :echo eh :kv kvh :tag "run-7")
;; expands (conceptually) to:
(:wat::core::let
  ;; (1) parent-side SWAP-CATCH — one ann-form per Peer' field: the handle's typed Dialable coord
  ;;     ascribed to the FIELD's Address'<S,R> (from field-types-of on :probe::enrich::Kwargs).
  ;;     A wrong-service handle fails HERE, at check time (the proven mechanism, probe-c2-typed-coordinate.wat).
  [_chk-echo (:wat::core::ann-form (:wat::capability::Dialable/coord eh)
               :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>)
   _chk-kv   (:wat::core::ann-form (:wat::capability::Dialable/coord kvh)
               :wat::kernel::Address'<probe::Kv::Op,probe::Kv::Reply>)]
  ;; (2) hand off to the impl: the config locus, the items, the work-fn's $impl, the FIELD-ORDERED
  ;;     service handles (grant + dial per worker), the FIELD-ORDERED data values (copied as EDN),
  ;;     and the ::Kwargs type-def (assembled in the child, then invoked via the companion — C1's mechanism, N=1→N).
  (:wat::bracket::uses' (:wat::spawn::process) ["a" "b" "c"]
    :probe::enrich$impl
    [eh kvh]                      ;; field-ordered Peer' handles → grant + dial
    ["run-7"]                     ;; field-ordered pure-data → copied
    :probe::enrich::Kwargs))
```
The `uses'` signature (how it takes the handle vector, the data vector, the `$impl`, the `::Kwargs` type) is the
one thing to PIN in the W3 build — but the parent check `(ann-form (Dialable/coord h) <field Address'>)` and the
child assemble+invoke (C1) are both already proven; W3 is generalizing C1's N=1 to N + wiring the data-copy path
(exigere: service-dial first).
