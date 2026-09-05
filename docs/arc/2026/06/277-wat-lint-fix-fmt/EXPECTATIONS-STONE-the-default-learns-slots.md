# EXPECTATIONS — STONE: the default learns slots

| # | what | expected |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★★★ **THE RET-SPEC IS ONE LINE** | `foldl-bare.wat`'s inner `fn` renders `-> :wat::core::i64` **on one line** — the builder's non-negotiable |
| 3 | ★ the Slot set is non-empty and PRINTED | a probe prints how many Slots were built. **A green over zero Slots proves nothing** |
| 4 | ★ `fn`'s slot is right | `Slot{head: ":wat::core::fn", glued: 3}` present |
| 5 | ★★ **the REFUSAL fires** | a synthetic grammar with a variadic before the arrow yields **no** Slot; shown, not asserted |
| 6 | `let` is unaffected | no arrow in its grammar → no Slot → `let-two.wat` byte-identical to before |
| 7 | every fixture idempotent | `IDEMPOTENT=true` across all fixtures |
| 8 | existing ruled shapes hold | `defn-multi`, `defn-empty`, `let-two`, `half-broken`, `all-four`, `claim-demo`, `assoc-ride`, `let-complex`, `unruled-*` |
| 9 | the three walls stand | disagreeing-kind sabotage raises; `ClaimedUnder` 0; `col` 0 in every rule file |
| 10 | `grep.wat` untouched | `git diff wat/grep.wat` **EMPTY** |
| 11 | comments survive | `run.wat` on `wat/io.wat` → **COMMENTS=28**, count printed |
| 12 | wat-scripts load | `every_wat_scripts_file_loads` 1 passed |
| 13 | floor (ORCHESTRATOR) | 5179+ run, **0 FAILED** |
| 14 | clippy (ORCHESTRATOR) | 0 |

**Runtime prediction:** 40-70 min. The grammar walk is proven; wiring `Slot` into R11 is the work.

## Trap-doors named in advance

- **Row 3 exists because a silent-empty join is this stone's signature failure.** If the head
  spelling is wrong, zero Slots are built, R11 behaves exactly as it does today, and rows 6-8 all
  pass. **Only row 2 and a printed Slot count distinguish "working" from "wired to nothing."**
- **Row 2 is the ruling.** No amount of green elsewhere substitutes for it.
- **Row 5 is a refusal shown firing.** Every wall in this campaign was sabotage-proven; a refusal
  asserted but never triggered is a comment.
- **The lazy-stream trap:** `filter` returns a stream and `length` on a stream raises. `into` a
  Vector first — `277-can-wat-read-its-own-grammar.wat` already does.
- **The driver-loading trap:** a file in `rules/` is not loaded until a driver `load-file!`s it.
- **The vacuous green:** row 11 prints the comment COUNT.
