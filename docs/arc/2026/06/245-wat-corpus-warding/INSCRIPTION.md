# Arc 245 — INSCRIPTION — warding the wat corpus, and the bar that learned to mean it

**Closed 2026-06-04. The wat stdlib — the surface every program composes on — is now a warded family, and the warding forged the honest bar it didn't have when it started.**

## The thesis

`src/` had warded Rust homes; the `wat/` stdlib — the surface users actually call — was untrusted-by-default. That is the *src-warded / wat-untrusted asymmetry*, and arc 244's doctrine names an unwarded member of a warded family a **defect, not a quirk**. Arc 245 set out to raise the wat corpus to a defined bar and inscribe the proof. What it discovered is that it had to *build* the bar first — and that the act of building it caught a class of lie hiding in every single file.

## The instrument (245.0)

A spawned intueri cast returned a flat **REUSE** verdict: don't mint a "wat-ward" — `vigilia` is already kind-adaptive (it names `cernere` for wat/DSL conformance), so the instrument is **vigilia cast with wat-kind selection** (intueri/cernere/conferre + circumspicere). The stamp stays `vigilatum`. The clippy-shaped L2 had no wat analog, so the bar was first written `checker-clean + suite-green`. That guess did not survive contact.

## The bar, forged honest

The first ward (`list.wat`, 245.1) **twice-refined the bar**:
1. **Teeth** (circumspicere): `suite-green` is toothless — a file passes the suite while its own forms are never executed. → the file's forms must be *exercised by a passing test*.
2. **Honesty** (grounding `green-gate.sh`): there *is* no green integration suite — it's excluded by design (arc-170 leaks). `suite-green` was a lie. → **`checker-clean + deftest-green(<name>)`**: a NAMED, deterministic, currently-green deftest exercising the file's own forms, self-verifiable at source. Routine-gating-against-rot is task #151 / arc 250, not a stamp claim.

That bar is the arc's central deposit. Every later file proved why it had to exist.

## The territory marked

**16 stdlib files WARDED** to L1+L2=0, each `deftest-green`:
- `list.wat` (245.1) — first ward; the bar's birthplace.
- `core.wat` (245.2) — the most important file; a 3-front stone (135 lines of arc-archaeology cut, the `=`/ordering/arithmetic deftests, a stale RED graveyard retired).
- `edn.wat` (245.3b) — the Tagged/NoTag newtypes; circumspicere caught roundtrip.wat testing the *shim* not the newtypes → a real in-band tag-drop deftest.
- `Record.wat` (245.3c) — the defrecord macros; conferre caught the D10 "class-safety OUT OF SCOPE" lie (the macro *emits* the guard); a record deftest + the 3/18-RED probe fixed.
- the **holon family** (245.4) — `holon.wat` + 11 leaf encoders; the biggest batch; conferre fixed holon.wat's stale 4-arg Hologram/get API; circumspicere forced 3 new deftests (Amplify/Bigram/Ngram shipped untested).

