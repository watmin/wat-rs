# SCORE — STONE 251.9: the catch-all that ate the declaration

No commit. Floor left to the orchestrator (read `^ +Summary`, never a tail). Lands on 251.8b (the stored tuple) and the match-arm close. Those were not reverted.

`head_fqdn` is the one door. Every converted head reader goes through it. A `WatAST::Symbol` reference is no longer `"this is not a declaration"`.

---

## Expectation 1 — malformed variant refused under both spellings

**PASSED.** `malformed_variant_is_refused_under_both_head_spellings`.

## Expectation 2 — missing mandatory purity marker refused under both

**PASSED.** `missing_mandatory_purity_marker_is_refused_under_both_head_spellings`.

## Expectation 3 — a symbol-headed declaration actually declares

**PASSED.** `a_symbol_headed_declaration_actually_declares`.

`--check` of the pair, one token different:

```
tests/resolve/probe_arc251_stone9__declares_kw.wat   EXIT=0
tests/resolve/probe_arc251_stone9__declares_sym.wat  EXIT=0
```

## Expectation 4 — `#[ignore]`s are gone

`grep -c '#\[ignore' tests/resolve/probe_arc251_stone9_symbol_head_declaration.rs` → **0**.

## Expectation 5 — the catch-alls actually fell

`grep -cE '_ => (return )?(false|None)' src/declare/parse.rs`

**15 → 14.** Strictly less. The four head-reader `_ => return false/None` arms are gone (`is_declaration_form`, `parse_defalias_form`, `try_parse_variadic_def_fn_form` def+fn). The remaining 14 are:

| line | what |
|---|---|
| 210 | **the door** — `head_fqdn`'s `_ => None` ("this node names nothing") |
| 221 · 248 · 288 · 303 · 567 · 589 | SHAPE — not a List / not a Vector |
| 259 · 263 · 584 | NAME (STOP-1, untouched) |
| 348 | MARKER metadata key (STOP-2, untouched) |
| 606 · 611 | SHAPE — args vector / `->` |
| 616 | TYPE `parse_type_keyword` (STOP-4, untouched) |

Two `_ => false` appeared on `is_struct_form` / `is_enum_form` because `matches!` became an explicit List-shape guard. They are SHAPE, not head readers.

## Expectation 6 — the compiler's population, with missed sites named

Converted through `head_fqdn` (literal equality / set-membership only):

**`src/declare/parse.rs`**
- `is_declaration_form`
- `parse_defalias_form` head
- `is_struct_form` / `is_enum_form`
- `try_parse_fn_shape_def` head + nested `:wat::core::fn`
- `try_parse_variadic_def_fn_form` head + nested fn
- `try_parse_user_variadic_def_fn_form` head + nested fn

**`src/declare/preregister.rs`**
- `preregister_acronyms`
- nested `do` / `let` recurse

**`src/declare/register.rs`**
- `register_defines` `do` / `let` / `defclause`
- `register_stdlib_defines` `do` / `let`
- `register_stdlib_runtime_defs`
- `register_runtime_defs_form`

### Finding — the room map missed the freeze door

**`src/types.rs` `classify_type_decl`.** Not in `src/declare/*`. This is the load/`--check` classifier: Keyword-only, so `(wat.core/defenum …)` never reached `parse_defenum`. The AMEND's `--check` failure is this site, not `is_declaration_form`. Converted: `head_fqdn` then the same literal match; the returned static tag (`"defenum"`) is not the FQDN (STOP-3 held).

The probe cannot pass without this site. A rider that converted `is_declaration_form` and stopped would have left `--check` red.

### Finding — outside blast radius, not converted

**`src/runtime.rs:12232` `head_of`.** Same Keyword-only catch-all, used by `eval-with-defs!` to skip evaluating already-registered declarations. Type decls are consumed by freeze before this runs, so the probe does not see it. Named, not patched — blast radius was `src/declare/*.rs` plus wherever the door lands.

`parse.rs:888` (`parse_type_slot` structured-type head) is STOP-4, not converted.

## Expectation 7 — no keyword-headed behaviour moved

**Not run** (orchestrator floor). Targeted: `every_ungated_wat_file_checks` PASS (stdlib still checks). Probe keyword controls still exit 0. `git diff` is declare/* + `types.rs` `classify_type_decl` + the probe un-ignore. No keyword arm deleted.

## Expectation 8 — clippy

`cargo clippy --release --all-targets --workspace` **0 errors**. 5 pre-existing dead-code warnings (`Coverage::Wildcard`, `pattern_coverage`, `ident_span`, `try_match_pattern_ast`, `substitute_many`). Three `clippy::question_mark` hits on Option-returning `let Some(head) = head_fqdn else { return None }` were rewritten to `?`.

## Expectation 9 — names were NOT touched

`git diff --stat -- src/function/parse.rs` → **empty**. NAME readers in `parse.rs` / `register.rs` (`items[1]` of defalias / def / acronyms) unchanged. STOP-1 held.

## Expectation 10 — markers were NOT touched

`parse.rs` metadata-map key reader still `WatAST::Keyword`. `src/function/parse.rs` not in the diff, so `:guard` / `:ensure` untouched. STOP-2 held.

---

## STOP rows

| STOP | result |
|---|---|
| STOP-1 NAME reader | **held.** None converted |
| STOP-2 MARKER | **held.** |
| STOP-3 `head_fqdn` used as a key | **held.** Every caller is `==` / `contains` / `match` on the FQDN string. `classify_type_decl` returns a static tag, not the Cow |
| STOP-4 TYPE reader | **held.** `parse_type_slot` / `parse_type_keyword` untouched |
| STOP-5 keyword-headed floor red | **not run.** Stdlib check lint green; keyword probe controls still 0 |

## The two lists were not unified

`DECLARATION_HEADS` and `RUNTIME_DECLARATION_HEADS` still answer different questions. The door reads a spelling; it does not merge the lists.

`ns_to_wat_path` was called, not edited.

## Targeted checks

```
cargo nextest run --release -E 'test(probe_arc251_stone9)'     3 passed
cargo test --release -p wat-doc -p wat-macros -p wat-edn -p wat-reader   ok
cargo nextest run --release --test lint -E 'not test(every_wat_scripts_file_loads)'
  124 passed
cargo clippy --release --all-targets --workspace               0 errors
```
