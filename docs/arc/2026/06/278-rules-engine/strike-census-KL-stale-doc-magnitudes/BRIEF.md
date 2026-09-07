# BRIEF — census K/L: amend two doc claims in `census.rs`

Read `DESIGN.md` first, including the note that **the audit's own L is half wrong** — you are
amending the audit as well as the code comments. **Comments only. No code outside a comment moves.**

## Read in order

1. `src/rete/kernel/census.rs:620-630` and `:884-890` — the two copies of the K claim. Both say the
   `alpha:*` marks fire PER FACT and name five children, three of them undersized.
2. `src/rete/kernel/fire/pass/alpha.rs:312` and `:421` — the only two `alpha:*` marks that exist,
   both `phase_end` at the end of a pass function. This is K's evidence.
3. `src/rete/kernel/census.rs:842-845` — the L claim, above `right_idx_appended`.
4. The three call sites and their `n` arguments — this is L's evidence, and note that **CATCHUP can
   emit zero**, which is where the audit is wrong:
   - `hash_join.rs:350` `STEP2` — `dr.iter().count()`, outside the loop
   - `hash_join.rs:216` `CATCHUP` — `n_all.saturating_sub(already)`
   - `fire/mod.rs:928` `MAINTAINER` — inside `if already < right_elements.len()`
5. `src/rete/kernel/tests/right_index_counter_invariant.rs:269`, `:300`, `:382`, `:388` — the
   consumer. `site_ran` is asked of CATCHUP and STEP2 only; the maintainer is read through `n > 0`.

## What to write

**K** — keep the 2026-08-01 measurement and mark it historical: it was taken when alpha had five
per-fact children, it found 26% of the reading was instrument, and **that finding is why they were
removed**. Then state today: two marks, once per pass. The column it justifies stays.

**L** — scope the claim to STEP2 and CATCHUP, both of which can be called with `n == 0`. Say that
MAINTAINER cannot, that its guard makes "ran and appended nothing" unreachable, and that no
consumer asks it.

## Verification

No mutation — these are comments; do not invent one. Report instead:

1. the `alpha:*` grep, showing two marks and their line numbers;
2. each site's `n` argument quoted, and which can be zero;
3. the floor Summary line.

## Blast radius

`src/rete/kernel/census.rs` only — three comment blocks (`:620-630`, `:842-845`, `:884-890`).
**No code. No instrument. No test.**

## STOP triggers

1. An `alpha:*` mark fires per fact → STOP; K's premise fails.
2. `MAINTAINER` can reach `n == 0` → STOP; L's premise fails and the audit was right.
3. Anything outside a comment changes → STOP.

## Prior result to copy for shape

`../strike-census-I-mixed-class/SCORE.md` — it reported a mild row as mild, without inflation.
Same here, and additionally: say plainly that the audit's L was partly wrong.
