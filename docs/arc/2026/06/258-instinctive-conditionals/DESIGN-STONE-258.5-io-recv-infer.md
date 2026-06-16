# DESIGN — Stone 258.5: `recv'` infers from the constraining consumer (the IO-cluster arrow, narrowed)

> Opened 2026-06-16. The IO-cluster step of the redundant-`-> :T` kill (NOTE-redundant-return-annotation-class
> § sequencing 2). Grounded against HEAD `67595322`. Pivoted into from arc-272 step-6a: building the
> capability handoff banged into `recv'`'s `-> :T` (a non-return arrow we refuse to propagate). 6a is
> PARKED on this stone.

## The narrowed problem (grounded — smaller than "kill the arrow everywhere")

`recv'` already has an **optional** `-> :T` ascription (arc-214 γ-1). The NOTE keeps it as a seed "only
for genuinely-ambiguous positions." So this is NOT "delete the ascription" — it is **"make `recv'`
infer its type wherever a consumer constrains it,"** leaving the seed for the genuinely-unconstrained.

- **Corpus surface:** `grep "recv'.*->" wat/ wat-tests/` → **ZERO**. All ~40 ascription uses are in
  `tests/` probes, and most are genuinely-ambiguous (the recv'd value exits to Rust for a `matches!`
  assert — no wat consumer to infer from). Those are **legitimate seeds**, not strip targets.
- **The real gap** (the 6a case + the class): `(connect' (recv' svc))` — `recv'` on a process handle
  returns `O` = a **fresh var** (`infer_process_prog_type`, check.rs:10827, fresh-by-design: the child's
  self-peer type is buried in its forms, unanalyzable at the spawn site). `connect'` then **rigid
  pattern-matches** its arg against `Address'<S,R>` (`infer_connect_prime`, check.rs:10485) and errors on
  anything that isn't a literal `Address'` parametric — a fresh `:?` can't match → `TypeMismatch`. So the
  inferable type **can't flow from the consumer**, and the ascription is the only source. That is the arrow.

## The decision (four-questions) — consumer-unify, not a checker rewrite

How does `recv'`'s type reach it without `-> :T`?

| approach | Obvious | Simple | Honest | UX | verdict |
|---|---|---|---|---|---|
| **consumer-unify** — rigid-matching consumers `unify` against `Expected<fresh,fresh>` so a fresh-var arg binds | YES (the consumer that knows the type binds the var) | YES (rigid-match → unify in the projection helpers; synthesis-compatible, no new mode) | YES (concrete arg still checks; wrong type still errors; seed kept for genuine-ambiguous) | YES (no ascription where context constrains) | **CHOSEN** |
| top-down expected-type push-down (new bidirectional mode) | — | **NO** (new checker architecture) | — | — | CUT — bigger than needed |
| handle carries its O (extract child self-peer type from forms) | **NO** (forms-buried, unanalyzable at spawn site) | NO | — | — | CUT — infeasible |

Consumer-unify is strictly safer than rigid-match: a concrete `Address'<i64,i64>` unifies (binds nothing
new), a fresh `:?` unifies (binds it to `Address'<s,r>` — the inference), a wrong concrete type (`i64`)
**still fails** unification → still a `TypeMismatch`. No precision lost.

## The fix

Replace the rigid `match … if head == "wat::kernel::Address'"` / `else TypeMismatch` shape with a
`unify(addr_ty, Address'<fresh,fresh>)` in `infer_connect_prime` (and the shared projection helpers the
other IO consumers use — `project_peer_io` etc.), so a fresh-var arg binds to the expected parametric.
`recv'`'s fresh `O` then flows from the consumer; the `-> :T` seed survives only where no consumer
constrains (value-exits-to-Rust / unconstrained return) — honest, per the NOTE.

## Gate probe (RED at HEAD) — check-level, isolates the inference

`tests/nursery/probe_arc258_recv_infers_from_consumer.rs`: a process child sends an `Address'` (or any
concrete type) over its self-peer; the parent does **`(connect' (recv' svc))` with NO ascription** and
the probe asserts **`startup_from_source` type-checks**. RED at HEAD: `connect'` rigid-matches the fresh
`recv'` result → `TypeMismatch` at check. GREEN once `connect'` unifies. (Check-level on purpose — isolates
the inference from arc-272 6a-i's separate `Address'`-EDN-decode runtime gap; the two compose to turn the
full 6a round-trip probe green.)

## Decomposition

- **258.5a — ✅ DONE** (`connect'` unifies its arg; the 6a unblock + the pattern). RED probe → GREEN;
  lib 919/36, nursery 896/4 (zero-new). `recv'` now infers `Address'` from the `connect'` consumer.
- **258.5b — DEFERRED (don't build the forcing function).** Generalize the unify-not-rigid-match pattern
  to other IO consumers only when one actually needs it. `connect'` is the sole consumer the arrow-kill
  has a caller for today (`send'` already unifies its payload; `accept'` takes a `Listener'`, not a
  `recv'` result). Build per-consumer as a real caller surfaces.
- **258.5c — OVERRIDDEN by 258.5b (2026-06-16).** The "genuine-ambiguous seed" concession is
  superseded. The `-> :T` ascription is FULLY KILLED — no seed, no optional form.
  **The EDN wire is self-describing** (post-234.7): records cross as `#wat.kernel/Foo {…}`,
  structs/enums tagged, scalars typed. `decode_trusted_wire(edn, sym.types())` reconstructs the
  exact `Value` from the wire's own tags + the type registry — no declared target type is needed.
  The `-> :T` branch was only doing coercion the self-describing wire makes redundant. The `tests/`
  uses that were "genuine-ambiguous seeds" (value-exits-to-Rust, e.g. i64 round-trip) are migrated
  to the no-ascription form — `decode_trusted_wire` with `sym.types()` handles scalar reconstruction.
  `recv'` is 1-arg only; `select'` likewise. `-> :T` on either form is now a hard checker error.
  Probe: `tests/probe_arc272_6c2_record_ipc_derisk.rs` (RED → GREEN on this kill).

Pairs the NOTE (redundant-`-> :T` class) + [[feedback_reach_stumble_is_the_signal]] +
[[feedback_deferred_dep_becomes_necessary_block_and_build]] (arc-272 6a blocks on this) +
[[feedback_optional_is_a_smell]] (resolved: the seed was NOT required — the wire already carries
the type; annihilated by construction rather than kept as an optional-is-a-smell).
