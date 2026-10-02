# SCORE — STONE 255.83: cutover 5b — STOP-1

Parent of this commit is `63e30b988`. The quasiquote escape, the rete `where` head, and the macro-template purity walk now read `canonical_identity_of`. `is_quasiquote_form` does not. No `scripts/replay/census.sh`, no `scripts/floor.sh`, no clippy.

## 1. What a live row is

A row is live only when one program, spelled once with the keyword and once with the symbol of the **same** identity, disagrees. `canonical_identity("wat.core/Option.None")` is `:wat::core::Option.None`. `wat.core.Option/None` is a different name (`:wat::core::Option::None`). Round-1 pairs that used the second spelling (`pat-none`, `pat-ok`, `pat-err`, `option-none-value`, `eval-none` in `/tmp/census-25583-c.log`) are not live rows. The door spelling of those patterns is SAME (`pat-none-door`, `pat-some-door`, `pat-ok-door`, `none-value-door`, `none-debug` in `/tmp/census-25583-h.log`).

Freeze step 7 rewrites reference symbols in code positions. The pairs that still disagree are data positions, walks that run on raw source, `eval-step!`, and `lower()`.

## 2. Measured differentials

SAME on the door spelling, before any cure in this stone (logs `census-25583-c.log`, `d.log`, `e.log`, `h.log`): `eval-if`, `eval-let`, `eval-nil`, `eval-holon-atom`, `defn-head`, `let`, `if`, `do`, `fn-lit`, `match-lit` (values 1, 1, 1, 2, 4, 5), `user-main` (both `MainSignatureError`), `tuple-slot`, `qq-src` and `qq-both-sym` (both `VAL str:3`), `pure-println`, `pure-lit`, `macro-body-qq` (both `VAL i64:1`), `defmacro-qq-router` (both `ProgramBodyIntroducesName`), `top-setter` (both `VAL i64:1`), `expect-call` (both `VAL i64:7`), `where-head` (both `MalformedClause`). `config-setter-expr` is the same path `:wat::config::set-capacity-mode!` with two context strings (`call head` vs `namespaced symbol ref`).

DIFF, same identity, before the cures:

| pair | keyword | symbol | function |
|---|---|---|---|
| `lower-atom`, `lower-bind` | `OK` | `MalformedCall` "first element is a keyword" | `lower_call` |
| `legacy-letstar` | `BareLegacyLetStar` | `UnresolvedReference` `:wat::core::let*` | `walk_for_bare_primitives` |
| `legacy-lambda` | `BareLegacyLambda` | `UnresolvedReference` `:wat::core::lambda` | same |
| `legacy-char` | retired `MalformedForm` `:wat::core::Char` | `UnknownNamedType` | same |
| `legacy-uuid` | retired `MalformedForm` `:wat::core::Uuid` | `UnknownNamedType` | same |
| `legacy-unit` | `BareLegacyUnitName` | `UnresolvedReference` `:wat::core::unit` | same |
| `digest-load` | `Fetch` file not found `()` | `MalformedLoadForm` "got symbol" | `parse_verify_algo` |
| `digest-iface` | `Fetch` file not found `()` | `MalformedLoadForm` "got symbol" | `parse_payload_interface` |
| `step-if-debug` | `StepNext` of `IntLit(1)` | `eval-step!` "no rule for op: symbol-head" | `step_list` |
| `pure-quote` | bool true | bool false | `classify_expr` |
| `pure-qq` | bool true | bool false | `classify_expr` |
| `entry-deftest` | passed 1, discovered 1 | passed 0, discovered 0, notes true | `source_has_config_setter` |
| `unquote-foozle` | startup `UnresolvedReference` path `foozle` | run `UnboundSymbol` `foozle` | `is_unquote_escape` |
| `macro-impure-inner` | `MalformedDefmacro` (`println` refused) | `VAL i64:1` | `validate_quasiquote_template` |

Wat-side, one program, `/tmp/census-25583-f.log` `RC=0`: `PAIR wat-preds VAL str:I1i0D1d0C1c0F1f0M1m0`. Each letter pair is keyword then symbol for `:wat::lint::if-head?`, `:wat::lint::is-defmacro-form?`, `:wat::lint::concat-head?`, `:wat::deporder::def-form?`, `:wat::fix::defmacro?`. Keyword true, symbol false, five functions. No `canonical-identity` verb exists under `wat/**/*.wat`.

