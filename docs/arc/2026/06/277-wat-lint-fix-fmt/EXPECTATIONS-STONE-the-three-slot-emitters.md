# EXPECTATIONS — STONE: the three slot emitters

⚠ **Every row here is verified by DIFFING BYTES against a committed golden.** The existing golden is
compared *structurally* (`assert_edn_matches_file!`) and therefore passed while its `:doc` was a
one-line escaped string and `print`'s was real newlines. **A layout stone cannot be verified by a
structural compare.**

⚠ **And no row may be read through a shell decoder.** wat has no raw stdout; a decoder that
mishandles `\n` inside a string is what made this session's output untrustworthy. Capture with
`io::write-file` or a `println!` inside the test, and `diff` the file.

| # | what | expected — EXACT |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★★★ **`#tag {…}` is ONE form** | `#wat.doc/Row {:a 1 :b 2}` → **`FORMS=1`**, and the tag stays on the brace's line. Today it is `FORMS=2` with a blank line between. **This is the blocker; nothing downstream works without it.** |
| 3 | ★★★ **`:args`, one entry per line** | five lines, each `[name type "desc"]`, aligned. Today one line of ~430 columns |
| 4 | ★★★ **`:ret` breaks only when over budget** | the `step-payload` pair is 137 columns → breaks. A short `:ret` stays on one line. **Both cases, or the rule is "always break" wearing a budget's clothes** |
| 5 | ★★★ **`:examples` is NOT laid out by Rust** | `git diff` on `crates/wat-doc/` touches `print_args` and `print_ret` **only**. `print_examples` is unchanged. **A second formatter in `wat-doc` is the failure this stone is shaped to avoid** |
| 6 | ★★★ **the post-pass dresses `:examples` to the RULED shape** | `format-source` over the emitted row gives `defrecord`'s name riding, `let` binders paired one per line, `defrule`'s `:when`/`:then` broken |
| 7 | ★★★ **and the post-pass no longer orphans the tag** | after row 2, the formatted row still opens `#wat.doc/Row {` on one line |
| 8 | ★★★ **BYTE-FOR-BYTE golden** | a new golden diffed with `diff`, not `assert_edn_matches_file!`. **Row 8 is how rows 2-7 are proven; a structural compare passes all of them while the bytes are wrong** |
| 9 | ★★ `:doc` is untouched | real newlines, prose indented to the content column. `push_edn_string` is already correct — **a diff that moves `:doc` is a regression, not an improvement** |
| 10 | ★★ the round trip still closes | `from_metadata(print(doc))` still yields the same `DocComment`. **Layout must not change meaning** |
| 11 | ★★ the corpus is untouched | `deporder c69f0460e9f91672` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` · `fmt cae5250235f652ec` |
| 12 | ★★ negative control | revert `print_args` to one line → the byte golden goes **RED**. Committed, not a scratchpad probe |
| 13 | floor (ORCHESTRATOR) | `5199+` run, **0 FAILED** |
| 14 | clippy (ORCHESTRATOR) | `0` |

**Runtime prediction:** 70-120 min. Row 2 is the unknown — it is a reader/formatter question, not a
layout one, and it gates everything else.

## Trap-doors named in advance

- **Row 2 is a BLOCKER, not a row.** If a tagged literal cannot be made one form, the post-pass
  cannot run and rows 6-8 are unreachable. **Do that first and report before touching `print.rs`.**
- **Row 5 is the design.** `:examples` holds wat FORMS; laying them out in Rust mints a second
  formatter beside `wat/fmt.wat`. `one_name_grammar` exists for exactly that class and went red on
  this arc's `:then` stone for exactly that reason.
- **Row 8 is how everything else is proven.** The existing golden passed for weeks with `:doc` in the
  wrong shape because the compare is structural. Diff bytes.
- **Row 4 needs BOTH cases.** A `:ret` that always breaks satisfies the 137-column half and is not a
  budget.
- ⚠ **`print.rs` CANNOT call the formatter** — `wat-doc` is a build-time dependency of the
  proc-macro crate `wat-macros`, so no runtime exists at print time and adding one is a cycle. If a
  design needs that call, **STOP**; it is not available.
- ⚠ **Do not read any output through a shell decoder.** `println`/`pprintln` EDN-encode; only
  `io::write-file` is raw.
