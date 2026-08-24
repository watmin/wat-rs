# BRIEF — a doc directive may wrap

A `///` line that does not start with `@` is currently **silently discarded** by both doc parsers. An
author who wraps an `@example` loses the second half and is never told, which is why the corpus carries
200-column one-liners. You will make a continuation line continue the directive — and nothing else.

Read `DESIGN-STONE-a-doc-directive-may-wrap.md` first. Copy the report shape of
`SCORE-STONE-the-last-comma-lives-in-a-symbol.md`.

⚠ **Check `git log -1` before you start.** This stone was queued behind another rider working in
`crates/wat-doc/src/lib.rs` (teaching the type check to use the reader). That work must be committed
first; if the tree is dirty in that file, STOP and report rather than building on an uncommitted base.

## STEP 1 — one helper, because there are two parsers

```
crates/wat-doc/src/lib.rs:265   pub fn parse(raw: &str)              recognized: … "@yields"
crates/wat-doc/src/lib.rs:567   pub fn parse_special_form(raw: &str)  recognized: … "@syntax"
```

Two copies of the line walk, and two copies of the recognized-tag list differing by one tag. **Do not
add the continuation rule twice.** One helper takes the raw doc lines and yields `(tag, payload)` pairs
with continuations already joined; both parsers consume it.

The termination rule — only the third row is new:

```
line starts with `@`   →  begins a new directive
blank line             →  ends the current one      (today's behaviour, preserved)
anything else          →  CONTINUES it, joined with a single SPACE
```

Wat is whitespace-insensitive, so a single space is correct for both an expression and a prose
description. Every existing directive arm consumes one `payload` string and must need **no change** —
if an arm needs changing, that is a finding worth reporting, not a detail to absorb.

⚠ **The unknown tag check must still fire.** Today an unrecognized `@foo` raises
`DocError::UnknownDirective`. A continuation line does not start with `@`, so it must not be mistaken
for a tag — and an unknown tag must not be silently swallowed as a continuation of the previous
directive. Row 4 is that.

## STEP 2 — controls, kept

Three tests, and the third is the one that matters:

1. A wrapped `@example` parses as ONE example with the joined payload.
2. A blank line ends the directive; following prose is not appended.
3. **A wrapped directive that was previously TRUNCATED now differs.** Write the negative control as it
   would have failed before: assert the joined payload contains the second line's content. Without
   this, the change is untestable and a later refactor can silently restore the discard.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★★ | a wrapped `@example` | one example; the payload contains BOTH lines' text |
| 2★★ | a wrapped `@arg` description | one arg; name/type unchanged, desc joined |
| 3★★★ | a blank line then prose | the prose is NOT appended to the previous directive |
| 4★★★ | an unknown `@foo` | still `UnknownDirective` — NOT swallowed as a continuation |
| 5★★ | every existing doc in the tree | still parses; no directive arm changed |
| 6★ | the 43 long lines | UNCHANGED — wrapping is now possible, not mandatory |

**Rows 3 and 4 decide it.** Row 1 goes green for a helper that joins everything to everything —
which would make a blank line meaningless and an unknown tag invisible. Only the terminators still
terminating proves the rule is a continuation rule rather than a concatenation.

## STOP triggers

- **STOP-1 — a directive arm needs changing** to handle a joined payload. Every arm takes one string
  today; if one does not, report which and why.
- **STOP-2 — joining changes an EXISTING doc's meaning.** Some doc in the tree may have prose after a
  directive that was being discarded and is now appended. Report every such site; that is real content
  that was invisible, and whether it belongs in the directive is a judgement, not a merge.
- **STOP-3 — the two parsers cannot share one helper** because their line handling genuinely differs.
  Report the difference rather than duplicating the rule.

## Boundaries

- `crates/wat-doc/src/lib.rs` and its tests.
- **Do NOT add style rules** — no column limit, no wrap policy, no reflow. That is wat-fmt's, and arc
  141 deferred it deliberately. Wrapped directives stay UNLINTED.
- **Do NOT rewrap the 43 long doc lines.** Wrapping becomes possible, not mandatory; a mechanical
  reflow needs the style rules this stone does not define.
- **Do NOT change any directive's grammar.**
- Do NOT commit, push, stash or amend. Keep the git index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- The orchestrator runs the full floor and clippy centrally.

Build with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000 cargo build --release`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## Your report

Rows 3 and 4 verbatim first — the blank line still terminating and the unknown tag still raising —
because those are the rows a concatenating helper fails. Then rows 1, 2, 5, 6. Whether one helper
served both parsers. Any site where joining surfaced previously-discarded content (STOP-2) — that
content was invisible and someone should look at it. Any STOP that fired, with the arm captured
verbatim BEFORE you diagnosed it. What surprised you.
