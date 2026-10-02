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

## 6. Amend — the whole-body template is checked too

Section 4 stays. The amend overrules it. `is_quasiquote_form` reads `canonical_identity_of`, so both spellings take the template route. That route calls `validate_template_escapes`, which calls `validate_quasiquote_template`. It does not send the template through `validate_macro_definition`. The keyword program that returned i64 1 is now `MalformedDefmacro`. That is the gate closing. STOP-2 does not fire on it.

Parent of the amend markdown is `04cb3317f`. This appendix is the cure on top of `5b5823914`.

Pinned by `tests/resolve/probe_arc255_83_quasiquote_router.rs` against co-located fixtures (`/tmp/census-25583-probes.log`, 7 passed, `RC=0`):

- Unquote of `foozle`: both spellings are startup `UnresolvedReference`, path `foozle`, context `call head — not a builtin, not a registered function`.
- Nested impure unquote: both `MalformedDefmacro`, reason `program-body macro purity check failed at definition: keyword head `:wat::kernel::println` refused at macro expand time — not on the pure-combinator allow-list (default-deny F5 gate, arc 249 stone 249.2b-i); only pure-total heads are permitted`.
- Whole-body impure quasiquote: both `MalformedDefmacro`, same refusal prefixed `quasiquote template purity check failed at definition of :user::m: `.
- Whole-body pure quasiquote (`:wat::i64::+` / `wat.i64/+`, then `(:user::m 1)` / `(user/m 1)`): both i64 2.
- The five wat predicates, both spellings: string `I1i1D1d1C1c1F1f1M1m1`.

The section 4 pin `whole_body_quasiquote_keeps_the_keyword_router` is gone. The two probe `.rs` files hold no wat form literals.

### STOP-4 does not fire

