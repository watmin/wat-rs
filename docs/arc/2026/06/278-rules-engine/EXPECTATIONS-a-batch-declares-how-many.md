# EXPECTATIONS — a batch declares how many

Written before the strike. Graded by my own re-run of every row.

⛔ = GATE: an invariant this stone controls. ▪ = REPORT: an observation, recorded not gated.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ★★ **the cap rejects, and nothing is enqueued** | scratch probe, 11 bodies into a cap-10 `send` | `RequestTooManyEntries{11,10}` **and queue depth unchanged** |
| 2 | ★★ **it fires at BOTH loci** | the same probe at `:wat::spawn::thread` and `:wat::spawn::process` | identical outcome at both — the byte guard is `peer-wire?`-gated; **this one must not be** |
| 3 | ★★ **10 still passes** | the same probe, 10 bodies | `Ok`, depth +10. A cap that rejects its own boundary is off by one |
| 4 | ⛔ **declaring the option without the variant is a COMPILE ERROR** | a case declaring `:max-entries` on a response lacking `RequestTooManyEntries` | fails at check time, **naming the missing variant**. Not a silent skip, not a fallback to `RequestTooLarge` |
| 5 | ⛔ **absent means uncapped** | a feature with no `:max-entries` | no guard emitted; behaviour byte-for-byte as today. **There is no default cap** |
| 6 | ⛔ **both emission sites guard** | read `wat/service.wat` at both | the same guard at each; a cap that depends on how you dialled is a failed strike |
| 7 | ⛔ **the unknown-option diagnostic learned the name** | declare a bogus option; read the error | it lists `:max-entries` among the recognized options |
| 8 | ⛔ **one adopter only** | `grep -rn "max-entries" --include=*.wat .` | `sqs.wat` `send` only. **No `Store::put/delete`, no `ack`, no `Seen::mark`** |
| 9 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ the count is reported, not gated |
| 10 | ⛔ **delivery still exact** | `circuit.wat` ×5 | `total=8000; distinct=8000` every run |
| 11 | **blast radius** | `git status --porcelain` | `src/types/surface.rs`, `wat/service.wat`, `sqs.wat`, probes/tests, the SCORE. Nothing else |

## REPORTS — recorded, not gated

| ▪ | what | why it is not a gate |
|---|---|---|
| a | `publish + drain` median ×5 | the circuit sends ≤ 4 bodies today, so the guard should never fire and should cost ~nothing — but a band is not an invariant |
| b | the per-op cost of the count guard, if measurable | `count` on a vector is cheap; worth a number, not a threshold |
| c | `dup` per run | `distinct` is the invariant |

## RUNTIME

45–75 min. The Rust parse is small with an exact exemplar; the macro work is the real content —
two emission sites and two derivations (the field accessor and the response ctor).

## TRAP DOORS

- ⚠⚠ **The cap must NOT be behind `peer-wire?`.** The byte guard is, because measuring bytes costs
  an `edn::write` that a thread-tier call should never pay. Counting a vector costs nothing, and a
  contract that only binds across a wire is not a contract. Row 2 exists for exactly this, and
  copying the exemplar too faithfully is the way to get it wrong.
- ⚠ **Off-by-one at the boundary.** SQS's limit of 10 means 10 is legal. Row 3 is the guard.
- ⚠ **`:max-request-bytes` has a default (512 KiB); `:max-entries` must not.** Copying the
  defaulting logic at `surface.rs:504` would silently cap every batch surface in the tree at
  whatever number was chosen — including the ones this stone deliberately does not adopt.
- ⚠ **Two emission sites.** The grep that finds one will find both only if it searches for the
  derivation, not the guard text.
- ⚠ **A compile error is easy to make un-triggerable.** Row 4 fails just as hard if the error
  cannot be provoked at all as if it never fires — the test must show it firing.
