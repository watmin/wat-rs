# SCORE — STONE 255.80: the recorded codemods reach wat inside Rust string literals

**Executor: a Sonnet subagent, working solo** (ran the floor itself). Drawn against local `main` @
`9fe5b47d8`, the brief's own draw commit `84a9eb067`. Commit locally on `main`; **not pushed**.

## 1. The probe (FM 2-bis) — committed before the tool

The brief asks for a literal in `src/macros/tests.rs` carrying an escape/line-continuation AND a
type-position `:wat::core::<24>` token. **Measured, not assumed:** a full character-by-character
scan of that file found exactly 3 backslash characters total (one `\` line continuation at line 403,
one `\"` pair at line 1360), neither anywhere near a type marker — the brief's premise does not hold
for that file. Per doctrine ("the code wins, say so"), the probe used the nearest real, non-hypothetical
analog in the same module — `aggregate_kwargs_companion_source` in `src/macros/parse.rs`, whose
`format!` template carries three `\` line continuations, a `\"`-escaped `{bare_name}` placeholder, two
`:wat::WatAST` type-position sites and a `:wat::core::Vector` head — strictly harder than the brief's
ask.

Composition proven end to end: decode the literal → substitute `{…}` placeholders with a same-length
`x`-run (so the offset map still lines up) → run `types-to-wat-type.wat` over a real temp `.wat` file via
`./target/release/wat ./wat-scripts/fixes/types-to-wat-type.wat` with the path on stdin (the documented
invocation) → diff old/new by parsing both with wat's own reader and walking the trees leaf-by-leaf
(isomorphic by construction, since `fix-text-apply` only ever rewrites a leaf's span text) → map each
changed leaf back to raw offsets via the decoded→raw char map → verify the raw slice is char-for-char
the old decoded text (none of the three edits sat inside an escape, so none were refused) → splice into
a copy of the real file → the result is syntactically valid Rust → the spliced literal, re-decoded,
equals the codemod's own output. **No STOP-1.**

(That exact literal is now converted for real by item 4's apply; the committed probe test —
`src/codemod_driver.rs`'s `probe_255_80` module — now asserts the post-conversion invariant on the same
literal: idempotent, decode→run→diff finds nothing left.)

## 2. The shared module — `wat::embedded_wat`

A `pub mod` inside the `wat` library crate (`src/embedded_wat.rs`), not a new `crates/` member and not
`pub(crate)`: the lint lives in the separate `tests/lint/` integration-test crate and needs public
visibility to reach it; a new workspace crate would be churn for one extractor. Factored out of
`no_inlined_wat_in_tests.rs`'s `extract_string_literals`/`decode_escape` (moved verbatim) and extended
with:

- `LiteralSpan { decoded, char_map, raw_quote_start, raw_quote_end, is_raw_string }` — for every decoded
  char, the raw `[start,end)` char range in the source it came from (an escape's whole raw sequence for
  one decoded char; a line continuation emits no decoded char and no map entry, so a splice region that
  swallows one automatically fails the raw-equals-decoded check downstream — no separate gap-tracking).
- `replace_placeholders_preserving_len` — a length-preserving sibling of the lint's own 6-char `__ph__`
  shrinker (which breaks offset alignment); `{{`/`}}` are left untouched, a lone `{…}` becomes a
  same-length `x`-run.

`no_inlined_wat_in_tests.rs`'s `extract_string_literals(src: &str) -> Vec<String>` is now a thin wrapper
(`extract_literal_spans(..).map(|s| s.decoded)`) — same name, same signature, same behavior; its own
unit tests needed no change (verified green, below).

## 3. The driver — `src/bin/wat-fix-rust.rs` ([[bin]] `wat-fix-rust`)

General tooling, not special-cased to this codemod: `wat-fix-rust <codemod.wat> [--dry-run]
[--wat-binary <path>] (--list <file> | <path.rs>...)`. Per **.rs file** (not per literal, and not one
giant cross-file batch): extracts every literal, filters to wat-shaped candidates
(`wat::codemod_driver::is_candidate_wat` — the same surface-agnostic List-headed-by-Keyword/Symbol
contract `is_inline_wat_form` uses), writes each candidate to its own temp `.wat`, runs the codemod
**once** over all of that file's candidates in one subprocess call ("one batch" scoped to a file —
bounds the blast radius of one malformed literal to one file, not the whole corpus run), then for each
changed candidate: parses OLD/NEW decoded text and walks both trees leaf-by-leaf
(`codemod_driver::diff_decoded`) to get exact per-token edits, maps each through the char map and
**refuses** (reports, never applies) any whose raw slice disagrees with the old decoded text
(`verify_and_map`), and splices every verified edit into the real file in one pass
(`splice_raw`). Dry-run prints each file's edit/refusal count and every `old -> new` change; it never
writes a `.rs` file through anything but the codemod's own output.

## 4. Apply `types-to-wat-type.wat` to `src/**` and `tests/**`

**Census before:** the brief's own 1,093-marker-adjacent-site / 84-file grep census (255.79's method) is
a rough, hand-inspected approximation (it itself undercounts/miscounts — see 255.79's SCORE §1, "this
undercounts the true type-position-site count"). The driver's own ground-truth scan over every tracked
`.rs` file (1,217 at apply time) is the authoritative number: **570 edits across 54 files, 0 refused**,
dry-run and apply agreeing exactly.

**A floor red this stone's own apply caused — captured, diagnosed, cured, never papered over.** The
first floor after the apply (`.floor/2026-10-01T21-28-56Z`) came back:

```
Summary [ 385.611s] 6362 tests run: 6338 passed (27 slow), 24 failed, 24 skipped
```

24 failures, three distinct mechanisms, all traced to specific literals before anything was touched a
second time:

**(a) The project's own loose-string-assert lint (`no_loose_string_assert`), 7 sites** — my own new code
(`src/codemod_driver.rs`, `src/embedded_wat.rs`) used `assert!(x.contains("…"))` where the convention
wants `assert_eq!` or a `rune:lint(loose-assert)` exemption. Tightened two (the idempotence test's whole
small fixture file, and the `{{`/placeholder regression test) to exact `assert_eq!`; the remaining four
(targeted presence/absence checks against a large real production literal, where asserting the whole
thing verbatim would couple the test to unrelated future wording of `aggregate_kwargs_companion_source`)
carry a per-site `rune:lint(loose-assert)` with reason. Verified green.

**(b) `check::tests::{bundle_of_list_of_holons_passes,list_mixed_types_rejected}`** — captured verbatim
(`.floor/2026-10-01T21-28-56Z/ARM.txt`):

```
thread 'check::tests::bundle_of_list_of_holons_passes' panicked at src/check.rs:25509:9:
assertion failed: check(r#"(:wat::holon::Bundle (wat.type/Vector :- [:wat::holon::HolonAST]
                 (:wat::holon::to-holon 1)
                 (:wat::holon::to-holon 2)))"#).is_ok()

thread 'check::tests::list_mixed_types_rejected' panicked at src/check.rs:25499:77:
called `Result::unwrap_err()` on an `Ok` value: ()
```

Root cause (not the conversion — a pre-existing harness drift it exposed): `check.rs`'s test-only
`check()` helper's own doc comment claims it "delegates to the canonical pipeline so the test
environment CANNOT drift from production," but it skipped `freeze.rs`'s documented step 7
(`normalize_symbol_refs`, run *before* `check_program`). Without it, a Symbol-headed `wat.type/Vector`
call never reaches `infer`'s `if let WatAST::Keyword(k, _) = head` dispatch table (the `:wat::core::Vector`
arm, which does element-type verification) — only a Keyword-headed `:wat::core::Vector` call did. A real
program never hits this: production always normalizes before anything downstream sees the AST; only this
hand-assembled test helper skipped it. **Cure** (`src/check.rs`, `tests` mod): added the missing
`normalize_symbol_refs(rest, &sym, &macros)` call in the same position `freeze.rs` runs it. Re-verified:
all 91 `check::` tests green, no other regression.

