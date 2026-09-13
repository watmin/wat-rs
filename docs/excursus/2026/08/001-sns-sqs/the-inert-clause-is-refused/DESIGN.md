# DESIGN — the inert clause is refused

Builder's ruling **D4-b**: *"`defservice` REFUSES `:deadline-ms` in `:satisfies` mode, with a diagnostic naming
the caller-side alternative."* **The last of the five.** It owed two probes; both are now run.

**Drawn 2026-09-13. NOT STRUCK.**

## ⭑⭑ BOTH PROBES ARE IN, AND THE CLAUSE IS INERT IN BOTH DIRECTIONS

```
probe 1  a :satisfies CALLEE declares :deadline-ms 300, its handler parks
         → the CALLER waits          outcome=TimedOut;elapsed-ms=10000;declared=300
         (wat-scripts/scratch-pad/probe-deadline-ms-is-callee-derived.wat, committed 004457a5d)

probe 2  a :satisfies service declares :deadline-ms 300, dials a parked peer from its OWN handler
         → its OUTBOUND call waits   handler-faced=TimedOut;handler-elapsed-ms=10000;declared=300
         (wat-scripts/scratch-pad/probe-deadline-ms-governs-nothing-outbound.wat, new)
```

> **`:deadline-ms` in `:satisfies` mode governs NOTHING — not what callers wait, not what the service's own
> handlers wait.** Both directions measured at the default 10 000 ms against a declared 300.

`wat/service.wat:2442` documents it as *"Optional `:deadline-ms` on the service. Default 10000. **Never
off**"* — a clause that is accepted, documented, and does nothing in the mode it is most likely to be written
in. **A convention wearing a wall's clothes.**

## ⭑ Probe 1b (`:ops` mode) is deliberately NOT RUN, and that is a measurement too

`:ops` is used **38×**, so the mode is real. But:

```
every real :deadline-ms declaration in wat/ wat-scripts/ wat-tests/ tests/  →  ONE
  wat-scripts/fanout/circuit.wat:2083   :deadline-ms 120000   (:satisfies mode)
:ops-mode services declaring :deadline-ms                     →  ZERO
```

⭐ **So the `:ops` answer cannot change the refusal's coverage: it is 0 sites either way.** Running it would
measure a combination that has never been written, to scope a refusal that would fire on nothing. **Named as
unmeasured rather than guessed — and named as *deliberately* unmeasured, with the reason.**

## ⛔⛔ AND THE REFUSAL BREAKS `circuit.wat` — which is the finding, not an obstacle

`:fanout::publisher` (`circuit.wat:2079–2083`) declares it, with a comment stating exactly why:

> *"stats is the join: it must outlive one publisher's share (~20 s at N=2000). Default 10000 would TimedOut
> mid-run and the parent would see a hang-shaped failure."*

⛔ **That declaration does nothing.** The publisher has been running on the 10 000 ms default all along.

⚠ **And the feared failure is not currently firing:** the happy path at n=2000 completes with
`distinct=8000;dup=0` and exit 0 on every run this session, so `stats` answers inside 10 000 ms at our sizes.
**The risk is latent, not active** — the comment's reasoning may simply be wrong about ~20 s, or the join may
be cheaper than the author feared.

## The one contract decision

> **Refuse the clause, DELETE the inert declaration, and FILE the latent risk — do not implement the author's
> intent in this stone.**

★ Deleting `:deadline-ms 120000` is **behaviour-preserving by construction**: it is measured to govern nothing,
so removing it changes nothing. That is provable and cheap.

⛔ **Converting the publisher's caller to `call-by-deadline 120000` would be a behaviour CHANGE** — the
*intended* one, but a change — and it deserves its own measurement (*does `stats` ever approach 10 s?*). Pairing
a refusal with a behaviour change would make one stone answer two questions and pin neither.

## The diagnostic must name the working alternative

A **caller-side** deadline does work — grok's STOP-5 on `the-store-can-fail`, and this stone's own probe used
`call-by-deadline … 40000`. So the refusal should read like its `:peers` siblings and end with the remedy:

```
<fqdn>: :deadline-ms is declared but governs nothing in :satisfies mode (measured: neither the
caller's wait nor this service's own outbound calls) — put the deadline at the CALL SITE with
(:wat::service::call-by-deadline peer op <ms> <fallback>), or drop the clause.
```

## The four questions

**Obvious?** YES — refusing a clause that provably does nothing, and naming what to use instead. **Simple?**
YES — one guard in `defservice` plus one deletion. **Honest?** YES, doubly: it makes an inert declaration
**unwritable**, and it refuses to smuggle in the behaviour change the deleted comment implies. **Good UX?**
YES — the next person who reaches for `:deadline-ms` is told, at compile time, where the deadline actually lives.

## Scope

**IN:** the refusal in `wat/service.wat` for `:satisfies` mode · the diagnostic above · deletion of
`circuit.wat:2083`'s inert declaration · a probe proving the refusal fires · a **FINDING** recording the latent
risk the deleted comment described.

**OUT = REJECTED:**
- ⛔ **Implementing the author's intent** (a `call-by-deadline 120000` at the publisher's caller). A behaviour
  change owing its own measurement. **The FINDING is the handoff.**
- ⛔ **Refusing it in `:ops` mode.** Unmeasured, and 0 corpus sites — refusing there would be a rule written
  against no evidence. ⚠ If the builder wants symmetry, that is a ruling, not this stone.
- ⛔ **Deleting the clause from the macro entirely.** It may be honoured in `:ops` mode; unmeasured. Refusing
  the measured-inert case is the honest rung.
- **Changing `wat/service.wat:2442`'s "Never off" comment.** It describes the default, which is real.

## Trap-doors

1. ⛔ **`circuit.wat` will fail to compile until the declaration is deleted.** Both changes are one stone; a
   floor run between them is red for a known reason.
2. ⛔ **Do not convert the publisher's caller.** Trap 1's fix is a **deletion**, not a migration.
3. **Mirror the `:peers` checks' wording** (`service.wat:896`/`:913`) — this is the fourth refusal in that
   family and should read like one.
4. **The floor count is 5244.** Four stones have moved it; a probe may move it again.
5. **`wat/service.wat` is stdlib** — but no Rust change, so no stash-dance.
