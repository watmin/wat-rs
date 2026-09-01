# NOTE — every `partire` range contains a neighbour. Six modules, six intruders.

> Measured 2026-09-01 across four shipped decomposition stones and two more sized.
> **No ruling.** This corrects the seam map for every remaining stone.

## The count

`partire` returns ranges. A range is a contiguous line span over a file whose functions are **not**
laid out by concern — so every range picks up whatever happens to sit inside it. Six for six:

```
check.rs cast   `restricted_call` 1343-1613   contained  is_atomizable            (1516-1612)
                  → holon bind/bundle predicate. Caught by the orchestrator; range corrected to 1343-1515.
runtime cast    `numeric_tower`               contained  dispatch_rete_op
                  → the dispatch spine. FLAGGED BY PARTIRE ITSELF, by name.
                  `declarations` 789-4496      contained  eval_tail                (4497)
                  → ONE PAST the end. The orchestrator's own off-by-one, not partire's.
                  `reflection`   8096-10646    contained  require_bundle           (9486)
                  → a holon helper. PARTIRE OMITTED IT CORRECTLY; the orchestrator's
                    line-range enumeration pulled it in.
                  `defclause_dispatch`         bounded by eval_let / bind_let_binding / eval_do
                  → the eval spine, adjacent by line only. Fenced by STOP before it fired.
                  `peer_protocol` (3 ranges)   FUSES THREE CONCERNS — see below.
                  `stepper`      14674-16054   contains  effectful_by_prefix · is_effectful_op
                  → the PURITY classifier (used by src/rete/purity.rs, src/intrinsic/mod.rs).
```

★ **The cast is not unreliable — its LISTS were right every time.** Twice partire named the intruder
itself (`dispatch_rete_op`) or correctly omitted it (`require_bundle`), and the orchestrator's
line-range reasoning put it back. **The defect is in treating a range as the unit of work.**

## ⛔ The one case where the RANGE ITSELF is over-broad: `peer_protocol`

`partire` fused three non-contiguous ranges into one module, justified by: *"the middle range's
outcome builders are called ONLY from the third range's `eval_peer_*` verbs (confirmed by reverse
grep)."* **That evidence covers the outcome builders and nothing else.** Enumerated from disk, the
middle range holds three concerns:

```
12801-13560   panic payload · fault values · died-error chains      33 items, 759 lines
              callers include src/intrinsic/kernel/error.rs         → KERNEL ERROR reporting
13600-14088   recv/send/try_send/close/signal/accept/connect_outcome  ~28 items
              callers: eval_peer_* AND src/kernel/address.rs        → genuinely peer/comms
14088-14400   runtime_error_to_eval_error_value · wrap_as_eval_result ·
              form_outcome · check_failed_cause                     → EVAL-ERROR machinery
              `wrap_as_eval_result` is called from src/intrinsic/holon/atom.rs — not peer at all
```

⚠ **A stone that took `peer_protocol` as cast would move kernel error reporting and holon-adjacent
eval-error machinery into a comms home.** The fusion argument was sound for the population it
measured and was then applied to a wider range than it covered.
`[[feedback_a_claims_support_does_not_travel_with_the_claim]]`

## The standing rule this produces

> **Move by the function LIST. Audit what a range CONTAINS, never where it ends.**

Already the standing brief instruction; this NOTE is why it is not merely prudent. And the corollary
earned here: **when a cast fuses non-contiguous ranges on a caller argument, check that the argument
covers every range** — not just the one it was measured on.

⬜ **`peer_protocol` is NOT re-cast here.** Whoever takes it should re-cast `partire` against the
current file, or cut only the well-attested part (range 1 + the outcome builders + the `eval_peer_*`
verbs) and leave the fault and eval-error clusters for their own stones. `src/kernel/` remains the
right home for the comms part — `connect_outcome_connected` is already called from
`src/kernel/address.rs`.
