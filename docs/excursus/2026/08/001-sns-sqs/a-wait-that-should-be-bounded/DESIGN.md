# DESIGN — a wait that SHOULD be bounded (the classification)

Builder: *"do the 6 strings then the 258 recvs"* → and, on being shown the 258 are three different kinds:
*"let's draw it."*

⚠ **EXECUTOR CHANGED.** grok's credits are exhausted for two days, so this stone is struck by a
**spawned Opus subagent**, not by pulsare. Named because a brief that implies one tier and runs another
is a brief that lies about what ran.

**Drawn 2026-09-15. NOT STRUCK.** ⛔ **REPORT-ONLY. This stone changes no code.**

## WHY — `recv 0/259` is TRUE and is NOT a defect count

`every-waiter-can-bound-its-wait` (`e7816e543`) measured **whether a wait is bounded**. It did not — and
could not — measure **whether boundedness is WANTED**, because that is a question about intent, not form.

⛔ **Reading the census as a defect list would do harm.** Measured on the 24 **live** bare-`recv` sites
(`wat/` + the three service scripts; the 259 figure is the whole corpus, mostly tests and probes):

```
circuit.wat:1208  (recv (after PeerKind::thread (Milliseconds ms) :done))
```

⭐ `await-timer-ms` recvs **on a timer**. It is bounded *by construction* — the peer's whole job is to
fire. Bounding it would be putting a timeout on a timeout. **Three sites are this shape** (circuit, sqs,
sns-fanout).

⭐ And `:user::park-receive!` (`sqs.wat:2053`) takes a `:queue::Queue::Wait` parameter — it is the
**long-poll**, where blocking **is the feature** (`queue-long-poll`, struck). Bounding it would undo that
stone.

⭑ **Same shape as `UNKNOWN` being first-class one level up:** the instrument answered its own question
honestly, and the orchestrator was about to read it as answering a different one.

## ⛔ THE ONE CONTRACT DECISION

**Classify every live bare-`recv` site into exactly one of four classes, from the CODE, with evidence —
and produce the defect list as the residue.**

| class | meaning | is unbounded a defect? |
|---|---|---|
| **TIMER** | the peer is an `after` — it fires by construction | **No** |
| **PARK** | blocking is the feature (long-poll, an idle serve loop awaiting work) | **No** |
| **BOUND** | a handshake or request/reply that can hang on a slow or silent peer | **YES — the defect list** |
| **UNKNOWN** | cannot be decided from the code | **report as UNKNOWN**, never folded |

⛔ **Read the CODE, not the census rows.** The orchestrator's own 24-site list came from grepping census
output and reading `enc=` names, and **four rows showed the same `enc=:fanout::worker`** — so some may be
one function counted repeatedly, or distinct recvs inside it. **The census row is a pointer, not a fact.**
This is the mistake this session has made most often; the brief exists to prevent it once more.

## ⭐ Three controls with known answers

The classifier is validated by sites whose class is already established:

| site | must classify as | why it is known |
|---|---|---|
| `:fanout::await-timer-ms` (and its sqs/sns twins) | **TIMER** | its peer is literally `(after … Milliseconds ms …)` |
| `:user::park-receive!` (`sqs.wat:2053`) | **PARK** | takes `:queue::Queue::Wait`; `queue-long-poll` is struck |
| `wat/service.wat` `:user::main` (child-main) | **BOUND** | ⭐ and **already fixed** (`7686bea24`) — it must appear as BOUND *and* as done, which also proves the classifier is reading current code |

⛔ **If any control misclassifies, the run is void** — say so and stop. A classifier that cannot sort the
three known sites cannot be trusted on the rest.

## Out of scope = REJECTED

- **Bounding anything.** Report-only. The defect list is an input to the builder's ruling.
- **The ~235 non-live sites** (tests, probes, scratch-pad, docs). A probe that blocks forever is a bad
  probe, not a shipped defect. ⚠ Report the count so the live/total ratio is visible, and stop there.
- **`send` (215), `readln` (161), `accept` (8), `poll` (14 UNKNOWN).** Same invariant, different
  primitives, and the same intent question applies to each. Not this stone.

## Blast radius

```
docs/…/a-wait-that-should-be-bounded/FINDING-*.md   the classification
everything else                                     0. REPORT-ONLY.
```

## Trap-doors named up front

1. ⛔⛔ **Mis-sorting a PARK as a BOUND would break a struck feature.** `queue-long-poll` exists so a
   worker stops burning a round trip; bounding its park undoes it. When in doubt → **UNKNOWN**, never
   BOUND.
2. ⛔ **A TIMER recv is not "already correct by luck."** It is correct *by construction*, and the
   distinction matters: a later stone must not "fix" it.
3. ⚠ **An idle serve loop is a PARK.** A service waiting for its next request must block indefinitely;
   that is what a service is. Only a wait for a **specific expected reply** is a BOUND candidate.
4. **The 24 is the orchestrator's count, from census rows.** Re-derive it from the code and report the
   real number **with the sites**; if it differs, the code wins.
5. **No `.wat` may be edited.** If a classification needs a probe to settle, write it under
   `wat-scripts/scratch-pad/` and say so — but prefer reading.
