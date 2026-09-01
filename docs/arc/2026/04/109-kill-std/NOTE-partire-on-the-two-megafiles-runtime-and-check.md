# NOTE — `partire` cast on the two megafiles: `runtime.rs` (SPLIT ×10) and `check.rs` (SPLIT ×5)

> Cast 2026-08-31, two independent read-only casts of the datamancy ward `partire`, ward text
> fetched from the signed MCP and embedded verbatim in each worker. **No stone drawn, nothing moved.**
> This is the seam map the decomposition campaign is cut from.
>
> **Builder's ruling that prompted it:** *"we break the mega files up first… we do not begin our
> crates migration until `wat-rs/src/*.rs` is cleaned up… once those partition lines are drawn in
> src, we begin the move to crates."* And the destination: *"`src/*.rs` is likely to only hold a
> `lib.rs`."*

## Why a ward and not a proposal

The orchestrator could have sketched a decomposition from the measurements it already had. That
would have been a guess wearing a plan's clothes. `partire`'s grounding clause is stricter than
anything self-imposed: **a proposed module lacking a standalone name, located regions, a distinct
reason-to-change, or independent-test evidence is WITHDRAWN, not reported** — and it names the
failure the orchestrator would most likely have committed, the **accidental seam** (splitting by
size, or by layer when both layers share one secret).

Both casts refused cuts. That is the evidence the ward was doing work rather than rubber-stamping.

---

## `src/runtime.rs` — VERDICT: **SPLIT**, ten modules

34,142 lines; 27,606 production (the trailing `mod tests` at `:27607` is out of scope — test
organisation is `complectens`, not `partire`). Target layout follows the repo's existing idiom
(`src/check.rs` + `src/check/`): the file stays the mod-root, each module becomes
`src/runtime/<name>.rs`.

| # | module | regions | Level |
|---|---|---|---|
| 1 | `kernel_signal` | `91–786` | 1 |
| 2 | `declarations` | `789–4496` | 1 |
| 3 | `defclause_dispatch` | `7120–8756` | 1 |
| 4 | `numeric_tower` | `9358–9527` · `9614–10691` | 2 |
| 5 | `pattern_matching` | `16140–16589` | 1 |
| 6 | `quasiquote` | `12089–12480` | 1 |
| 7 | `reflection` | `12481–15030` | 1 |
| 8 | `record_construction` | `17474–18700` | 2 |
| 9 | `stepper` | `22809–24199` | 1 |
| 10 | `peer_protocol` | `19952–20531` · `20946–22232` · `24694–27606` | 1 |

★ **~13,600 lines relocatable.** The cast explicitly did NOT attempt to partition the remaining
~14,000 (the central `eval`/`dispatch_keyword_head_value` spine and the one-off primitives it
calls): *"I have call-graph confidence for the ten below but not for further cuts inside that
remainder, so nothing there is proposed."* **An honest boundary on its own claim.**

### Two exclusions it named so a line-range copy-paste does not drag them along

- **`dispatch_rete_op` (`9528–9613`)** sits textually *inside* `numeric_tower`'s range but recurses
  into `dispatch_keyword_head_value` — it belongs to the dispatch spine, not to arithmetic.
- **`eval_type` (`16590–16632`)** sits immediately after `pattern_matching` but is an unrelated
  `:wat::core::type` classifier. Accidental adjacency.

### Refused cuts

- **"everything `dispatch_keyword_head_value` calls" as one module** — the shared secret would be
  *"gets called from the big match,"* not a domain. The ward's layer-sharing-one-secret failure.
- **`eval_type` into `pattern_matching`** — adjacency, not concern.
- **`keyword_accessor_record`/`_struct`/`synthesize_fn_body` (`6967–7119`)** — withdrawn for lack of
  a verified call graph. *"An ungrounded guess is worse than leaving them in place."*

### Practitioner's-call splits (the concern graph admits two readings)

1. `register_defclause` + `preregister_stdlib_defclause_stub` — by lifecycle (with `declarations`,
   as placed) or by feature (with `defclause_dispatch`).
2. `quasiquote` vs `reflection` — a zero-cross-call boundary at exactly `:12481`; both reify AST, and
   a maintainer could merge them into one metaprogramming module.

