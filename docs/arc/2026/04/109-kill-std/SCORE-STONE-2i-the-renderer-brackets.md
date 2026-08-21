# SCORE — ②-i: the renderer brackets. **And the rider refused a wrong correction from me.**

Floor **4855/4855**, clippy **0**. Two flights, ~20 min. No `wat/` file touched.

| # | what | result |
|---|---|---|
| 1 | parametric args bracket, both modes | ✅ `(wat.type/Vector [wat.type/i64])` |
| 2 | nesting nests | ✅ `(wat.type/Vector [(wat.type/Vector [wat.type/i64])])` |
| 3 | ★ the `Fn` arm untouched | ✅ `[wat.type/i64 :-> wat.type/bool]` |
| 4 | ★ COLON mode emits the rust-ish head | ✅ `(:wat::core::HashMap [:wat::core::String :wat::core::i64])` |
| 5 | sibling verb registered in all three places | ✅ `check.rs` · `runtime.rs` · `macros/eval.rs` |
| 6 | goldens updated, none weakened | ✅ 20 files |
| 7 | floor | ✅ **4855/4855, 70.7s** |
| 8 | clippy | ✅ 0 |
| 9 | goldens (orchestrator step) | ✅ 5 bumped 25326 → 25330, hunks verified above the pin |
| 10 | no `wat/` corpus migration | ✅ |

## ⛔⛔ I SENT A CORRECTION THAT WAS WRONG. THE RIDER MEASURED INSTEAD OF COMPLYING.

I rendered both modes with `write-forms`, read

```
(:wat.core/Vector [:wat.core/i64])
```

and sent the rider back to fix "a third spelling". **My probe was the defect, not its code.**

`write-forms` → `watast_to_edn` is hardcoded to `Carriage::Display`, whose `Keyword` arm
**unconditionally** calls `keyword_from_wat_path` and re-spells *every* `::`-keyword into the
EDN-dotted form. The verbatim printer is `ast->source` → `write_wat_source`, whose `Keyword` arm is
`out.push_str(k)` — a byte-literal copy (`src/edn_shim.rs:746`, verified).

The rider's probe, run against my build, shows both at once:

```
rung1-parametr verbatim(ast->source)  : (:wat::core::HashMap [:wat::core::String :wat::core::i64])
rung1-parametr display  (write-forms) : (:wat.core/HashMap [:wat.core/String :wat.core/i64])
rung3-usertype verbatim               : :wat::holon::HolonAST
rung3-usertype display                : :wat.holon/HolonAST
```

**The node was always correct.** COLON mode builds the `::`-string directly and never routes rungs
1/2 through `wat_keyword_to_clojure_symbol` — that call is shared malformed-path *validation* whose
symbol result is discarded on the Colon arm.

★ **It made NO code change**, and said why: *"changing rung 1/2 to emit Clojure-dotted text would make
the actual stored node wrong to satisfy a printer that isn't the right instrument for this check."*
That is the correct call, and it is the harder one to make when the orchestrator has just told you
you are wrong. It built an instrument that shows BOTH printers side by side, so the
display-vs-verbatim gap was visible rather than argued.

⚠ **Tenth instrument error of the session, and the same family as the rest:** I used a tool that
*transforms* its input and read the transformation as the value.
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

★★ The thing that makes this outcome good rather than merely survivable: **the brief told the rider to
verify by rendering rather than by inspection, and it did — then applied the same standard to my
claim.** A rider that had simply complied would have corrupted the renderer to match a broken probe,
and the corruption would have shipped green, because `:wat.core/Vector` parses.

## Honest deltas

- **20 goldens, not my estimated 29 — and its number is better than mine.** Step ① made the parser
  accept `(Head [args])` *in addition to* `(Head args)`; it is a dual-read, not a replacement. So
  fixtures using the flat form as hand-written *source* for parse-acceptance tests
  (`assert!(…is_ok())`, no string comparison) stay green untouched. My 29 was a grep over occurrences
  that never asked which were RENDERER assertions and which were SOURCE literals.
- **All brief line numbers were exact** — `:1200`, `:1204`, `:1225`, `:1249-1253`, `:1255-1262`,
  `:1284`, `check.rs:18800`, `runtime.rs:5214`, `macros/eval.rs:667`. Confirmed by matching code.
- **`TypeExpr::Tuple`'s head left out of `mode`** — flagged as a judgment call, documented in the fn's
  doc comment. Not in Room 2's 4-way ladder and not exercised by the contract suite. ⚠ It will need a
  decision before the Clojure flip; recorded rather than guessed.
- **Rung 4 (type-var) is the same in both modes** — a bare `T` was never colon-qualified in any
  surface, so there is nothing for COLON to change. Not a STOP; correct.
- **A stale doc reference found in passing:** `runtime.rs:14462` and `check.rs:3538` both name
  `holon_type_ast_to_wat_type_form`, a function that **does not exist** anywhere in `src/` — dead
  since the arc-294.f rewire. Out of scope; a reader chasing that name hits nothing.

## What ②-i unblocked

`②-ii` can now be written: a wat-fix codemod that rewrites type-shaped keywords via
`keyword/to-type-form-colon`, producing `(:wat::core::Head [args])` — the shape the corpus migrates
to, with the head spelling unchanged.

★ And `300/NOTE-the-type-converter-emits-the-superseded-form.md` is **closed** — the flat splice it
names as a blocker for 300.1 is now bracketed.