One whole-body template outside the new probes: `tests/macros/probe_arc249_macro_engine_impure.wat.bad` line 1, `:my::impure-cu`, body `` `~(:wat::kernel::stopped?) ``. That file is the gap proof. `.wat.bad` is not in the census. Tracked `*.wat` flipped nothing (below).

### Live rows

Nine Rust functions and five wat predicates. Under 40. STOP-3 does not fire. Keyword text is unchanged. A reference symbol goes through the identity door.

Driven in `src/` (`/tmp/census-25583-diff2.log`, `RC=0`) and the fixture probe:

| pair | both spellings |
|---|---|
| lower atom | i64 7 |
| config setter | true for `set-redef!` and for `:wat::test::probe` / `wat.test/probe`; false for a plain defn |
| verify load | `not-found:sum.txt`; bad prefix `malformed:verification algorithm keyword must start with :wat::verify::; got :wat::core::if`; bare symbol `malformed:verification algorithm must be a :wat::verify::<kind>-<algo> keyword; got symbol` |
| verify runtime | algo `sha256`; string payload `deadbeef`; file-path `no-loader::wat::verify::file-path` |
| step `if` | next i64 1 |
| quote / quasiquote / holon literal | a quoted `println` is data; an unquoted `println` is the head `:wat::kernel::println` |
| legacy walker | `BareLegacyLetStar`, `BareLegacyLambda`, `BareLegacyUnitName`; Char reason contains `Stone 242.1`; Uuid reason contains `arc 255.77` |
| `:fn(wat.type/i64)` | `BareLegacyLowercaseFn` |
| `wat.core/fn` | not `BareLegacyLowercaseFn` |
| dotted `:wat.core/Char` | the Char retirement does not fire |
| `:wat::core::Option/expect` / `wat.core.Option/expect` | i64 7 |

`spelling_key` clones a keyword and runs `canonical_identity` on a reference symbol. A Type/method symbol becomes the `::method` join, so that symbol does not equal a raw `/` literal. The expect pair above agrees at i64 7. That measurement is this verb.

`:wat::core::canonical-identity` (String → String) is the door for `:wat::lint::if-head?`, `:wat::lint::is-defmacro-form?`, `:wat::lint::concat-head?`, `:wat::deporder::def-form?`, and `:wat::fix::defmacro?`. Each compares that verb to the same keyword literal.

Head-by-text left on `ast-name`, recorded and not codemoded:

- `wat/fix.wat`: `:wat::core::if` (81), `"->"` (83, 133, 637), `"<-"` (132, 692), `"&"` (643), `":-"` (965), `:wat::enum::Pure` (971), `:wat::enum::Impure` (972), `:wat::core::first` (1384), `:wat::core::drop` (1391). Compares to the codemod operand `old` at 1064 and 1120. `defmacro?` at 655 is routed.
- `wat/lint.wat`: `:wat::core::=` at 143. `if-head?` (126), `concat-head?` (323), and `is-defmacro-form?` (340) are routed.
- `wat/core.wat`: `:wat::core::agg-positional` (643), `":-"` (723), `"&"` (780, 864).
- `wat/Record.wat`: `:wat::core::unquote-splicing` (153, 249).
- `wat/kernel/readln.wat`: `:max-buffer-bytes` (85).
- `wat/service.wat`: `"T"` (997), `:locus` (2729, 2853).
- `wat/rete/compile.wat`: `:from` (623, 933–937), `":-"` (882).
- `wat/rete/oracle/stratify.wat`: `:wat::rete::exists` (233), `:from` (239–243).
- `wat/fmt.wat`: `":-"` (363).
- `wat/deporder.wat`: `def-form?` is routed (95, 102). `ast-name` contains `"::"` at 60. That is not a head-identity compare.

`"->"`, `"&"`, `":-"`, `":from"`, and `":locus"` are unchanged by `canonical_identity`. Routing every `:wat::` compare would also make a dotted keyword such as `:wat.core/if` match. That is a keyword-text change. These sites stay.

### Ledger

`LEDGER_TOTAL` is 64. Trajectory on this amend: 139 → 138 (`is_quasiquote_form`) → 119 → 114 → 64. Shape A and shape B in `src/` are 0. What remains is shape E: `check_subpattern` 12 [Ex12], `infer_match` 1 [Ex1], `infer_polymorphic_time_arith` 4 [Ex4]. `effectful_by_prefix` was not edited; the ledger dropped it because its callers hand it a door value.

`/tmp/census-25583-ab1.log` is the shrink 114 → 64 (`RC=101` until the ratchet). `/tmp/census-25583-ab2.log`: the frozen census matched 64 and `the_discriminator_reaches_the_tree_and_its_passes_converge` panicked at `tests/lint/keyword_heresy_ledger.rs:962` with `only 281 decision sites found`. This session's `== Some(":wat::…")` compares are invisible to the literal walker. They were rewritten to a bare `key == ":wat::…"`. The `> 300` floor was not lowered. `/tmp/census-25583-ab3.log`: 4 passed, 0 failed, `RC=0`. The site count after that rewrite was not printed.

### Gates

Census `.census/2026-10-02T21-01-01Z.txt`: files=2278, `RC=0`. Against `.census/2026-10-02T18-57-51Z.txt`, both sides are `0:2068, 1:208, 101:2`. Zero rc flips of 0 → nonzero. `scripts/replay/census.sh --diff` printed `census-diff: no STOP-8` (`/tmp/census-25583-stop8.txt`, `RC=0`). The positive fixtures were added after that census. They check (the probe log and the green floor). They were not a second census.

Clippy `cargo clippy --release --offline --all-targets -- -D warnings`. First run `/tmp/census-25583-clippy.log` `RC=101`: one `clippy::needless_borrow`, `spelling_key(&h)` where `h` is already a reference (`src/check.rs`). Fixed to `spelling_key(h)`. Second run `/tmp/census-25583-clippy2.log` `RC=0`, `Finished release profile [optimized] target(s) in 12.53s`.

Floor `.floor/2026-10-02T21-02-22Z` is red. It was not re-run. Summary from `ARM.txt`:

```
Summary [ 404.680s] 6388 tests run: 6387 passed (28 slow), 1 failed, 24 skipped
```

The arm is `assert!(violations.is_empty(), …)` at `tests/lint/no_inlined_wat_in_tests.rs:267`, test `no_inlined_wat_in_tests::tests_carry_no_inlined_wat`. The failing block:

```
        FAIL [   0.106s] ( 166/6388) wat::lint no_inlined_wat_in_tests::tests_carry_no_inlined_wat
  stdout ───

    running 1 test
    test no_inlined_wat_in_tests::tests_carry_no_inlined_wat ... FAILED

    failures:

    failures:
        no_inlined_wat_in_tests::tests_carry_no_inlined_wat

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 374 filtered out; finished in 0.10s

  stderr ───

    thread 'no_inlined_wat_in_tests::tests_carry_no_inlined_wat' (2082350) panicked at /home/john/work/holon/wat-rs/tests/lint/no_inlined_wat_in_tests.rs:267:5:


    🔥🔥🔥 INLINED-WAT IN TESTS — 2 file(s) still carry a string literal that wat's own
    reader parses as a form (surface-agnostic: rust-scheme `(:wat::core::…)` AND faithful
    Clojure `(wat.core/…)` both count).

    THE FIX — move the wat into a co-located `.wat` fixture and drive it lint-clean via ONE of
    two idioms. RUBRIC (which to reach for): docs/CONVENTIONS.md § 'Test idioms — EDN-over-stdio
    vs just-eval'. In short:
    • just-eval      — `call_beside_value(file!(), ":user::compute")`: run a fixture's named entry
    fn in-process, inspect its typed Result<Value, RuntimeError>. For a
    VALUE/TYPE claim (a fn's return; a compile-time/freeze property, which
    often needs only `startup_beside(file!())`, no call).
    • EDN-over-stdio — `run-hermetic` runs `:user::main` as a real process; it `println`s its
    result as EDN and the test `edn::read`s it back (lossless round-trip).
    For a PROGRAM claim (a crash/exit + reason, stdio effects, IPC fidelity,
    cross-loci behavior).
    One-line: 'the PROGRAM does X' -> EDN-over-stdio ; 'this VALUE/TYPE is X' -> just-eval.
    A legitimately-inline case (e.g. a parser/reader test) earns a per-site
    `// rune:lint(no-inlined-wat) — <reason>` (the reason must earn it).

    Drive it to ZERO. Literal-hit breakdown so far: 2 format!-driver, 8 faithful-surface,
    7 other parse-body. Offenders:

    tests/resolve/probe_arc255_83_differentials.rs
    tests/resolve/probe_arc255_83_quasiquote_router.rs
```

The wat moved into the co-located fixtures named above. No rune. `/tmp/census-25583-lint2.log`: `tests_carry_no_inlined_wat` ok, `RC=0`.

The new floor is `.floor/2026-10-02T21-14-00Z`. Launch log `/tmp/census-25583-floor2-launch.log`, `RC=0`. Summary:

```
Summary [ 405.023s] 6388 tests run: 6388 passed (29 slow), 24 skipped
```

The baseline `.floor/2026-10-02T19-08-53Z` at `581478c9c` was 6374 passed, 24 skipped. This floor ran 6388. Skipped stays 24.

STOP-1, STOP-2, STOP-3, and STOP-4 do not fire.
