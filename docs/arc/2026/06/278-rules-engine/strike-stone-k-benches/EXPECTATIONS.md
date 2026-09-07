# EXPECTATIONS — Stone K moves 2–4, written BEFORE the strike

## Scorecard

| what | command | expected |
|---|---|---|
| the bench builds and runs | `cargo bench --bench binding_repr` | runs, prints the three diagnostics' output |
| ⭐ the floor's SKIPPED falls by exactly 3 | `scripts/floor.sh` | **`5480 run / 19 skipped`**, 0 fail |
| the two live gates survive | `cargo nextest run --release -E 'test(bind_key_construction) or test(binding_cardinality_distribution)'` | 2 passed |
| the three are gone from the test binary | `cargo nextest list \| grep -c binding_key_cost` | 0 |
| the reasons survived | `grep -c '2028.8' benches/binding_repr.rs` | ≥1 — the measured margin is still on the record |
| ward-rune gate | `cargo nextest run --release -E 'test(no_unknown_ward_rune)'` | green, whichever form the runes took |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 — note `--all-targets` covers `benches/` |

## Runtime prediction

30–50 min. The move is mechanical; splitting the shared helpers between the two homes is the part
that needs judgement.

## Trap doors, named in advance

- **⛔ An item-level move that drops the module header.** The file's `:1-5` header carries
  `partire`'s finding — the very evidence that makes this move legal. Moving five functions and
  leaving the header behind loses it. Read the whole file.
  `[[an-item-level-move-drops-what-is-not-an-item]]`
- **The reasons deleted with the `#[ignore]`.** Once there is no attribute, the rune has nowhere to
  sit — and the measured margins are the only record of why these are not gates. Losing them
  invites exactly the "fix it into an assertion" that reddened this floor before.
- **`clippy --all-targets` newly covers a file that was never linted as a bench.** Expect warnings
  the test harness tolerated; fix them rather than allowing them.
- **A silently duplicated helper.** If both homes need one, that is STOP-2, not a copy-paste.
- **The skipped count moving by something other than 3.** That is the whole structural claim of
  this stone; a different delta means an unrelated test changed state in the same commit.
