# EXPECTATIONS — the store says what it deleted

**Written BEFORE the strike**, from `9a6ee5b76`, so the result cannot move the goalposts.

## ⛔ THE ROW THAT MATTERS MOST, AND WHY THE FLOOR CANNOT GIVE IT

Every match arm in this migration binds `_`. **Nothing in the corpus reads `deleted` when this stone
lands.** So a green floor is compatible with the count being right, always-`1`, always-`0`, or
base+GSI. **Row 5 is the entire proof of this stone**, and it exists only because it was written here
first. This is the third stone in a row where the default gate measures something else; say so in the
grading rather than letting green stand in for proven.

## The scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the variant carries the field | `grep -n 'deleted <- :wat::core::i64' wat/query.wat` | **1** hit, inside `DeleteResponse` at `:534` |
| 2 | no nullary `DeleteResponse::Success` left | census of `DeleteResponse::Success` heads with 0 binders | **0** in arm-head position, **0** in value position |
| 3 | the count is threaded, not invented | `grep -n 'Result :- \[:wat::core::i64 :wat::sqlite::Error\]' wat/query/sqlite-store.wat` | `delete-one-key` **and** `delete-rows` both widened |
| 4 | `delete-rows` ACCUMULATES | read the recursion | `n +` the recursive call — **not** the last key's count |
| 5 | ⭑⭑ **the number is REAL, both backends** | scratch probe: delete 3 keys, 2 exist | **`deleted = 2` on mem AND on sqlite** |
| 6 | the GSI clear is not summed | same probe, on a table **with** indexes | still **2** — not `2 + 2·|indexes|` |
| 7 | a missing key contributes 0 | same probe | delete 1 key that does not exist → **`deleted = 0`**, variant still `Success` |
| 8 | the codemod is recorded | `ls wat-scripts/fixes/` | a new `.wat` migration committed, idempotent on a second run |
| 9 | codemod touched arms only | `git diff` the 3 construction sites | `mem.wat:677`, `sqlite-store.wat:91`, `probe-entry-three-of-ten.wat:137` changed as **logic**, never as `_` |
| 10 | all four `.wat` homes migrated | `git diff --name-only` | files under `wat/`, `wat-scripts/`, `tests/`, `docs/` all present |
| 11 | integrity — nothing else moved | strip the intended change, compare to HEAD | every other byte identical |
| 12 | floor | `./scripts/floor.sh` → **Summary line** | **5237 passed, 0 FAIL**, no `ARM.txt` |
| 13 | tests compile | `cargo nextest run --release --no-run` | clean — the build alone does not compile them |
| 14 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | exit **0** |
| 15 | happy path | `circuit.wat 2000 4 3 8192 true 1000` | `distinct=8000;dup=0` |
| 16 | chaos | `… 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` | exit 0 |
| 17 | no Rust change | `git diff --stat -- src/` | **empty** — the count already exists in Rust |
| 18 | ⚠ `rt-store` does NOT improve | happy-path `rt-store` vs ~6628 | **unchanged** — and that is CORRECT, see below |

## ⚠ Row 18 is a null result I am committing to in advance

This stone lands the *protocol*, not the optimisation. `PATCH-measure-variant.diff`'s **−17.1 %
`rt-store` / −60.9 % `count`** is explicitly out of scope (DESIGN §Out of scope), because it also needs
an `:Unknown` state for the unknowable-put path — a builder decision. **If `rt-store` moves at all,
something unintended happened and I want to know.** A strike that reports the win as landed here has
misread the scope.

## Runtime prediction

**35–55 minutes.** Three stdlib files of real logic (small), one codemod (the long pole — a new
structural rule, piloted on a `/tmp` copy first), nine mechanical arms, one differential probe. Floor
~490–520 s on a quiet box; clippy ~2–4 min.

## Trap-doors, ranked by how likely they are to bite

1. ⛔ **The GSI clear summed into `deleted`** (rows 5/6). `delete-one-key` runs the base DELETE *then*
   `clear-index-projections`; only the first count is the answer. This is the one failure that produces
   a plausible wrong number.
2. ⛔ **`delete-rows` returning the last key's count** instead of the sum (row 4). Silent for
   single-key batches — which is most of the corpus — and wrong exactly where the queue cares.
3. **The codemod rewriting a construction as `_`** (row 9). `probe-entry-three-of-ten.wat` carries both
   forms, so a file allow-list cannot separate them; the rule must be arm-head position.
4. **`tests/**/*.wat` or `docs/**/*.wat` missing from the path list** (row 10). Cost this campaign a
   217-failure floor once.
5. **A piped exit code.** A type-error run this session reported `$?` = 0 through `| head`; its true
   exit was **3**, on stderr. Read the Summary line and the stderr, never `$?` after a pipe.
6. **The three `wat/` files must move together** — `wat/query.wat` is frozen into the binary at build
   time, so a partial edit refuses the rebuild rather than failing a test.

## What would make me reject a green result

- Row 5 not run, or run on only one backend.
- Row 18 showing an `rt-store` improvement (scope was misread).
- A golden patched instead of STOP-3 firing.
- A census quoted without a positive control — the pattern `DeleteResponse::Success` **cannot see the
  declaration**, and any count taken with it is a count of uses only.
