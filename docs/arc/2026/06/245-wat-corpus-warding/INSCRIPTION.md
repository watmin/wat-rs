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
