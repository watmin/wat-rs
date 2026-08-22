# EXPECTATIONS — un-park the wall, and fix what it finds

Written BEFORE the strike, against `65b834811`. Floor at HEAD: 4866/4866.

## The scorecard

| # | what | my command | expected |
|---|---|---|---|
| 1★★ | the nine phantoms are gone | `--check` a one-line innocent program | EXIT 0, empty output |
| 2★★ | the wall is NOT blind | the uncalled-phantom fixture | EXIT 1, names `:user::NoSuchType` |
| 3★ | it names the TYPE, not the caller | the phantom-with-a-caller fixture | EXIT 1, **no `TypeMismatch` on parameter #1** |
| 4 | the five parked fixtures | scoped `binary_id(wat::resolve)` | all pass |
| 5 | the hand-list is gone | `grep -rn known_builtin_leaf_types src/` | no hits |
| 6 | the alias fix is structural | a parametric-surface-method probe | checks clean; alias carries the method's params |
| 7 | `symbol_table.rs` untouched | `git diff -- src/value/symbol_table.rs` | **empty** |
| 8 | the floor | `scripts/floor.sh` | **4866 + N new tests**, 0 FAIL |
| 9 | clippy | `-D warnings` | 0 |

**Row 8 is NOT "unchanged".** I made that mistake last stone: "the floor is unchanged" is not
well-formed for a stone that ships tests. The well-formed expectation is **no pre-existing test
changes outcome, and the count moves by exactly the number of tests added** — which I will verify by
counting `+ fn ` in the diff, not by accepting a number.

## The rows that can lie, and this stone has the worst pair yet

**Rows 1 and 2 are meaningless alone and decisive together.** Row 1 — "no unresolved output on a
clean program" — is satisfied perfectly by a wall that reports NOTHING. Deleting the sweep passes row
1. Excluding `*/Request`/`*/Response` aliases passes row 1. Adding the free variables to a bound set
they do not belong to passes row 1.

Row 2 is the only row that proves the wall can still fire. **A report that gives me row 1 without row
2's verbatim output is Mode B on its face**, however green everything else is. This is the
non-vacuity pairing, and it is the single thing I will look at first.

**Row 6 guards the same axis one level down.** Rows 1+2 prove the wall is honest; row 6 proves the
ALIAS is honest. A fix that made the nine names resolve by registering `D`, `I`, `O`, `W` as builtin
types would pass 1 and 2 and be a catastrophe — it would teach the registry that single capital
letters are types. STOP-1 forbids it; row 6 detects it.

## Independent prediction

**Runtime: 50-90 minutes.** Part A is mechanical (cherry-pick, delete a function, re-point one call).
Part B is small in lines and real in judgement: two sites, and the union hypothesis has to be checked
rather than applied.

**Trap-doors named in advance:**
- **Row 1 achieved by silencing.** The likeliest failure and the one STOP-1 exists for. It will not
  look like cheating from inside — it will look like "these synthesized aliases aren't user code, so
  they shouldn't be swept."
- **The union is wrong at one of the two sites.** `runtime.rs:2018` mints variant constructors for a
  synthesized enum, not aliases for a surface; its scope may legitimately differ. STOP-2 covers it,
  and I expect this to be the one that fires if any does.
- **A fourth failure class appears.** Once the nine are fixed and the builtin names resolve, whatever
  is left is new information. STOP-4 says report before touching. That list is worth more than the
  stone.
- **`no_loose_string_assert` fires again.** Third time today, and the brief warns explicitly with the
  cure (ask through the door; do not add a rune that would lie about the site).

## Mode

- **Mode A** — rows 1 AND 2 verbatim and together, the union hypothesis settled by evidence at both
  sites, any residual violations reported before being touched.
- **Mode B** — row 1 without row 2, or the phantoms cleared by narrowing the sweep rather than fixing
  the alias.
- **Mode C** — a STOP fires. Ship nothing; the report is the deliverable. **STOP-2 and STOP-4 firing
  are GOOD outcomes** — they are the stone's real yield.
