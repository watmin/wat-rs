# EXPECTATIONS — STONE: the FORM is the authority for `->`

| # | what | expected |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★★★ **a GENERIC `fn`'s ret-spec** | `generic-fn.wat` renders **`-> :wat::core::i64` — BOTH TOKENS ON THE SAME LINE** |
| 3 | ★ a NON-generic `fn` | `foldl-bare.wat` — same, no regression |
| 4 | ★ `defmacro` | a fixture with a `defmacro` — ret-spec both tokens, same line |
| 5 | ★ `defn` | `defn-multi.wat` — `-> :wat::core::i64` same line (R1's own path, must not regress) |
| 6 | ★ `Slot`'s consumer count REPORTED | a printed count of rules joining `Slot`. **Expected 0. Do not delete it** |
| 7 | `defclause`'s nested arrow untouched | if a fixture exists, its shape is unchanged; otherwise say so |
| 8 | every fixture idempotent | `IDEMPOTENT=true` across all |
| 9 | ruled shapes hold | `let-two`, `half-broken`, `all-four`, `claim-demo`, `assoc-ride`, `type-ctor`, `type-nested`, `let-complex`, `unruled-*` |
| 10 | type applications unaffected | declaration one line; constructor glues + explodes |
| 11 | three walls stand | disagreeing-kind sabotage raises; `ClaimedUnder` 0; `col` 0 in every rule file |
| 12 | comments survive | `run.wat` on `wat/io.wat` → **COMMENTS=28**, count printed |
| 13 | wat-scripts load | `every_wat_scripts_file_loads` 1 passed |
| 14 | floor (ORCHESTRATOR) | 5179+ run, **0 FAILED** |
| 15 | clippy (ORCHESTRATOR) | 0 |

**Runtime prediction:** 20-35 min. One condition in one rule file.

## Trap-doors named in advance

- **Row 2's wording is deliberate: BOTH TOKENS, SAME LINE.** The previous stone's row said
  *"ret-spec on its own line"*, which `->` and the type each occupying a line satisfies — and the
  defect shipped. **A row that can be satisfied by the defect is not a row.**
- **Rows 3, 4 and 5 are the no-regression set.** `Slot` served `fn`, `rete::fn` and `defmacro`;
  `defn` has its own rule. All four paths must still produce a single-line ret-spec.
- **Row 6 is a REPORT, not an action.** A consumer count of 0 is expected and correct.
- **The `->` inside a defclause's `[-> :T]` is a VECTOR element, not a sibling in the enclosing
  form** — a sibling-index test should miss it. Confirm; do not assume.
- **The vacuous green:** row 12 prints the comment COUNT.
