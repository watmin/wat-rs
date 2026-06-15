# Arc 232 — defprotocol / extend-type (the open type-bound)

> **Status: DESIGN (unparked 2026-06-14).** Arc 232 was a reserved name with no artifacts; designed
> fresh here. Unparked because arc 209's host-agnostic `start` proved it a hard dependency: a
> disconfirming probe showed an abstract forwarded arg gets `NoMatchingClauseAtCallSite` — there is
> no open type-bound in wat, and a closed host sum is the antipattern. defprotocol IS that bound.
> Grounded against HEAD `ef1c1462`. Builder: *"block our current work and build the dep we need."*

## Why (the driving consumer)

A defservice must be host-agnostic over a GROWING transport set (thread, process, +future
localhost-TCP, remote-mTLS). The honest seam is `start [host <- :Host] -> Handle` where `:Host` is
an OPEN bound any transport's opts can join. wat has no such bound today:

- **No anonymous unions** (ADT language) — can't write `ThreadOpts | ProcessOpts`.
- **defclause dispatch is closed at the call site** — `assignable(arg, clause_expected)` per clause;
  an abstract `:H` matches no concrete clause → `NoMatchingClauseAtCallSite` (check.rs:5483).
  Proven by `probe_diagnostic_shared_return_defclause_forward` (deleted; finding in
  [[feedback_deferred_dep_becomes_necessary_block_and_build]]).
- **A closed `Host` sum** = central surgery per transport (the rejected antipattern).

`defprotocol` is the open, ADT-honest bound (Clojure's protocol, typed): operations closed,
implementors open. New transport = `extend-type :TcpOpts :Host (…)`, zero edit to `start`.

## What it is (Clojure-faithful, typed)

```clojure
;; declare the protocol + its method signatures (self is the protocol-bound receiver)
(:wat::core::defprotocol :wat::kernel::Host
  (listen [self <- :wat::kernel::Host  s <- :S  r <- :R] -> :wat::kernel::Endpoint<S,R>)
  (spawn  [self <- :wat::kernel::Host  prog <- :P]        -> :wat::kernel::SpawnHandle<R,S>))

;; a concrete type joins the protocol (the satisfaction edge + the impls)
(:wat::core::extend-type :wat::spawn::ThreadOpts :wat::kernel::Host
  (listen [self s r] …mint crossbeam…)
  (spawn  [self prog] …spawn-thread'…))
(:wat::core::extend-type :wat::spawn::ProcessOpts :wat::kernel::Host
  (listen [self s r] …autobind UDS + readback…)
  (spawn  [self prog] …spawn-process'…))

;; now a fn can be typed over the OPEN bound; methods dispatch on the concrete type at runtime
(:wat::core::defn :my::counter/start [host <- :wat::kernel::Host  s0 <- :State] -> :my::counter::Handle
  …(:wat::kernel::listen host …)… …(:wat::kernel::spawn host …)…)
```

(`:Host`/`listen`/`spawn`/`Endpoint`/`SpawnHandle` names are illustrative — **intueri at draw**.)

## Grounded integration points

1. **Type model** (`types.rs:70`): a protocol is a `TypeExpr::Path` naming a registered protocol.
   No new `TypeExpr` variant needed — the protocol-ness lives in a registry + `assignable`, exactly
   as record-subtyping does today. (Confirm at strike: no Parametric-protocol need for v1.)
2. **Satisfaction** (`assignable`, check.rs ~3186 precedent): add one edge — `assignable(T, :P)`
   holds iff `T` has an `extend-type … :P` entry. Mirrors the `is_subtype`/`:wat::Record`
   precedent. This is the single change that makes `start [host <- :Host]` accept any extender.
3. **Dispatch** (check.rs:5416-5491 is the CLOSED defclause path; protocol methods need the OPEN
   analog): a protocol-method call types via the protocol's declared method signature (accept any
   arg `assignable` to `:P`, return the declared method return); at RUNTIME it dispatches on the
   receiver's concrete type via the extend-type registry (open — any extender, not a fixed clause
   list). This is the key divergence from defclause: registry lookup, not closed first-match.