**(c) 21 further failures across 14 files** — NOT a checker bug. Every one is a literal whose *job* is
to byte-match something that itself stays old-spelled, which the apply's type-position rule cannot
distinguish from genuine source:

- **`src/lower.rs`, `tests/kernel/mvp_end_to_end.rs`** (2 failures) — `lower()` is called directly on raw,
  un-normalized AST ("bypassing the checker," per `lower.rs`'s own test comment). A Symbol-headed
  `wat.type/Vector` never reaches `lower()`'s Keyword-only dispatch, so `LowerError::BundleShape` fired
  where the Keyword spelling worked. No production path is affected (same reason as (b)).
- **`src/intrinsic/mod.rs`** (1 failure) — `typeexpr_to_doc_string`'s `Tuple` arm is a `format!` template
  whose one job is to render text matching `reckoner.rs`'s hand-written `///` doc comment
  (`@ret (:wat::core::Tuple :- …)`) for a cross-check test. Comments are out of scope for this stone
  (255.79's own "stone 7" residue class) — so the renderer must keep tracking the comment's current
  spelling, not the canonical type position.
- **12 files** (`tests/comms/probe_arc214_stone46b_select_prime.rs`,
  `tests/resolve/probe_arc251_parametric_target.rs`,
  `tests/services/probe_arc255_24_defservice_declares_what_it_emits.rs`,
  `tests/types/probe_arc214_stone46i_typed_peer.rs`, `probe_arc255_{18,19,27,28,48,52}…`,
  `probe_stone118_3b_seqable_parametric_satisfaction.rs`, `probe_stone_118_b1a_neg.rs`) — a
  `matches!`/`assert_eq!` guard's *expected* string, compared against a real `CheckError`'s
  `expected`/`got` field as rendered by `format_type` — confirmed by reading `format_type`
  (`src/check.rs:18545`): its `Path`/`Parametric` arms still emit `:wat::core::X` for every ordinary
  type, untouched by this stone. Converting the guard's literal broke the match against the
  (correctly-unmigrated) rendered message.

**Cure:** reverted exactly these 15 files to their pre-apply spelling —
`git checkout e28d03236 -- <path>` (the probe commit, predating any corpus apply) — not a revert of the
stone's work, a narrowing of the apply set: **39 files / 506 edits** (570 − 64 across these 15), 0
refused. Verified file-by-file before the fresh floor: all 62 `tests/types`/`tests/resolve`/
`tests/comms`/`tests/services` tests named above green, plus `lower_bundle`,
`bind_vs_bundle_of_same_atoms_differ`, `doc_arg_ret_types_match_checker_scheme` green.

**Audited for the same shape elsewhere:** grepped every remaining converted file's diff for
`expected ==`/`got ==`/`ReturnTypeMismatch`/`TypeMismatch`/`typeexpr_to_doc_string` context near a
changed line. Three more hits (`src/edn/render.rs:1222`, `src/intrinsic/reflect.rs:847`,
`src/reflect/verbs.rs:1587`) — all a hardcoded `RuntimeErrorKind::TypeMismatch { expected: "…", … }`
*production* diagnostic string, not a test-comparison target; no test anywhere asserts that exact string
(confirmed by grep), so nothing requires it to match `format_type`'s current output. Left converted.

## 5. Tests

- Extractor offset map: escape, raw string + hash-count, line continuation (no decoded char, no map
  entry), `{{`/`}}` escapes, same-length placeholder substitution — `src/embedded_wat.rs` `tests` mod,
  9 tests.
- Driver: idempotence on a fixture `.rs` (`driver_is_idempotent_on_a_fixture_rs_file`), a splice that
  must be refused (`driver_refuses_a_splice_when_an_escape_hides_inside_the_changed_span` — a `\x3a`
  hex-escaped `:` inside a type-position keyword's own span), ordinary Rust source left untouched —
  `src/codemod_driver.rs` `driver_tests` mod, 3 tests.
- The probe itself, 1 test (now a post-conversion idempotence regression on the same real literal).

13 new tests total — matches the floor delta exactly (6349 → 6362, §6).

## Gates

| gate | command | result |
|---|---|---|
| the probe | its own run (`src/codemod_driver.rs::probe_255_80`) | composition proven; no STOP-1 (now re-asserts post-conversion idempotence on the same literal) |
| the lint unchanged in verdict | `cargo nextest run --release -E 'test(/no_inlined_wat_in_tests/)'` | `Summary [ 0.057s] 10 tests run: 10 passed, 6373 skipped` — same unit tests, same green verdict |
| idempotent | the driver again over the final 1,218-file tracked `.rs` list, dry-run | `15 file(s) changed, 64 edit(s)` — **exactly and only** the 15 deliberately-excluded files; every one of the 39 applied files: 0 edits |
| residue | the 15-file exclusion census above, re-derived; refused splices | 0 refused anywhere, both apply and this dry-run |
| release floor (first run, RED) | `scripts/floor.sh`, foreground, `timeout: 600000` | `.floor/2026-10-01T21-28-56Z`: **24 failed** — captured verbatim in §4, cured, never re-run before capture |
| release floor (fresh run after the cure) | `scripts/floor.sh`, foreground, `timeout: 600000` | `.floor/2026-10-01T21-47-14Z`: **`Summary [ 388.019s] 6362 tests run: 6362 passed (27 slow), 24 skipped`**, exit 0 — 6349 baseline (`817e2003f`) + 13 new tests, exact |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- **No STOP-1.** The probe composed cleanly (§1).
- **No STOP-3.** 0 refused splices across the whole apply and every re-check.
- **A red this stone's own change caused (§4):** 24 floor failures, three mechanisms, all diagnosed to
  an exact arm/literal before anything was re-run; the first floor run is the untouched, verbatim
  capture. Cured: (a) the project's own loose-assert lint on my new code, tightened/exempted; (b) a
  pre-existing test-harness drift from production (`check()` missing `normalize_symbol_refs`) the
  conversion exposed, fixed to match `freeze.rs`'s own documented pipeline; (c) 15 literals whose
  textual role is "match something that stays old-spelled" (a low-level AST consumer bypassing
  normalization, a doc-comment-mirroring renderer, and 12 diagnostic-comparison guards against
  `format_type`'s unmigrated output) — reverted to the pre-apply spelling, narrowing the apply to 39
  files / 506 edits. None of this is a STOP: every cause was a known, nameable mechanism with a
  mechanical fix, not a new design question for the builder.
- **If this brief contradicts the code:** the brief's own probe literal (src/macros/tests.rs) does not
  exist as described — measured and reported in §1, worked around with the nearest real analog rather
  than halting.

## Commit

Locally on `main`, not pushed. Four commits this stone:

- `84a9eb067` — the brief's own draw (doc only, pre-existing).
- `e28d03236` — the probe + shared module (`embedded_wat.rs`) + the engine's core (`codemod_driver.rs`)
  + lint rewiring.
- `b184330b3` — the driver binary (`wat-fix-rust`) + the corpus apply (54 files / 570 edits at the time)
  + the `check.rs` `normalize_symbol_refs` cure.
- `0982145d2` — the loose-assert lint cure + the 15-file revert, narrowing the final apply to **39
  files / 506 edits**.

`git diff e28d03236 --stat -- src tests Cargo.toml`: **42 files changed, 811 insertions(+), 426
deletions(-)**.