That is 9 Rust functions plus 5 wat functions. Under 40. STOP-3 does not fire. The other A/B rows were not given a same-identity differential, so they are not in this live count. The `:fn(` prefix inside `walk_for_bare_primitives` was not probed.

## 3. Cures that landed before the stop

`is_unquote_escape` and `is_where_form` take `&WatAST` and match `canonical_identity_of`. `validate_quasiquote_template` does the same for quasiquote, unquote, and unquote-splicing. The literal-fn exception in `validate_pure_total` does the same for `:wat::core::fn`.

Remeasured on that tree (`/tmp/census-25583-h.log`, `RC=0`), keyword side unchanged:

- `unquote-foozle` SAME. Both startup `UnresolvedReference`, path `foozle`, context `call head — not a builtin, not a registered function`.
- `macro-impure-inner` SAME. Both `MalformedDefmacro` refusing `` `:wat::kernel::println` ``.
- `where-head` SAME. Both `MalformedClause`.
- `expect-call` SAME. Both `VAL i64:7`. `walk.rs`'s `stem.replace('.', "::")` is not routed through the door.
- `macro-body-qq` SAME. Both `VAL i64:1`.

The ledger test named the shrink before the ratchet was tightened: `146 → 139`, gone entirely, none merely reduced — `validate_pure_total` Ax1, `validate_quasiquote_template` Ax3, `is_unquote_escape` Ax2, `is_where_form` Ax1. `LEDGER_TOTAL` is 139. Shape E was not touched. After the ratchet edit, `the_heresy_ledger_matches_its_frozen_census` passed (`/tmp/census-25583-i.log`, 4 tests, `RC=0`).

## 4. STOP-1 — `is_quasiquote_form`

`src/macros/expand.rs` `is_quasiquote_form` is still a keyword match on `:wat::core::quasiquote`. `parse_defmacro_form` runs `validate_macro_definition` only when this returns false. The comment on the function (stone 255.11) records why: teaching it the symbol spelling makes a symbol-spelled quasiquote body **skip** that check.

Before the template cure, `macro-impure-qq` was SAME (both `VAL i64:1`): the symbol body was not routed as a template, but `validate_quasiquote_template` did not see a symbol `unquote`, so `println` was template data and the check passed. After the template cure the symbol `unquote` is real code. The pair is now DIFF (`/tmp/census-25583-h.log`):

- Keyword body `(:wat::core::quasiquote (:wat::core::unquote (:wat::kernel::println "x")))`: startup succeeds, `:user::p` returns i64 1. The router skips the purity check.
- Symbol body `(wat.core/quasiquote (wat.core/unquote (:wat::kernel::println "x")))`: startup `MalformedDefmacro`, reason `program-body macro purity check failed at definition: keyword head `:wat::kernel::println` refused at macro expand time — not on the pure-combinator allow-list (default-deny F5 gate, arc 249 stone 249.2b-i); only pure-total heads are permitted`.

Those two outcomes are pinned by `tests/resolve/probe_arc255_83_quasiquote_router.rs` (`whole_body_quasiquote_keeps_the_keyword_router`, and the two SAME pairs above).

The two spellings should keep this difference. Widening `is_quasiquote_form` makes the symbol program skip the check and return i64 1, which is the permissive direction 255.11 forbids. Sending the keyword program through `validate_macro_definition` changes it from i64 1 to `MalformedDefmacro`, which is STOP-2. This row stays keyword-only.

No further A/B rows were cured. `lower_call`, `walk_for_bare_primitives`, `parse_verify_algo`, `parse_payload_interface`, `step_list`, `classify_expr`, `source_has_config_setter`, the wat predicates, and `stem.replace('.', "::")` are unchanged.

## 5. Gates not run

STOP means stop. Not run: `scripts/replay/census.sh`, `scripts/floor.sh`, `cargo clippy --release --all-targets -- -D warnings`. The brief's floor to beat, read and not re-run, is `.floor/2026-10-02T19-08-53Z` at `581478c9c`: 6374 passed, 24 skipped.