4. **The two forms**: `defprotocol` (register protocol name + method schemes) + `extend-type`
   (register `(P, T) → impl bodies` + the satisfaction edge). Likely special forms or wat macros
   over a registry primitive — **decide at draw** (mirror how defclause/recordtype register).
5. **Registry**: a `protocol → methods` table + a `(protocol, type) → impls` table, in the
   SymbolTable / CheckEnv (alongside `get_defclause_clauses`, check/env.rs:301).

## Four-Q

- **Obvious?** YES — `defprotocol`/`extend-type` are the Clojure names; a reader knows them.
- **Simple?** YES — one satisfaction edge in `assignable` + one open-dispatch path; no new TypeExpr
  variant. (If v1 needs Parametric protocols or default methods, that's a smell to decompose — cut.)
- **Honest?** YES — it names the open abstraction the host seam needs; no closed sum lying that the
  transport set is fixed; the wrong impl (a type not extending P passed where :P is required) is
  uncompilable via `assignable` ([[feedback_no_magic_that_lets_llm_fake_correctness]]).
- **Good UX?** YES — new transport = one `extend-type`, zero central edit. The organic seam.

## The RED probe (the gate the design rests on)

Re-create the killed probe in protocol form: `defprotocol :t::P (make [self] -> :t::Out)` +
`extend-type :t::OptA :t::P` + `extend-type :t::OptB :t::P` + a fn `fwd [o <- :t::P] -> :t::Out
(make o)` + `(fwd (:t::OptA))`. RED at HEAD (defprotocol unknown). GREEN once the mechanism ships.
This is the exact shape host-agnostic `start` needs.

## Decomposition (strikes — each RED-probe-gated, delegated, weighed)

1. **232.1 — `defprotocol` + `extend-type` parse + registry** (declare protocol/methods; register
   impls + satisfaction edge). No dispatch yet; gate = the forms register + a registry read.
2. **232.2 — `assignable(T, :P)` satisfaction edge** — a `:P`-typed param accepts an extender.
   Gate = a fn `[x <- :P]` accepts a value of an extending type, rejects a non-extender.
3. **232.3 — protocol-method dispatch** (check-time via the method scheme; runtime via the
   registry on the concrete type). Gate = the RED probe above goes GREEN.
4. **232.N — INSCRIPTION.**

Then arc 209 resumes on this foundation: `SpawnHandle` sum + `Endpoint` record + `listener'`
uniform mint + `Host` protocol + host-agnostic `start`.

## Scope / out (rejected here, not deferred-silently)

- **Default method impls / protocol inheritance / Parametric protocols** — OUT of v1 unless a strike
  proves them load-bearing for the host seam (they are not). Clojure-parity extras are their own arc.
- **The host consumer itself** (`Host` protocol, `SpawnHandle`, `Endpoint`, host-agnostic `start`) —
  OUT of arc 232; that's arc 209's resumption ON this mechanism. 232 ships the mechanism + a generic
  proof, not the host wiring.
- **Migrating existing defclause kernel intrinsics to protocols** (arc 256, banked) — separate.

## Open (intueri / four-Q at draw — do NOT punt)

- All names (`defprotocol`/`extend-type` are likely fixed by Clojure parity; the host-consumer names
  `Host`/`listen`/`spawn`/`Endpoint`/`SpawnHandle` → intueri when arc 209 resumes).
- Whether `defprotocol`/`extend-type` are Rust special forms or wat macros over a registry primitive
  (mirror defclause/recordtype — ground at 232.1).
- Whether protocol-method dispatch reuses any of the defclause dispatch machinery or is a clean
  separate path (the open-vs-closed distinction suggests separate; confirm at 232.3).
