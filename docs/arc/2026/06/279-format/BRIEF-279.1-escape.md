# BRIEF — Arc 279.1: `format` `{{`/`}}` literal-brace tokenizer

You are a single-hop sonnet executor. **Do NOT spawn sub-agents.** Do NOT run `git`. Build, run the
named tests, report. The orchestrator weighs your kill on its own build.

## The work (one paragraph)

The `format` macro in `wat/core.wat` cannot emit a literal brace. Make it support the **doubled-brace
escape**: `{{` → a literal `{`, `}}` → a literal `}`, collapsed at expand time. Rewrite the macro's
template **parse section** as a single-pass character-walk state machine (the current split-by-`{`-then-`}`
parser cannot host doubling). The foundation is already in place and proven — you only edit `wat/core.wat`
and un-ignore one test file.

## The contract — implement EXACTLY the algorithm in the DESIGN

Read **`docs/arc/2026/06/279-format/DESIGN-279.1-escape.md` § "The algorithm"** and implement it verbatim:
a two-pass design — **Pass 1** tokenizes the char vector via `foldl` over a `Tuple(mode, pending, buf,
segments)` accumulator into a `Vector<Tuple(kind, payload)>`; **finalization** inspects the returned
accumulator (lone-brace / unclosed-name errors, flush final text); **Pass 2** maps segments → concat
pieces (`Vector<WatAST>`) + the used-set. The transition table in the DESIGN is the spec — every brace
case and every macro-error message is named there. Do not invent variations.

## Read in order (the rooms)

1. `docs/arc/2026/06/279-format/DESIGN-279.1-escape.md` — the algorithm + the worked cases. THE SPEC.
2. `tests/probe_arc279b_subs_tuple_macro_eval.rs` — **the worked reference**. It is GREEN and shows the
   exact shape you copy: build the char vector with `(map (fn [i] (string::subs str i (+ i 1))) (range 0 (length str)))`,
   `foldl` a `Tuple` accumulator, `first`/`second` to read it, `=` to compare a char, build a
   String-literal AST node via the read-string trick. Copy this shape.
3. `wat/core.wat:506-541` — the `format` doc comment. UPDATE it: replace the `\{`-not-supported note
   with the `{{`/`}}` doubling rule + the two-pass tokenizer summary.
4. `wat/core.wat:543-736` — the `format` macro. Rooms within it:
   - `:548-561` — template extraction + the `"`-guard. **KEEP unchanged.**
   - `:563-595` — the kwargs-fold (`kwargs-map`). **KEEP unchanged.**
   - `:597-705` — the parse section (split-by-`{`, the `pieces` foldl, the `used-set` foldl).
     **REPLACE** with the two-pass char-walk (Pass 1 tokenize + finalization + Pass 2 emit/used).
   - `:707-722` — the strict unused-kwarg check. **KEEP unchanged** (it reads `used-set`/`kwargs-map`).
   - `:724-736` — the emit tail (empty/single/concat). **KEEP unchanged** (it reads `pieces`).
5. `tests/probe_arc279b_format_escape.rs` — the feature gate. Remove the three `#[ignore = "arc 279.1 …"]`
   attributes so all three tests run.

## Implementation sketch (you fill it; the shape is fixed)

```clojure
;; in the format macro's let-bindings, REPLACING the seg-by-open/leading/init-pieces/pieces/used-set block:
[chars   (:wat::core::map
           (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::String
             (:wat::core::string::subs tmpl-str i (:wat::core::i64::+ i 1)))
           (:wat::core::range 0 (:wat::core::string::length tmpl-str)))
 ;; Pass 1: foldl over chars → Tuple(mode pending buf segments) per the DESIGN transition table.
 tok-state (:wat::core::foldl (:wat::core::fn [acc <- :wat::core::Tuple c <- :wat::core::String]
              -> :wat::core::Tuple
              (:wat::core::let [mode (:wat::core::first acc) pending (:wat::core::second acc)
                                buf (:wat::core::third acc) segs (:wat::core::last acc)]
                ;; ... the transition table (if/= ladder over mode×pending×c) ...))
              (:wat::core::Tuple "text" "none" "" (:wat::core::Vector :wat::core::Tuple))
              chars)
 ;; finalization: inspect tok-state — error on lone brace / unclosed name, else flush final text segment.
 segments  (... )
 ;; Pass 2: foldl segments → pieces (Vector<WatAST>) + used-set (or two folds; the existing used-set
 ;;         check at :707-722 reads `used-set`, so produce one).
 pieces    (... )
 used-set  (... )]
```

Factor the String-literal-node builder (used by the leading flush, the text-segment flush, and Pass 2's
text segments) into one expand-time helper to avoid the triplicated read-string trick.

## STOP triggers (halt + report; do not improvise)

1. If `string::subs` is rejected at macro-eval ("not pure-total" / similar), STOP — the allow-list line
   may be missing; report it (the orchestrator added it; do not re-add or work around).
2. If `first`/`second`/`third`/`last` do **not** read a 4-field `Tuple` as expected, STOP and report the
   exact error — do not switch to a different state container without surfacing it first.
3. If any of the **preserved** arc-279 cases regress (`"{a} {b}"` substitution, missing/unused-kwarg
   macro-errors, non-literal-template macro-error), STOP — the kept sections must keep working.

## Blast radius

`wat/core.wat` (the `format` macro + its doc comment) and `tests/probe_arc279b_format_escape.rs`
(remove 3 `#[ignore]`s) **only**. No Rust source edits — `subs` is already wired and allow-listed. No
new files. No `git`.

## Verify before you report (run these, paste the output)

```
cargo test --release -p wat --test probe_arc279b_format_escape          # 3/3 GREEN (the feature gate)
cargo test --release -p wat --test probe_arc279_format                   # arc-279 base still GREEN
cargo test --release -p wat --test probe_arc279b_subs_tuple_macro_eval   # foundation still GREEN
cargo test --release --test test 2>&1 | grep "test result"               # deftest binary: 257 passed / 1 failed (pre-existing run_string_entry_direct)
cargo test --release --test test_stdlib_load_order 2>&1 | grep result    # deporder gate: 1 passed / 0 failed
```

Report: the diff summary (what you changed in `core.wat`), the five command outputs verbatim, and any
delta from the expected results.
