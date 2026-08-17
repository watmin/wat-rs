# EXPECTATIONS — 294.k · a fabricated home is a lie

Written **before** the strike so the result cannot move the goalposts. Scored against the
orchestrator's **own** re-run, never the rider's report.

## ⛔ THIS STONE HAS TWO SUCCESSFUL OUTCOMES

Unusually, a **red floor is not a failure here.** The strike imposes a wall to discover whether a
branch is reachable. Score it as:

| outcome | verdict | what it means |
|---|---|---|
| floor **green**, grep 0 | **SUCCESS — dead arms** | the fabrication never fired; five sites gone, a wall in their place |
| floor **red**, every offender named verbatim | **SUCCESS — live arms** | the fabrication WAS firing and silently erasing identities. The offender list is the next stone and is worth more than a green run. |
| floor red, offenders summarised rather than quoted | **FAILURE** | `[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]` |
| a fallback re-added to make the floor green | **FAILURE** | that is the defect, restored |

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | `.local` gone | `grep -rn 'wat-edn\.local' src/ crates/ tests/ wat/ wat-scripts/ wat-tests/` | **0** |
| 2 | `.opaque` gone | `grep -rn 'wat-edn\.opaque' src/ crates/ tests/ wat/ wat-scripts/ wat-tests/` | **0** |
| 3 | ★ **differential** | the new test | `tag_from_type_path` ≡ `struct_tag_for` on every input, raise included |
| 4 | the raise names the path | read the error | the offending path appears in the message |
| 5 | negative control kept | `git status` | the raise's test is in the tree, not deleted after the write-up |
| 6 | floor | `scripts/floor.sh` → **Summary line** | green, **or** red with every arm quoted verbatim |
| 7 | clippy | `cargo clippy --release --all-targets` | **0** |
| 8 | waterline | `grep -rnE '^[[:space:]]*#\[ignore' … \| wc -l` | **13** |

Rows **3** and **6** are load-bearing. Row 3 is the mechanism that stops the mirror pair diverging
again; row 6 is the measurement the whole stone exists to take.

## Independent prediction

**Runtime: 15–30 minutes.** Basis: far smaller than 294.i (24 sites) or 294.j (72). Two functions,
seven call sites, one new differential test. The uncertainty is entirely in the floor's answer, not
the edit. **2× cap = 60 minutes.**

**Expected diff:** small and net-negative in `src/` — two fallback arms deleted, one raise added,
possibly a signature change threaded through seven call sites — plus one new test file.

## Trap-doors — named before the strike, so a hit is data rather than a surprise

1. **`tag_from_type_path` returns a bare `Tag`; `struct_tag_for` returns `(String, String)`.** Neither
   can express failure. The rider must choose `panic!` (house pattern, 294.j's encode wall) or
   `Result` threaded through seven sites, and **say which and why**. STOP-2 bounds it: a `Result` that
   escapes `edn_shim.rs` is not this stone's to push.
2. **The measurement is of the OBSERVED set, not the reachable set.** Every `type_path` grepped is
   `::`-separated — but a `type_path` can be built at runtime (`format!("{type_path}::{variant}")` at
   `:2545`). If the floor screams, that is the reachable set correcting the observed one, which is
   precisely what this stone was drawn to find out.
3. **A silent behaviour change.** Today an undeliverable path produces a *tag*; tomorrow it produces a
   *raise*. If some caller currently swallows the bad tag and carries on, the raise turns a silent
   wrong into a loud stop — correct, but it may surface in a test that looked unrelated. Read such a
   red as **the wall working**, and capture it rather than routing around it.
4. **The differential (row 3) may fail on day one** — the two functions may already disagree on some
   input beyond the fallback. That is not a reason to weaken the test; it is the third instance of
   this class in this arc and exactly what row 3 exists to expose. Report the disagreement.

## How this gets scored

Every row re-run by the orchestrator before any commit. The rider's report is a **hypothesis**; a
current `file:line` or a fresh Summary line is the only evidence that counts. Then one commit — strike
and test together, on a tree whose state is honestly described, green **or** red-with-the-offenders-named.
