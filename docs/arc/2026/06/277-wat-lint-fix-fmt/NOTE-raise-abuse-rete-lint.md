# NOTE — the RAISE-ABUSE lint (a rete-rule lint; the 278→277 bridge) (2026-07-23)

**Origin:** arc 278 R57 `IGNORANTIAM DELEMVS, NON LABOREM TIMEMVS`. Chasing the self-scheduling stone,
`send'` on a gone peer was found *raising* a reason-free `"channel disconnected"` — a raise that unwinds
past the reader, masking the failure. `recv'` was already walled (R53 `RecvOutcome`); `send'` never was —
the LAW's "complete" (R55) was half. The builder: *"can we enact a lint via rete rules to identify these
heretics?"* — which is arc 277's whole purpose (the self-hosted `wat-lint`) meeting arc 278 R21's thesis
(*"we need rete for writing lints; lints are rete rules"*). This note tracks it.

## What the lint identifies

A **raise-abuse**: a raise (`RuntimeError`/`MalformedForm` in the runtime, or `raise!`/`assertion-failed!`
in wat) on an **expected runtime condition** — a peer/address/listener gone, a channel closed, a resource
dropped, an I/O that failed — that **should be a faced VALUE** (an outcome the caller matches), not a raise.

The line is sharp, and the lint must hold it:
- a raise on a **program error** (wrong type, arity, malformed syntax) is *correct* — a located crash, the
  substrate's job. **NOT a heretic.**
- a raise on an **expected condition** the caller could/should handle is the abuse. **Heretic.**

Grounded surface (2026-07-23): of ~1727 `RuntimeError` sites, only ~a dozen name an expected condition, and
they cluster in the **peer/IO verb family** — `listener'` / `connect'` / `accept'` / `send'` / `try-send'`
/ `recv'` / `close'` / `select'` / `poll'` — plus a few resource ops (`TempDir`/`TempFile`/pipe
already-dropped). `recv'` is walled; `send'` is being walled (arc 278, `DESIGN-send-outcome-wall.md`); the
rest are the candidate heretics.

## Discovery vs enforcement — the lint is DISCOVERY; the checker wall is the KILL

Keep these separate — they are different weapons, and the lint must not pretend to be the stronger one:
- **Enforcement = the outcome wall + the checker force.** Once a verb returns an outcome
  (`SendOutcome`/`RecvOutcome`) and the checker forces the match, an unfaced raise is a **compile error** —
  *unrepresentable*, not flagged. That is the annihilation. A lint is redundant (and weaker) for a walled
  verb.
- **Discovery = the rete lint.** Its value is FINDING the heretics — which raise sites are abuses vs legit
  — as a **re-runnable, queryable program** instead of a one-time manual audit. This is the programmable
  database ("police the pattern") on the substrate's own quality; the chaos engine (R25) turned inward.

So the lint's job is the **audit, made living**: assert the raise sites as facts, fire rules
(`heretic ⟸ peer/IO-verb ∧ raises-on-expected-condition ∧ ¬returns-an-outcome`), query the heretic list.
Its output drives the checker wall; the wall does the killing.

## The honest gap — the abuses live in RUST; rete lints WAT

`wat-lint`/`wat-fix` walk **wat** form-trees (`fix-source`, `deporder.wat`); the raise-abuses live in the
**Rust** `eval_*_prime` sites (`runtime.rs`). A native rete-over-wat lint sees wat, not Rust. Two ways
across, both real work:
- **Fact-extraction bridge (the north star):** a pass that scans the Rust raise sites and emits facts —
  `(raise-site :op :send' :reason-kind :disconnected :verb-family :peer :returns-outcome? false)` — into
  rete; the rules above deduce the heretics; query the list. This IS the "rete-as-datalog over code" build
  arc 278 points at, and it makes the audit a permanent, re-runnable guard. It is *infrastructure* (a
  Rust→facts scanner), not a quick lint.
- **Wat-surface lint (native, transitional):** rete over the wat corpus flags any call to a *known-raising*
  peer/IO verb in unfaced position. But once the walls + checker force land, those are compile errors — so
  its window is only the transition.

## The plan — set the tone in 278, then pivot here

1. **Arc 278 sets the tone** — the `send'` OUTCOME WALL establishes the *pattern* (a peer/IO verb returns
   `{Sent, Closed, Lost[cause]}`, the checker forces the match), and the **raise-abuse audit** establishes
   the *classification* (which verbs are heretics, the exact rule that distinguishes abuse from program
   error). Both are ground-truth the lint's rules need.
2. **Then pivot into arc 277** and build the real rete-lint — with the rules already written and
   ground-truthed by the audit (fold "the datalog rules that DEFINE a raise-abuse" into the audit's
   deliverable, so the lint's brain is banked before its eyes are wired). Decide then: the Rust-facts bridge
   (the durable guard) vs the wat-surface lint (the transitional one).

This is the full circle R21 named — *the linter, rete, and forms are all just data transforms* (278 R8) —
made concrete on the substrate's own honesty. Kin: 278 R21 (rete for lints), R53 (the recv' wall), R57
(`IGNORANTIAM DELEMVS` — the raise-abuse annihilation), `DESIGN-send-outcome-wall.md`; 277's existing lint
framework (`DESIGN.md`, the concat-abuse rule as the shape of a rule-that-carries-its-fix).
