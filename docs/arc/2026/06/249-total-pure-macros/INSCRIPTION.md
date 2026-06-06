# INSCRIPTION — Arc 249: macros are total, pure programs over forms

**Closed:** 2026-06-06. Branch `arc-170-gap-j-v5-deadlock-state`; closing tree at
`5ce98e0b` (every stone committed + pushed individually — see the per-stone
record below).

## What the arc set out to do, and did

249 opened as a small threading-macro verdict and revealed its true scope while
building it: a wat macro body was a *template*, not a *program*, so the entire
idiomatic Clojure macro family was inexpressible in wat — a ceiling on the
clojure-on-rust identity. The arc lifted that ceiling and made the macro layer
the third axis of "Clojure-shape + added rigidity" (after type strictness, arc
237, and dialect honesty, arc 247):

> **A wat macro body is a total, pure program over forms.** Expansion always
> terminates (combinators, no open recursion — totality is free), and is a
> deterministic effect-free function `forms → forms` (the kernel namespace is
> the effect boundary; the engine's default-deny allow-list enforces it).
> Same source → same expansion → same canonical hash, on any machine, always.

## The stones — every DESIGN commitment shipped

- **249.1 — the canary** (`6ba27ca0`): threading verdict + Rust desugar,
  REMARKABLE one-shot. Its probe became the behavioral contract for 249.3.
- **249.2a — the home**: `src/macros.rs` (flat, 2415 lines) lifted into the
  warded `src/macros/` home (registry/parse/expand/eval/error/tests, names by
  intueri cast). The `vigilatum` stamp was deliberately HELD at this stone —
  a stamp claims annihilation, and two findings (F5, the eval-context) stood
  open until the engine closed them by enforcement.
- **249.2b — the engine**: the fenced macro-eval engine (`src/macros/eval.rs`)
  — `macro_eval` + `validate_pure_total`, a default-deny allow-list mirroring
  the runtime's pure-total dispatch arm. It SUBSUMED the arc-143 computed-
  unquote path: expand-time eval is gated to the pure set **by enforcement,
  not convention** — F5 (the impurity/determinism hole that let an impure
  `,(expr)` make the hash depend on runtime state) closed structurally.
- **249.3 (a/b) — threading reborn as wat**: engine form-vocabulary + purity
  fence + `~@`-splice + `List?` (249.3a), then `->`/`->>` as plain wat
  defmacros in `wat/core.wat` over the engine, passing the 249.1 contract
  probe — and the Rust `thread_desugar` HARD CUT (249.3b). Threading proved
  the model.
- **249.4 (a/b) — cut what the engine obsoletes** (Honest-forced):
  `keyword/of` promoted from a Rust built-in (`construct_keyword_of`,
  DELETED) to a pure-total wat macro (249.4a). `for` was not rehomed — it was
  **ANNIHILATED** (249.4b): the engine's `map` + `~@` splice IS the
  comprehension, so a `for` macro would have been a second way to do one
  thing; `match_for_comprehension` cut with it. The one-canonical-path
  doctrine settled this in-arc.
- **249.5 (a–g) — the hygiene completion** (grew from 249.2a's ward: the
  "capture is structurally impossible" claim was a lie — the expander's scope
  tags were inert at resolution). The macro-variable-capture CLASS was
  annihilated across all three keying surfaces and proven last by
  enumeration: runtime resolution via `env_key` (249.5b + the ArgSpec
  root-fix 249.5d, which deleted the strip-and-re-walk class), check-side
  resolution (249.5e), and hash identity via canonical first-appearance scope
  renumbering (249.5f) — then GATED so it cannot silently reopen (249.5g, the
  scopes-reader gate). `src/scope/` was minted as the home for the
  sets-of-scopes primitives (249.5a).
- **249.N — the ward-close**: every drifted or held stamp re-earned against
  the COMPLETE updated vigilia (the 2026-06-05 release with conditional
  triggers), each home driven to L1+L2=0 through inward rounds + a
  circumspicere perimeter + verified-first fight sweeps:
  - `src/scope/` STAMPED (`a30b3132`) — 10-spell guard.
  - `src/macros/` STAMPED (`8cb0f9c5`; timestamp retrofit `c0c6b230`) — the
    stamp HELD since 249.2a finally earned: 13-spell guard, 47 findings
    fought across three sweeps, 4 L1 killed (among them the unwitnessed
    hash-IS-identity claim — now alias-vs-direct hash-equality-tested), 6
    invariants behind living gates incl. the pre-validated-caller gate.
  - `src/collection/` RE-STAMPED (`330e4a6c`) — 10-spell guard; 5 L1 killed,
    the standout by elimination: typed `List<T>` now flows through
    polymorphic `length`/`empty?` (check ≡ runtime); record-assoc
    single-evaluation made TYPE-enforced; 13 new witnesses.
  - `wat/core.wat` RE-STAMPED (`5ce98e0b`) — spec/DSL 5-spell guard; cernere
    + exigere converged 0+0; all 6 conferre divergences were SPEC-side and
    fixed in `docs/USER-GUIDE.md`; the false "loads early" rationale killed
    at both sites; THE NAMED THREADING DEFTEST shipped
    (`wat-tests/core/core-threading.wat`, 10 deftests incl. zero-step
    identity) + the alias witnesses (`core-collection-aliases.wat`); the
    corpus grew 217 → 236, all green.

  The canonical ward records: `WARD-MACROS-UPDATED-GUARD-AGGREGATE.md`,
  `WARD-COLLECTION-REEARN.md`, `WARD-COREWAT-REEARN.md` (all in this arc dir).

## Closing gates (orchestrator-run, first-hand, at close)

- `cargo build --release --tests -p wat` — clean
- `cargo test --release --lib -p wat` — **920 / 0 / 1**
- `cargo test --release -p wat --test test` — **236 / 0 / 53** (the corpus)
- The hygiene + gate probe family — all green (capture-prevention ×3,
  argspec-rest ×1, check-scoped ×2, hash-renumber ×3 incl. the alias-hash
  witness, scopes-reader gate ×2, pre-validated-caller gate ×2, threading ×8,
  collection-transform ×13)
- `cargo clippy --release -p wat` — empty on every warded home

## Scope boundaries (affirmative)

- **The idiomatic macro library (`cond->`, `as->`, `when`, `condp`, `case`,
  …) and `&form`/`&env`:** arc 249 intentionally does NOT cover these. The
  engine enables them but shadows nothing by their absence — there is no
  redundancy forcing them in-arc, and no caller has surfaced demand. If/when
  a caller surfaces, a NEW ARC opens; this INSCRIPTION does not commit to
  one.
- **Type-level `pure total` effect (user-fn macro-callability, "v2"):** out
  of arc 249's scope; substrate-architectural reason: it reconnects the
  purity axis with the type system and stands on its own design; not tracked
  elsewhere because the macro layer is complete without it — user `defn`s
  are simply not macro-callable, which the engine enforces today.
- **Corpus pedagogy completeness (the defn-metadata corpus demo and the
  broader test-surface ward):** out of arc 249's scope; tracked in the
  test-surface ward (task #181, opened by the arc-245 corrective: the tests
  ARE the demos).
- **The empty-list-step asymmetry (`->` rejects loudly at expansion; `->>`
  splices to `(acc)` and fails at eval):** documented and witnessed at its
  empirical shapes in this arc; arc 249 intentionally does NOT redesign the
  behavior — both paths fail loudly, and a semantic change is a language
  decision a ward-close does not own. If/when the design question is taken
  up, a NEW ARC opens; this INSCRIPTION does not commit to one.

## What the arc deposited (the durable capabilities)

1. **The two invariants, enforced**: TOTAL (combinators, depth-gated,
   constant == behavior) + PURE (default-deny allow-list; the deny direction
   is the only drift direction, and the witnesses prove the denials).
2. **Hygiene as a closed class**: minted at one site, read at exactly two,
   gated against new readers — capture is now *provably* structurally
   impossible, the very claim that started 249.5 as a lie.
3. **Hash-IS-identity for macro programs**: cross-run (scope renumbering) AND
   cross-source (alias-vs-direct equality), both witnessed.
4. **The kill-oscillation at scale**: four homes through cast → fight →
   fresh-eyes re-cast → circumspicere-last → stamp, termination by judgment
   each time; the perimeter lens earned its place at every single home
   (an unwitnessed substrate commitment, a cross-pass egress, a false
   load-order rationale — none visible to any inward lens).
