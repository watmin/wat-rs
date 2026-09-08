# EXPECTATIONS — STONE N

Written BEFORE the strike. Bars derived from the rule.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⛔ the toolchain RUNS | `./target/release/wat <any corpus .wat>; echo $?` UNPIPED | **not 3** |
| 2 | ⛔ fixture-local errors 0 | per worklist path, count errors whose `:file` is that path | **0** |
| 3 | the bare spelling is UNREPRESENTABLE | `(:wat::core::Some 1)` and `:wat::core::None` in a typed slot | REFUSED, message names the qualified form (STOP-3, STOP-6) |
| 4 | the legacy `:None` too | a bare `:None` form | REFUSED |
| 5 | corpus is clean of the heresy | `grep -rho ":wat::core::Some\b" --include=*.wat . \| wc -l` etc. | **0** for all four; `:None` 0 |
| 6 | the 118 repaired, not unwrapped | `grep -c "unwrap-alias" <the codemod run log>` | not run (STOP-2) |
| 7 | the table was DERIVED | read the codemod's gensym→FQDN construction | derived from declarations, not a literal list (STOP-4) |
| 8 | idempotent | re-run both codemods over the same paths | **0 further changes** — row 5 of M2 went unmeasured; it is mandatory here |
| 9 | the five probe rows | `cargo nextest run --release -E 'test(probe_arc296_enum_map_ctor)'` | 5 passed, `#[ignore]` = 0 |
| 10 | the floor | **ORCHESTRATOR**, `^ +Summary` UNPIPED | see below |
| 11 | clippy | `cargo clippy --release --all-targets --workspace` | 0 errors |

## ⚠ ROW 10 DECIDES THE STONE — AND THE TIMEOUTS ARE THE TELL

At M the floor was `2447 passed / 2773 failed / 18 timed out`. **N must return the full band with
0 failed AND 0 timed out.** The 18 timeouts were spawn-tests whose child died at startup; nothing
but a working toolchain clears them, so a floor with timeouts remaining means the migration is not
finished no matter what the fail count says.

★ Expected residue, and ONLY this: **goldens that pin a stdlib `wat/*.wat` line.** The migration
moves lines in `service.wat` and friends;
`109/NOTE-a-golden-that-pins-a-stdlib-line.md` rules it — RECAPTURE, KEEP PINNING, do NOT extend the
normaliser. **Anything else is a finding**: name it with the arm and the verbatim block.

## RUNTIME PREDICTION

120-180 min. The wall is small and surgical; the corpus rename is large but mechanical; the wrap
already exists and needs its table derived rather than extended by hand. Two release builds for the
dance, plus one for the wall.

## TRAP DOORS, NAMED

- **The screams will not match the DESIGN's grep numbers, and the screams are right.** If they come
  back close to 2,039 + 31, be suspicious rather than relieved — a form-tree census normally finds
  MORE than a text one, especially across 32 macro-generated enums.
- **Patterns as well as constructions.** `[:wat::core::Some {:value v} …]` is a match arm head and
  carries the bare spelling too. A rename that moves only constructions leaves every arm broken.
- **Order is load-bearing**: rename, THEN wrap. Reversed, the wrap hits two-segment heads and
  reproduces the exact 118-site regression this stone exists to repair.
- **`wat/fix.wat` rewrites its own source** (67 sites at last count). Safe during the run — the
  executing copy is frozen into the binary — but if the post-dance build fails, suspect it first.
- **Partial success is failure.** The tree does not RUN until the last site moves; there is no
  useful intermediate state and no progress meter to read.
