# SCORE — STONE: `pprintln` is the one printer

No commit. Floor and clippy left to the orchestrator. `crates/wat-doc/` not touched. `wat/fmt.wat` not touched. No existing rule file.

`pprintln` of a `:wat::doc::Row` is the ruled form: `#wat.doc/Row {` on one line, `:doc` as prose (literal newlines, continuations at the content column), `:examples` dressed by fmt rules. Scope is the record class, not a flag, not a third verb. Capture is the binary's stdout.

---

## WITHDRAWN — `STONE-the-three-slot-emitters`. Reverted, not folded.

Halt reached mid-strike. Two files were dirty.

- **`crates/wat-doc/src/print.rs`** — the three slot emitters. **Reverted.** STOP-4: this stone does not touch wat-doc. `git diff crates/wat-doc/` is empty.
- **`wat/fmt.wat` tag-stitching** (`tag-symbol?` + `emit-tops`) — **reverted.** It solves a real defect (`#wat.doc/Row` orphaned from its map when a row is read back as source / `FORMS=2`), but **nothing on this path re-reads a row**, so it is not needed here. Untested. Not folded into this stone. Keep it as its own small stone if it stands; this SCORE does not carry it.

Untracked three-slot files (`print_tests__five_args.edn`, `print_tests__short_ret.edn`, `tests/cli/tagged_literal_is_one_form.rs/.wat`) were removed with the revert. Tree is clean of withdrawn work.

---

## Row 1 — it builds

`cargo test --release --test cli -- pprintln_doc_row` compiles and runs. `--check` of both fixtures is **0**. `every_ungated_wat_checks` (stdlib, including `wat/doc.wat`) **PASS**.

## Row 2 — the row prints as the ruled form

From a **record value** (`:wat::doc::from-map` of `metadata-of :wat::rete::step-payload`), stdout opens:

```
#wat.doc/Row {
  :doc "…
  …
}
```

Tag and brace on **one line**. Two-space entries. `}` alone. No tag string prepended onto a map.

The walker emits that opener because it has to intercept `:doc` and `:examples`. The spelling is the one a record already prints (`#ns/Name {` from `Aggregate` class `wat::doc::Row`). We did not stitch a tag onto a separately printed map — that was the withdrawn stone.

## Row 3 — `:doc` has REAL newlines, indented to the content column

Golden `tests/cli/pprintln_doc_row__step_payload.edn`:

```
  :doc "`(:wat::rete::step-payload …) -> :wat::rete::DerivationStep`

        Arc 278 Stone P12c — the explain payload builder. …
```

Blank prose lines stay blank (no margin spaces). Continuation lines indent to the content column (one past the opening `"`). Copy of `push_edn_string` (`crates/wat-doc/src/print.rs:104`); `print.rs` itself unmodified.

## Row 4 — `edn::read` still round-trips it

`printed_row_edn_read_round_trips`: `wat_edn::parse_owned` of the printed row **succeeds**. Literal newlines are the prose mode; the reader takes them back.

## Row 5 — `:examples` is in the RULED shape

Same golden. `defrecord`'s name rides:

```
        (:wat.core/defrecord :probe/StepPayloadExampleTemp
          [celsius <- :wat.core/i64])
```

`defrule`'s `:when`/`:then` broken:

```
        (:wat.rete/defrule :probe/step-payload-example-rule
          :when [(:probe/StepPayloadExampleTemp
                   (?c <- :celsius)
                   (:wat.rete.i64/< ?c 20))]
          :then [(:probe/StepPayloadExampleResult ?c)])
```

`let` binders paired one per line (blank binder-gap lines from `let-blank` captured as-is, trailing spaces included).

**Named gap, not a row-5 miss.** Slash-in-name field accessors print as `#wat.ast/Keyword {:path ":wat::rete::Explained/support"}` (the field-access / slash-in-name codec), not `:wat.rete.Explained/support`. Still parseable EDN. The rules dressed the forms they can see.

Routing: `collect-rules :fmt` is an **intrinsic**, not in `SymbolTable.get`. The walker `eval_inner`s `(:wat::fmt::format-source "<doc-row>" src (:wat::rete::collect-rules :fmt))`. Source fed to format-source is dotted EDN (`wat_edn::write(&watast_to_edn(ast))`); FQDN `ast->source` (`:wat::core::do`) is rejected by the EDN keyword lexer (`keyword begins with ::`).

`eval_edn_write-pretty` is unchanged (generic). `pprintln` / `epprintln` are the pretty printers.

## Row 6 — the prose mode is SCOPED, not global

`pprintln` of `:probe::Note` with `"line one\nline two"`:

