# SCORE — STONE 218.7: the clj oracle runs to dry

Branch: `main`. **Committed, not pushed.** Stone `49c9900d6`.
Drawn against `53946dac3`. Parent brief: `BRIEF-STONE-218.7-the-oracle-runs-to-dry.md`.
Floor/clippy: orchestrator's row (not run). 8d not started. 219 not reverted.

## The ruling, as struck

`wat-edn` is spec-correct **EDN data**. `wat-reader` (arc 300) is the Clojure-dialect **code** reader.
The ward's old sentence *"wat must accept everything clj accepts"* is gone. Three categories:

| | meaning | example |
|---|---|---|
| `clj:OK / wat:OK`, `clj:ERR / wat:ERR` | parity | most rows |
| `clj:OK / wat:ERR` **and the construct is EDN** | **a wat bug** | `a:b` (219 stands, exempted) |
| `clj:OK / wat:ERR` **and NOT EDN** | **CORRECT** | `'x`, `^:m x` |
| `clj:ERR / wat:OK` | **a wat bug** unless exempted | `{:a 1 :a 2}` (fixed) vs unknown tag (exempt) |

## Bugs fixed (spec-correct EDN)

| input | before | after |
|---|---|---|
| `9223372036854775808` | ERR (i64 cap) | `Value::BigInt` |
| `123456789012345678901234567890` | ERR | `Value::BigInt` |
| `{:a 1 :a 2}` | accepted, **kept both** | `ErrorKind::DuplicateMapKey` |
| `#{1 1}` | accepted, **kept both** | `ErrorKind::DuplicateSetElement` |

`num-bigint` was already a dep; `Token::BigInt` / `Value::BigInt` already existed. The lexer now
promotes an i64 overflow of a digit-run instead of `InvalidNumber`. Maps/sets are still `Vec`;
the duplicate is rejected **before** push, so it no longer survives into the value.

`.wat` source is `wat-reader`'s, not this crate's. The two collection probes the 251 measurement
named (`Duplicate key: 1`) are unaffected by this crate becoming strict.

## Oracle wins, spec discrepancy reported

Doctrine: if `clojure.edn` and a reading of the spec disagree, **the oracle wins and we report it**.

| input | spec | clj | wat-edn now |
|---|---|---|---|
| `.5` / `.1` | silent on a missing integer part; symbol rule says if `.` is first, second must be non-numeric | OK `.5` | **float** (was: lex_symbol → refuse) |
| `5.` / `42.` | a present fractional part must have a digit | OK `5.0` | **float** (was: int 5 + leftover `.`) |

`comprehensive.rs` (`float_leading_dot_is_a_float`, `float_trailing_dot_is_a_float`) and
`spec_strict.rs` (`leading_dot_digit_is_a_float`) were retargeted from the old refusals.
`integer_overflow_errors` → `integer_overflow_promotes_to_bigint`.

## Spec-strict (clj is a superset) — CORRECT under the ruling

These are `clj:OK / wat:ERR` on purpose. Each `exemption()` names the clause.

| input | why we refuse |
|---|---|
| `'x` | quote is a Clojure reader macro, not EDN |
| `^:m x` | metadata is a Clojure reader macro, not EDN |
| `a/b/c`, `a/b/c/d`, `clojure.core//` | spec: `/` once only; neither prefix nor name empty. `value.rs` no longer claims `clojure.core//` is a spec exception. |
| `#inst "1985-04-12"` | spec wants RFC-3339; a bare date is not a timestamp. clj promotes to midnight UTC. |
| `\ ` (backslash + space) | spec: *"Backslash cannot be followed by whitespace."* clj reads it as `\space`. |

`` `x `` / `~x` / `~@x` / `@x`: **parity** — clj also refuses (`Invalid leading character`). They stay
in the corpus as negative controls. The exemption is belt-and-suspenders; the ward matches on
verdict first.

Unknown tags (`#myapp/Foo …`, `#myapp/Person …`): `clj:ERR / wat:OK`, already exempted
(*"read any and all edn"*).

## 219 — premise failed, ruling STANDS, blast measured, not reverted

`vocab.rs` now quotes the omitted sentence: `: #` ARE allowed as constituent characters other than
the first. `is_symbol_continue` still refuses them.

**Blast** (comment+string stripped, tracked `.wat`, 2190 files):

| class | tokens | files |
|---|---|---|
| keyword `::` wat-path (`:wat::core::i64`, …) | 154612 | 2145 |
| symbol `::` wat-path (`wat::core::i64`, `->wat::core::i64`, …) | **85** | **27** |
| real `a:b` / `a#b` symbol identifiers | **0** | **0** |

