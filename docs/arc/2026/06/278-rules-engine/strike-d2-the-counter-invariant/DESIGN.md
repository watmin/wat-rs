# DESIGN — D2's own row named the instrument nobody built

## Why

**D2 is the last row on the 2026-08-30 vigilia.** It stands as a bounded negative: *"the code
asymmetry is REAL; no constructed input reaches it. LATENT, not live."* Its closing line names what
was missing:

> *"nothing here inspected `right_idx_n` directly. A Rust-level probe that reads the counter after
> each round…"*

**That probe was never built.** Both drives that "found nothing" were **end-to-end** — native vs
oracle fact counts, and a query whose rows mirror the join chain. The row itself records that
`seen_insert` dedups the facts. **An end-to-end differential is blind to a doubled bucket by
construction** — C16's exact shape, and C16 was blind to a live fact-drop for the same reason.

## Re-driven at HEAD `974e0d859` — the row's evidence is FALSE, and the defect is worse than stated

The row's proof was: *"`hash_join_delta` has ZERO mentions of `right_idx_n`; it is not even a
parameter."*

**Today `right_idx_n` IS a parameter** — `pass/filter_after_join.rs:24`, `pass/join_after_filter.rs:38`,
threaded through `pass/mod.rs:81` as `indexed_n`. `partire`'s split (`584130360`, `0cb16a818`) gave
both passes the counter.

**They still do not bump it.** Three sites append to the right index; one maintains the counter:

| site | appends `right_idx` | maintains `indexed_n` |
|---|---|---|
| `fire/mod.rs:802` | ✅ | ✅ reads `already` `:799`, writes back `:815` |
| `pass/hash_join.rs:185` | ✅ | ❌ |
| `pass/hash_join.rs:298` | ✅ | ❌ |

**This moved the defect DOWN the extirpare ladder while looking like plumbing.** A missing parameter
is a compiler-visible gap; an unused `&mut` is a convention. The refactor made it easier to violate
and harder to notice.

## The contract decision, pinned

**Assert the counter invariant directly, and prove the unmaintaining sites are REACHED.**

- The invariant: for every join id `J`, `indexed_n[J] == right_idx[J].len()` after each round.
  Anything else means an append bypassed the maintainer.
- ★ **Non-vacuity is the strike.** If `hash_join.rs:185`/`:298` never fire in the probe's workload,
  the invariant holds trivially and proves nothing — the same green-over-nothing this arc has found
  in C9's corpus, C16's filter, C14's counter and the `assert!(!ok)` idiom. **The probe must
  demonstrate both sites executed**, by census or by instrumentation, before its verdict counts.
- **Either outcome closes the row honestly.** Invariant breaks → **D2 is LIVE** and the negative was
  wrong. Invariant holds with both sites proven reached → **D2 is a bounded negative WITH an
  instrument**, which is what a bounded negative is supposed to have.

## Out of scope = REJECTED

- **Another end-to-end differential.** Structurally blind here; that is why two drives found nothing.
- **Fixing the asymmetry.** If the invariant holds, there is nothing to fix. If it breaks, the fix is
  a separate strike drawn on evidence.
- **Threading the counter into the two sites "for safety".** That is a hot-path edit on an unproven
  premise — the exact trade C10 forbids.
