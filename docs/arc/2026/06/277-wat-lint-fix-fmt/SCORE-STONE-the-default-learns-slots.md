# SCORE — STONE: the default learns slots from the registry

No commit. Floor and clippy left to the orchestrator. `wat/grep.wat` untouched.

## The ruling

`foldl-bare.wat`'s inner `fn` renders the ret-spec on **one line**:

```
(:wat::core::fn
  [acc <- :wat::core::i64 x <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::i64::+ acc x))
```

`IDEMPOTENT=true`. Head spelling from the parsed grammar's child 0 (`:wat::core::fn`), not `Row/name`.

## The Slot set, printed — not a silent empty join

```
SLOTS=3
  SLOT head=:wat::core::defmacro glued=4
  SLOT head=:wat::rete::core::fn glued=3
  SLOT head=:wat::core::fn glued=3
FN_SLOT=true  REFUSAL=true
```

36 grammars parse; 3 have a top-level `->`. `let` has none — `let-two.wat` byte-identical to before.

## The refusal, shown

`slot-of-syntax` on `(:wat::core::fn <x>+ -> :T y)` — a variadic child before the arrow — yields **no** Slot. `REFUSAL=true`.

## What this does not fix (measured, named)

Type applications still split (`HashMap :- [T]`). They have no `@syntax`. The lexical `->`/`:-` glue is the builder's next call, not this stone.

## Walls

Disagreeing-kind sabotage still raises `fmt: conflicting Breaks for node 11 — block vs align`. Deleted after. `ClaimedUnder` 0. `col` 0 in every rule file.

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| `run-slots.wat` | **SLOTS=3**, FN_SLOT=true, REFUSAL=true |
| `run-all.wat` on `foldl-bare.wat` | `-> :wat::core::i64` **one line**, IDEMPOTENT=true |
| `run-let.wat` on `let-two.wat` | unchanged |
| every other fixture | idempotent; ruled shapes hold |
| `run.wat` on `wat/io.wat` | **COMMENTS=28** |
| kind-conflict sabotage | **raises** |
| `every_wat_scripts_file_loads` | **1 passed** |

---

## ORCHESTRATOR VERDICT — 2026-09-05

**ACCEPTED. No edit.** ⛔ **And a number of mine was wrong by an order of magnitude.**

| what | result |
|---|---|
| ★★★ **row 2 — the ret-spec is ONE LINE** | `-> :wat::core::i64` together, `IDEMPOTENT=true` |
| row 3 — the Slot count PRINTED | **`SLOTS=3`**, non-empty; the silent-empty join did not happen |
| row 4 — `fn`'s slot | `head=:wat::core::fn glued=3` ✓ |
| row 5 — the refusal FIRES | variadic-before-arrow → **no Slot**, `REFUSAL=true` |
| row 10 — `grep.wat` untouched | `git diff` **EMPTY** |
| floor | **5179 run, 5179 passed, 0 FAILED, 18 skipped** · clippy **0** |

The builder's non-negotiable holds. Head spelling came from the parsed grammar's child 0, so
STOP-1's silent-empty join never occurred.

## ⛔ MY CORRECTION — "36 forms at once" was 3

I told the builder this would *"make the default correct for **36 forms at once**"*. **It is three.**

```
SLOTS=3
  :wat::core::fn          glued=3
  :wat::rete::core::fn    glued=3
  :wat::core::defmacro    glued=4
```

36 rows carry a grammar; **only 3 of them contain a top-level `->`.** I read "36 grammars" and
reported it as "36 forms fixed" without asking how many of those grammars have the thing the rule
looks for. **The measurement was right and my inference from it was not**
(`[[feedback_a_pattern_that_matches_a_subset_is_not_a_census]]`).

★ The stone is still worth it — `fn` was the ruled defect and it is fixed, the refusal is proven,
and the grammars are now demonstrably machine-usable. But the *yield* is 3, not 36.

## ⭐ AND THAT REVERSES THE CASE ON THE ALTERNATIVE — the same output shows it

The very run that fixes the ret-spec still contains:

```
  [xs <- (:wat::core::Vector :-
           [:wat::core::i64])]        ⛔ a type application, still torn across two lines
```

| | registry-derived (this stone) | lexical `->` / `:-` glue |
|---|---|---|
| slots produced | **3** | every `->` **and** every `:-`, anywhere |
| type applications | **not covered** — no `@syntax` | **covered** |
| needs the registry | yes | no |
| rests on | 36 grammar strings staying accurate | two tokens that are LANGUAGE, not policy |

**I argued for the registry route and the measurement now argues against it.** The honest reading:
`->` and `:-` are not per-form policy that a registry must declare — they are **language syntax**,
and a rule about them belongs where syntax lives, not where declarations do.

⚠ **What this stone bought is not wasted**: it proves the grammars parse, the head spelling problem
is real and solvable, and the refusal discipline works. If the lexical rule lands, this becomes the
*general* mechanism for anything a grammar knows that syntax alone cannot — and there will be such
things. **But for `->` and `:-` specifically, the lexical rule is smaller and covers more.**

**That is the builder's call, and it is now backed by a number rather than my preference.**

## Not disputed

STOP-2 through STOP-5 held. `wat/grep.wat` untouched. The three walls still stand — the
disagreeing-kind sabotage still raises, `ClaimedUnder` 0, `col` 0 across every rule file. Every
fixture idempotent, `wat/io.wat` still **COMMENTS=28**.
