# DESIGN — Stone 245.8: ordering is relational (the 237.8b future-stone, opened)

**Status:** STRIKE-READY 2026-06-06. Room 1 of the full clear
(`TRIAGE-FULL-CLEAR.md`). Kills 46 of the 147 (the biggest cluster).

## The gap, grounded

- All 46 `wat_arc148_ord_buildout` failures are one shape:
  `NoMatchingClauseAtCallSite` on `:wat::core::</<=/>/>=` over non-numeric
  types — the ordering defclauses (wat/core.wat) carry exactly two clauses
  each (i64,i64) and (f64,f64).
- The runtime engine is ALIVE: `values_compare` (src/runtime.rs ~9187) covers
  i64/u8/f64 (+cross-numeric arms), String, bool, keyword, Instant, Duration,
  Vec (lexicographic, incl. Bytes), Tuple, Option (None<Some), Result
  (Err<Ok) — recursive over elements; returns `Option<Ordering>`, `None` for
  non-orderable. `eval_compare` (~9265) wraps it. Arc 148 built it; 237.8b
  narrowed only the REACHABILITY.
- The narrowing was a recorded promise: `DESIGN-STONE-237.8b.md:211` —
  "Comparison primitives for non-numeric types (String/Char/time `<` etc.) —
  future stone after 9 ships." 237.9 shipped; the stone never opened. This is
  that stone.

## The decision (four-questions, driven; the inscribed discriminant applies)

The PARTITION doctrine (marked at check.rs `fn infer_list` +
runtime.rs `dispatch_keyword_head*`; the 237.8c equality reversal is the
exemplar): **clause = monomorphic; intrinsic = type-level computation, and the
RELATIONAL flavor is a constraint flowing BETWEEN args.** Ordering over
parametric containers (`Vec<T> < Vec<T>`, `Option<T>`, `Tuple<…>`,
`Result<T,E>`) requires `unify(arg0, arg1)` — arg↔arg type-var flow a finite
clause list cannot express (the bool return is the same trap that briefly
mis-shaped equality). Therefore:

**Ordering (`<`, `<=`, `>`, `>=`) becomes a relational intrinsic, the sibling
of equality** — one check-side inference mirroring `infer_equality`'s shape
(unify the two args; then gate on the ORDERABLE class), one runtime
keyword-head dispatch routing to the existing `eval_compare`/`values_compare`.

- Hybrid (numeric defclauses + structural intrinsic) — REJECTED: fails Simple
  (two dispatch mechanisms for one op).
- Reject structural ordering (delete the 40 tests) — REJECTED: fails Honest
  (contradicts 8b's recorded promise and the living engine).

**Doctrine preserved:** cross-type ordering stays rejected — `unify(i64, f64)`
fails (same-type-only survives the mechanism change; the error KIND changes
from NoMatchingClause to the unify-failure TypeMismatch). Equality's
cross-numeric acceptance is equality's own (arc 238 value-comparison);
ordering does NOT inherit it at check time.

**The orderable class (check-side gate, mirroring the runtime's truth):** the
unified type must be orderable — i64, u8, f64, String, bool, keyword
(`:wat::core::keyword`), Instant, Duration, and recursively: Vector<orderable>,
Tuple<orderable…>, Option<orderable>, Result<orderable, orderable>. NOT
orderable (reject at check, teaching error): HashMap, HashSet, enums/Records/
Structs, unit, fn types, HolonAST, channels/handles. TypeVars: defer/accept as
unresolved (mirror infer_equality's policy — verify its arm before choosing).
The runtime's `values_compare → None` remains the eval-side backstop.

## Blast radius (verify-first at every site)

1. **check.rs** — new relational-intrinsic arm for `<`/`<=`/`>`/`>=` beside
   `infer_equality` (name by the established family convention — the
   `infer_equality` precedent names its sibling; confirm the convention by
   reading the family before naming). Remove whatever check-path currently
   routes ordering to the defclause grid for these four heads.
2. **runtime.rs** — keyword-head dispatch arms for the four ops →
   `eval_compare` (verify what currently dispatches them — the defclause path
   — and what `eval_compare`'s callers/shape expect).
3. **wat/core.wat** — RETIRE the four ordering defclauses (the section + its
   header prose). The `i64::<` / `f64::<` per-Type leaf families STAY (the
   type-locked tier, registered in Rust). Note: this DRIFTS core.wat's fresh
   vigilatum — expected; the stamp is re-earned after the stone (re-ward on
   touch). Update the stamp line's guard claims ONLY if the stone's sweep
   changes what they assert; the re-stamp itself is the orchestrator's, after.
4. **tests/wat_arc148_ord_buildout.rs** — the CONTRACT: 40 fill tests go green
   unmodified; the 6 `*_raises_type_mismatch` tests modernize to assert the
   check-time rejection (read each test's intent; assert on the real new
   error kind).
5. **wat-tests/core/core-arithmetic.wat** — the cross-type ORDERING rejection
   deftests (lines ~281-297 per the prior census) assert the defclause-absence
   error; modernize the assertion to the intrinsic's rejection shape (the
   INTENT — cross-type ordering rejected — is unchanged).
6. **docs/USER-GUIDE.md** — the ordering prose updated this week says
   "wat defclauses over per-Type leaves … NoMatchingClause" (the comparison
   cheatsheet row + the Tier-1 block): update to the intrinsic truth
   (relational intrinsic over the orderable class; cross-type rejected by
   unify failure; structural ordering now matches structural equality's
   reach).

## Gates (the contract)

- `cargo test --release --test wat_arc148_ord_buildout -p wat` → **46/46**
- `cargo test --release --lib -p wat` → 920-baseline (+ any new in-module tests)
- `cargo test --release -p wat --test test` → 236-baseline green (with the
  modernized cross-type assertions)
- `cargo build --release --tests -p wat` clean;
  `cargo clippy --release -p wat` no new findings in touched files
- The hygiene/gate probe family stays green (no keying surfaces touched, but
  run it: the dispatch arms are near them)

## Affirmative bounds

- Char ordering: out of this stone's scope — wat has no Char type surface to
  order (8b:211 named it speculatively); nothing to fill.
- The cross-numeric runtime arms in `values_compare` (i64↔f64) stay as-is:
  unreachable from type-checked user code for ordering (check rejects first),
  live for equality's value-comparison path which owns them.
- core.wat's re-stamp happens AFTER this stone lands (the orchestrator's
  ritual, not the executor's).
