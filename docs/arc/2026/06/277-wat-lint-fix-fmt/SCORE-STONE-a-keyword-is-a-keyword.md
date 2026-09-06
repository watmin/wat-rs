# SCORE — STONE: a keyword is a keyword, not a tagged value

No commit. Floor and clippy left to the orchestrator. `verbatim_keyword` not modified. `try_ns` not modified. `print.rs` not touched. Lands on top of the uncommitted `pprintln` stone — both stay uncommitted together.

The split was the bug. A name containing `/` now splits on that `/` (`receiver`/`method` on the door). `:wat::rete::Explained/support` renders `:wat.rete.Explained/support` and reads back as the same wat keyword. Seven tagged carriers leave the doc row.

---

## Row 1 — it builds

`cargo test --release --lib -- slash_in_name_keyword_renders_as_keyword_and_reads_back` compiles and runs. `--test cli -- pprintln_doc_row` **3 passed**. `--test wat_lang` **248 passed**. `--test lint` **119 passed**.

## Row 2 — a slash-in-name keyword renders as a KEYWORD

```
keyword_from_wat_path(":wat::rete::Explained/support")
  → Keyword ns="wat.rete.Explained" name="support"
  → ":wat.rete.Explained/support"
```

Not `#wat.ast/Keyword {:path …}`. Same for `:wat::holon::Hologram/make`.

## Row 3 — AND IT READS BACK AS THE SAME WAT KEYWORD

`edn_to_watast` of the written EDN yields `:wat::rete::Explained/support`. Same for `:wat::holon::Hologram/make`. `stop_trigger_slash_in_name_keyword` now notes the STOP is resolved: `:wat::core::HashMap/length` encodes `:wat.core.HashMap/length` and round-trips.

**Decode is not free.** `ns_to_wat_path` used to join with `::` always, which would have decoded `:wat.rete.Explained/support` as `:wat::rete::Explained::support` — STOP-1, strictly worse than the tagged carrier. The inverse is the same heuristic already in `crates/wat-macros/src/edn_doc.rs` `fqdn_of`: last namespace segment uppercase and the name *not* uppercase → join with `/`. Enum variants (`Purity/Pure`) and operators (`i64/<`) stay `::`. Split uses `identifier::leaf`/`path`, not a hand-rolled `rsplit`. BLAST RADIUS named "the split only"; the CONTRACT DECISION required this inverse. `try_ns` untouched.

## Row 4 — the doc row loses all seven

`tests/cli/pprintln_doc_row__step_payload.edn`, recaptured from the binary. `grep -c '#wat.ast/Keyword'` = **0**. The call is:

```
           support (:wat.rete.Explained/support ex)
```

The other six: `:wat.core.Option/expect` (×2), `:wat.rete.Support/token`, `:wat.rete.Token/matches`, `:wat.rete.Token/bindings`, `:wat.rete.Explained/session`. pprintln rows 2–8 from the previous SCORE stand — container, `:doc` prose, scope by record class.

## Row 5 — the verbatim carrier still EXISTS and still fires

`:a/b/c` — after `receiver`/`method` the ns still contains `/` (two slashes on the wire). `verbatim_keyword` fires. Tagged `#wat.ast/Keyword {:path ":a/b/c"}` parses and decodes back to `:a/b/c`. Function body unchanged.

## Row 6 — the change is CONFINED to the slash-in-name shape

`:wat::core::first` still renders `:wat.core/first` and reads back `:wat::core::first`. `:doc` still `:doc`.

## Row 7 — the corpus formatter is unmoved

`wat/deporder.wat` `wat/spawn.wat` `wat/io.wat` `wat/fmt.wat` — git status clean. The EXPECTATIONS hashes are a different instrument; this stone did not rewrite those files.

## Row 8 — every golden that moves is EXPECTED and enumerated

Each is the `X/y` shape (`#wat.ast/Keyword {:path "…Type/method"}` → `:Type.ns/method`). No bulk regenerate.

