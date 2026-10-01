# BRIEF — STONE 255.80: the recorded codemods reach wat inside Rust string literals

**Drawn 2026-10-01 against `main` @ `9fe5b47d8`.** **Executor: a Sonnet subagent, working solo** (it runs the floor; pass
`timeout: 600000` on that Bash call, and on any build or codemod run that may pass two minutes, or the harness moves it to
the background after 120 s). A tool, its probe, one application, tests. Commit locally on `main`
(`git add -- <paths>`, never `-A`); **do not push**. Your final message is your report.

## The rulings (builder, 2026-10-01)

- **R1:** the 1,093 type-position sites in wat embedded in Rust (255.79's STOP-1) move by the **same recorded codemod**
  as the `.wat` corpus, not by hand. *"We have been planning for this kind of tooling for months."* This supersedes arc
  109's note that wat in a Rust string is hand-edited (`109-kill-std/BRIEF-STONE-the-match-arm-RELAND-2-…:69`).
- **X2:** no new Rust parsing library. The one literal reader is the tested extractor in
  `tests/lint/no_inlined_wat_in_tests.rs` (`extract_string_literals`, `:154`; escapes, raw strings with `#` counts, `\`
  line continuations, nested block comments, char literals vs lifetimes; it decides "is this wat" with wat's own reader,
  `is_inline_wat_form`, `:80`). Factor it into **one shared module** that also records, for every decoded character, its
  raw source offset; the lint and the rewriter both use it.

## Why it matters past this stone

The embedded corpus is **8,447 forms across 363 `.rs` files, 7,099 in `src/`**
(`251-types-as-forms/BRIEF-STONE-251.8d-ii-FOURTH-the-bootstrap.md:114`), and it is what keeps 251's terminal "keyword
heads illegal" cut unruled. Cutover stones 4b, 5 and 7 need the same reach. Build it as general tooling: it must run
**any** recorded codemod, not only this one.

## The work

1. **Probe first (FM 2-bis), committed before the tool:** take one wat-shaped literal from `src/macros/tests.rs` that
   contains at least one escape (`\"` or `\n`) or a `\` line continuation and a type-position `:wat::core::<24>`. Decode
   it with the extractor, write the decoded text to a temp `.wat`, run `wat-scripts/fixes/types-to-wat-type.wat` over it
   (paths on stdin, as today), diff old against new decoded text, map each changed region to raw offsets, check that the
   raw slice is exactly the old text, splice, and show the resulting Rust compiles and the literal decodes to the
   codemod's output. If this composition fails, **STOP-1**.
2. **The shared module.** Move the extractor out of the lint into one module both can use (a `#[path]` include, a small
   crate under `crates/`, or a `pub(crate)`/test-support module: say which and why), add the decoded→raw offset map, and
   point `no_inlined_wat_in_tests` at it with its unit tests unchanged and passing. **`format!` templates:** a `{…}`
   placeholder is replaced by a **same-length** stand-in before the codemod runs (the lint's `replace_placeholders`
   changes length; an offset-preserving variant is needed), and `{{`/`}}` escapes are respected.
3. **The driver,** recorded beside the codemods (e.g. a `src/bin/` tool or a `scripts/` entry; say which): given a codemod
   and a list of `.rs` paths, it extracts every wat-shaped literal, runs the codemod over all of them in one batch, and
   splices the edits back. It refuses to splice any region whose raw slice does not equal the old text (escaped content
   inside a changed region), reporting it instead. Dry-run mode prints the per-file edit count and a diff. Idempotent.
4. **Apply** `types-to-wat-type.wat` through the driver to `src/**` and `tests/**`. Census before (the 1,093 sites in 84
   files, re-derived) and after (the residue, classified like 255.79's).
5. **Tests:** the extractor's offset map (escape, raw string, continuation, placeholder, `{{`); a splice that must be
   refused; the driver's idempotence on a fixture `.rs`.

## Gates

| what | how | expected |
|---|---|---|
| the probe | its own run | the literal rewritten, the Rust compiles, the literal decodes to the codemod's output |
| the lint unchanged in verdict | `cargo nextest run --release -E 'test(/no_inlined_wat_in_tests/)'` | green, same unit tests |
| idempotent | the driver again over the converted files | 0 edits |
| residue | the census in item 4, re-run | no type-position `:wat::core::<24>` left in embedded wat; refused splices listed |
| release floor | `scripts/floor.sh`, **in the foreground, `timeout: 600000`**, nothing else running | all passed; the count against 6349 at `817e2003f` (`.floor/2026-10-01T20-18-57Z`), plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change (a converted literal, a golden or assertion whose only change is the spelling,
  its own tests): capture it **verbatim** from `.floor/<stamp>/`, cure it, run a **new** floor. Never re-run unchanged
  code for a green.
- **STOP-1:** the probe's composition fails (the codemod cannot run over a decoded literal, or the edits cannot be mapped
  back faithfully). Report exactly where and STOP before building the tool.
- **STOP-2:** a converted literal changes what its test asserts or computes (beyond the spelling). Quote it and STOP.
- **STOP-3:** more than 50 refused splices. List them and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. The driver edits `.rs` files only through the recorded codemod's output, never a regex of
its own. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Run every build, floor and clippy in the
foreground and block on it. Never write a number, file:line or example you did not measure. If this brief contradicts the
code, the code wins: say so. Write `SCORE-STONE-255.80-codemods-reach-wat-in-rust-strings.md` beside this brief, commit
it, **do not push**.
