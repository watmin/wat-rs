# SCORE — F1 row 5, weighed against the orchestrator's own re-run

> Re-run here at `58a10e1f8`. **The first Class F strike.**

## Row 1 — the live question, answered

**No gate in `tests/lint/` is vacuous today. STOP-1 did not fire.** Driven, not read: every
discovering gate instrumented at the point its population is computed, run under `--no-capture`.

| gate | visits |
|---|---:|
| `every_parity_script_is_invoked` | 4 |
| `docs_wat_loads_or_declares_why_not` · `hunt_tooling_selftests` | 11 each |
| `wat_record_from_sources_are_loaded` | 13 |
| `gen_doc_surface_matches` | 27 verbs / 9 doc-qualified names |
| `no_new_broken_doc_link` | 34 diagnostics |
| `no_ceiling_raise_in_rete` · `no_mutex_in_rete` | 57 |
| `no_stale_path_in_doc` | 125 |
| six gates over `src` | 213 |
| `no_angle_type_in_diagnostic` | 230 |
| `no_rc_use` · `no_rpds_rebuild_loop` | 236 |
| `wat_scripts_fixes_load` | 445 |
| `no_inlined_edn` · `no_inlined_wat_in_tests` | 727 |
| four gates incl. `no_loose_string_assert` | 998 |

**A missing guard was a risk, not a live defect** — and because that ran first, every floor written
into these gates is a *measured* number rather than one chosen for symmetry with a sibling.

## The count, from the instrument

**24 in scope, 19 undeclared.** Not the row's *10 of 15*; not my audit grep's *16 of 24*. Of the 19,
**six already had a real guard** and only lacked a declaration, twelve had none, one needed the rune.
This is F0 working exactly as the builder specified: the stone carried no number, and the instrument
answered.

| # | after |
|---|---|
| 1 | ✅ driven; no gate vacuous |
| 2 | ✅ `every_walking_gate_declares_non_vacuity.rs` |
| 3 | ✅ `no_new_broken_doc_link.rs` accepted **with a rune**, not by name — an allowlist would rot |
| 4 | ✅ self-guard + a **positive control**, both driven |
| 5 | ✅ refused-reason list; `N/A` driven RED |
| 6 | ✅ `tests/lint/` only, **no `src/` change** |
| 7 | ✅ lint 119 → **134** |
| 8 | ✅ `Summary [ 389.439s] 5248 tests run: 5248 passed (1 slow), 21 skipped`, zero FAIL rows |
| 9 | ✅ clippy rc=0 |

## ★ THE STRIKE BIT ITS OWN EXECUTOR, AND DRIVING IS WHAT CAUGHT IT

The gate scans `tests/lint/`, so **its own prose is data**. The `///` doc on its
`Declaration::Rune` variant parsed as a rune declaration — **it was one run from vouching for itself
with its own documentation**. Neither that nor the second bite (`no_loose_string_assert` flagging the
comment explaining how it avoided `no_loose_string_assert`) was visible by reading; both surfaced in
the first driven run.

That is the memory I promoted one strike ago — *a cure can carry the defect one level down* — landing
inside the very strike drawn to prevent vacuous gates.

**My own re-run then found the cure's doc overstated.** I turned a `NON-VACUITY` marker into
`/// NON-VACUITY` expecting RED; the gate stayed **green**, because `DOC_HEADS` is consulted at
exactly one site — the rune path. The behaviour is **correct** and the asymmetry is real: a rune's
*reason text* is its evidence, so a description reads as an answer; a marker's evidence is the
*assertion beneath it*, which `is_assert` refuses to read from any comment. But the constant's doc
claimed a general rule — *"only a plain `//` comment declares"* — that it does not have. Narrowed at
the site, with the drive and the reason.

## ⛔ Where MY brief was thin

- **A. Read-order item 3 did not transfer.** `no_unknown_sequi_rune.rs` models checking a rune's
  *category against a table*; here there is one rune name and the hard problem is **which files must
  carry one**. Copying it yields a gate that checks spelling and is silent about files with no rune.
  The precedents that paid were `no_ceiling_raise_in_rete.rs` and `every_parity_script_is_invoked.rs`.
- **B. ★ "Drive every walking gate" named no mechanism, and the only honest one is expensive.**
  Reading a collector cannot see what it visits; it took instrumenting 27 population sites and a full
  `--no-capture` run. **Unnamed, a rider reads the collectors, calls it driven, and row 1 is lost** —
  which would have cost the whole strike, since row 1 is what made the guards measured.
- **C. STOP-3's seeded-ledger option was a trap here.** Nineteen flagged files *sounds* like a split.
  Because row 1 ran first, none was vacuous, so each guard was a measured floor and a five-line
  insert. A ledger would have frozen 19 undeclared gates behind a row reading "open".
- **D. My survey grep missed a real guard.** `wat_scripts_fixes_load.rs` asserted
  `!entries.is_empty()` after its loop, worded outside every phrase I searched. F0 in miniature,
  again.

## Reported, not fixed — three rows

- **`no_loose_string_assert` reads comment lines**, so any comment *describing* the banned pattern is
  flagged. Its own header records an earlier comment-scoping bug of the same family.
- **`gen_doc_surface_matches`'s extractor is unguarded** — it parses 9 doc-qualified names from a
  *named* file and would pass on 0. Same class, different population (present file, blind extractor);
  deliberately outside this gate's cut and **named in its header so it is tracked, not silent**.
- **Two known gaps in the new gate, written into its own header**: the check is file-level (a second
  walking test in an already-declared file is not demanded), and a rune's reason is read from its own
  line only.

## Arms not driven, named

`in_scope.len() >= 18` — **reachable but not driven**: mutation 4 would have fired it, but the
positive-control assert sits above and panicked first. A second, weaker net under the same failure.
Six `Hollow` sub-arms — **proven by 14 permanent detector unit tests**, each executing the arm and
asserting its verdict, **not** additionally tree-mutation-proven.
