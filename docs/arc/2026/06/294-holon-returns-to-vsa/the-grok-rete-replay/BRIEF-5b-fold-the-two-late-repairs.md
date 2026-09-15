# BRIEF 5b — fold batch 2's two late repairs into the steps that need them; widen the step gate

> Batch 2 replayed #61–#125 (SCORE-5). Its #125 checkpoint went red twice; both repairs are RIGHT, and
> both were committed AFTER #125 (`1c6d6af3e`, `00cc59ff4`). The 4b ruling (4-YES, SEAM § THE REPLAY):
> a composition defect found late FOLDS into the step that needs it — this branch becomes main, so no
> REPLAY commit may be knowingly red. Today #95–#125 (31 commits) are. Finding 20.

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Never use worktrees.

## Why — measured

1. **#95** (`7cb9994cb` → `6144dfc09`) adds `:wat::rete::keyword::{from-string,to-string}` — `RETE_OPS`
   Alias rows with a `TypeScheme` and no registry row. Two of main's in-crate walls go red from #95 on:
   `every_row_is_admitted` (`src/rete/vocabulary.rs`; `:wat::rete::keyword::` not in `RETE_MODULES`) and
   `registry_membership_gap_a_is_named_and_frozen` (`src/intrinsic/mod.rs`; the two names NEW to Gap A).
   `00cc59ff4` is the sanctioned repair — the gate's own message offers "add each to
   `REGISTRY_MEMBERSHIP_GAP_A`", and Gap A's header names `RETE_OPS` Alias rows as its population.
2. **#108** (`3f03b7d33` → `9313bbb83`) adds a four-space-indented list under `///` in `Ret`'s doc
   (`src/rete/vocabulary.rs`); main's floor runs `cargo test --doc` and grok-rete's did not, so rustdoc
   compiles it. `1c6d6af3e`'s ```` ```text ```` fence is the repair.
3. **Why the step gate missed both:** it runs the lint binary, the census, and stone 3's gate — none of
   which runs the library's own unit tests or the doctests.

## A — the fold (the 4b recipe)

1. `git tag replay-pre-fold-5 replay/grok-rete` (local), `git switch -c fold 6f4999462` — #94, the
   parent of #95 (verified: `-S` finds the converter rows first at #95, the indented doc list only at
   #108).
2. `git cherry-pick 6144dfc09` (#95), apply `00cc59ff4`'s diff (`git cherry-pick -n 00cc59ff4`),
   `git commit --amend` with a `Composition:` paragraph (main's registry ratchet + `RETE_MODULES` meet #95's
   rows; the sanctioned repair).
3. `git cherry-pick 6144dfc09..9313bbb83^` (#96–#107), `git cherry-pick 9313bbb83` (#108), apply
   `1c6d6af3e` (`-n`), amend with a `Composition:` paragraph (main's doctest floor meets #108's doc).
4. `git cherry-pick 9313bbb83..848a2bf6e` (#109–#125), then every commit after `00cc59ff4` on the old tip
   (SCORE-5 and this brief's commit). Drop the two FIX commits — their content now lives in #95/#108.
5. `git branch -f replay/grok-rete fold && git switch replay/grok-rete && git branch -d fold`.

**Its proof:** `git diff replay-pre-fold-5 HEAD` is EMPTY (the fold moves the repairs; it changes no
file at the tip), and the 65 `REPLAY(grok-rete #` subjects #61–#125 are unchanged. Then delete the tag.

## B — the step gate gains the two walls it could not see

From #126 on, BRIEF-1 § "One step" 3 adds, on every step with a `.rs` change:
- the library's unit tests: `cargo nextest run --release -E 'kind(lib)'` (1471 tests, 16 s measured
  on a warm build at `07489fc1e`);
- the doctests: `cargo test --doc --release` (2 s warm).
Well under a minute a step — cheaper than one fold, which is what their absence cost here.
A red is STOP-11: an in-crate wall or a doctest red at the step that caused it.

## The bar

- A's proof; the floor at the new tip GREEN (5540/5540), clippy 0.
- B written into BRIEF-1 § "One step" and its STOP list.

## STOP triggers — rejections; report verbatim

- **STOP-1:** a cherry-pick conflicts, or A's diff is not empty.
- **STOP-2:** a floor is red — paste the whole block, do not re-run.

## Tier

Commit on green. **Do not push.** Yield with `SCORE-5b.md` and `REPLAY-LOG.md` (new hashes #95–#125).
