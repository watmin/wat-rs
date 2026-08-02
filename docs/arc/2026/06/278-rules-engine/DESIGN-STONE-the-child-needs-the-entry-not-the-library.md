# DESIGN-STONE — the child needs the ENTRY, never the LIBRARY

> **Status: DESIGNED, builder-ruled 2026-08-02 (option B of a four-questions fork).** The RED gate
> already exists and is standing: 14 `*_on_process` failures on the current floor. No probe to author.

## How it surfaced

The namespacing wall's `Reserved` arm, armed for scalar `def` (`BRIEF-scalar-def-reaches-the-gate`),
turned 14 process-locus service tests red. Every one is a child `StartupError` carrying
`#wat.check/CheckErrors`, and every span points at **`src/wat_edn_bridge.rs:442` / `:468`** — not a
source file. That is the signature of forms *reconstructed from the wire*: the shipped
`service-forms`, checked in the child. The thread-locus twins all pass (`sift_logs_…` PASSes,
`sift_logs_…_on_process` FAILs) because a thread shares the parent's universe and re-declares nothing.

## The mechanism, and it accounts for every error

`wat/service.wat:1785` emits

```clojure
(:wat::core::defn :<fqdn>::service-forms [] -> :wat::core::Vector<wat::WatAST>
  (:wat::core::concat ~peers-forms-node ~own-forms-call))
```

For `:wat::query::mem-store` the two halves expand to exactly the 11 names the child refuses:

| half | site | re-emits | n |
|---|---|---|---|
| `peers-forms-node` | `service.wat:1778-1784` | `:wat::query::Store::surface-forms` + its 4 `*-MAX-REQUEST-BYTES` | **5** |
| `own-forms-call` | `service.wat:1749-1763` | `mem-store::{serve,init,stop-project,hibernate-project,dispatch-admin,extract-addr}` | **6** |

5 + 6 = the child's *"11 type-check errors"*, name for name. The shape is fully accounted for; nothing
is inferred.

**Why it was ever legal:** privilege does not survive a process boundary. In the child these are the
post-`register_defines` USER residue, so a `:wat::`-rooted `def` is precisely what
`resolve::gate -> Reserved` exists to refuse. It compiled for as long as it did only because a scalar
`def` never reached that gate.

**And the child already has all 11.** The stdlib bake is exhaustive — `src/stdlib.rs`, 96 files,
`include_str!`, no load gate; its own header states the forms register *before* user entry forms reach
macro expansion. A child that boots `wat/query/mem.wat` and `wat/query.wat` from its own bake needs
none of them shipped.

## The rejected option, and why it is rejected

**(A) — make the check-side door's presence test consult the population the name actually lives in**
(so a stdlib-baked companion returns `Equivalent → NoOp`). This was the tempting small fix. It fails
the four questions:

- **Simple? NO.** One `contains_key` doing two jobs — *"is this the substrate's own form replayed?"*
  and *"is someone claiming a taken name?"* — braided into a single presence test that cannot separate
  them.
- **Honest? NO, and severely.** `gate`'s ordering short-circuits `Equivalent → NoOp` **before** the
  reserved check (`src/resolve/registration.rs:80`). So (A) would make *any* user `def` whose name
  matches an already-registered stdlib symbol silently discarded rather than refused — a hole across
  the whole language, punched in the wall this stone's sibling was built to erect. A forged
  `(def :wat::core::first …)` would be met with silence.

(A) would also have turned 14 tests green while leaving the redundant shipping in place — the exact
shape R59 `NISI FRANGAS, NIHIL PROBAS` names.

## The ruling — the child needs the ENTRY, never the LIBRARY

Ship the child only what it cannot already have.

**And the refinement that a naive "don't ship it" would have broken:** `own-forms-call` ends in
`~child-main-form` (`service.wat:1763`) — the agnostic `:user::main` that binds the launch coordinate
`:user::spawn::service-locus`. That is generated per service, is in no bake, and IS the child's entry
point. Drop it and the child has nothing to run.

So:

| the service's fqdn | ship |
|---|---|
| **`:wat::`-rooted** (baked stdlib) | the `child-main-form` ONLY |
| **anything else** (user / app) | everything, exactly as today |

## Why the discriminator is an INVARIANT, not a heuristic

`:wat::` is reserved (`src/resolve/reserved.rs:25-27`) and the gate refuses a user-privilege
registration under it. Therefore **a `:wat::`-rooted service name can only have been declared by baked
stdlib source** — there is no other way for one to exist. The wall we just armed is what makes this
optimization sound; it is derived from the reservation, not guessed from a naming convention.

The macro already holds both discriminands at expansion: `fqdn-base` (used at `service.wat:1664` to
build `service-forms-kw`) and `proto-base` (used at `:1771-1772` to build `surface-forms-kw`). No new
plumbing, no privilege plumbed through the wat layer, no `MacroRegistry::stdlib_privilege` reached for
(that flag is Rust-side and is not visible to a wat macro — checked).

## Open, and it is the one thing that decides the stone's residue

Whether `:wat::cache::lru-svc<K,V>`'s **parametric instantiation** is resolvable from the child's own
bake of the generic, or genuinely must cross the wire. The bake carries the generic declaration; if
monomorphization happens in the child at check time, there is no exception and this stone covers
everything. **Do not assume either way — the strike discovers it:** if `lru-svc` goes green with only
its `child-main-form` shipped, there is no residue. If it stays red, *that* is the genuine provenance
case the SEAM was reaching for — one real case, not a family.

## STOP triggers

- **STOP-1 — a battery/distribution service under `:wat::`.** `installed_dep_sources` (`src/source.rs`,
  via `src/stdlib.rs`) admits dependency sources into the stdlib set. If such a source can declare a
  `:wat::`-rooted service AND can be absent from the child's image, the invariant above is punctured.
  The exec'd child re-runs the same binary, so it should hold — but STOP and report rather than
  assume; the invariant is the whole stone.
- **STOP-2 — the child needs more than the entry.** If dropping the internals leaves a child unable to
  resolve something (a hygiene-scoped name, a per-instantiation record), STOP and report exactly which
  form and why. Do NOT restore the whole `own-forms-call` to get green — that is the defect.
- **STOP-3 — a user service regresses.** The non-`:wat::` path must be byte-identical to today.
  Verify a user-declared `defservice`'s emitted `service-forms` is unchanged; if it is not, STOP.

## The gate

The 14 `*_on_process` failures ARE the acceptance test — they go green, and no thread-locus twin
regresses. Floor weighed centrally by the orchestrator's own `--release` re-run against the
post-stone baseline.
