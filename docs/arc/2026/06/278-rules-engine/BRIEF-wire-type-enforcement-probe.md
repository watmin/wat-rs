# BRIEF — MEASUREMENT probe: does the service wire decode ENFORCE a declared payload type?

> Builder-ruled 2026-07-25: option **(a)** — the parametric protocol — after four-questions (A: 4×YES;
> concrete-messages failed Obvious, Simple, Honest). **This is stone 1 of (A), and it is a MEASUREMENT,
> not a fix.** Draw the wire probe before any plumbing.

## Why this comes first

Extending the protocol synthesis means threading type parameters (`Op<K>`/`Reply<K>`, the client fn, the
serve-loop arms, `Peer'<Op,Reply>`) across `wat/service.wat` and `synthesize_surface_protocol`. That work is
**moot** if the wire erases the type on arrival — you would be threading a parameter to a boundary that
ignores it.

The four-questions passed (A)'s *Honest* **conditionally**, and this is the condition, verbatim from the
ruling: *the decode must actually enforce `K` at the boundary — if a `Vector<K>` decodes without checking
`K`, the parameter is decoration.* Nobody knows the answer. The previous rider could not reach it
(the parametric path is blocked) and correctly declined to infer it.

## THE DELIVERABLE IS AN ANSWER, NOT GREEN CODE

**Do not fix anything. Do not make anything pass.** Report what the substrate *does*, with evidence. A
finding of "the wire does NOT enforce declared payload types" is a complete success for this strike and
changes the campaign — it is not a failure to be worked around.

## The question, decomposed

**TIER 1 — the decisive one, testable TODAY with no parametric anything.**
Does a service's wire decode enforce the **declared concrete** request-payload type?

Concretely: a service whose op declares `req <- SomeRequest` where `SomeRequest [items <- Vector<String>]`.
Deliver a payload that is **well-formed EDN but wrong-typed** — e.g. `{:items [1 2 3]}` (i64s where
`Vector<String>` is declared). Does the server reject it with a named, located failure, or accept it and
hand the handler a mistyped value?

If concrete types are **not** enforced, generic ones certainly will not be, the *Honest* condition fails,
and **(A) needs re-ruling** — STOP and report that, loudly.

**TIER 2 — only if Tier 1 says ENFORCED.** Ground *how*: which function performs the check, what it compares
against (the registered `TypeDef`? the surface member's `TypeExpr`?), and — the load-bearing part — whether
that mechanism would have a bound `K` available at instantiation, or whether it resolves the declared type
statically in a way a type parameter could not reach. Cite `file:line`. This tells the next stone what must
carry `K` and where.

## ⚠ THE PROBE MUST WALK THE REAL PATH

A unit test of the decoder in isolation **proves nothing** — this project has been burned by exactly that
(a probe that skated past the production mechanism and produced a false GREEN). It must be a **real
`defservice`**, stood up on a **real locus**, reached by a **real `connect'`**, with the payload crossing
the **actual wire**, using the same send/recv verbs production uses. Model on the shape of
`wat-tests/service-parametric-two-params.wat`.

The awkward part is *delivering* a wrong-typed payload, since the generated client fn is typed and the
checker will reject a mistyped call at compile time. That is the point — you must get **underneath** the
typed client to put raw EDN on the wire. Ground how before you write it: look at how the peer send/recv
verbs accept a `Value`, how `ServiceEvent::{Malformed,Rejected}` are produced, and how the serve loop
decodes an inbound frame into the op's request type. **If you cannot find an honest way to put raw bytes on
the wire, STOP and report that** — do not fake it by calling the decoder directly and presenting it as a
wire result.

## Rooms (start here, but ground your own path)

- `wat/service.wat` — the serve loop's inbound decode + dispatch arms.
- `src/runtime.rs` — `ServiceEvent::Malformed` / `Rejected` construction (grep them; they name what the
  decode *can* reject with today).
- `src/edn_shim.rs` — `edn_to_value` (STRICT) vs `edn_to_value_foreign`; `reconstruct_record` walks a
  declared field schema and hard-errors on a *missing* declared key while silently ignoring extras. Whether
  it also checks each field's *type* is precisely the question.
- `wat-tests/service-parametric-two-params.wat` — the stand-up/`connect'`/round-trip shape to copy.

## Blast radius

A probe and a report. **No production code changes.** If the probe needs a scratch `.wat`, it goes in
`wat-scripts/scratch-pad/` (loader-gated — must be GREEN). Do NOT commit.

## STOP triggers

1. Tier 1 says NOT enforced → STOP, report. That is the campaign-changing finding.
2. No honest way to put raw bytes on the wire → STOP, report; do not substitute a direct decoder call.
3. Any temptation to *fix* what you find → STOP. This strike measures.

## Gate

- A definitive Tier-1 answer with the evidence that produced it (the exact payload sent, the exact
  observed result — a structured error, or the handler receiving a mistyped value).
- Tier 2 grounding with `file:line`, if Tier 1 was ENFORCED.
- `cargo nextest run --release` unchanged at the floor: **4173 passed, 314 skipped** (you are adding no
  production code; confirm you broke nothing).
- FOREGROUND only. **Do NOT commit.**

## Your report

The Tier-1 verdict and how you proved it (payload in, result out — quote both). The Tier-2 mechanism with
citations. Whether a bound `K` could reach that mechanism. Anything you could not verify — and say so
plainly rather than inferring.