```
#probe/Note {
  :text "line one\nline two"
}
```

Byte-identical to `pprintln_doc_row__note.edn`. Ordinary multi-line strings stay escaped.

## Row 7 — the scope is derivable from the VALUE

`write_pretty_wat_value` matches `Value::Aggregate` whose class is `"wat::doc::Row"`. No caller flag. No `pprintln-doc`. Two printers remain two.

## Row 8 — `:args` unchanged

One triple per line, same shape as `pprintln` of the metadata map already produced:

```
  :args [
    [:session :wat.rete/Session "the compiled session (network read via `session_network`)"]
    [:alpha_id :wat.core/i64 "the AlphaNode id for this condition"]
    …
  ]
```

## Row 9 — BYTE golden, diffed

`doc_row_pprintln_matches_byte_golden`: `assert_eq!` of stdout against `include_str!("pprintln_doc_row__step_payload.edn")`. Capture: `target/release/wat tests/cli/pprintln_doc_row.wat` → stdout. Not `assert_edn_matches_file!`. Not `io::write-file`. Not a shell decoder.

## Row 10 — negative control

Reverted the `:doc` arm to generic `append_pretty_field` (prose rendering off). `doc_row_pprintln_matches_byte_golden` went **RED** at `tests/cli/pprintln_doc_row.rs:32`.

Left (`:doc` escaped, one line):

```
  :doc "`(:wat::rete::step-payload session alpha-id bindings sfact supporting) -> :wat::rete::DerivationStep`\n\nArc 278 Stone P12c — the explain payload builder. …
```

Right (golden, real newlines, content-column indent):

```
  :doc "`(:wat::rete::step-payload session alpha-id bindings sfact supporting) -> :wat::rete::DerivationStep`

        Arc 278 Stone P12c — the explain payload builder. …
```

Prose restored. The three tests **3 passed** after restore. The committed control is the byte golden: un-arm `push_prose_string` and row 9 goes red. Not a scratchpad probe.

## Row 11 — `print.rs` is untouched

`git diff crates/wat-doc/` is empty. wat-doc crate tests **58 passed**.

## Row 12 — the corpus is untouched

`wat/deporder.wat` `wat/spawn.wat` `wat/io.wat` `wat/fmt.wat` — git status clean. Not touched. The EXPECTATIONS hashes (`c69f0460` / `1c4bbb19` / `ba89b65c` / `cae52502`) are a different instrument; this stone did not rewrite those files.

## Row 13 — floor (ORCHESTRATOR)

Not run.

## Row 14 — clippy (ORCHESTRATOR)

Not run.

---

## Other named cuts

- **`:wat::doc::of` is unused by the test.** Passing a registered function name as the arg type-checks as the function, not the keyword (`metadata-of` is special-cased; user defns evaluate registered names). Fixture is `from-map (Option/expect (metadata-of :wat::rete::step-payload) …)`.
- **Row fields are all `:wat::core::Value`.** Lifted from a heterogeneous `metadata-of` HashMap. Typed `String`/`Purity`/… failed get/Option. Same wall as the whole-row stone: do not annotate as HolonAST.
- **`--check wat/doc.wat` as a user file** fails reserved-prefix. Stdlib load (after `Record.wat`) is the path; `every_ungated_wat_checks` covers it.
- **`call_beside_value` cannot `load-file!` the fmt rules** (InMemoryLoader, relative path misses). The note control is a separate program without `load-file!`. The row fixture loads the twelve `wat-scripts/fmt/rules/*.wat` files then `pprintln`s.
- **`FORMS=2` is not on this path.** Named, not scoped.

## Targeted checks (executor)

```
cargo test --release --test cli -- pprintln_doc_row     3 passed
cargo test --release -p wat-doc -p wat-macros -p wat-edn -p wat-reader   clean
cargo nextest run --release --test lint                 119 passed, 0 skipped
target/release/wat --check tests/cli/pprintln_doc_row.wat       0
target/release/wat --check tests/cli/pprintln_doc_row_note.wat  0
```

## Files

```
wat/doc.wat                                      :wat::doc::Row + from-map + of
src/load/stdlib.rs                               load wat/doc.wat after Record.wat
src/services/verbs.rs                            write_pretty_wat_value; pprintln/epprintln
tests/cli/pprintln_doc_row.rs                    byte golden + round-trip + row 6
tests/cli/pprintln_doc_row.wat                   load rules, pprintln of the Row
tests/cli/pprintln_doc_row__step_payload.edn     row 9 golden
tests/cli/pprintln_doc_row_note.wat              row 6 control
tests/cli/pprintln_doc_row__note.edn             row 6 golden
```
