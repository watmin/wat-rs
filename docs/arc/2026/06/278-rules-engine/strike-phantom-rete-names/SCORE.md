# SCORE — F2's codemod row, weighed against the orchestrator's own re-run

> Re-run here at `f4800ef97`.

| # | pre-value | after |
|---|---|---|
| 1 | an invented head **ran clean** | ✅ the gate REDs on it |
| 2 | phantoms in codemod + scratch probe | ✅ gone from both |
| 3 | 41 pairs, 2 unresolvable | ✅ **39**, reason at the table |
| 4 | `foldr`/`nth` in comments | ✅ **still there, gate green** — driven by me |
| 5 | `defn` in 15 files | ✅ not flagged (36 code uses) |
| 6 | — | ✅ the new gate declares its own non-vacuity |
| 7 | *"All wat stays correct, always"* — **driven false** | ✅ struck; two gates named, and what is still unproven stated |
| 8 | lint 134/134 | ✅ **153/153** |
| 9 | floor 5248/5248 | ✅ `Summary [ 421.594s] 5267 tests run: 5267 passed (3 slow), 21 skipped`, zero FAIL rows |
| 10 | clippy rc=0 | ⚠ **went RED** — see below; cured, rc=0 |

**The comment-vs-code pair, driven by me, both directions:**

- the same two phantoms appended **as prose** → `19 tests run: 19 passed`
- `:wat::rete::core::map` in a **`def` body** → `FAIL … probe-…-hof.wat:97 :wat::rete::core::map — no `RETE_OPS` row … and not a known form`

The classifier discriminates. The green is meaningful, not permissive.

## ⛔ Clippy went red, and the tier split is why it was caught

`clippy::unnecessary_get_then_check` at `rete_names_in_wat_scripts_resolve.rs:746`, in a new unit
test. The rider's `binary_id(wat::lint)` run was **153/153 green** — **nextest runs tests, clippy
lints**, so that check structurally could not see it. This is the same shape as the lint reds that
prompted adding `wat::lint` to the rider's checks, one layer up. Cured with clippy's own
prescription.

## ⛔ Where MY brief was thin — and the ★ was false as written

- **A. ★ THE ONE CONTRACT DECISION WAS FALSE.** *"A `:wat::rete::` name written in CODE resolves"* —
  but **a recorded codemod's OLD column is code and must name what it removes**, and a
  negative-control probe (`probe-f64-comparator-bogus-head.wat`) deliberately calls an unminted head
  **as another brief's non-vacuity proof**. Four such names across three files. A gate enforcing my
  sentence literally would have destroyed that proof — trap 1's shape (a gate demanding the deletion
  of accurate history) in **code** rather than in a comment, which is the form I did not anticipate.
  They carry a per-name `rune:lint(rete-name-unminted)` now, with the same reason discipline as the
  rest of this tree.
- **B. ★★ A NAIVE UNION WOULD HAVE VOUCHED FOR ITSELF.** Measured: **every one of the 79 `RETE_OPS`
  rows is also attested in a code position elsewhere.** Under a flat `rows ∪ attested` universe the
  registry half resolves **exactly zero** names by itself — emptying it changes no verdict, and the
  resolver-blinding mutation passes green with the row set doing nothing. My EXPECTATIONS said the
  non-vacuity floor must catch it, and it would have — but **a floor is not a resolver**, and it
  would have been the only thing there. Splitting by namespace makes each half the sole authority
  for its own family: blinding rows leaves 71 names unresolved, blinding attestation 63. Promoted to
  memory.
- **C. My "deleted, not corrected" warning was right for a shallower reason than the truth.** A pure
  head-rename to `mapv`/`filterv` **does not even compile** — the loader gate REDs with *"no clause
  of `:wat::core::filterv` matches arity 2 … clauses attempted: `(Vector :- [T])`; `(Stream :- [T])`"*,
  because the container had to change too. **That red is the strike's best single fact:** with a head
  that resolves, `every_wat_scripts_file_loads` finally had something to check. It had nothing for
  three months while the head was invented.
- **D. Trap 4's grep cannot answer STOP-1.** `:wat::core::map`/`filter` appear in 79 files, 24 of
  which also contain a `where` — none inside one. The rider paren-matched `where` subtrees instead:
  **0 occurrences across 328 `where` forms in 1,653 `.wat` files.** STOP-1 clear by structure, not by
  grep — the same substring lesson that has cost me three radius estimates.

## The rider's own near-miss, reported unprompted

Mutation 6b first reported **green** — because the mutation had not landed (`perl -0pi -e` with
`\x{2014}` silently failed without a UTF-8 flag). It re-ran with a substitution that asserts the file
changed, and it RED-ed correctly. **A mutation that does not land is indistinguishable from a gate
that does not fire**, and it nearly recorded the former as the latter. That belongs in every future
mutation step: assert the mutation landed before believing its result.

## The doctrine question, ruled

The rider hand-edited three `.wat` files rather than routing through wat-fix, and flagged it for
overrule. **Ruled: correct.** `CLAUDE.md:99` governs *"a structural rewrite across many `.wat` files
(a rename, a record→enum migration, a form flip)"*. These are three single-site edits in three
different files, each carrying bespoke hand-written prose no rename table can express. The codemod
rule is not implicated, and reaching for it here would have been ceremony.

## Arms not driven, named

Rune-with-no-separator (driven as a unit test, not at corpus level); `KNOWN_FORMS`' two self-check
asserts; the subject-walk non-vacuity floors; `prose_control_holds` (driving it means deleting the
tree's `foldr`/`nth` record); `prose_in_rust_does_not_attest_a_name` — **not reachable in Rust**, but
the rider measured the equivalent out-of-tree: scanning `src/` *without* stripping comments does
admit `:wat::rete::core::map`, so the control is live rather than decorative.
