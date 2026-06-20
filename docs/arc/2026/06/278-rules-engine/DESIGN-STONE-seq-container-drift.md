# DESIGN — seq/collection checker↔runtime container drift (Phase ① parity prerequisite)

## What + why
Three collection ops accept a container representation at **runtime** that the **type-checker** rejects — so
the checker false-rejects valid programs the runtime would run correctly. Surfaced 2026-06-20 when a
returns-the-fact de-risk probe reached for `(:wat::core::first <PersistentVector>)` and the checker refused it.

Root cause (confirmed across three arcs, one signature): a new container repr was added to the runtime and
never propagated to the checker — the "build the runtime half, skip the `check.rs` half" pattern. All three
drifts are **false-REJECT** (checker stricter than runtime); a full audit found **zero** false-ACCEPT (no
corruption / no runtime-crash path). 17 of 20 collection ops are clean.

This is a Phase ① feature-parity prerequisite: the chosen feature set must be coherent before we measure it,
and returns-the-fact will read PV-of-facts results, so the sharp edge must go first.

## The three drifts (each weighed against the disk this session)
| op | runtime accepts | checker accepts | missing on checker |
|---|---|---|---|
| `first`/`second`/`third` (`infer_positional_accessor`, check.rs:9991 ↔ `eval_positional_accessor`, runtime.rs:10944) | Tuple, Vec, List, PersistentVector, WatAST::List | Tuple, Vector, List | **PersistentVector, WatAST::List** |
| `rest` (check.rs:5301 ↔ `eval_rest`, collection/eval.rs:~1409) | Vec, List, PersistentVector, WatAST::List | Vector, List | **PersistentVector, WatAST::List** |
| `conj` (`infer_conj`, collection/infer.rs:129 ↔ runtime.rs:12388) | Vec, List, PersistentVector, HashSet | Vector, PersistentVector, HashSet | **List** |

## The fix (this stone — add the missing arms, sized to each runtime set)
For each drifted checker arm, add the missing container arms so the checker's accepted set EQUALS the runtime's,
with the honest return type:
- **positional accessors** (`first`/`second`/`third`): add `PersistentVector<T> → Option<T>` and
  `WatAST::List → Option<wat::WatAST>` arms to `infer_positional_accessor`. Mirror the existing `Vector<T>`
  arm (returns `Option<T>`). Update the error message to list all five containers.
- **`rest`**: add `PersistentVector<T> → PersistentVector<T>` and `WatAST::List → wat::WatAST` arms (rest
  preserves container identity — runtime.rs returns a new PV from a PV, a WatAST::List from a WatAST::List).
  Update the error message.
- **`conj`**: add the `List<T> → List<T>` arm to `infer_conj`. Update the error message to include `List<T>`.

These are forced, NECESSARY changes to `check.rs` / `collection/infer.rs` — sized to correctness, not "tiny".
Both sides of each op now agree. No runtime change (the runtime is already complete and correct).

## The contract decision (pinned)
The checker's accepted container set for a collection op MUST equal the runtime's. The return type per arm:
positional accessors → `Option<elem>` (out-of-bounds is a runtime fact); `rest` → same-container<elem>
(identity preserved); `conj` → same-container<elem>. No widening, no Value-escape.

## The probe (the drift tripwire — the achievable extirpare rung)
`tests/probe_seq_container_parity.rs`: for each drifted op × each missing container, a wat snippet that MUST
type-check AND run to the right value. RED at HEAD (checker rejects → eval returns Err). GREEN when the arms
land. This pins checker≡runtime so any FUTURE one-sided arm goes red. Strong value assertions on the
PersistentVector cases (the rete-relevant ones); WatAST cases assert "checks + runs without error".

## Out of scope = the NEXT stone (named, not deferred-vaguely)
The **structural unification** — route positional/rest/conj through a single shared element-extractor (extend
`extract_seq_elem`, collection/infer.rs:500, which today covers only `{Vector, PersistentVector}`) so a new
container repr added once reaches all consumers and a one-sided arm becomes unrepresentable. This is genuine
design (Tuple is heterogeneous and cannot pass through a single-elem-type extractor; per-op-family container
subsets differ), so it earns its own DESIGN + strike. The probe from THIS stone guards the class in the interim.
That stone is the top-of-ladder cure; this stone is the check-rung fix + tripwire.

## Done = green
`tests/probe_seq_container_parity.rs` → all green. No regressions: lib floor 941/36; the rete differential
probes (8a/8b/8custom/7exists/fence-hof) unchanged; `cargo build --release` clean.
