# SCORE — `assertion-failed!` takes kwargs

No commit. Floor left to the orchestrator (read `^ +Summary`, never a tail). Lands on 251.9 and the match-arm close. Those were not reverted.

The bare name is a kwargs macro. The primed positional primitive is `:wat::kernel::assertion-failed!'`. A plain fail is `(assertion-failed! :message "msg")`. The two trailing Nones are gone, not rewritten as `:actual :None`.

---

## Expectation 0 — the probe existed and was red

**Discharged by the AMEND** before this strike. Four rows on `ef2c20bf2`: control PASS; kwargs rows FAIL `ArityMismatch expected 3 got 2`; positional FAIL `left 0 right 0`. Un-ignored here.

## Expectation 1 — the kwargs form is accepted

**PASSED.** `a_plain_failure_needs_only_a_message` and `the_optional_kwargs_are_still_accepted`. `--check` of both kwargs fixtures EXIT=0, equal to the control.

A string-literal `:actual`/`:expected` is wrapped in `Some` in the quasiquote template so the probe's `"a"`/`"e"` type as `Option<String>`; an already-Option form (`(:wat::core::Some …)`) is spliced as-is. Corpus call sites use the latter.

## Expectation 2 — the two Nones are GONE, not rewritten

**PASSED.** Measured on `*.wat` / `*.wat.bad`:

| | before | after | delta |
|---|---|---|---|
| `:wat::core::None` tokens | **5865** | **665** | **−5200** |
| adjacent `None None` pairs | 2621 | 18 | |

The AMEND predicted ≈5176 deleted. Spot-check of the diff: a plain fail becomes `:message "…" ` with neither key; a real actual/expected keeps both keys; a one-sided None is dropped (`assert-coincident` keeps `:actual` only). No call site writes `:actual :wat::core::None` as a kwarg (the four remaining `:actual :None` hits are `Failure` record constructors in `wat/spawn.wat` and a scratch probe, not this verb).

`grep -c 'assertion-failed![^)]*:wat::core::None :wat::core::None'` over `*.wat` still hits 8 files. They are **not live surface calls**:

- `tests/kernel/probe_arc109_assertion_kwargs__positional.wat` — STOP-3 control, skipped by the codemod
- `wat/kernel/assertion.wat` — the **prime** expansion `` `(assertion-failed!' ~msg :wat::core::None :wat::core::None) ``
- `wat-scripts/fixes/*.wat` — comments / string templates of the **old** form those historical codemods emit

## Expectation 3 — the positional form is REFUSED

**PASSED.** `the_retired_positional_form_is_refused`. `--check` of the positional fixture EXIT=101:

```
#wat.kernel/AssertionFailure
  :message "assertion-failed! takes kwargs :message / :actual / :expected; the positional (message actual expected) form is retired"
```

Control EXIT=0. One convention.

## Expectation 4 — the corpus moved BY CODEMOD

**PASSED.** `wat-scripts/fixes/assertion-failed-to-kwargs.wat` is the recorded migration (shape copied from `positional-to-kwargs.wat` / `match-arm-to-bracket-map-pattern.wat`). Dry-run on a `/tmp` copy of `wat/test.wat` first; then `printf '[…453 paths…]\n' | ./target/release/wat ./wat-scripts/fixes/assertion-failed-to-kwargs.wat`. Idempotent (kwargs first-arg is a keyword → skip). The positional probe was skipped by name.

No hand-edited `.wat` call site. Exceptions (named, not `.wat` forms):

- inline wat in Rust strings (`src/runtime.rs`, `src/kernel/spawn.rs`, `src/intrinsic/edn.rs` `@example`)
- MCP jsonl (`tests/cli/wat_mcp__{assertion_panic,counter_across_turns,thread_counter_across_turns}.jsonl`) — string-exception analog of RELAND 2

## Expectation 5 — the None population fell

**−5200** from 5865 to 665. Not `5784 − 2532`: the AMEND's pair count was the right order of magnitude (≈5176). The residue is real Option/Result Nones plus the prime's default and the STOP-3 control.

## Expectation 6 — the floor

**Not run.** Orchestrator. Targeted: `every_ungated_wat_file_checks` PASS (stdlib still checks). Probe 4/4 PASS.

## Expectation 7 — clippy

`cargo clippy --release --all-targets --workspace` **0 errors**. Same 5 pre-existing dead-code warnings.

---

## STOP rows

| STOP | result |
|---|---|
| STOP-1 probe not first | **does not fire.** AMEND committed it red; this strike un-ignored it |
| STOP-2 hand-edited `.wat` call | **held.** Corpus via wat-fix. Rust strings + jsonl named as the string exception |
| STOP-3 both forms accepted | **held.** Positional refused at expand (`Option/expect` of a past-end `get`) |
| STOP-4 Nones rewritten as kwargs | **held.** Dropped. Spot-checked |
| STOP-5 sibling `raise!` | **held.** `src/check.rs` `raise!` registration untouched |

## F5 note

The first macro draft constructed `(:wat::core::None :wat::WatAST)` as a call; F5 refused the `:wat::core::None` head. Optional slots are 0-or-1 vectors; a missing required value is `get` past the end. Arithmetic is `:wat::i64::{+,*,/,>,rem}` — `:wat::core::+` is not expand-time legal.

## Targeted checks

```
cargo nextest run --release -E 'test(probe_arc109_assertion_kwargs)'   4 passed
cargo test --release -p wat-doc -p wat-macros -p wat-edn -p wat-reader  ok
cargo nextest run --release --test lint -E 'test(every_ungated_wat_file_checks)'  PASS
cargo clippy --release --all-targets --workspace                       0 errors
```
