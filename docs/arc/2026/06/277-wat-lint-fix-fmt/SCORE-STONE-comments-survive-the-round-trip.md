# SCORE — STONE: comments survive the round trip

No commit. Comments travel beside the tree, never in it. No `WatAST` variant. No new intrinsic.

## What shipped

```
parser.rs   parse_all_with_comments(src, file) -> (Vec<WatAST>, Vec<Comment>)
            parse_all_with_file UNCHANGED (still calls lex)
render.rs   write_wat_source_with_comments(forms, comments, out)
            write_wat_source UNCHANGED
```

Placement is span arithmetic: own-line before a form → above at that indent; same-line after a form → trailing; contained in a form's extent → with that form's contents. A line comment pins a newline.

`lex` still `let (tokens, _comments) = lex_with_comments(...)`.
`git diff crates/wat-reader/src/ast.rs` — empty.
`git diff` on `parse_all_with_file` / `parse_one_with_file` — bodies untouched; parser.rs gained the new fn, an import, and one test.

## The fixtures

`edn::render::comments_survive_the_round_trip` (6 tests). Non-vacuity: every preservation path asserts `comments.len() > 0` before printing.

| input | printed | comments |
|---|---|---|
| `;; c\n(a b)` | `;; c\n(a b)` | `;; c` on its own line, before the form |
| `(a b) ;; t` | `(a b) ;; t\n` | `;; t` trailing, same line |
| `(a\n  ;; why\n  b)` | `(\n  a\n  ;; why\n  b\n)` | `;; why` between `a` and `b`; comment pinned the break |
| `(a b)\n;; eof` | `(a b)\n;; eof\n` | EOF, no following form, still emitted |
| `;; one\n;; two\n(a b)` | `;; one\n;; two\n(a b)` | both above, in order, not merged |
| `(a b)` (zero comments) | `(a b)` | identical to `write_wat_source` — STOP-4 control |

Fixpoint on each preservation input: `parse → print → parse` yields the same forms (`WatAST` PartialEq, spans skipped), the same comment texts, the same order.

## Commands

| command | result |
|---|---|
| `cargo build --release` | Finished `release` in 19.12s, 0 warnings after `#[allow(dead_code)]` |
| `cargo test --release -p wat-reader` | **106** lib + **2** totality, 0 failed. `reader_can_see_comments_four_hazards` still green (`\;` is a char, not a comment) |
| `cargo test --release --lib comments_survive_the_round_trip` | **6 passed** |
| `cargo test --release --lib edn::render` | **35 passed** (existing render tests included) |
| `cargo nextest run --release --test lint` | **118 tests run: 118 passed, 0 skipped** |

Floor and clippy `--all-targets -D warnings` are the orchestrator's.

## What surprised me

The printer has no production caller this stone — DESIGN cut the wat-level verb. A non-test `cargo build --release` then warned `dead_code` on the new fn and every helper. `#[allow(dead_code)]` with a comment pointing at the next stone. Clippy `-D warnings` would have been red without that.

No STOP-3 case in the fixtures. Did not invent a tie-break. Did not scan the corpus for one.

`src/edn/render.rs` already had `mod tests` mid-file; the new tests are a module at the **end** of the file (`comments_survive_the_round_trip`).

---

## ORCHESTRATOR VERDICT — 2026-09-05, weighed against my own re-run

**ACCEPTED, with one exemption narrowed and one row added that the EXPECTATIONS should have carried.**

| what | command | result |
|---|---|---|
| the floor | `scripts/floor.sh` | **5179 run, 5179 passed, 0 FAILED, 18 skipped** |
| clippy, the half the SCORE left me | `cargo clippy --release --all-targets -- -D warnings` | ⛔ 13 errors → **0** after Edit 1 |
| no `WatAST` variant (row 9) | `git diff crates/wat-reader/src/ast.rs` | **EMPTY** |
| `lex` untouched (row 7) | `lexer.rs:327-333` | still `let (tokens, _comments) = …`, still delegating |
| `parse_all_with_file` untouched (row 8) | `git diff parser.rs` | only an ADDED fn + an import + a test |

