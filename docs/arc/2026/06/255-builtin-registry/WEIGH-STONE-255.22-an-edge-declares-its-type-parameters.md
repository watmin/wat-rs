# WEIGH — STONE 255.22: an `extend-type` declares its type parameters — ACCEPTED

**Executor commit `dbe400181`**, on the brief + addendum `fc70847a0`. Weighed by the orchestrator against
disk on 2026-09-24.

## Re-measured

| row | measured | result |
|---|---|---|
| tree | `git status`; `git diff dbe400181 -- src tests` | clean; the tree equals the commit |
| floor | `.floor/2026-09-24T08-06-09Z/clean.log` | `6053 tests run: 6053 passed (9 slow), 22 skipped`; all **9** `probe_arc255_22_*` tests appear as PASS |
| ⭐ the `Elem` hello-world | `wat …_hello_Elem.wat` | runs, prints `"hello, world!"`. The binderless original is rc=1 on the pre-stone build (`74410c56f`) |
| the free-letter wall | `--check …_free_letter.wat.bad` | rc=1 `#wat.type/EdgeFreeTypeName … :T in the child is neither declared in the edge's binder nor a known type` |
| the absent-parameter wall | `--check …_param_absent_from_child.wat.bad` | rc=1 `#wat.type/EdgeParamAbsentFromChild … binder parameter T does not appear in the child type` |
| the binderless `Elem` probe on the new binary | `--check probes-255.22/hello-Elem.wat.txt` | rc=1: a free letter is now an error |
| the C-b3 witness (`255-21-coord-claims-either-transport.wat`) | `--check` | **rc=0: still open**, as expected (C-b3) |

⛔ **An orchestrator instrument error, caught before it was written down:** the first pass printed both
wall fixtures as rc=0. The cause was `echo "$(basename $f) new=$?"`, where `$?` is basename's status, the
same defect as earlier this session. Read directly, both refuse. It is recorded as memory
`feedback_dollar_q_after_command_substitution`.

⚠ The IDE's diagnostics for this commit (*"`register_generic_edge` is never used"*, *"mismatched types"*
in the test file) are **stale**. The floor compiled and passed those tests.

Taken from the report without re-running:

- clippy 0;
- census `no STOP-8` (215 → 215; the only change is the two deleted probes);
- delta NEW 3 / RECOVERY 0;
- ledger 215;
- a two-binary `--check` sweep over 2507 files changed only the three expected verdicts.

## What landed

- **The form:** `(extend-type :- [P…] <child> <target> …)`, read through one helper,
  `types::extend_type_operands`. Only 3 readers were positional, not the brief's 11: the other 8 go
  through `parse_extend_type_form` or match only the head.
- **A structured generic edge:** (binder, child, target), keyed by the child's head. Matching fits the
  child, binding only the binder's names, then instantiates the target and unifies it; only a single
  solution is accepted.
- **Removed:** the spelling-keyed `transport_satisfier_heads` / `transport_edge_keys` (the `:T`/`:Xt`
  guesses) and 118.3-B's positional zip. `family_extends` now reads generic edges, which also fixes
  runtime defclause dispatch for an `Elem`-spelled edge.
- **The four `Seqable` edges** read *a Vector holding T is a Seqable of T*. defservice's three edges emit
  `:- [~@handle-tp-syms]`.
- **The 118.3b fixture** tests the real `Seqable` through `into`, with new rows: a concrete
  `(Seqable :- [i64])` bound, a wrong element type, and a type with no edge. **Lost:** the bare-surface
  row, which needed the private `BareSeqable`. The arm it guarded is still exercised by every
  surface-method receiver check.
- **Deleted:** the two August probes. `probe-seqable-is-spellable-today.wat` is concrete-only and
  unchanged (rc=0 on both binaries, prints `"3,4"`).

## Reported under STOP-1, accepted

A 20-line change in the method-scheme lookup (118.B2d, `check.rs` ~:5401): when exactly one generic edge
matches the receiver, its bindings instantiate the method's type. Without it, an edge spelled `E` leaked
`:E` into the result type (the `lie_mixed` row named `got :E`). `satisfier_method_keys` and its `:T`/`:Xt`
key guess are **untouched**. That is C-b3, and the `coord` witness above is still rc=0.

## Brief errors, recorded

- "11 positional readers" were 3.
- `is_type_param_letter` played no part in edge registration. The spelling dependence lived in the
  guessed keys.
- The addendum's `Elem` row cannot use `:wat::core::Seqable` itself: method keys are global, and
  `Vector/seq` is already registered. It uses a same-shaped private surface.
