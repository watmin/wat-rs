# NOTE — `TestSummary`: `passed + failed` can exceed `total`

**Found 2026-09-06** by the arc 278 rete census sweep, which picked it up as section J of
`docs/arc/2026/06/278-rules-engine/vigilia-2026-09-05/recon/census-name-audit.md`.

⛔ **NOT worked there — it is out of scope.** That session is rete-only by the builder's ruling, and
`src/test_runner.rs` is the `wat test` runner, not rete. Written down so the finding is not lost
with the sweep that found it.

**Parked in arc 109 at the builder's direction**, which is where this tree collects substrate
findings that are surfaced but not worked. The code's own arc is
`docs/arc/2026/04/015-wat-test-for-consumers` (`test_runner.rs` is its slice 1) — that is where a
cure would land; this note is only the record that the defect is known. There is deliberately no
second copy over there: two copies is how two files drift.

## The defect

`total` counts **tests**; `failed` counts **tests AND file-level errors**. They are not over the
same population, so `passed + failed` can exceed `total`.

- `src/test_runner.rs:260` — `summary.total += discovered.len();` — discovered tests only.
- `failed` is incremented for things that are not tests:
  - `:189` — a directory read error
  - `:219` — a file read error
  - `:255` — a freeze error
  - and genuinely per-test at `:301`, `:308`, `:319`, `:333`.

## What a user sees

`:270` announces `println!("running {} tests", summary.total);` and `:354-355` then reports
`"test result: {}. {} passed; {} failed"`. With one unreadable file and three passing tests, the
run announces **3 tests** and reports **3 passed; 1 failed** — four results from three tests. The
arithmetic does not close, and the reader has no way to tell which of the four was not a test.

## Why it matters beyond tidiness

A CI step that derives a rate — `failed / total`, or asserts `passed + failed == total` — is wrong
whenever a file-level error occurs, and it is wrong in the direction that **understates** how much
did not run: a file that failed to read contributed zero to `total`, so every test it contained is
invisible to both numerator and denominator.

## Shape of a cure (not prescribed here)

The two populations want separating rather than a bigger `total`: file-level failures are a
different kind of event from a failing test, and the display already has to explain which is which.
`no_tests_discovered` (`:85-89`) is the existing precedent in this struct for "a distinct outcome
gets its own field rather than being folded into a count."

Whoever takes it should decide whether `total` grows to mean "results expected" or whether a
`file_errors` field joins the struct — this note deliberately does not choose.