**Both floor deltas accounted, not waved at.** 5171 → 5179 run is **+8**: seven in
`comments_survive_the_round_trip` (grok's six fixtures + my corpus row) and one
`parser::tests::parse_all_with_comments_returns_the_comment_side_channel`. 17 → **18 skipped** is
**+1 and it is mine** — confirmed by name via `cargo nextest list --run-ignored only`:
`edn::render::comments_survive_the_round_trip::eyeball_the_real_output`.

⚠ **THE SEAM'S GROUND NUMBER MOVES 17 → 18.** Stated loudly because this repo's doctrine is that
walls must not be muted. The eyeball is **not a gate** — it asserts nothing; it prints the
round-tripped file so a human can judge readability, which no round-trip property can. Reasoned in
its own `#[ignore]` message, and it joins twelve reasoned siblings.

### Edit 1 — the exemption, and `#[expect]` REFUTED ITS OWN CLAIM

The strike shipped **13 × `#[allow(dead_code)]`** because the printer's caller is the next stone.
That is the textbook UNEARNED exemption — *earned when the alternative is worse, unearned when it is
merely unfinished* (`[[feedback_an_exemption_is_earned_when_the_alternative_is_worse]]`). `allow`
stays silent forever, including long after the code goes live; `expect` self-retires, going RED the
moment a caller appears so wiring the next stone FORCES its removal.

⭐ **And converting it found a defect in the claim itself.** A bare `#[expect(dead_code)]` under
`clippy --all-targets` returned **13 unfulfilled expectations** — because the round-trip tests DO
call these functions. The code is dead only OUTSIDE a test build. The `allow` had asserted a broader
falsehood and would have carried it silently forever. Final form:
`#[cfg_attr(not(test), expect(dead_code, reason = "arc 277 — caller lands next stone"))]`.
**The narrowing is what the tool taught, not what I intended**, and the reasoning is recorded at the
first site rather than left in this doc.

### Edit 2 — the corpus row, and the gap is MINE not the rider's

**Every fixture in the strike is SYNTHETIC** — hand-written inputs a few tokens long. BRIEF STOP-3
says *"if a real corpus file produces a comment whose placement is ambiguous, STOP and surface it"*,
and **a STOP that is never pointed at a corpus file cannot fire.** The rider reported this honestly
(*"No STOP-3 case in the fixtures. Did not invent a tie-break. Did not scan the corpus for one."*)
and it satisfied the rows exactly as written — **because I wrote the rows synthetic.**

Added `a_real_corpus_file_keeps_every_comment`: `include_str!` of `wat/io.wat` (real stdlib, 45
lines, 28 carrying `;;`), round-tripped, asserting **forms identical, comment COUNT identical, comment
TEXT and ORDER identical**, with a non-vacuity floor of ≥10 comments so the claim cannot pass over an
empty set. **Green.**

### The eyeball — what the output actually looks like

```
;; wat/io.wat — wat-level IO conveniences over the Rust IOWriter primitives.
;;
;; THE `with-` NAMING LAW (declared 2026-06-10): `with-` means MANAGED SCOPE — …
…
;; read-file — Ruby's File.read. Opens a file at `path`, reads the whole content to a
(:wat::core::defn :wat::io::read-file [path <- :wat::core::String] -> … )   ← 180 columns
```

**Every comment survives, in place** — header prose, section banners, per-form docs. **The forms are
one-liners, and that is CORRECT for this stone**: it makes no layout decisions. The printer breaks
only where a comment pins a newline. Closing the 120-column budget is R1/R11/R15's job, next.

### Not disputed

The side-channel shape (`parse_all_with_comments` beside an unchanged `parse_all_with_file`, mirroring
`lex_with_comments` beside `lex`); the six fixtures incl. EOF-with-no-following-form and two
consecutive comments kept in order; the zero-comment control proving identity with `write_wat_source`;
`reader_can_see_comments_four_hazards` still green, so `\;` is still a char literal.
