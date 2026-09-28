# AMEND 3 — STONE 255.67: goldens are not programs (X-G), and the stone lands green

**Drawn 2026-09-28.** **Executor: a fresh Sonnet subagent.** Commit locally on `main`; **do not push**.

## The ruling (builder, 2026-09-28): X-G

A golden that holds an expected **output** (a form a codemod or a renderer produces, compared by a test) is not a program.
It carries a role suffix, **`<stem>__<case>.wat.golden`**, the same precedent as `.wat.bad`. That puts it outside
`git ls-files '*.wat'` (`scripts/replay/census.sh:47`), so neither the census nor a corpus codemod sees it.
(How goldens are *compared*, as data rather than text, is a separate question, still open. Do not change comparisons.)

## Why now

The floor at `.floor/2026-09-28T06-49-04Z` is `6212 passed, 1 failed`. The one failure is
`probe_arc283_1_rename_typearg::rename_reaches_type_arguments`: its golden
`tests/types/probe_arc283_1_rename_typearg__renamed.wat` is the expected output of `rename-keyword-prefix` on an
old-spelling input string, and the corpus conversion (`fd04778e4`) rewrote the golden's content. Stone 5 (the head
codemod) would do the same to every `.wat` golden.

## The work

1. **Find every golden** in the X-G sense: a `.wat` file read by a test as an **expected output**, compared against
   something the test produces (`include_str!`/`read_to_string` in `assert_eq!`, or an equivalent). 255.58's SCORE
   (`SCORE-STONE-255.58-measure-goldens-compared-as-text.md` § 1) lists 37 such `.wat` outputs; re-derive the list
   from the tree. A `.wat` that a test **loads and runs as a program** is not a golden.
2. **Rename each to `.wat.golden`** with `git mv`, and update the path in its test. Nothing else in the test changes.
3. **Restore `probe_arc283_1_rename_typearg__renamed.wat.golden`'s content** to its pre-conversion bytes
   (`git show fd04778e4^:tests/types/probe_arc283_1_rename_typearg__renamed.wat`), because that is what the codemod under
   test actually produces. For every **other** renamed golden that `fd04778e4` changed, check whether its test still
   passes. If it does, keep it as is. If it does not, restore its pre-conversion bytes the same way, and list it.
4. **Dead fixtures:** 255.58 measured six `tests/resolve/probe_arc251_keyword_to_type_form__contract-*.wat` that no test
   reads (`02-parametric`, `03-nested-parametric`, `04-type-var-bare`, `05-multi-arg`, `07-empty-tuple`,
   `08-nested-tuple`). Re-verify each is read by nothing (search the whole tree for its file name). Delete the ones
   confirmed unread, and list them.
5. **Gates:** the release floor (`scripts/floor.sh`), clippy, census `--diff` against `.census/2026-09-28T04-23-04Z.txt`
   (renamed files leave the census; name them), delta. Append to `SCORE-STONE-255.67-cutover-2-types.md`: the renamed
   list, the restored list, the deleted list, and every gate verbatim.

## STOP triggers (checked against the work list: none fires on a site it orders changed)

- **STOP-1:** a floor red that is not a renamed or restored golden's own test. Do not re-run it; quote the block
  verbatim and STOP.
- **STOP-2:** a file you cannot classify as golden vs program. List it and leave it as a `.wat`.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. If this amendment
contradicts the code, the code wins: say so. Commit with `git add -- <paths>` (and `git mv`), never `-A`. **Do not push.**
