# FINDING — the publisher's stats join may outlive the default deadline

**Filed 2026-09-13** by `the-inert-clause-is-refused` (D4-b). Not fixed here.

## The deleted comment

`:fanout::publisher` (`wat-scripts/fanout/circuit.wat`, the `:deadline-ms 120000` line that
used to sit under this comment) said:

> *"stats is the join: it must outlive one publisher's share (~20 s at N=2000). Default 10000
> would TimedOut mid-run and the parent would see a hang-shaped failure."*

## Why the line is gone, and why the worry is not

D4's two probes measured that `:deadline-ms` in `:satisfies` mode governs **nothing** — not
the caller's wait, not the service's own outbound calls. Both directions: **10000 ms observed
against 300 declared.** Deleting `120000` is behaviour-preserving by construction.

The comment's feared failure is **latent, not active.** The happy path at n=2000 completes
with `distinct=8000;dup=0` on every run this session, so `stats` answers inside 10 000 ms at
our sizes. The join may simply be cheaper than the author feared, or the ~20 s figure may be
wrong. **Neither has been measured.**

## What this stone must not do

Converting the publisher's caller to `call-by-deadline 120000` would be the *intended*
behaviour change, and it owes its own measurement (*does `stats` ever approach 10 s?*). Pairing
that with the refusal would make one stone answer two questions and pin neither.

The caller-side deadline is real (`call-by-deadline`). If a later stone finds the join
approaching 10 s, that is the tool — not a `:deadline-ms` clause on the publisher.

## Provenance

- Deleted from: `wat-scripts/fanout/circuit.wat` (was `:deadline-ms 120000` on
  `:fanout::publisher`).
- Probes: `wat-scripts/scratch-pad/probe-deadline-ms-is-callee-derived.wat` (probe 1,
  committed `004457a5d`); `wat-scripts/scratch-pad/probe-deadline-ms-governs-nothing-outbound.wat`
  (probe 2).
- Kin: `NOTE-a-callee-deadline-ms-is-inert.md` (probe 1 only; this finding is the outbound
  half plus the corpus deletion).
