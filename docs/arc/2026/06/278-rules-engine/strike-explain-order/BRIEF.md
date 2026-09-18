# BRIEF — F1: sort the oracle's node walk, and make the raw walk unwritable

Cure **and** gate in one strike. **Floor GREEN when you are done.**

## Read in order

1. **`DESIGN.md` beside this file** — it pins the contract (one shared verb) and lists the five sites.
2. **`wat/rete/oracle/fire.wat:145-165`** — **the law and the mechanism, already written.** The
   `WHY sort` comment is the argument; `:159` is the code. Copy both.
3. **`wat/rete/oracle/explain.wat:10-49`** — `harvest-support`, the driven defect; and **`:53`**, the
   doc claiming it matches the native.
4. **`wat/rete/oracle/pass.wat:177`, `:224`, `:395`** and **`wat/rete/oracle/fire.wat:124`** — the
   four to classify.
5. **`../vigilia-2026-09-05/probes/probe_vig_explain_order.{rs,wat}`** — the probe. Re-driven at this
   HEAD by the orchestrator: native stable 8/8, oracle three distinct answers, 0/8 agreement.

## Implementation sketch

```wat
;; wat/rete/oracle/<wherever the shared oracle helpers live> — ONE definition
(:wat::core::defn :wat::rete::topological-node-ids
  [network <- :wat::core::PersistentMap] -> (:wat::core::Vector :- [:wat::core::i64])
  ;; WHY sort: compile mints ids left-to-right, so ascending id IS topological.
  ;; PersistentMap/keys is HAMT order — not that. (fire.wat's law, now in one place.)
  (:wat::core::sort (:wat::core::into (:wat::core::Vector :wat::core::i64)
                      (:wat::core::PersistentMap/keys network))))
```

Then `explain.wat:49` and `fire.wat:159` both call it, and each remaining site either calls it or
carries a rune saying why its walk is order-insensitive.

**`.wat` is include_str!'d into the binary — a `wat/` change needs a rebuild before it takes effect.**
Do not drive a `wat/` edit against a stale binary.

## The gates

**Gate A (load-bearing):** a lint over `wat/rete/oracle/**` — no `(:wat::core::PersistentMap/keys network)`
outside `topological-node-ids`, except per-site `;; rune:lint(<category>) — <reason>`. **Mutation-prove
it**: re-introduce a raw walk, confirm RED, restore. Follow `tests/lint/`'s existing shape and give it
a non-vacuity guard (the file list must not be silently empty).

**Gate B:** land the probe as `tests/rete/probe_arc278_explain_order.{rs,wat}`, ≥8 producing rules,
keeping the single-producer control. Assert native == oracle. Green after the cure.

## Blast radius

`wat/rete/oracle/**` + one lint + one test pair. **No `src/` change** — if you find yourself editing
`src/rete/`, the cure has been mis-shaped.

## STOP triggers

1. **If sorting any of the four other sites changes an existing test's result, STOP and report.**
   That means the site was order-sensitive AND something depended on the old order — a finding.
2. **If a site is order-insensitive, prove it in the rune's reason** (what the fold builds and why
   order cannot reach the result). "It looks fine" is not a reason.
3. **If Gate A cannot be mutation-proved RED, STOP** — an un-reddenable gate is the defect this arc
   keeps finding.
4. **On any RED elsewhere: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-left-idx-latch/` — cure and gate together, fixture unchanged from the probe that found
the defect, floor green, unrepresentability proven rather than asserted.
