# Arc 163 — Slice 3e EXPECTATIONS

**Drafted 2026-05-07.** Pre-spawn predictions for sonnet's substrate
container-head FQDN sweep.

## Independent prediction

**Mode A.** ~30-50 minutes wall-clock.

Mechanical sweep across 5 short→FQDN substitutions. ~118 writes +
9 reads + 7 match arms + 1 canonicalize arm reshape + ~3-5 typealias
doc-comment rewrites. Per memory `feedback_simple_is_uniform_composition.md`:
N identical changes IS simple. The substitution is unambiguous
(quote-literal pattern + `.into()` suffix), so per-file
`replace_all: true` is safe.

The sweep is compiler-driven — once the canonicalize arm is reshaped,
all matchers and constructors must agree on the new FQDN form or
build fails. Compiler errors guide remaining edits.

## Hard scorecard

| Row | Pass criterion |
|---|---|
| R1 | Workspace pre-fix: 2041 passed / 0 failed (baseline) |
| R2 | Workspace post-fix: 2041 passed / 0 failed (no regressions) |
| R3 | `cargo build --release` exits clean |
| R4 | Audit grep `head: "Vec"\\|head: "Option"\\|head: "Result"\\|head: "HashMap"\\|head: "HashSet"` returns **0** Bucket A sites (live writes) — Bucket C/D residuals OK |
| R5 | Audit grep `head == "Vec"\\|head == "Option"\\|head == "Result"\\|head == "HashMap"\\|head == "HashSet"` returns **0** Bucket A sites |
| R6 | Canonicalize arm in `src/types.rs` (lines ~60-72) container-head match arms DELETED — only primitive arms (`:wat::core::i64` etc.) remain |
| R7 | Match arm in `src/runtime.rs:3661-3665` left side updated to FQDN keys; right side (value_tag strings) UNCHANGED |
| R8 | Renderer `format_type` at `src/check.rs:9817` emits `:wat::core::Vector<T>` (not `:Vec<T>`) for the new internal head — verify by reading any test assertion that pretty-prints types |
| R9 | Sonnet's report includes per-phase counts + at least 2 honest deltas |
| R10 | NO primitive-path edits (`":i64"` etc. unchanged — that's slice 3f scope) |
| R11 | NO value_tag-right-side edits (`value_tag == "Vec"` UNCHANGED — that's separate Value::type_name() system) |

## Path classifications

- **Mode A**: clean sweep, all rows pass, audit greps confirm. ~30-50 min.
- **Mode B**: sweep lands but with self-correction (e.g., touched a
  Rust-language `Vec` / `Option` outside-quotes site by mistake;
  caught by build error and reverted). Acceptable; flag in report.
- **Mode C**: build doesn't compile after sweep, OR audit grep R4/R5
  returns > 0, OR test count regressed, OR slice 3f scope violated.
  Sonnet stops + reports; orchestrator decides next step.

## Time-box wakeup

2× upper-bound = 100 min. Wakeup at T+100 min.

## Honest deltas to flag

- **If a substrate site has hybrid usage** (e.g., string `"Vec"` used
  as a head AND elsewhere as a value_tag in the same file): note the
  context to confirm only the head usage updated.
- **If test assertions hardcode legacy head strings** in expected-output
  comparisons: list them; orchestrator may need to update tests
  separately if sonnet's mechanical sweep doesn't cover them.
- **If there are >120 head-write sites** discovered: flag — the audit
  may have undercounted; the sweep is still mechanical but the report
  should state actual counts.
- **If renderer pretty-printing changes** show up in test diffs (e.g.,
  a test that compared `format_type(...)` output and now sees
  `:wat::core::Vector<T>` instead of `:Vec<T>`): list each test;
  orchestrator updates expected output.
- **If you encounter a head-string usage in NON-substrate code** (e.g.,
  an example or a doc-only file): note the location; classify as
  Bucket A/B/C/D and apply the appropriate action.

## What "done" looks like

After this slice, the substrate's wat-internal type representation
is FQDN throughout for these 5 container heads. Source FQDN flows
through unchanged; source bare forms still rejected by the walker
(unchanged). Reading any substrate file shows `head: "wat::core::Option"`
consistently — no mixed convention. The "internal bridge looks like
our form" inconsistency that user flagged 2026-05-07 is closed for
container heads.

Slice 3f then applies the same rule to primitive paths.
