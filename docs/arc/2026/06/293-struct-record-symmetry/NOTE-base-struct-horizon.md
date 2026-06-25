# NOTE — the base-struct horizon: one foundation, kind-as-a-capability-label

**Status: HORIZON / north-star (builder, 2026-06-26, pre-compaction).** Captured durably before the gap. NOT
the immediate next strike — the current work (293.2-parity, `/from-map`, surfaces-for-records) proceeds on the
3-variant model and *converges toward* this; the repr unification is a later, bigger strike.

## The builder's model (verbatim intent)

> *"it is best to have struct and record just be built on the same foundation and the 'struct-ness' vs
> 'record-ness' vs 'holographic-record-ness' are just labels on an underlying 'base-struct' or something… in
> my mind they really are just structs… and they control what you can do with their fields."*

**One underlying aggregate — a "base-struct" (named, typed, fixed fields). The three kinds are CAPABILITY
LABELS on that base, controlling what you may do with the fields:**

| label | = today's | capability the label grants/withholds |
|---|---|---|
| (bare) | **struct** | non-EDN, in-locus, holds resources; **may NOT cross the wire** |
| EDN-portable | **core record** | EDN-representable; **may cross the wire** |
| holographic | **holon record** | EDN + **VSA ops** (similarity/holographic encoding) |

Everything else — construction, named accessors, `/from-map`, structural surfaces, width subtyping — is the
**base-struct**, shared identically. Only the label differs. This is the **holder × surface** model taken to
the *value-representation* level: one repr, the label IS the holder (the nominal capability tag).

## Why it's the truest decomplection

Today there are THREE `Value` variants (`Value::Struct`, `Value::wat__Record`, `Value::wat__holon__Record`)
with parallel machineries. The base-struct collapses them to **ONE repr + a label** — the maximal "operate on
them uniformly," and it matches the builder's actual mental model ("they really are just structs"). It is the
endpoint the whole arc points at.

## THE CONSTRAINT THAT MUST SURVIVE (the one thing to get right)

R8 / 4b-i made EDN-portability a **categorical KIND wall**: *"a struct shall never cross the wire — by kind,
not by field-check."* That wall is load-bearing (the security spine — "shared memory becomes only values"; a
non-portable resource can NEVER leak across a boundary). Collapsing to base-struct + label is only correct
**if the label is as un-leakable as the variant was** — i.e. the label is a NOMINAL, immutable, type-level
property the checker enforces categorically (a bare-labeled base-struct can NEVER be assigned to a portable
slot, the same hard wall, just keyed on the label instead of the variant). **If the label degrades the wall
to a soft runtime/field check, the unification is WRONG** (it would reopen exactly the leak R8 annihilated).
So: base-struct + label is the ideal endpoint **iff the label preserves the categorical wire wall.** That is
the single design question to answer before this strike.

## Blast radius (why it's a later, bigger strike, not now)

Collapsing the 3 `Value` variants → 1 + label touches: serialization (EDN encode/decode keys off the variant
today), the `is_portable_type` wire gate, `closure_extract`, and every exhaustive `match` on the three
variants. High blast radius — hence "horizon," converged-toward, not bolted on mid-arc.

## Pairs
`STRIKE-4b-struct-state.md` (R8 / the wire-boundary law this must preserve) · `DESIGN.md` § "Out of scope —
unifying the Value reprs" (this NOTE is the builder's answer to that future-arc pointer: yes, eventually, with
the wall-preserving constraint) · `REALIZATIONS.md` R1 (row polymorphism + the nominal-holder fusion) ·
`feedback_uniform_operation_or_decomplect_is_catastrophic`.
