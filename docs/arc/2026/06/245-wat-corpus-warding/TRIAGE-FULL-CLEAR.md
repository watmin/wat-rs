# 245-reopen — THE FULL CLEAR (no survivors)

> Builder directive 2026-06-06: "wipe the dungeon clean - diablo 2 style - the
> quest isn't done until there's no survivors." The 245.7 baseline
> (`INVENTORY-245.7-baseline.tsv`) is the dungeon map: default tier 152 pass /
> **33 FAIL** binaries; tests 1460 / **147 FAIL** / 59 ignored. This ledger is
> the room-by-room clear record — update it as each room falls; it survives
> compaction so the clear never loses its place.
>
> METHOD per room: ground (run the binary, read the corpses) → conferre (real
> substrate gap = FILL / stale pre-clojure-ification test = MODERNIZE-or-DELETE
> — greening a stale test re-enshrines a retired form) → strike (design if
> fill; sweep if modernize) → gates → commit → next room. Leak-contained runs
> only (the failing 33 are all in the non-leaky default tier; named
> `cargo test --release --test <name> -p wat` is sanctioned). ENDGAME: when
> the tier is green, fold scripts/integration-run.sh into green-gate.sh
> (task #151) so it can never rot silently again.

## The rooms

| room | binary(ies) | tests | verdict | status |
|---|---|---|---|---|
| 1 | wat_arc148_ord_buildout | 46 | CONFERRE'D: 40 FILL (real gap — 237.8b's recorded future-stone, DESIGN-STONE-237.8b.md:211, never opened) + 6 MODERNIZE (rejection moved to check-time) → **STONE 245.8: ordering is relational** | **CLEAR** (`f681d1d0`, 46/46) |
| 2 | wat_arc098_form_matches_{runtime,typecheck} | 15+7 | CONFERRE'D: FILL — matches?'s pattern arg is DSL data; the resolver never got its boundary when strictness landed (quote-family precedent mirrored, resolve.rs); struct-head validation owned by check.rs infer_form_matches (live witness) | **CLEAR** (`35dfc10d`, 15/15 + 9/9) |
| 3 | wat_arc150_variadic_define | 14 | CONFERRE'D: 3 layers — (i) 11 test argspecs CORRUPTED (phantom `_b <- &` triples, modernized); (ii) under that, REAL GAP: top-level user variadic defn never registered — try_parse_fn_shape_def's `.ok()?` SILENTLY swallowed RestBinderNotSupported (the 249.5d rest support reached macros/defclauses, never the user defn path); FILLED across registration/check/eval + reflection; (iii) 4 negative tests re-grounded at empirical shapes. NOTE: src/function/ stamp drifts — re-ward owed at campaign end (with core.wat) | **CLEAR** (`f3d1fc9e`, 16/16) |
| 4 | wat_core_cond | 9 | — | pending |
| 5–33 | the long tail (29 binaries) | ~56 | — | pending |

Separate buckets (NOT this tier, cleared after): crates/wat-holon-lru 19
(struct-rot, named in `94261f45`); the 67 excluded arc-170 process binaries
(leaky-signal; gated behind #151's run-tier).

## Room 1 — the ordering stone (245.8)

Census of the 46: Instant ×4 · Duration ×4 · Bytes ×4 (incl. prefix-tie) ·
Vec ×6 (incl. recursion) · Tuple ×6 · Option ×6 (None<Some) · Result ×6
(Err<Ok) · holon-algebra Vector ×4 → **FILL** (every one backed by a LIVING
`values_compare` arm, runtime.rs ~9187 — the arc-148 engine survived 237.8b;
only the check-side reachability died). HashMap/HashSet/enum/Struct/unit/
HolonAST `*_raises_type_mismatch` ×6 → **MODERNIZE** (they expect rejection;
they now get it EARLIER, at check).

Four-questions verdict (driven): **ordering joins equality as a relational
intrinsic** — the inscribed PARTITION rule (check.rs `infer_list`; the 237.8c
equality-reversal precedent) applies verbatim: ordering over parametric
containers needs `unify(arg0, arg1)` = arg↔arg type-flow = INTRINSIC; a
defclause is monomorphic and cannot express it. Hybrid fails Simple (two
mechanisms, one op); rejection fails Honest (contradicts 8b's recorded promise
+ the alive engine). Cross-type stays rejected (unify(i64,f64) fails — the
same-type doctrine survives the mechanism change). `values_compare` returning
`Option<Ordering>` IS the orderable-class discriminant; the checker mirrors it.

Owned consequences: the 4 ordering defclauses in wat/core.wat RETIRE
(one-canonical-path) → core.wat's fresh stamp DRIFTS (correct; re-ward on
touch, re-stamp after); `i64::<`/`f64::<` leaves STAY (the type-locked tier);
the corpus cross-type ordering rejection tests + USER-GUIDE's
ordering-is-defclause prose update to the new truth; the 6 rejection tests
modernize to check-time assertions.
