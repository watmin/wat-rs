# EXPECTATIONS — the ```edn doc row is imposed

Written BEFORE the strike. Every bar derived from a measured fact or a stated rule.

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the fence is read, not ignored | delete a required key from the converted row's map | a **DocError naming it** — not silence, not a `panic!` |
| 2 | a bad tag is refused | `#wat.doc/Nonsense {…}` on the converted row | compile error naming the tag |
| 3 | the converted row is byte-identical | `metadata-of` + `render-doc`, before vs after | identical strings, character for character |
| 4 | every `@`-form row is untouched | registry census | **571 rows · 85 SpecialForm · 52 alias**, unchanged |
| 5 | no third decoder | `grep -c from_metadata crates/wat-macros/src/*.rs` | ≥1 — the fence path REACHES it |
| 6 | wat-macros' own tests hold | `cargo test --release -p wat-macros` | green |
| 7 | wat-doc's own tests hold | `cargo test --release -p wat-doc` | green |
| 8 | the floor holds, doctests included | orchestrator, centrally, once | 5139/5139, doctests exit 0 |
| 9 | clippy holds | `cargo clippy --release --all-targets -- -D warnings` | 0 |

Row 4's derivation: the census measured **571 / 85 / 52** at `18c534027`. Converting a row changes
HOW it is declared, never WHAT it declares — so any movement in row 4 means the conversion is not
faithful, and that is the finding rather than a number to update.

Row 1 is the honesty row. The entire value of a tagged record over `@name value` is that the shape
is *validated* instead of *scanned*. If a missing key produces silence, the new form buys nothing
and this stone has not delivered — no matter how green everything else is.

## Independent prediction

**40–70 minutes.** Parts 2 and 3 are small. Part 1 is the work, and the EDN→`WatAST` conversion is
where it lives: `from_metadata` expects a specific map shape and `wat_edn::Value` is a different
tree. That conversion is the stone.

## Trap doors — named before, not after

1. **The EDN→WatAST conversion may not be total.** `wat_edn::Value` has variants with no `WatAST`
   spelling. If a doc map needs one, STOP-1 fires and the design owes an answer. I do not know
   whether it is total, and I am not assuming it.
2. **`from_metadata` may accept a map shape the ```edn form cannot express**, or vice versa. The
   two forms must mean the same thing; row 3 is what would catch a divergence, on one row.
3. **The doctest gate is armed now.** An ```edn fence in a doc comment already survived it (probe
   b), but that was a fence in a `///` block with nothing else new. A fence inside a row the
   proc-macro also reads is a second situation.
4. **`@example` round-tripping.** The wat-side map already carries
   `:examples [["src" "expected"]]` and parses — but through the WAT reader. Through the EDN reader
   the strings must survive identically, quotes and `#=>` included. Row 3 catches it if the chosen
   row has an example, so **choose a row that has one**.

## What I will do on return

Re-run rows 1–7 myself before scoring; rows 8 and 9 are mine alone and are the only verdict on
green. The rider's numbers are a hypothesis until a current `file:line` confirms them.