Re-permitting `:`/`#` as constituents is a cheap correctness win on **identifiers** (zero of them).
The 154k keyword `::` tokens are 8d's, not 219's. Builder rules before any revert.

## Corpus: 72 → 190. Loop to dry.

Generator: `crates/wat-edn/tests/clj_oracle/generate_corpus.py` (grammar enumeration + the original
72 as prefix + negative controls). Golden: `clj -M crates/wat-edn/tests/clj_oracle/regen.clj`
against `/usr/local/bin/clj` (Clojure 1.12.4). **No hand-written golden verdict.**

| round | action | new unexempted divergences |
|---|---|---|
| 1 | generate 72→190, regen golden, run ward | **4** — `.1`, `.5`, `5.`, `\ ` |
| 2 | `.`+digit → `lex_number`; trailing `.` as float; `\ ` spec-strict exemption (**do not `trim()`** — that ate the space and hid the row) | 0 leftover |
| 3 | regenerate: **190 identical**, golden identical | **0** — DRY |

Round 3 reconfirmed this scoring turn: `python3 generate_corpus.py` → `wrote 190 rows`; `diff` vs
pre-copy empty.

`exemption()` over-match tightened before commit: `{:` is not a 219 symbol, `#inst "not-a-timestamp"`
is not date-only, `::foo` is auto-resolve not constituent. Locked by `every_exemption_names_a_reason`.

## Live exemptions (divergence, not unused classification)

clj:OK / wat:ERR:

- `'x` — quote, not EDN
- `^:m x` — metadata, not EDN
- `a:b` `a#b` `foo:bar` `x#y` `:a:b` `:a#b` — 219 stands
- `a/b/c` `a/b/c/d` `clojure.core//` — `/` once only
- `#inst "1985-04-12"` — not RFC-3339
- `\ ` — backslash cannot be followed by whitespace

clj:ERR / wat:OK:

- `#myapp/Foo {:x 1}` `#myapp/Person {:first "F"}` — unknown tag, intentional superset

Duplicate-key rows are **parity** now (both ERR). Not exempted (`exemption("{:a 1 :a 2}").is_none()`).

## REPL — actual output, this machine, this turn

```
IN="clojure.core//" => OK clojure.core//
IN="a/b/c" => OK a/b/c
IN="a:b" => OK a:b
IN="a#b" => OK a#b
IN="{:a 1 :a 2}" => ERR Duplicate key: :a
IN="#{1 1}" => ERR Duplicate key: 1
IN="9223372036854775808" => OK 9223372036854775808N
IN="123456789012345678901234567890" => OK 123456789012345678901234567890N
IN="#inst \"1985-04-12\"" => OK #inst "1985-04-12T00:00:00.000-00:00"
IN="^:m x" => OK x
IN="`x" => ERR Invalid leading character: `
IN="~x" => ERR Invalid leading character: ~
IN="@x" => ERR Invalid leading character: @
IN=".5" => OK .5
IN="5." => OK 5.0
IN=".1" => OK .1
IN="\\ " => OK \space
IN="'x" => OK 'x
```

The brief asked for "the three REPL one-liners (5, 6, 7 above)" — those numbered items are not in
the struck brief (they lived in an earlier draft). The table above is the oracle set the brief
actually names.

## Harness limits (reported, not worked around)

- **One case per LINE.** A raw newline in the input (multi-line string, `;` comment then a form)
  cannot be a corpus row. `"tab\tnl\n"` is an escape sequence, not a newline in the file.
  `comprehensive.rs` already has `string_multi_line`; `spec_strict.rs` has `multi_line_string`.
  The ward is blind to that class; other tests cover it. Smallest fix if it starts to matter:
  a second NUL-delimited corpus, or a `\u000a` encoding decoded before `read-string`/`parse_owned`.
  Not done here.
- **`edn/read-string` reads ONE value.** Trailing content is not an error (`1 2 3` reads as `1`).
  Accept/refuse parity on multi-form input is **not** what this ward measures.

## Test count

Predicted delta: **+1** `#[test]` (`every_exemption_names_a_reason`). Four tests **renamed**, not added.

- `git grep '^#\[test\]' crates/wat-edn`: HEAD 303 → 304
- `cargo nextest list --release -p wat-edn`: **349** (Finished line excluded)
- `cargo test --release -p wat-edn --offline`: **349** unit+integration + **3** doctests, all pass

## Walls I ran (not the floor)

- `/usr/local/bin/clj` one-liners — pasted above
- `python3 crates/wat-edn/tests/clj_oracle/generate_corpus.py` — 190, idempotent
- `cargo test --release -p wat-edn --offline` — green
- `cargo nextest list --release -p wat-edn` — 349

Floor + clippy: **not run** (brief: orchestrator, uncontended). Do not push. Do not start 8d.

## STOP

None.
