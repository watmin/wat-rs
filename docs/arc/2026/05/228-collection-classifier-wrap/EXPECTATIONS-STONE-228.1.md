# EXPECTATIONS — Arc 228 Stone 228.1 — Substrate collection classifier-wrap + Pascal-Case constructor verbs

Mode A target: 14/14 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | NEW `:wat::holon::Map` constructor verb minted | Rust fn (`eval_algebra_map` or similar); dispatch table entry; TypeScheme + check.rs registration; takes `Vec<HolonAST>` of Bind pairs; produces `Bind(Atom("Map"), Bundle(items))` |
| 2 | NEW `:wat::holon::Set` constructor verb minted | Same shape as Map; takes `Vec<HolonAST>`; produces `Bind(Atom("Set"), Bundle(items))` |
| 3 | NEW `:wat::holon::Vector` constructor verb minted | Takes `Vec<HolonAST>`; substrate auto-applies positional Bind keys; produces `Bind(Atom("Vector"), Bundle(positional Bind pairs))` |
| 4 | NEW `:wat::holon::List` constructor verb minted | Takes `Vec<HolonAST>`; produces `Bind(Atom("List"), Bundle(sequential bare items))` |
| 5 | NEW `:wat::holon::Tuple` constructor verb minted | Same internals as Vector (positional Binds) but distinct outer classifier `Atom("Tuple")` |
| 6 | `to_holon_inner` HashSet arm — classifier-wrap | Produces `Bind(Atom("Set"), Bundle(items))` instead of bare Bundle |
| 7 | `to_holon_inner` Vec arm — classifier-wrap | Produces `Bind(Atom("Vector"), Bundle(positional Binds))` instead of bare Bundle |
| 8 | `to_holon_inner` Tuple arm — classifier-wrap + distinguished from Vec | Produces `Bind(Atom("Tuple"), Bundle(positional Binds))`; NOW DISTINCT from Vec at substrate (was identical) |
| 9 | `to_holon_inner` HashMap arm — classifier-wrap | Produces `Bind(Atom("Map"), Bundle(K-V Binds))` instead of bare Bundle |
| 10 | `to_holon_inner` List arm — classifier-wrap | Produces `Bind(Atom("List"), Bundle(sequential bare items))` instead of bare Bundle (if List arm exists; mint if missing) |
| 11 | NEW helper `extract_classifier(&HolonAST) -> Option<String>` | Returns classifier name if outermost form is `Bind(Atom(String(name)), _)`; None otherwise |
| 12 | `eval_holon_from_holon` updated to dispatch by classifier | First tries `extract_classifier`; dispatches by name ("Map"/"Set"/"Vector"/"List"/"Tuple"); no fallback to bare-Bundle heuristic (per HARD CUT discipline — bare-Bundle decode errors with helpful diagnostic) |
| 13 | Arc 216 probe tests updated for new encoding | `probe_arc216_stone1_hashset_roundtrip` / `stone2_vector` / `stone3_hashmap` / `stone7_tuple` — assertions updated to check for classifier-wrapped Bind shape instead of bare Bundle; all PASS |
| 14 | All test suites green + holon-rs untouched | `cargo build --release -p wat` 0 errors; `cargo test --release --lib -p wat [--skip 5 signal]` PASS; all arc 216 probes PASS; arc 221/143/mvp_end_to_end PASS; `cargo test -p wat-edn` PASS; `cargo clippy --release --all-targets -p wat-edn -- -D warnings` 0 warnings; `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` empty |

## Independent prediction (calibration record)

**Target runtime:** 120-240 min Mode A
**Upper bound:** 300 min
**Confidence:** medium-high

**Rationale:**
- Closest precedent Stone 225.1 v3: ~68 min for ~150-200 site rename + 5 deliverables (faster than predicted due to mechanical nature)
- This stone is more substrate-internal (less wat-side caller sweep) but adds 5 NEW constructor verbs + dispatch refactor
- Encoding-cascade may break arc 216 probes; sonnet updates assertions per Phase 4 cascade methodology

**Risks:**
- Forward-correcting arc 216 doctrine — assertions in probes will fail with NEW expected vs OLD actual; sonnet needs to update assertions (NOT the encoding); per Stone 221.3 Delta 1a framing (broken-by-this-stone honest framing)
- HashMap arc 216 Stone 5b/5c (impl Hash for Value) — round-trip must still work with classifier-wrap on both sides
- `extract_classifier` helper is NEW code — sonnet needs to choose the right return type (`Option<String>` vs `Option<&str>` vs `Option<HolonAST>`); pick consistent with rest of substrate

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- holon-rs changes (arc 230)
- wat-edn changes
- Type predicates (arc 226)
- User-defined types (arc 227)
- EDN-form parser-level minting (arc 222)
- WatAST honesty (arc 223)
- Quasiquote evaluator (arc 229)
- INSCRIPTION (Stone 228.4)
- Aliases (HARD CUT)

## Honesty deltas accepted

- Rust function names for new constructor verbs may vary if sonnet finds a more honest name during the sweep — document in SCORE
- `extract_classifier` return type signature — sonnet picks; load-bearing point is "returns classifier name on classifier-wrapped form; None otherwise"
- Number of arc 216 probe updates may exceed pre-flight estimate; cascade absorbs
- The arc 216 in-code comment headers — sonnet decides which to update (Phase 6); fix what you touch

## Honesty deltas NOT accepted

- "Pre-existing failure" framing for arc 216 probe failures (those are broken-by-this-stone per Delta 1a)
- Skipping any classifier-wrap arm per "didn't want to break the probe" — STOP. Hard cut means hard cut. Forward-correct arc 216.
- Touching holon-rs — STOP per STOP-4
- Adding aliases for the bare-Bundle decode path — STOP. HARD CUT.
- Extending scope to type predicates or user-defined types — STOP per STOP-5; surface as finding; orchestrator decides

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** unexpected substrate compile errors (not from rename cascade)
- **STOP-2:** test failure beyond cascade-rename consequences after green build
- **STOP-3:** 300 min elapsed
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** scope-extension surfaced beyond 5 collection types
- **STOP-6:** round-trip semantics break (e.g., HashMap silently corrupts)
- **STOP-7:** bash discipline — cargo hang from accidental pipes
