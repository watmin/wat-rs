# DESIGN — STONE: the table rule claimed a definition, and the file lost its end

`wat-fmt` makes three of the four real files it has ever been pointed at **worse than it found
them**, against the ruled 120. One rule causes essentially all of it, and it is not a design
question — it is a rule reaching in and claiming forms that were never data rows.

## THE MEASUREMENT — ablation, one rule at a time, `wat/deporder.wat`

```
without table       over120=0    worst=102     ← ★
without kwargs      over120=252  worst=420
without siblings    over120=107  worst=862
without atoms       over120=259  worst=420
without defrecord   over120=260  worst=420
without match       over120=261  worst=420
without let         over120=285  worst=421
without defn        over120=260  worst=420
ALL (control)       over120=259  worst=420
```

Across every real file:

| file | today | without `table` |
|---|---|---|
| `wat/deporder.wat` | over120=**259** worst=420 | **0 / 102** |
| `wat/grep.wat` | over120=**66** worst=226 | **0 / 98** |
| `wat/spawn.wat` | over120=**175** worst=265 | **9 / 174** |
| `wat/fmt.wat` | over120=**669** worst=**1649** | **2 / 147** |
| `wat/io.wat` | 0 / 104 | 0 / 104 — **no group forms at all** |

⚠ `io.wat` is why this survived ten stones: it is 3 forms of one-liners, and it was the only real
file this arc measured before `[[NOTE-the-formatter-had-never-met-a-real-file]]`.

## WHY — and it is not a long string

`(:wat::core::defn :wat::deporder::collect-kwds …)` comes out at **352 columns, of which 222 are
spaces**:

```
(:wat::core::defn :wat::deporder::collect-kwds␣×10 [node <- :wat::WatAST]␣×187
    -> (:wat::core::Vector :- [:wat::core::String])␣×25 (:wat::core::if
```

Padding accounted, every stretched top-level line in `deporder`:

```
                    w     padding   code
collect-kwds       352     222      130
collect-form-refs  352     217      135
build-file-syms    136       7      129
build-symbol-map   387     160      227
build-pos-map      353     171      182
verify             353     188      165
stdlib-sources     353     240      113   ← code alone FITS; 240 chars of pure padding
verify-stdlib      360     234      126
```

`table-start` fires on any 2+ adjacent `list` siblings sharing a head and key-sequence. At a file's
top level, consecutive `defn`s of the same arity satisfy that exactly — same parent (the root), same
head, same positional signature. It **claims** them, which suppresses `defn`'s own ruled break, and
then pads each column to the group's widest member.

**Two damages, stacked, and neither causes the other:**

1. **The claim suppresses the ruled break.** `build-symbol-map` is 227 characters of code *before any
   padding* — head, name, arg-spec, ret-spec and the body's opening all riding one line.
2. **The padding has no budget.** `stdlib-sources` is 113 characters of code, which fits, stretched to
   353 by padding alone. `table.wat`'s own header calls naming no width a virtue — right for
   *indent*, which is structural, and wrong for a construct whose entire job is *horizontal*
   alignment. `atoms.wat` already carries the correct clause: inline iff every value is an atom
   **and it fits**.

## THE GUARD IS STRUCTURAL, NOT A HEAD LIST — and the census is why

The first draft of this stone listed five definition heads. **The corpus has twelve at column 0:**

```
1883 :wat::core::defn      276 :wat::core::defrecord   148 :wat::rete::defrule
  92 :wat::load-file!       82 :wat::rete::defquery     37 :wat::core::defenum
  36 :wat::core::defsurface 35 :wat::core::defmacro     24 :wat::core::defclause
  22 :wat::core::def        21 :wat::core::defstruct    17 :wat::service::defservice
```

A five-head list left `spawn.wat` at **75** over-120, because it missed `defservice`. A twelve-head
list is a list that will be patched again. `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`

**The honest predicate is structural: a table is DATA, and data is never at the top level.** One
`:where` on the parent — `(:wat::rete::i64::not= ?p 0)` — and it never needs to know what a
definition *is*:

| file | today | 5-head list | ★ `?p != 0` | no table at all |
|---|---|---|---|---|
| `deporder` | 259 / 420 | 0 / 102 | **0 / 102** | 0 / 102 |
| `spawn` | 175 / 265 | 75 / 222 | **9 / 174** | 9 / 174 |
| `grep` | 66 / 226 | 1 / 141 | **1 / 141** | 0 / 98 |
| `fmt.wat` | 669 / 1649 | 16 / 1430 | **16 / 1430** | 2 / 147 |

On `spawn` the structural guard equals removing the rule entirely; the head list does not.

⚠ **Could the substrate answer instead?** No — checked, per this arc's own
`[[NOTE-the-registry-already-knows-the-slots]]`. `defn` / `defrecord` / `defstruct` / `deftest` are
wat-level MACROS with **0 registry rows**; the registry cannot report that they define anything.
Asking the substrate is unavailable here, so the structural guard is the highest rung the material
allows.

⚠ **Both faults are still needed — measured, not assumed.** The guard leaves a real tail that is
exactly the fit test's: `grep`'s single 141-column line, and `fmt.wat`'s **1430**-column line, both
from genuine NESTED groups that align without ever asking whether they fit.

## THE THIRD THING — the file must not end on a blank line

**Builder, 2026-09-06:** *"the last line of the file is not blank - we need a guard on that"*

```wat
;; some/path/file.wat
(wat.core/defn user/some-fn
  [x :- wat.type/i64]
  :- wat.type/i64
  (wat.core/+ x 1))          ;; no trailing newline separator at the end
```

`emit` currently applies `ensure-blank` after the fold, so **every** formatted file ends `)\n\n`.
That is rule 3 ("one blank after each top-level form") reaching the **last** form, where there is no
following form to separate from.

★ **It also lands on the arc-255 path.** An isolated doc example comes back as `…))\n\n`:

```
(:wat::core::mapv …)  →  "(:wat::core::mapv\n  …\n  (:wat::core::Vector 1 2 3))\n\n"
```

That is the whole of the `CHANGED 330→614 · INLINE 284→0` shift the last SCORE reported as
arithmetic, and `crates/wat-doc/src/print.rs` would embed that blank in every `:examples` entry.

## WHAT THIS STONE IS NOT

- **Not `if` / `cond`.** Both are ruled (the `if` test rides its head line; `cond` clauses align) and
  both are NEW RULE FILES. They are the next stone, and they go second **because their acceptance
  rows are unmeasurable until this one lands** — every real file still carries 259/175/66 over-120
  lines from the table fault, so "did the if rule help" cannot be read off `deporder` today.
- **Not defect E.** Where a trailing comment goes is still the builder's call.
- **Not a new record and not a new rule file.** Two `:where` guard sets on an existing rule, one fit
  test, one word in `emit`.

## THE CONTRACT DECISION

**The fit test refuses to align; it never truncates and never re-breaks.** A group that does not fit
120 formats as if `table.wat` had not fired — each member by its own rules. This is exactly
`atoms.wat`'s existing shape, and it is the only behaviour that cannot manufacture padding.

## FILES

```
wat-scripts/fmt/rules/table.wat    the definition-head guards + the fit test
wat/fmt.wat                        emit's terminal ensure-blank → ensure-nl
```

⚠ **`wat/*.wat` is FROZEN into the release binary.** Editing the emitter and re-running proves
nothing without `cargo build --release`. Three sabotages in this arc were read as evidence before
that was established; two of them were self-consistent renames that could not have failed.
