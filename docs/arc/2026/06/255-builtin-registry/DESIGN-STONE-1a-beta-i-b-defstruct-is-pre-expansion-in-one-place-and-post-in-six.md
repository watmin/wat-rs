# DESIGN — STONE 1a-β-i-b: `defstruct` is PRE-expansion in one place and POST-expansion in the rest

> Raised by 1a-β-i's STOP-5: `:wat::core::defstruct` is a stdlib `defmacro` (`wat/core.wat:2030`)
> that rewrites to `structtype` during `expand_all`, so `parse_type_decl`'s `"defstruct"` arm is
> dead. **The stone that follows is not "delete the dead arms" — it is "which arms are dead?", and
> the answer is not uniform.**

## ★★★ THE DISCRIMINATOR, and it is measured on both sides

`expand_all` erases every `:wat::core::defstruct` head from the program. So a consumer that runs
**after** it can never see one; a consumer that runs on **raw, unexpanded** AST still can.

Both poles are proven by probe, not by reading:

```
POST — parse_type_decl.   --check on a malformed (:wat::core::defstruct :probe::Bad) answers
       :head "structtype", with parse_structtype's own arity text. The "defstruct" arm never fired.

PRE  — refuse_mutation_forms.  (:wat::eval-ast! '(:wat::core::defstruct …)) answers
       "eval refused mutation form: :wat::core::defstruct". The literal head reached the guard.
```

★ **`eval-ast!` evaluates user AST that was never macro-expanded.** That is why one `defstruct` arm
in this tree is load-bearing while six others are not — and nothing in the tree currently records
the difference.

## THE ONE CONTRACT DECISION — pinned

**Every `:wat::core::defstruct` site is classified by ONE question — *can this code run before
`expand_all`?* — and the answer is written at the site.** Dead arms are removed; live arms STAY and
gain the sentence that says why, so the next sweep cannot mistake them for leftovers of this one.

⛔ **The failure mode this stone must not create is the opposite of the one it fixes.** A rider told
"remove the dead `defstruct` arms" will remove all seven, and the two that guard `eval-ast!` are a
real refusal. **A half-swept name is how `is_mutation_head` and `is_mutation_form` came to disagree
in the first place.**

## The seven sites, with my classification — to be VERIFIED per site, not inherited

| # | site | runs | disposition |
|---|---|---|---|
| 1 | `freeze::is_liftable_declaration_head` | `split_body_prelude` ← `extract_closure`, **runtime** | POST → **remove** |
| 2 | `types.rs` `parse_type_decl`'s `"defstruct"` arm | after `register_types` | POST → **remove** |
| 3 | `types/defstruct.rs` `parse_defstruct` (~62 ln) | only caller is #2 | dead → **remove** |
| 4 | `types/defstruct.rs` `validate_defstruct_arity` | only caller is #3 | dead → **remove** |
| 5 | `types.rs` `classify_type_decl`'s arm | routes to #2 | POST → **remove** |
| 6 | `declare/parse.rs` `is_struct_form` | callers in `preregister.rs` | ⬜ **MEASURE** |
| 7 | `closure_extract.rs` `walk_free_symbols`'s arm | runtime closure extraction | POST → likely remove, **MEASURE** |
| 8 | `runtime::is_mutation_head` · `freeze::is_mutation_form` | guard `eval-ast!` | **PRE → KEEP, and say why** |

⚠ **#6 and #7 are marked MEASURE and must not be swept on my say-so.** `preregister` runs early and I
have not established which side of `expand_all` it sits on. `[[feedback_a_pattern_that_matches_a_subset_is_not_a_census]]`
— a table I wrote is a prediction until each row is probed.

⚠ **Most of `src/types/defstruct.rs` is LIVE** and the file is not the unit. `parse_defstruct_metadata`
and `parse_aggregate_fields_with_splices` are called from `types.rs:4678`/`4684` through
`parse_aggregate`, which `parse_structtype` uses. Measured, because I nearly reported a 581-line
module as dead: the dead set is **two functions and one arm**, roughly 90 lines.

## ⛔ THE GATE — the live arm must become un-sweepable

Removing dead arms while leaving live ones is a distinction only prose currently carries, and prose
is the convention rung. So the stone pins the PRE-expansion behaviour with a **probe**:

> `(:wat::eval-ast! '(:wat::core::defstruct …))` must still be refused with
> `EvalForbidsMutationForm`, naming `:wat::core::defstruct`.

★ That is what makes the surviving arm un-deletable silently: a future sweep that "finishes the job"
goes RED with a message naming the exact form and the exact guard.
`[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`

## The meter

```
MISSING  4  →  3        (def · defalias · defmacro)     ← removing site #1 from the domain
FOREIGN  0  →  0
```

★★ **And this is the point of the stone, not a side effect.** `defstruct` is a MACRO: it has no
declare-time fn and never will, so while it sits in `is_liftable_declaration_head`'s domain **MISSING
can never reach 0** and 1a-β-ii can never flip the consumer. The dead arm is not untidiness — it is
the thing standing between the campaign and its first hand-list kill.

## THE FOUR QUESTIONS

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **classify all 7, remove the dead, pin the live with a probe** | YES | YES | YES | YES | ✅ **PICKED** |
| remove every `defstruct` arm | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| remove only site #1 (the meter's blocker) | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| leave it, special-case `defstruct` in the meter | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| classify all 7, no probe | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |

- **remove-all Honest? NO** — it deletes a real refusal. Measured, with the probe above.
- **only-#1 Honest? NO** — it takes the meter's number while leaving the dead router arm and its two
  dead functions, and leaves the next reader to re-derive the whole macro-shadowing story.
- **special-case Honest? NO** — an exemption for a name the domain should not contain. The hand-list
  is wrong; exempting it in the meter launders that.
- **no-probe Good UX? NO** — the live/dead distinction survives only as a comment, and the next
  sweep is the one that deletes the guard.

## Blast radius

`src/freeze.rs` (one arm) · `src/types.rs` (two arms) · `src/types/defstruct.rs` (two fns) ·
possibly `src/declare/parse.rs` + `src/closure_extract.rs` **pending measurement** ·
one new probe test. No `.wat` corpus change. No registration. **No hand-list that guards `eval-ast!`
is touched.**

## Acceptance

| what | command | expected |
|---|---|---|
| the meter moved | `liftable_declaration_head_missing_and_foreign` | MISSING 4 → 3, domain 9 → 8 |
| ⛔ the PRE guard survives | the new probe | `eval-ast!` still refuses a literal `defstruct` |
| ⛔ the probe can FAIL | drop `defstruct` from `is_mutation_form` | RED, naming the form |
| the dead fns are gone | `grep -c "fn parse_defstruct\b"` | 0 |
| ⛔ the LIVE defstruct.rs half is untouched | `parse_defstruct_metadata` · `parse_aggregate_fields_with_splices` | still called from `types.rs` |
| a real `defstruct` still works | `--check` a `(:wat::core::defstruct :geo::Pt [x <- :i64])` | clean, as today |
| floor | `scripts/floor.sh`, exit UNPIPED | 5125/5125 |
| clippy | `-D warnings --all-targets` | 0 |

## Out of scope = REJECTED

- **`def`/`defmacro`/`defalias`, the consumer flip, deleting the hand-list.** 1a-β-ii, which this
  stone unblocks by making MISSING reachable.
- **The other macro-shadowed names, if any.** This stone measures `defstruct`. Whether `defn` or
  others have the same shape is a separate census with its own probe.
- **`NOTE-an-orphaned-impl-annotation-is-silently-discarded`'s drainage gate.** Different mechanism,
  its own four questions.
