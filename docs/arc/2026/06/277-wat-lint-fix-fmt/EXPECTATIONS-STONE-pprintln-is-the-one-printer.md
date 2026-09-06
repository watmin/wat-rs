# EXPECTATIONS — STONE: `pprintln` is the one printer

⚠ **Capture output by running the binary on a script and reading stdout.** `pprintln` of a VALUE is
already unescaped — no decoder, no `write-file`. Believing otherwise is what produced the withdrawn
stone.

⚠ **Goldens are diffed as BYTES.** `assert_edn_matches_file!` is structural and passed for weeks
while `:doc` was in the wrong shape.

| # | what | expected — EXACT |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★★★ **the row prints as the ruled form** | `#wat.doc/Row {` on ONE line · two-space entries · `}` alone. **From a record value — no tag string prepended by hand** |
| 3 | ★★★ **`:doc` has REAL newlines, indented to the content column** | the prose reads as prose. Today: `"…`\n\nArc 278…"` on one line |
| 4 | ★★★ **and `edn::read` still round-trips it** | `edn::read` of the printed row yields an equal value. **This is the safety row — literal newlines violate strict EDN, and the whole prose mode rests on our reader taking them back** |
| 5 | ★★★ **`:examples` is in the RULED shape** | `defrecord`'s name rides · `let` binders paired one per line · `defrule`'s `:when`/`:then` broken. Today: generic EDN, one element per line |
| 6 | ★★★ **the prose mode is SCOPED, not global** | an ordinary multi-line string elsewhere in the corpus renders as it does today. **A global change is a far wider stone than a doc row, and row 4 tolerating it is not a licence** |
| 7 | ★★★ **and the scope is derivable from the VALUE** | not a caller-passed flag, not a `pprintln-doc` sibling. **Two printers is the thing this stone exists to stop becoming three** |
| 8 | ★★ `:args` unchanged | one entry per line — already correct, and a diff that moves it is a regression |
| 9 | ★★★ **BYTE golden, diffed** | committed, `diff`ed. **Rows 2-5 are proven by row 9; a structural compare passes all of them while the bytes are wrong** |
| 10 | ★★ negative control | revert the prose rendering → the byte golden goes **RED**. Committed |
| 11 | ★★ `print.rs` is untouched | `git diff crates/wat-doc/` is empty. **It serves the proc-macro path; this stone is the runtime path** |
| 12 | ★★ the corpus is untouched | `deporder c69f0460e9f91672` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` · `fmt cae5250235f652ec` |
| 13 | floor (ORCHESTRATOR) | `5199+` run, **0 FAILED** |
| 14 | clippy (ORCHESTRATOR) | `0` |

**Runtime prediction:** 70-120 min. The record and the container are free (measured). The prose mode
and its scoping are the work; `:examples` is a call into rules that already produce the right shape.

## Trap-doors named in advance

- **Row 6/7 are the design.** The easy build is a second verb or a flag — and then the corpus has
  three printers instead of two. The scope must fall out of the value.
- **Row 4 is why row 3 is allowed at all.** Literal newlines are not strict EDN; the only reason this
  is legitimate is that `edn::read` round-trips them. **If a change makes that stop holding, STOP.**
- **Row 5's rules are reachable from `pprintln` and NOT from `print.rs`** — the runtime exists at
  print time here. That asymmetry is the stone's whole reason for pointing this way.
- **Row 9 is how the rest is proven.** Every wrong claim in the session that produced this stone came
  from measuring something adjacent to the artifact.
- ⚠ **The container is FREE — do not build it.** A record already prints `#ns/Name { … }`; verified
  with a two-field probe. If a design starts prepending a tag string, it has gone wrong.
- ⚠ **`FORMS=2` does not apply here.** That blocker is about *re-reading* a row as source; nothing on
  this path re-reads. Do not import it.