**Honest non-wards:**
- `runtime.wat` (245.3a) — **retired** (a formless vestige; the namespace is Rust-defined).
- `stream.wat` (245.3d) — **deferred → lazy-seqs** (doomed; the eager thread-per-stage impl the lazy rework replaces).
- the **kernel family** (245.5, 7 files) — **deferred → arc 170** (the concurrency layer itself; channel is typealias-only, the rest are fork/spawn/leaky — un-deftest-green-able until 170 closes the leaks).
- `wat-tests/*` (245.6) — **skipped** (tests are coverage/scaffolding, not the warded surface; the bar doesn't apply to a deftest).

## The deposits (the real yield)

1. **The deftest-green bar** — the central refinement; a stamp that attests only what the build can re-verify.
2. **circumspicere is the load-bearing lens for wat.** The inward guard clears comments and forms; circumspicere caught the *file's-own-forms-unexercised* trap on **list, core, edn, Record, AND holon** — every file had shipped surface no test touched. Without it, five stamps would have lied.
3. **Stale arc-archaeology is wat's borrow-checker.** The dominant L1/L2 class across the whole corpus was historical narration that rotted while code moved: `DispatchRegistry` gone but "remains," `infer_*` citations pointing at pre-arc-246 locations, the D10 out-of-scope lie, the `per-058-NNN`/`arc-0NN` coordinate residue in every holon leaf, Hologram's pre-arc-076 4-arg examples. Reconciliation (conferre grounds the lies, intueri strips the residue, history → the arc record) is the corpus pattern.
4. **Adjacent debt dragged into the light + cleared:** the arc-237 mixed-promotion graveyard (`wat_polymorphic_arithmetic.rs`), the arc-241 struct-rot (`:wat::core::struct`→`defstruct` in roundtrip.wat; banked for struct-to-form + counter-service), the arc-242 `Char/of`→`char/of` rename, the 3/18-RED record probe, redef conflicts, and ~5 encoder coverage gaps.
5. **Two disciplines banked** (`feedback_weighty_fork_still_four_questions` — a weighty-feeling fork is STILL four-questions, recurred 3×; the substrate-brief shell hygiene — plain commands, the agent doesn't run the gate) + a doctrine extension (typealias-only files ward as checker-clean, deftest-green N/A).

## FM-11 — DONE, no false deferral

Every ward is verified against the *committed* tree (form-integrity diffs, green-gate, deterministic deftests run twice). Every non-ward is honest: runtime *deleted* (not stub-shimmed); stream and kernel *deferred with named target arcs and recorded reasons* — not "we'll do it later" hand-waves, but four-questions-grounded "this cannot be deftest-green now, and here is who owns it." The territory the arc could honestly mark, it marked; the territory it couldn't, it labeled with the arc that will.

## The close

Arc 245 closes. The wat stdlib's deterministic surface is warded and *means it*; the concurrency layer (stream, kernel) waits for its arcs (lazy-seqs, 170), labeled not lied-about. The bar that started as a guess — `suite-green` — was forced by the first file and the lens that turns around into `checker-clean + deftest-green(<name>)`, a stamp that cannot go false without a named green test going red.

The resumption gate advances: **~~245~~ ✓ → 249 → 235 → rejoin 232.** The wat corpus is no longer untrusted-by-default. 🔦🗡️

---

# Arc 245 — INSCRIPTION-II — the REOPEN: the full clear + the test-surface ward

**Closed 2026-06-06.** The arc-245-v1 INSCRIPTION above is preserved as
shipped — the wat stdlib warded, the bar forged, 16 files stamped. This
section records the REOPEN that the v1 close itself named with one honest
deferral and one corrected scope.

## The reopen, named at v1's close

V1 marked `wat-tests/*` as "skipped — tests are coverage/scaffolding, not the
warded surface." The user's correction, on reading that: **"that's like the
most important thing to ward — the tests are our demos of how to use wat
well."** That single sentence reopened the arc against its v1 close.

Two months later (after 246, 247, 248, 249, 250 shipped), the reopen
discovered the v1 stdlib warding alone wasn't the corpus's whole problem.

## The full clear (2026-06-06)

The default integration tier ran red in the dark for weeks — the routine
gate was lib-only, leaving 33 fail binaries / 147 fail tests invisible.
245.7 built the leak-contained runner (`scripts/integration-run.sh`) +
recorded `INVENTORY-245.7-baseline.tsv` as the ground-truth map. Then the
clear, in five strikes:

- **Stone 245.8 — ordering relational** (46/46, `f681d1d0`): the 237.8b
  recorded future-stone, opened. The runtime `values_compare` engine was
  alive but unreachable from check; `infer_ordering` minted as the sibling
  of `infer_equality`. The four ordering defclauses retired from
  `wat/core.wat`. Cross-type stays rejected (unify(i64,f64) fails);
  same-type doctrine preserved. The NoMatchingClause class went **extinct**
  in the tier.
- **room 2 — matches? boundary** (22/22, `35dfc10d`): the resolver never
  taught about `matches?`'s DSL pattern argument; cleared by the
  quote-family precedent.
- **room 3 — variadic defn resurrected** (16/16, `f3d1fc9e`): the silent
  `.ok()?` swallow of RestBinderNotSupported in
  `try_parse_fn_shape_def`. Top-level user `defn` never registered its
  variadic form (macros + defclauses got rest in 249.5d; user defn never
  did). Threaded across registration / check / eval / reflection.
- **room 4 — cond `:else`** (10/10, `2d518d78`): the finer matches?-sibling
  resolver boundary.
- **the long tail** (55/55, `ce655a68`): 27 binaries, three classes —
  third-wave resolver strictness (single-segment keyword accessors), error
  shape drift (assertion-update to live variants), arc-242 nil-rot in
  embedded sources. Plus the lru bucket separately (`e148540d`, 19/19 —
  retired arity-suffix rot killing the service driver SILENTLY behind
  `(Err _)` discards; the doctrine generator for #181's own findings).

**Result: 1646 / 0 / 59 across 187 binaries. THE TIER IS GREEN.** Then
the ENDGAME (#151, `ef585672`): the runner folded into `green-gate.sh` as
check 3/3 — the tier cannot silently rot again.

## The splice stone — re-diagnosed and cut (`9d7f93e6`)

A carry-out from the clear. Old framing: "runtime param-binding gap." The
re-diagnosis was sharper: the failing pattern is textbook ANAPHORIC
CAPTURE, which the 249.5 hygiene work refuses BY DESIGN (it only ever
passed under pre-hygiene name-only resolution). Re-cast as the permanent
witness `anaphoric_splice_capture_refused_by_hygiene`. The REAL gap,
probed three layers deep: `walk_quasiquote`'s Vector arm never learned the
249.3a `~@` splice — env-bound list form-values landed as one lump in
argspecs. Fixed by mirroring the List arm's depth-1 splice loop; the
hygienic macro pattern (body names computed from the spliced material) is
now fully expressible (live contract:
`hygienic_splice_adder_binds_via_spliced_names`).

**The discipline note worth carrying:** an interim `#[ignore]`'d "red
contract for a future stone" was caught by the builder as a
survivor-in-the-making and reverted; the stone was cut immediately
instead. The dungeon-clear's "no survivors" doctrine extends to tests
written *for* future work.

## #181 — the test-surface ward (the v1 reopen, paid)

Six casts of the test-kind guard (the guard's first full muster — cernere,
intueri, probare, vocare, exigere, complectens) + circumspicere LAST. The
findings vindicated the whole reopen:

- **15 latent bombs** (cernere): live retired forms hidden inside
  `#[ignore]`'d proof files — they detonate at startup the day arc-170
  lifts the ignores. The dark-corner-inside-the-dark-corner class. All
  15 defused (`096237e2`); the trickiest two struct-restricted →
  defstruct migrations contained-un-ignore-verified (the panic moved
  from `test_runner.rs:459` startup to `:487` run phase — CHECK CLEAN,
  failing only on the arc-170 concurrency they are correctly ignored for).
- **5 framework-doc lies** (intueri): the deftest expansion docs showed
  retired `define`, "deftest currently expands to run-hermetic" (flipped
  long ago), `deftest-hermetic` named via a retired mechanism, the README
  taught a `test-` prefix rule the runner dropped 2026-04-25. All fixed.
- **4 hollow tests** (probare + vocare): `assert-eq true true` sentinels
  proving nothing, one on-mismatch-only helper passing silently, one
  vocare CANNOT-FAIL test. All realified.
- **4 tmp-* regression guards** in scratch clothing (probare): renamed to
  purpose under `core/` (`9b91368e`, then deftest namespaces
  tmp::→core::).
- **4 exigere deferrals** rewritten to present-truth.
- **circumspicere LAST**: F5 (2/3 unexercised public verbs witnessed —
  run-in-scope honestly skipped, its ScopedLoader-for-load! distinguishing
  behaviour requires a fixture-file demo outside this stone's scope), F6
  (2 underscored filenames renamed to the README's own hyphen convention),
  F7 (7 internal helpers marked); F4 → **#181-followon: arc-170
  ignore-removal gate** (task #183, named-stone — the gate's *shape*
  needs slow-head, not an inline tail; #151-doctrine sibling).
  All landed `46273b73`.
- **`wat/test.wat` STAMPED** (2026-06-06T09:50:50Z, this commit) — the
  framework is the warded unit; the corpus (`wat-tests/**/*.wat`) is its
  demonstrated surface, running 238/0/53 corpus-green and inscribed into
  green-gate check 3/3 by #151 so the demos cannot silently rot.

## Affirmative scope cuts (the test-surface ward's bounds)

- **9 fully-dark proof files + complectens 8 L2 sibling-deftest gaps** →
  the composition wards CANNOT be earned while the arc-170 concurrency
  layer leaks; affirmatively scoped to the **arc-170 reanimation arc**.
  When those tests run again, they get the layering ward. Named, not
  silent. (Cf. the test-surface ward record: `WARD-TEST-SURFACE.md`.)
- **`run-in-scope` fixture-demo** (F5): the distinguishing behaviour
  requires a load-fixture file that doesn't belong in the framework's own
  test file. Named in the wat/test.wat stamp's invariant #2 as an honest
  gap, not a phantom witness.
- **The arc-170 ignore-removal gate** (F4 → #183): a slow-head sibling
  stone tracking the enforcement mechanism for "lift the 53 ignores when
  arc-170 closes." Cited in the wat/test.wat stamp's invariant #4 so the
  stamp doesn't lie about F4 being closed.

## The deposits (REOPEN's real yield)

1. **The full tier as a routine gate.** `green-gate.sh` now runs the
   leak-contained integration tier as check 3/3 (#151). The single
   highest-leverage change of the whole REOPEN: the tier that ran dark
   for weeks cannot ever again. The 67 leaky-signal binaries excluded
   by the heuristic remain the arc-170 frontier.
2. **The diagnosis-evolves-honestly pattern, three times.** The lru bucket
   (suspected struct-rot → actually retired-arity-suffix driver-silent-death
   behind `(Err _)`); the splice stone (suspected runtime gap → actually
   hygiene-refusing-anaphoric-capture + a real `~@` Vector-arm gap); the
   #181 Half-B defuse (suspected purely-mechanical → actually surfaced
   that `struct-restricted` was itself resolve-broken pre-migration). The
   discipline: when probes contradict the brief, the *brief* updates.
3. **"The tests are the demos" inscribed as discipline.** The corpus-as-
   teacher doctrine forced F5's witnesses, the hollow-test realification,
   and the tmp-* renames — not for code quality, for *pedagogical truth*.
   A test the reader can copy is the unit; a sentinel that teaches
   nothing is a lie even if it passes.
4. **The contained-un-ignore verification.** The strategy doc demanded
   the Half-B defuses be proven beyond the gates' reach (gates can't see
   ignored bodies). The verification surfaced findings the gates never
   could AND a *better* truth than the brief assumed.

## FM-11 — wrap-proof grep

`tr '\n' ' ' | tr -s ' ' | grep -oE "<full pattern>"` — the wrap-blind
fix shipped at the 249 close, applied here. Run + judged. Every match
falls in an accepted affirmative-scope form (`out of <arc>'s scope`,
`tracked in <named-arc>`, `when a caller surfaces` paired with NEW ARC
language). No L1 deferral language survives.

## The close

Arc 245 closes — *again*, and this time the corpus is not just stamped
but **gate-protected**, **its demos teach**, and **its dormant bombs are
out of it**. The resumption gate advances exactly as v1 named:
**~~245~~ ✓✓ → 249 ✓ → 235 → rejoin 232.** 🔦🗡️