### ⚠ A duplication it flagged in passing (not a partire finding)

`stepper` carries **its own** pattern matcher, `try_match_pattern_ast`, with zero calls to
`pattern_matching`'s `try_match_pattern`. Two independent implementations of the same match
semantics. **That is a `solvere` question and a real coupling risk** — whoever edits one and forgets
the other is the hazard.

---

## `src/check.rs` — VERDICT: **SPLIT**, five modules

22,556 lines. The file already declares `pub mod env; pub mod error; pub mod error_edn;` with
`src/check/` populated, so this extends an established decomposition rather than starting one.

| # | module | regions | Level |
|---|---|---|---|
| 1 | `check::legacy_lint` | `774–1343` | 1 |
| 2 | `check::restricted_call` | `1343–`**`1515`** ⚠ corrected | 1 |
| 3 | `check::pattern_coverage` | `6033–7501` | 2 |
| 4 | `check::concurrency` | `1613–1772` · `9688–12304` | 1 |
| 5 | `check::builtins` | `16569–21711` (`register_builtins`) | 1 |

★ **~10,200 lines move; `check.rs` lands ~12,300.**

### ⛔ ORCHESTRATOR CORRECTION — the cast's own ranges OVERLAPPED

The report gave `restricted_call` as `1343–1613` **and separately refused** `is_atomizable` as a
`holon_ops` module for lack of independent-test evidence. But `is_atomizable` is at **`1516–1612`** —
*inside* the range it proposed to move. The report also mis-cites it at `1476`, which is
`caller_matches_prefix_list`; that 40-line slip is likely how the overlap survived.

Verified on disk:

```
1343  extract_prefix_list_from_metadata   ┐
1430  walk_for_restricted_call            ├─ restricted_call  = 1343..1515
1476  caller_matches_prefix_list          ┘
1516  is_atomizable                       ← STAYS. Not a caller-whitelist concern.
1613  find_process_join_before_drain      ← concurrency
```

★ **This is the ward's own accidental seam, inside the ward's output: a cut drawn by LINE SPAN
rather than by reason-to-change.** `is_atomizable` has nothing to do with caller whitelists; it is
called from `infer_list`'s holon arm (`:3664`, `:3690`) and one test (`:22439`). It stays in
`check.rs` — which is exactly what the report's own refusal section concluded.

⚠ **The lesson for the campaign:** a proposed range is a claim about *every line inside it*, not just
its endpoints. Verify what a range CONTAINS, not merely where it starts.
`[[feedback_a_census_without_attribution_is_not_a_census]]`

### Refused cuts

- **`infer_list`'s ~3,484-line dispatch (`2549–6033`)** by call-target group — every test reaches it
  through `let`/`def`/`if`/apply scaffolding, so the pieces have *no test surface distinct from "any
  check test at all."* They are shared substrate, not a separable secret. **This is the biggest
  tempting cut in the file and the ward refused it.**
- **The substitution/instantiation core (`16152–16569`)** — used by every inference path including
  all five proposed modules; extracting it relocates coupling into an import.
- **`is_atomizable` as `holon_ops`** — no dedicated test directory isolates holon bind/bundle
  checking; only two in-file unit tests.
- **`MUST_USE_TYPES` family (`7605–7718`)** — a real, small, distinct policy with no isolating
  fixture.

### Practitioner's-call splits

1. `infer_serve_dispatch_op`/`infer_retag_op` (`11598–11681`) — kept with `concurrency`, but tested
   under `tests/services/`; a `defservice`-axis reader would draw a `check::services_dispatch`.
2. `infer_rete_form` (`2363–2549`) — grounded and independently testable against `tests/rete/`, but
   186 lines for one function may not earn its own file under minimum-cuts.

---

## What this NOTE does NOT decide

The **order**. `[[NOTE-the-crate-boundary-is-the-real-cut-and-eight-homes-are-cyclic]]` measured that
eight of nine existing homes are in a dependency cycle with `runtime.rs` (`value` alone is acyclic;
`types` is **one** reference away), and the builder has ruled that the src decomposition comes first
and the crate migration second. Which module moves first — and whether the ordering is chosen to
leave each home acyclic — is not drawn here.

⚠ And nothing in either cast has been executed. **Both are seam maps, not stones.**