| golden | X/y |
|---|---|
| `tests/cli/pprintln_doc_row__step_payload.edn` | seven accessors (row 4); recaptured from the binary |
| `tests/wat_lang/wat_arc144_lookup_form__struct_head.edn` | `__internal/type-decl` |
| `tests/wat_lang/wat_arc144_hardcoded_primitives__length_primitive.edn` | `__internal/primitive` |
| `tests/reflection/wat_arc144_uniform_reflection__special_form.edn` | `__internal/special-form` |
| `tests/reflection/wat_arc144_uniform_reflection__primitive_empty.edn` | `__internal/primitive` |
| `tests/reflection/wat_arc144_uniform_reflection__type_defstruct.edn` | `__internal/type-decl` |
| `tests/resolve/probe_arc258_stone3_fix_source__contract-06-preserves-option-expect.wat` | `Option/expect` |

Needle (not a golden file): `tests/wat_lang/wat_arc144_special_forms.rs` `contains` of the sentinel, now the EDN spelling `:wat.core.__internal/special-form`. Same X/y. Existing `rune:lint(loose-assert)`.

No other `#wat.ast/Keyword` remains under `tests/`.

## Row 9 — the doctest gate still passes

`--test reflection` (includes `probe_arc255_ivb2b_verify_examples`) was in the passing group with wat_lang/resolve/program/cli. Exit **0**.

## Row 10 — negative control

Gated the `/` split off (`if false && stripped.contains('/')`). 

- `slash_in_name_keyword_renders_as_keyword_and_reads_back` → **RED** at `src/edn/render.rs:4815`: `expected Keyword, got Tagged(Tag { namespace: "wat.ast", name: "Keyword" }, Map([(:path ":wat::rete::Explained/support")]))`.
- `doc_row_pprintln_matches_byte_golden` → **RED**: left has seven `#wat.ast/Keyword` blocks; right is the recaptured golden with `(:wat.rete.Explained/support ex)`.

Split restored. Both tests **passed** after restore. Un-arm the `contains('/')` branch and rows 2–4 go red. Not a scratchpad probe.

## Row 11 — floor (ORCHESTRATOR)

Not run.

## Row 12 — clippy (ORCHESTRATOR)

Not run.

---

## Other named cuts

- **`__internal` decode.** Last ns segment is not uppercase, so `ns_to_wat_path` still joins with `::`. The goldens above are encode-only. A decode of `:wat.core.__internal/primitive` would currently yield `:wat::core::__internal::primitive`. Named, not papered over. The DESIGN's row-3 example (`Explained/support`) is the uppercase case and round-trips.
- **Two-slash guard.** After `receiver`/`method`, if the ns still contains `/`, we send to verbatim rather than emit unreadable `:a/b/c` text. `try_ns` does not refuse a slash in the namespace (STOP: `try_ns` is not modified).
- **Division** `:wat::core::/` — method is empty after the split; `try_ns` fails; verbatim. Unchanged.

## Targeted checks (executor)

```
cargo test --release --lib -- slash_in_name_keyword…     1 passed
cargo test --release --lib -- unspellable_keyword…       1 passed
cargo test --release --lib -- plain_keyword_bytes…       1 passed
cargo test --release --test cli -- pprintln_doc_row      3 passed
cargo test --release --test wat_lang                     248 passed
cargo test --release --test reflection/resolve/program   (with cli/wat_lang, exit 0)
cargo test --release -p wat-doc -p wat-macros -p wat-edn -p wat-reader  clean
cargo nextest run --release --test lint                  119 passed
```

## Files

```
src/edn/render.rs                                              the split + ns_to_wat_path inverse
tests/cli/pprintln_doc_row__step_payload.edn                   row 4 golden (on the pprintln stone)
tests/wat_lang/wat_arc144_lookup_form__struct_head.edn         row 8
tests/wat_lang/wat_arc144_hardcoded_primitives__length_primitive.edn
tests/wat_lang/wat_arc144_special_forms.rs                     sentinel needle
tests/reflection/wat_arc144_uniform_reflection__*.edn          three goldens
tests/resolve/probe_arc258_stone3_fix_source__contract-06-preserves-option-expect.wat
```
