# FINDINGS — vigilia 2026-09-07, rete

> ⛔ **THIS FILE IS THE ONLY PLACE A ROW'S STATUS LIVES.** Edit in place; never append a closure
> below a row. `reports/` holds each ward's verbatim return as EVIDENCE — it carries no status, and
> a status read from a report rather than from this table is a status nobody maintains.
>
> The rule and its cost: `WORK-LIST.md:10-13` of the 2026-09-05 cast said *"One row, one place"*,
> and `RETE-BOARD.md` — same directory, four days later — shipped its own status column anyway.
> Both copies then rotted in lockstep: **nine rows read OPEN while every one was already cured.**

> ⭐ **EVERY ROW CARRIES A RE-DERIVATION** — the command, gate or grep that decides its status
> *now*. A row without one is a claim, and this arc has proved that claims rot silently while
> nobody re-checks them. If you cannot write the re-derivation, the finding is not yet understood
> well enough to row.

**Severity:** L1 = a correctness lie · L2 = a structural mumble · L3 = taste (noted, does not count
toward convergence). Passed through from each ward verbatim — this table never re-classifies.

**Priority is not written down.** Apply `vigilia`'s rule at read time — L1 before L2, most upstream
ward first — against status you have just re-derived. The last board's static order named a cured
row first for days.

## Rows

| id | ward | target | site | finding | sev | status | re-derivation |
|---|---|---|---|---|---|---|---|
| **I1** | intueri | 1 | `fire/pass/accumulate.rs:18` | `accumulate_pass`'s doc promises only *"dispatch the accumulate nodes"*; the body ALSO runs pass 3.20 — pre-dispatching `Test` parents that feed an accumulate, so `filter_pass` can skip them. A reader trusting the doc misses that a sibling pass's inputs are seeded here. | L2 | **OPEN** · ✅ I VERIFIED | `grep -q dispatch_where_tests src/rete/kernel/fire/pass/accumulate.rs && sed -n 18p … \| grep -qiv pre-dispatch` — closed when the doc names both responsibilities, or 3.20 becomes its own fn |
| **P1** | purgare | 1 | `wat/rete/oracle/fire.wat:231-254` | `retain-supported` is DEAD — superseded by `factbag::retain` during the factbag-one-owner strike (drawn this session) and never deleted. Zero call sites. | L2 | **OPEN** · ✅ I VERIFIED | `grep -rn 'retain-supported' wat/ wat-scripts/ wat-tests/ tests/ src/` → 3 hits, all in `fire.wat`, none a call |
| **P2** | purgare | 1 | `stratify.rs:444-471`, consumed `:1045` | `TerminationProof`'s two variants are constructed but the DISCRIMINANT is never read — the only consumer does `let _ = proof;`. | L2 | **OPEN** · ⚠ ward-reported | closed when something branches on the variants, or it collapses to a marker, or it carries a rune |
| **S1** | solvere | 1 | `stratify.rs:1-2` vs `:330-1067` | Module doc claims stratum-numbering only; ~69% of the file is a fixpoint-TERMINATION verifier — a different concern, unnamed in the header. | structural | **OPEN** · ⚠ ward-reported | closed by a split or an honest header |
| **S2** | solvere | 1 | `stratify.rs:532-544` | The two desugared constructor heads are hardcoded here AND maintained in `purity.rs`'s ratchet. Drift fails silently. | incidental | **OPEN** · ⚠ ward-reported | `grep -c 'kwargs-construct' …` — closed by one shared constant |
| **S3** | solvere | 1 | `fire/rules.rs:80-96` + `:522-530` | Forward→reverse children inversion hand-written twice in one file. | incidental | **OPEN** · ⚠ ward-reported | closed by one `build_rev_children` helper |
| **S4** | solvere | 1 | `fire/rules.rs:551-571` + `:401-416` | Query closure computed twice per non-monotonic requery — once for a boolean gate, once for the work. Defended on COST, not architecture. | incidental | **OPEN** · ⚠ ward-reported | closed when the gate hands its closure to the harvest step |
| **S5 ★** | solvere | 1 | `fire/pass/mod.rs:78-139`; dupes at `join_after_filter.rs:84-113`, `filter_after_join.rs:208-243` | ⭐ The helper's doc header opens **"ONE COPY"** and it is called from exactly ONE of three sites. The full body is still inline at both originals. Fire-hot-path join logic, and a doc comment that lies. | structural | **OPEN** · ✅ I VERIFIED | `grep -rn 'left_activate_join' src/rete/kernel/` → 1 call; both other ranges still hold the body |
| **S6** | solvere | 1 | `filter.rs:231-288`, `filter_after_join.rs:120-166` + `:247-297` | Negation/Exists dispatch triplicated; the one real divergence is `filter.rs`'s `leading_emitted` dedup. | structural | **OPEN** · ⚠ ward-reported | closed by one shared helper |
| **S7** | solvere | 1 | `fire.wat:276,347,509`; `explain.wat:66,92`; `insert.wat:52` | The "unwrap a must-succeed Outcome" 3-arm protocol re-derived at ~7 sites. | incidental | **OPEN** · ⚠ ward-reported | closed by `expect-fired` / `expect-inserted` helpers |
| **S8** | solvere | 1 | `pass.wat:459-502` vs `explain.wat:10-49` | "Derive facts for a token via RHS" walked twice; the two diverge only in what they do with the fact. | incidental | **OPEN** · ⚠ ward-reported | closed by a shared `derived-facts-for-token` |
| **S9** | solvere | 1 | `accum-pass.wat:28-142` | A justified braid MISSING ITS RUNE — forced by wat's invariant parametric types, documented in prose at `:11-16`, but no `rune:solvere(load-bearing-coupling)`. | structural (paperwork) | **OPEN** · ⚠ ward-reported | `grep -c 'rune:solvere' wat/rete/oracle/accum-pass.wat` → 0 |
| **T1 ★** | struere | 1 | `fire/mod.rs:1527`, `:1744`, `:1822` | The join-key family **panics** — 11 sites — where `kernel/outcome.rs` states the law *"a dynamic failure is a value a caller must match, never a raise."* `driver_of` in the SAME FILE returns `Result<_, EvalBreak>` for the identical hazard. | L2 | **OPEN** · ✅ I VERIFIED | `driver_of` → `Result<&CondDriver, EvalBreak>`; the three key fns → 11 `panic!` |
| **T2** | struere | 1 | `fire/acc.rs:13-15` | `rune:struere(host-constraint)` whose reason (*"would duplicate `BindView`"*) is duplication-avoidance, not a missing language mechanism. | L2 | **OPEN** · ⚠ ward-reported | closed when recategorised or the reason names a real host limit |
| **T3** | struere | 1 | `accumulate.rs:222,308`; `filter.rs:161` | The 4-field `BindIntern{…}` literal hand-spelled at three production sites; `FireSession::bind_intern` does this wiring but is `#[cfg(test)]`-gated. | L2 | **OPEN** · ⚠ ward-reported | closed by a production `BindIntern::from_wm` |
| **T4** | struere | 1 | `fire/pass/alpha.rs:76-89` | `ClassPlan::observe -> bool` conflates *"defer"* with *"not a leaf class"*. The codebase cured this exact shape with `OperandSlot`. | L2 | **OPEN** · ⚠ ward-reported | closed by a two-variant enum |
| **T5** | struere | 1 | `fire/pass/mod.rs:162`; `accumulate.rs:59-89`, `filter.rs:60` | `pre_dispatched` is a bare `HashSet` two passes must both honour; the invariant holds only by `delta.rs`'s call ORDER. ⚠ Same site as **I1**, different lens. | L2 | **OPEN** · ⚠ ward-reported | closed by an owned ledger with one door |
| **T6** | struere | 1 | `wat/rete/oracle/fire.wat:57-76` | `walk-filter-ids` dispatches three passes. **Ward's own verdict: no action** — the `rune:intueri(naming)` already covers it. | L2 (self-disclosed) | **CLOSED — no action** · ✅ I VERIFIED | the rune at `:54` is accurate |
| **C1 ★★** | conferre | 1 | `stratify.wat:138-160` vs `stratify.rs:88-94,103-122`; claim at `stratify.rs:26-27` | ⭐ **L1 — AND IT FALSIFIES A CLAIM I WROTE TODAY.** The oracle's `rule-negates` recognises negation ONLY when `:not` is the top-level head; Rust's `negate_types` recurses through `And`/`Or` and finds a nested `:not`. On `(or A (and B (not C)))` the oracle contributes nothing and Rust pushes `C` → different stratum numbers. `stratify.rs:26-27`, dated **2026-09-07** by `66a24d288`, asserts every other listed fn *"DOES mirror"*. | **L1** | **OPEN** · ✅ I VERIFIED | oracle `:150` tests `hd = ":wat::rete::not"` and returns `acc` otherwise; Rust `:108-112` recurses `And\|Or` passing `under_not`. Closed when the semantics is decided and one side fixed, or the claim is scoped |
| **C2** | conferre | 1 | `stratify.rs:35-37` | `fact_type_head`'s doc says it mirrors *"the inline `ast-name` + colon-strip done identically in both `rule-produces` and `rule-negates`"*. After today's cure, `rule-produces` no longer colon-strips — it resolves via `return-type-of`. Only `rule-negates` still does. | L2 | **OPEN** · ⚠ ward-reported | `stratify.wat:52-60` says there is no colon-strip fallback; closed when the comment names only `rule-negates` |
| **C3** | conferre | 1 | `fire/mod.rs:262` | *"Mirrors seed-token (wat:544-551)"* — but `pass.wat:535-551` is inside `binding-extensions`, an unrelated fn. `seed-token` is at `:76-90`. The ALGORITHMIC claim holds; the citation drifted. | L2 | **OPEN** · ⚠ ward-reported | `grep -n 'seed-token' wat/rete/oracle/pass.wat` → 76/81/134. Closed by fixing the range or dropping line numbers as every sibling comment does |
| **Q1** | sequi | 1 | `session.rs:1778-1788` | `session_ceiling_breach` reads the global `alloc_counter::session_bytes` and its result decides a hard `Err` in the fire loop — an unmarked hidden-state read, while sibling global reads in the same crate (`arm.rs:705`, `census.rs`) ARE runed. ~30 lines of prose at `:1759-1777` already read like a rune's reason. | L2 | **OPEN** · ✅ I VERIFIED | `grep -c 'rune:sequi' ` over `session.rs:1770-1790` → **0**; `arm.rs:705` → tagged. Closed by adding the rune citing the existing rationale |
| **Q2** | sequi | 1 | `arm.rs:705` + `:710` | `rune:sequi(ambient-context)` on `ARM_TABLE`, whose OWN adjacent comment says it *"holds DOMAIN state (the armed network + its lease count)"* — and that category is defined as *"doesn't carry domain state."* The rune's category contradicts its own prose. ⚠ Ward judged the CHAIN clean (the table is an upstream build cache, threaded downstream as an explicit `arm:` param); this is a vocabulary defect, not a break. | L3 | **OPEN** · ✅ I VERIFIED | both strings present at the cited lines. Closed by re-categorising; `excusare` may reach the same site independently |
| **M1** | temperare | 1 | `filter_after_join.rs:71` + `:98` | Both branches re-fetch and clone `d_beta[hj_id]` INSIDE the `for filter_id in filter_kids` loop (opened `:59`) — keyed on the OUTER `hj_id`. The structurally identical sibling `join_after_filter.rs:62` binds it ONCE above its child loop. Dimension: filter fan-out per join × tokens in that delta. | L2 | **OPEN** · ✅ I VERIFIED | `:59` opens the loop; `:71` and `:98` both `d_beta.get(&hj_id)`; `join_after_filter.rs:62` hoists. Closed by hoisting to match the sibling |
| **M2** | temperare | 1 | `round_census.rs:103` + `:109` | `right_idx.per_join_marks()` called twice building one `RoundCensus` — same receiver, no args, same round. `#[cfg(test)]`-gated, but the file's own header stresses its numbers back real cost claims, so a duplicated rebuild skews the instrument. | L2 | **OPEN** · ⚠ ward-reported | bind once, reuse for both fields |
| **X1 ★** | excusare | 1 | `alpha.rs:338` | Bare `#[allow(clippy::too_many_arguments)]`, **ILLEGITIMATE-AT-BIRTH**. The only comment above it addresses `#[cold]`/`#[inline(never)]`, not argument count — and being a COLD path, the hot-loop excuse its siblings use would not even apply. Least defensible of the six. | L1 | **OPEN** · ✅ I VERIFIED | line above is a `#[cold]` note, not a reason; contrast `hash_join.rs:39-41` which carries an on-point one |
| **X2** | excusare | 1 | `fire/mod.rs:469`, `:2128`, `filter_after_join.rs:17`, `round_census.rs:25` | Four more bare `#[allow(clippy::too_many_arguments)]` with no argument-count reason. Each has a doc comment above describing what the fn DOES — a comment's presence is not a reason's presence. `round_census.rs:25` is `#[cfg(test)]`-only, where the lint is arguably MORE valid. | L1 | **OPEN** · ✅ I VERIFIED | read all four lines above: *"Drain the frontier…"*, *"Exists/Not Leaf: probe…"*, etc. None mentions arity |
| **X3 ★★** | excusare | 1 | `wat/rete/oracle/fire.wat:54` | ⭐⭐ **THREE WARDS, ONE RUNE, TWO VERDICTS.** `excusare` strikes `rune:intueri(naming)` as ILLEGITIMATE-AT-BIRTH: *"rename would fork every oracle fire caller"* is a cost-of-change plea, not a domain-truth, and names no target. But `intueri` graded the same rune **CLEAR**, and `struere` (T6) said **no action required**. Vigilia forbids me re-classifying a ward's verdict — so both stand, and the disagreement IS the finding. | L2 (excusare) vs CLEAR (intueri) | **OPEN — NEEDS A DECISION** · ✅ I VERIFIED both readings | the rune's text at `:54-56` is a historical explanation + a refactor-cost claim; whether that is a warrant is the open question |
| **E1** | exigere | 1 | `stratify.rs:620-632` | *"deliberately not attempted"* / *"the fence half stays punted"* — but `RETE-OPEN-WORK.md:1471` records that exact idea as **CLOSED — REFUSED**, permanently, because population for `i64`/`f64`/`String` is an insert-time input rather than a static type property. The comment describes a final refusal in punt-language; a reader who has not seen the arc doc reads open work. | L1 by pattern, **wording-only in substance** | **OPEN** · ✅ I VERIFIED | `:632` says *"stays punted"*; `RETE-OPEN-WORK.md:1471` says *"THE FENCE HALF IS CLOSED — REFUSED"*. Closed by rewording to a refusal; no work is owed |
| **F1 ★** | conformare | 1 | `stratify.rs:288-294`; caller `fire/rules.rs:648`,`:707`,`:880` | A NEGATION CYCLE is a user rule-authoring mistake, and it is raised with `rust_caller_span!()` — while `fire_rules_on_session`, ONE FRAME UP, holds `span: &Span` and its own doc says *"a refusal a user can reach must name the line the user wrote, not a line in this file."* The span was available and discarded at the boundary. | L2 | **OPEN** · ✅ I VERIFIED | `stratify.rs:288-294` uses `rust_caller_span!()`; `fire/rules.rs:648` has `span: &Span`. Closed by threading it through `native_stratify`/`fire_rules_from_deps` |
| **F2** | conformare | 1 | `session.rs` `pm_to_*` / `to_transient_*` — 28 raise sites | The whole decode family takes no `span` and raises every `TypeMismatch` with `rust_caller_span!()`, reached from two span-bearing entry points that don't forward it. ⚠ Ward explicitly could NOT resolve whether a malformed Session-memory shape is user-reachable at all — if it is not, the fix is 28 `rune:conformare(spanless-by-domain)` marks, not code. None carries a rune today. | L2 | **OPEN** · ⚠ ward-reported, domain question unresolved | `grep -c rust_caller_span session.rs` → **27**; `insert.rs` → **0** |
| **N1 ★** | cernere | 1 | `session.rs:754`, `:981`, `:1159` | ⭐ **A PHANTOM FORM IN USER-FACING ERROR TEXT.** The three transient-decode helpers set `const OP: &str = ":wat::rete::to_transient …"`. **`to_transient` is attested by NO authority** — not a `RETE_OPS` row, not a `check.rs` TypeScheme, not a `runtime.rs` dispatch arm, not a wat `defn`. It is a plain internal Rust fn, and the label is spelled in Rust `snake_case` inside a wat-namespaced string where every resolving sibling is kebab-case. A caller whose malformed `Session` trips this gets a `TypeMismatch` naming a form they cannot find in the language — the form they actually invoked was `fire-rules`/`insert`. ⚠ Same decode family as **F2**, different lens. | L2 | **OPEN** · ✅ I VERIFIED | I enumerated ALL 11 distinct `:wat::rete::` OP labels in target 1: **8 resolve** (`arm-session` 5 authorities, `fire-rules` 10, `insert` 21, …), **3 are phantom — all of them `to_transient`**. Closed when the label names the entry verb (e.g. `":wat::rete::fire-rules (session decode)"`), matching `arm.rs:1252`'s correct convention |
| **R1 ★** | probare | 1 | `outcome.rs:22-25` | ⭐ **A DEFERRAL DECISION RESTING ON A COUNT AND A CITATION, ONE OF WHICH IS FLATLY WRONG.** The header declines to push the outcome enum into `fire_fixpoint_delta_armed` because *"it has three callers (`fire-once`, `fire-rules`, and the query path at `fire/rules.rs:425`)"*. **`fire/rules.rs:425` contains no call** — it is a session-field data literal; the query call is 8 lines later at `:433`. In-target call sites number **four**, not three: `fire/mod.rs:1177`, `fire/delta.rs:266`, `fire/rules.rs:193`, `fire/rules.rs:433` — `fire-rules` alone reaches it by TWO routes (stratified and unstratified). Nothing in `rete_header_claims_are_asserted.rs` covers `outcome.rs`. | **L1** (ward's severity, passed through) | **OPEN** · ✅ I VERIFIED | `grep -rn 'fire_fixpoint_delta_armed(' src/ \| grep -v 'fn '` → 4 in-target + 2 in `tests/`; `sed -n '423,427p' fire/rules.rs` → data literal. ⚠ **The two halves are not equally strong** — see my note below. Closed by correcting the citation and stating the count in a form a gate can hold |
| **R2** | probare | 1 | `mod.rs:15`; `arm.rs:706`; `wat/rete.wat:199-207` | *"Session stays 8 fields"* is called **THE ONE CONTRACT** of `DESIGN-STONE-intern-zero-mutex` — and nothing asserts it. It is TRUE today (I counted the `defrecord`: 8). But the sibling claim one door over, `FireCtx`'s field count, IS gated by `rete_header_claims_are_asserted::fire_ctx_field_count_matches_its_doc` — minted precisely because *"its doc said thirteen while the struct held fourteen."* The cure exists and this contract never received it. | L2 | **OPEN** · ✅ I VERIFIED | `sed -n '199,208p' wat/rete.wat` → exactly 8 fields; `grep -rn '8 fields' tests/` → **0**. Closed by a gate arm mirroring the `FireCtx` one |
| **V1 ★** | perspicere | 1 | `pass.wat` (12 sites), `accum-pass.wat` (2) vs `fire.wat` (11) | ⭐ **THE ALIAS EXISTS AND ONE ORACLE FILE ADOPTED IT WHILE TWO DID NOT.** `wat/rete.wat:157-161` declares `:wat::rete::AlphaMemory` / `BetaMemory` / `ProductionMemory`. `fire.wat` uses those names **11 times**. `pass.wat` spells the longhand `(PersistentMap :- [i64 (PersistentVector :- [Element])])` **12 times and uses the alias zero times**; `accum-pass.wat` 2 and zero. ⛔ **I CORRECTED THE WARD HERE:** it reported the alias *"simply isn't being used"* — false, it is used 11×. The true shape is a **per-file split inside one corpus**, which is stronger: not an unminted noun but a migration that stopped halfway. | L2 | **OPEN** · ✅ I VERIFIED | `grep -c ':wat::rete::AlphaMemory\|BetaMemory\|ProductionMemory' wat/rete/oracle/*.wat` → `fire.wat:11`, others 0; `grep -c 'PersistentMap :- \[:wat::core::i64'` → `pass.wat:12`, `accum-pass.wat:2`. ⛔ Closed by a **wat-fix codemod**, never a hand-edit — `CLAUDE.md` mandates it for exactly this shape |
| **V2** | perspicere | 1 | `census.rs:503` vs `:108,130,194,630,676,735,837` | ⛔ **I INVERTED THE WARD'S VERDICT ON THE FACTS, AND THE ROW IS THE INVERSION.** The ward called `census.rs:509` (`RefCell<Option<GatherKeyMap>>`) *"a real gap"* — an eighth TLS slot that should have been runed like its seven siblings. **It is the opposite.** `GatherKeyMap` is a typealias declared six lines above at `census.rs:503`; `:509` is therefore depth-2 **with the noun already named** — it is precisely the cure this ward prescribes, already applied. It needs no rune because it is not deep. The seven runed siblings are the ones that never got an alias. **So `census.rs` contains its own cure, applied once and not to the other seven.** | L2 | **OPEN** · ✅ I VERIFIED | `grep -rn 'type GatherKeyMap' src/` → `census.rs:503`. Closed by either aliasing the seven to match `:503`, or recording `:503` as the file's model form so the next hand copies it |
| **V3** | perspicere | 1 | `filter.rs:237`, `filter_after_join.rs:130`, `:256`; and 9 further shapes | Eleven repeated nested-generic shapes with no alias, the strongest being `Option<std::sync::Arc<[Value]>>` at three sites across two files **carrying the identical local name `hoisted_keys` at all three** — the noun is already agreed, just never written as a type. Others: `Result<Arc<InternedNetwork>, EvalBreak>` ×3 in `arm.rs`; `impl Iterator<Item = Result<i64, EvalBreak>>` ×3 in `acc.rs`; `Result<HashMap<String, i64>, EvalBreak>` ×2 in `stratify.rs`, which the ORACLE already names `type-strata` in prose at `stratify.wat:41`. | L2 | **OPEN** · ⚠ ward-reported (I verified only the `hoisted_keys` and `GatherKeyMap` claims myself) | ward's own whole-repo greps per shape; closed shape-by-shape, not as one sweep |
| **W1 ★★** | circumspicere | 1 | claims: `session.rs:1759-1765`, `fire/delta.rs:472-473`, `wat/rete/oracle/insert.wat:63-66` · code: `delta.rs:715`, `insert.rs:205` vs `:214` | ⭐⭐ **THE CEILING IS SAMPLED AT BOUNDARIES; THE SHIPPED SENTENCE SAYS IT IS A BOUND.** Three sites ship the builder's ruling verbatim — *"the session is the boundary — it may not consume more than the configured amount of memory, 1G by default"* — and the oracle's says *"enforced at BOTH doors."* But `session_ceiling_breach` runs **after** a whole round's passes have materialised `next_delta`, and `check_insert_ceiling` runs **after** `vector_concat_inner` has materialised the entire batch (`insert.rs:205`, checked `:214`). A single round or a single `insert-all` big enough to breach does so **before** any check can fire — the same 2.5M-facts/4.0 GB failure the ceiling was built for, one level down. | **L1** (ward's severity, passed through) | **OPEN** · ✅ I VERIFIED | `grep -rn 'session_ceiling_breach' src/` → 2 decision sites in-target; `sed -n '196,216p' insert.rs` → concat at `:205`, check at `:214`. ⚠ **Read my note below before acting — the qualification EXISTS, in one file, and never travels.** Closed by qualifying the three claim sites + a `rune:circumspicere(accepted-by-design)` pointing at `delta.rs:450` |

## Cast log

| # | target | cast at | wards mustered | returns in `reports/` | L1 | L2 |
|---|---|---|---|---|---|---|
| 1 | `src/rete/kernel/` + `wat/rete/oracle/` | 2026-09-07 | 13 inward + circumspicere last | RETURNED (14 of 14 — TARGET 1 COMPLETE): cernere · probare · perspicere · circumspicere · intueri · purgare · solvere · struere · conferre · sequi **CONV** · temperare · excusare · exigere **CONV** · conformare — **ALL FOURTEEN CAST.** | **6** | 22 |

## Verified by the orchestrator, not taken

Every row above was re-read against the disk before it was rowed; a ward's verdict is a hypothesis
until a `file:line` confirms it.

⛔ **A ✅ in the status column means I re-read the disk myself. A ⚠ means the row is the WARD's claim,
carried unverified.** Both are rowed, because dropping the unverified ones would hide work; but only
the ✅ rows may be cited as fact. This distinction is the whole reason the status column exists.

- **I1** — CONFIRMED. `accumulate.rs:59` clears `pre_dispatched`, `:76` calls `dispatch_where_tests`,
  `:88` inserts into it. The second responsibility is real and the doc at `:18` does not name it.
- **P1** — CONFIRMED. `grep -rn 'retain-supported'` over `wat/ wat-scripts/ wat-tests/ tests/ src/`
  returns three hits, ALL inside `fire.wat`: the doc header `:231`, the `defn` `:242`, and a
  self-reference `:327`. No call site anywhere. ⚠ And it is MY leftover — the factbag-one-owner
  strike I drew this morning replaced it with `factbag::retain` and did not delete it.
- **S5 ★** — CONFIRMED, and worse than the row alone conveys. All three sites carry the identical
  `keyed_join_persistent` / `FilterJoinIdx` / `FireCtx` / `record_tokens` shape, and
  `grep -rn left_activate_join src/rete/kernel/` finds ONE call (`filter_after_join.rs:75`). The
  helper's doc header at `mod.rs:78` opens with the words **"ONE COPY."** The extraction described
  in that header was never applied to the two sites it names.

## Runes weighed

| rune | verdict |
|---|---|
| `rune:intueri(naming)` at `wat/rete/oracle/fire.wat:54` — *"the name is the historical walk-sorted-ids split, not filter-alone"* | **CLEAR, confirmed.** `walk-filter-ids` does dispatch `accumulate-pass`, `filter-pass` AND `hash-join-pass` (verified over `:57-76`). The rune names the mismatch honestly and gives a checkable cost for the rename. |

## Convergence, per ward

| ward | verdict |
|---|---|
| intueri | 0 L1 + 1 L2 — **DIVERGES** (narrowly) |
| purgare | 0 L1 + 2 L2 — **DIVERGES** |
| solvere | 9 rows (2 structural ★) — **DIVERGES** |
| struere | 6 L2, one self-closed (T6) — **DIVERGES** |
| conferre | **1 L1** + 2 L2 — **DIVERGES** |
| sequi | ⭐ **CONVERGED** — *"chains followed to their ends; no hidden domain-state break found in the fire path"* (`reports/sequi.md:5`). Q1/Q2 rowed as vocabulary notes, *"neither is a red"* — the ward's own words |
| temperare | 2 L2; 5/5 runes upheld, 5/5 prior-closed rows skipped — **DIVERGES** |
| excusare | 65 weighed / 65 enumerated · 59 HOLD · **6 struck** (X1, X2 are L1) — **DIVERGES** |
| exigere | ⭐ **CONVERGED** — TODO-family zero re-derived independently, not inherited (`reports/exigere.md:42`); E1 is wording-staleness, not an open deferral |
| conformare | ⚠ **THE WARD'S OWN VERDICT AND ITS OWN FINDINGS DISAGREE.** `reports/conformare.md:59` ends **"CONVERGED."** — yet the same report returns F1 and F2, both rowed L2 above and both verified by me. `vigilia` forbids the aggregator re-classifying a child, so BOTH stand as returned. Read it as *"the analysis converged"*, not *"the target is clean"* — but it is recorded, not smoothed, because a report whose last line contradicts its own body is exactly the shape that gets quoted later without its findings |
| cernere | 1 L2 — **DIVERGES**, narrowly. ⭐ But its headline is the CLEAN half: ~130 distinct `:wat::rete::` names across the six oracle files, **all resolve**; 44 distinct `:wat::` tokens across the 22 Rust files, all resolve but one. The live question — *is `wat/rete/oracle/**` covered by anything, given the name-resolution gate scans `wat-scripts/` only?* — came back **measured, not assumed**: every oracle fn is on a forced entry path, so no unforced-`def` phantom exists there today |
| probare | 2 (1 L1 + 1 L2) — **DIVERGES**. ⭐ Notable for refusing its own headline metric: told the target is deliberately comment-dense and that the spell exempts doc-comment-rich code, it ran the 28-file ratio table anyway, marked every exemption explicitly instead of dropping rows, and declined to verdict on the Rust declaration-count measure because it *"produces nonsense ratios like 1:57"* here. Third independent re-derivation of the zero TODO/FIXME count |
| perspicere | 3 L2 — **DIVERGES**. ⭐ Its headline is the adjudication, not the findings: told seven of ten runes are near-identical boilerplate and warned off a block verdict, it grepped each of the seven exact type strings whole-repo and found **each genuinely unique** — seven distinct instruments, not one template. **All 10 runes CLEAR.** ⚠ Two of its three findings I had to correct on the facts (V1, V2) — in opposite directions, both ending stronger. ⛔ It also closed a divergent report with the word CONVERGED, as `conformare` did: a defect in the brief's wording, not the ward's judgment |
| circumspicere | **1 L1 — and it is the cast's best argument for itself.** Cast last, given the aggregate of the other thirteen, it found the one thing an inward lens structurally cannot: a gap between what the engine SHIPS as a guarantee and where the guarantee is actually sampled. `sequi` read the same ceiling chain and pronounced it clean — correctly, on its own axis. Also adjudicated the single `rune:circumspicere` VALID under the heaviest rune burden in the grimoire |

- **C1 ★★** — CONFIRMED, and the falsified claim is my own. Oracle `stratify.wat:150` tests
  `hd = ":wat::rete::not"` on the TOP-LEVEL head and returns `acc` unchanged for anything else —
  it never descends into an `:and`/`:or`. Rust `stratify.rs:108-112` recurses `And | Or` passing
  `under_not` through, so a nested `:not` IS found and its type pushed. The shape is grammatically
  legal: `wat-scripts/perf/grid/where-nested-combinators.wat:29` carries it verbatim.
  ⚠ **The ward correctly WITHDREW the stronger claim** — no `defrule` exercises this today, so no
  observed stratum difference is asserted. That withdrawal is the grounding clause working, and it
  is also a `peragrare` cell in waiting: a legal shape no fixture visits.

- **Q1 / Q2** — CONFIRMED. `session.rs:1770-1790` contains zero `rune:sequi`; `arm.rs:705` carries
  `rune:sequi(ambient-context)` while `:710` says the table *"holds DOMAIN state"*. The category is
  defined as the absence of exactly that.

⭐ **sequi is the first ward to CONVERGE**, and that is a result, not an absence. It followed the
whole fire round chain — `fire_fixpoint_delta_armed` through all seven passes — and found the state
threaded explicitly end to end, with `production_delta` returning its delta rather than writing an
out-parameter. It also grepped the oracle for `set!`/atoms/globals and found **zero**. A ward that
comes back clean having said what it looked at is the evidence the full guard was worth casting.

- **M1** — CONFIRMED. `filter_after_join.rs:59` opens `for filter_id in filter_kids`; `:71` and
  `:98` each call `d_beta.get(&hj_id)` inside it, keyed on the OUTER variable. `join_after_filter.rs:62`
  binds the same fetch ABOVE its child loop. Same shape, one hoisted, one not.

⭐ **temperare upheld 5 of 5 runes and skipped 5 of 5 prior-closed rows** rather than re-reporting
settled work — the outcome the briefing's prior-art list exists to produce. It also singled out
`fire/mod.rs:494` as the model form for a `rune:temperare` reason: dated, naming its floor run, and
carrying a MEASURED ratio (206 / 245583 = 0.084%) rather than an estimate. That rune came out of
this session's own combinator-inner strike.

- **X1 / X2** — CONFIRMED. All five sites carry a bare `#[allow(clippy::too_many_arguments)]`. Four
  have a doc comment above, but every one describes what the function *does*
  (*"Drain the frontier pass 3.6 produced…"*, *"Exists/Not Leaf: probe the token's bucket…"*) —
  none mentions arity. The contrast is `hash_join.rs:39-41`, which carries a real on-point reason:
  *"…purely to satisfy a lint, and the parameters here are already the fire pass's working set
  rather than an accidental pile."* Two siblings justified, five silent.

⭐⭐ **X3 IS THE CAST'S MOST INTERESTING RESULT: THREE WARDS DISAGREE ABOUT ONE RUNE.**
`intueri` weighed `rune:intueri(naming)` at `fire.wat:54` and graded it **clear**. `struere` reached
it independently and said **no action required** — the existing rune already does the work.
`excusare`, whose entire remit is weighing exemptions, struck it **ILLEGITIMATE-AT-BIRTH**: the
reason explains *how* the name went wrong and asserts a *cost* to fixing it, and neither is a
domain-truth. **`vigilia` forbids the aggregator re-classifying a child's verdict**, so all three
stand and the disagreement is itself the deliverable — a decision for the builder, not for me.

⭐ **And the aggregate is the answer to "prove the others don't find anything":** 65 exemptions
weighed, **59 HOLD**. This subsystem's runes are, overwhelmingly, real. Classes B and H (19 of the
59) were checked mechanically — against the actual `#[cfg(test)]`/no-op-twin shape, and against the
actual absence of each cited symbol — not read for plausibility.

- **E1** — CONFIRMED. `stratify.rs:632` reads *"which is why the fence half stays punted"*;
  `RETE-OPEN-WORK.md:1471` reads *"★★★★★★★ THE FENCE HALF IS CLOSED — REFUSED."* A permanent
  refusal described in the language of an open punt. No work is owed — only the wording misleads.

⭐ **exigere CONVERGED, and re-derived the zero rather than inheriting it.** I told it a prior ward
had reported zero `TODO/FIXME/XXX/HACK` and asked it to confirm or refute independently; it ran its
own greps and got zero. Two wards agreeing from separate sweeps is evidence. One repeating the
other's number is an echo.

## ⚠ THREE COUNTS OF ONE POPULATION, AND THEY DISAGREE

The runes in this target have now been counted three times, by different methods, with no two
agreeing:

| counter | figure | method |
|---|---|---|
| `excusare` | **56** applied (+2 excluded as prose) | enumerated then weighed each |
| `exigere` | **54** markers | `rune:[a-z-]*` grep |
| me, just now | **57** tagged (+1 bare `rune:` = 58 lines) | `grep -o 'rune:[a-z]*'`, per-occurrence |

Nobody is badly wrong and no finding turns on it — but a population **nobody can state to the unit**
is a population no later claim should be built on. Recorded rather than smoothed, because a count
that varies with who ran it is a count that needs its method stated beside it. This is the fifth
sweep-arithmetic wobble of the session.

- **F1 ★** — CONFIRMED. `stratify.rs:288-294` raises *"negation cycle detected — rule set is not
  stratifiable"* with `rust_caller_span!()`; `fire/rules.rs:648` declares `span: &Span` and uses it
  for exactly one other call. **And the comparator is decisive:** `insert.rs` contains **zero**
  `rust_caller_span!()` — all five of its error sites thread `list_span` — while `session.rs`
  contains **27**. Same subsystem, two opposite conventions.

⭐ **conformare reported SCOPE before findings, which is why its findings are worth having.** It
established first that the target defines no error type of its own and has no `From` impls at all —
both by grep, both reported as zero — and only then ran the one audit surface that applies. The
brief told it that answering *"this target defines none"* would be a complete result; it did the
work anyway and found two real call-boundary gaps. A ward that can say "not here" is a ward whose
"here" means something.

⚠ **F2 carries an unresolved domain question and is rowed WITH it.** The ward could not determine,
from the target alone, whether a malformed Session-memory shape is ever user-reachable. If it is
not, these 28 sites are `driver_of`'s legitimate class and the fix is documentation, not threading.
It said so instead of guessing — and that honesty is why the row is actionable.

- **N1 ★** — CONFIRMED, and I enumerated the whole population rather than trusting the three cited
  sites. Every distinct `:wat::rete::` OP label in target 1, tested against `check.rs` /
  `runtime.rs` / `vocabulary.rs` / `wat/`: `adopt-session-lease` (3), `arm-session` (5),
  `fire-once` (6), `fire-rules` (10), `fire-rules-explain` (5), `insert` (21), `insert-all` (6),
  `release-session` (4) — **eight resolve.** The only three that resolve NOWHERE are the three
  `to_transient` spellings. So the ward named the entire class, not merely where it noticed it.
  ⚠ **An extra tell the ward did not claim and I checked:** every resolving label is wat
  kebab-case; `to_transient` is Rust `snake_case`. The phantom is spelled in the HOST language's
  convention inside a wat-namespaced string — the "author was thinking in another tongue" shape
  `cernere` names by name.
  ⭐ **And it converges with `conformare`.** `session.rs:754` builds
  `RuntimeError::new(rust_caller_span!(), TypeMismatch { op: OP, … })` — the same decode family
  F2 rowed for discarding the caller's span. Two wards, two lenses, one site: F2 says the error
  cannot name the user's line; N1 says it cannot name a real form either. Neither alone shows that.

- **R1 ★** — CONFIRMED, but **the two halves of this finding are NOT equally strong, and I will not
  let the strong half vouch for the weak one.**
  · **The citation is simply wrong, and that half is not arguable.** `fire/rules.rs:425` is inside a
    session-field data literal (`("network", q_net), ("rules", empty_rules), …`). The query-path
    call to `fire_fixpoint_delta_armed` is at `:433`. I read both.
  · **The count is arguable.** The header says *"three callers"* and then names them as verbs —
    `fire-once`, `fire-rules`, the query path — and its next clause reasons about *"the second
    door"*, which is door-language, not call-site language. Read as VERBS the three is defensible;
    read as CALL SITES it is four, because `fire-rules` dispatches to two distinct routes
    (`fire_unstratified` → `delta.rs:266`, and `fire_rules_stratified` → `rules.rs:193`). The ward
    graded the whole finding **L1**; `vigilia` forbids me re-classifying a child's verdict, so L1
    stands as returned — but the L1 rests on the citation, not on the count.
  ⭐ **AND THE CURE ALREADY EXISTS IN THIS REPO.** `rete_header_claims_are_asserted.rs` carries an
  arm named `the_termination_verifier_still_has_exactly_one_call_site` — a caller-count claim,
  mechanically held. The gate already knows how to pin exactly this kind of assertion. `outcome.rs`
  simply never got one. That is the arc's signature shape again: the fix is driven, proven and
  shipped on a sibling path that never received it.

- **R2** — CONFIRMED. `wat/rete.wat:199-207` declares `Session` with exactly eight fields
  (`network`, `rules`, `alpha-memory`, `beta-memory`, `production-memory`, `facts`, `next-id`,
  `query-memory`). `grep -rn '8 fields' tests/` returns **zero**. The claim is true and unheld,
  while its sibling — `FireCtx`'s field count — is gated by an arm minted after that doc said
  thirteen and the struct held fourteen. ⚠ Note the asymmetry that makes this worth a row rather
  than a shrug: `arm.rs:706` calls it **"THE ONE CONTRACT."** A claim carrying that weight with no
  gate is the exact shape this arc keeps paying for.

⭐ **probare's most valuable act was refusing the metric it was sent to apply.** Told that this
target is deliberately comment-dense and that its own spell exempts doc-comment-rich code, it ran
the ratio table anyway, marked every exemption explicitly rather than dropping rows, and then said
so: the literal Rust declaration-count measure *"produces nonsense ratios like 1:57"* for files that
are one large function with heavy rationale, so it declined to verdict on it. It also independently
re-derived the zero TODO/FIXME count — the **third** separate measurement of that zero this cast,
by a third method. Three wards agreeing from three sweeps is evidence; one inheriting another's
number is an echo.

- **V1 ★ / V2** — CONFIRMED, **and I had to correct the ward on both, in opposite directions.**
  Its hypotheses were directionally right and factually off, and in each case the true fact is the
  stronger finding. That is the weighing earning its keep: a report credited on sight would have
  shipped two wrong sentences into the record.
  · **V1** — the ward said the `AlphaMemory`/`BetaMemory`/`ProductionMemory` aliases *"simply
    aren't being used."* They are: `fire.wat` uses them **11 times**. What is actually true is a
    per-file split — `fire.wat` adopted the alias, `pass.wat` (12 longhand, 0 alias) and
    `accum-pass.wat` (2, 0) did not. A migration that stopped halfway inside one corpus is a
    sharper finding than a noun nobody minted, and it names its own cure: a codemod.
  · **V2** — the ward called `census.rs:509` *"a real gap"*, an eighth TLS slot missing the rune its
    seven siblings carry. **The fact inverts it.** `type GatherKeyMap` is declared at
    `census.rs:503`, six lines above; `:509` reads `RefCell<Option<GatherKeyMap>>` — depth 2 with
    the noun named. It carries no `rune:perspicere` because **it does not need one**: it is the
    cure, not the gap. The seven are the sites that never got an alias. The file holds its own
    model form, applied once.

⭐ **perspicere did the thing the brief most asked for, and it is worth recording.** Told that seven
of the ten runes are near-identical boilerplate — all `read-once`, all *"alias would be a mumble"* —
and warned NOT to issue a block verdict, it checked each of the seven exact type strings against a
whole-repo grep and found **each genuinely unique**. Seven distinct instrumentation shapes, not one
template stamped seven times. All ten runes pass. It also separated the two halves of each reason
unprompted: the `read-once` half is checkable and true; the *"alias would be a mumble"* half is
subjective and weaker. That is the honest reading, and it is the answer to *"prove the others don't
find anything instead of assuming they won't"* — the boilerplate LOOKED like the failure and was not.

⚠ **AND A SECOND WARD CLOSED A DIVERGENT REPORT WITH "CONVERGED."** `perspicere`'s final section is
headed **CONVERGED** and its very next sentence reads *"The result is not 'clean': there are 13
genuine findings."* `conformare` did the same thing (`reports/conformare.md:59`). Two of twelve.
⛔ **This is a defect in MY BRIEF, not in the wards.** Every brief carried the clause *"If you
converge, say CONVERGED — and say what you looked at."* Read one way that means *the target is
clean*; read another it means *my sweep converged / I was exhaustive*, and both wards took the
second reading. **The wording must be fixed before targets 2–4 are cast** — see the README's resume
protocol. A ward is not wrong to answer the question it was asked.

- **W1 ★★** — CONFIRMED on every coordinate, **and the nuance is load-bearing enough that acting on
  the row without it would overcorrect.**
  · **The three claim sites are real and none carries a rune.** `grep -c 'rune:'` over
    `session.rs:1750-1760` → 0, over `delta.rs:465-475` → 0. The oracle's `insert.wat:63-66` states
    *"The session ceiling is enforced at BOTH doors"* with no qualifier at all.
  · **The placement is real.** `insert.rs:205` materialises the whole concatenated batch; the check
    is at `:214`. The fire door's check sits after the round epilogue.
  · ⛔ **BUT THE AUTHOR KNEW, AND SAID SO — IN ONE FILE.** `delta.rs:707` reads: *"A single round can
    allocate without bound (this file's own header: `fanout` derives 40_000 facts in one round), so
    'it converged' is not evidence it was cheap."* That is the exact hazard, disclosed at the CHECK
    site — where it appears as the reason the check was moved above the `break`, not as a
    qualification of the contract. And `delta.rs:450-451` says the round cap *"bounds
    NON-TERMINATION, not memory … a legitimate workload shape this deliberately does not limit."*
  · **So this is not an engine that does not know its own limit. It is a QUALIFICATION THAT NEVER
    TRAVELS.** `grep -n 'without bound\|does not limit'` across the three claim files returns
    **three hits, all in `delta.rs`** — none in `session.rs`, none in the oracle. The honest
    sentence exists and stayed where it was written; the universal-sounding one is what shipped to
    the other two sites and to the wat-facing spec.
  ⭐ **That makes the closure clearly documentation, and the ward said so unprompted** — it applied
  the four questions, judged this an *inherent cost* rather than a one-line default, and pointed at
  `delta.rs:441-443` as the precedent for how this codebase already discloses the round cap's
  version of the same gap. A ward that proposes the smaller correct fix over the larger dramatic one
  is a ward worth casting.

⭐ **AND IT ADJUDICATED THE ONE `rune:circumspicere` AS VALID, WHICH IS THE HARDER VERDICT.** Its own
spell gives this rune the heaviest burden in the grimoire — the reason must name WHERE the bound is
documented, and an undocumented "it's fine" IS the blind spot. `arm.rs:717` cites two design stones;
the ward confirmed both exist and are substantive, and noted the bound was already tightened once
after an unwind-safety bug in the pre-guard `release-session` path. It called it the model form.

⛔ **THE LAST WARD FOUND THE THING THIRTEEN LENSES WALKED PAST, WHICH IS THE ENTIRE ARGUMENT FOR
CASTING IT.** `sequi` read this exact ceiling chain and pronounced it clean — correctly: the state
threading IS clean, and it rowed only the unruned global read (Q1). What it could not see, because
it faces inward at the code, is that the sentence the code SHIPS describes a stronger guarantee than
the placement delivers. Thirteen inward lenses passed over `session.rs:1759` and none was pointed at
the gap between a claim and its enforcement. That is the surround, and it is exactly what
`circumspicere` exists for.

---

# TARGET 2 — `src/rete/**` minus `kernel/` + `wat/rete*.wat` (25 files, 23,886 lines)

Cast opened 2026-09-07. Muster derived with measured triggers in `README.md`. Returns land verbatim
in `reports-target2/`. **Same rules: this table is the only status home; every row carries its
re-derivation; ✅ means I re-read the disk myself, ⚠ means the row is the ward's claim.**

| id | ward | site | finding | sev | status | re-derivation |
|---|---|---|---|---|---|---|
| **2C1** | conferre | `validate/mod.rs:709` vs `:720-725` and `matcher.rs:977-985` | The doc says the accepted set is *"a literal resolves"* — unqualified. `ast_literal_value` accepts **only** `IntLit`/`FloatLit`/`BoolLit`/`StringLit`; `RationalLit`, `BigIntLit` and `NilLit` are literals that resolve to `None`. ⭐ **The code is RIGHT** — its own `matches!` omits exactly those three — and the file already contains the precise phrasing 40 lines below, in the user-facing error at `:748`: *"an integer / float / boolean / string literal."* The loose word is only in the doc comment above the function. | L2 | **OPEN** · ✅ I VERIFIED | `sed -n '977,986p' matcher.rs` → 4 arms then `_ => None`; `grep -nE '(RationalLit\|BigIntLit\|NilLit)\('` → all three variants exist (`hash.rs:189,201,218`). Closed by copying `:748`'s own wording up to `:709` |
| **2F1** | conformare | `purity.rs:1513` | `classify_native_fn` raises `AxisViolation::at(rust_caller_span!(), …)` — the **one** production site in ~20 that does not thread a real user span. ⭐ **The exception is legitimate and documented** — `purity.rs:168-169` says *"`classify_native_fn` / unregistered names use `rust_caller_span` (no body AST)"* — but the prose sits at the STRUCT definition, **1,300 lines from the site**, and carries no `rune:conformare(spanless-by-domain)`. A reader at `:1513`, and every rune census, sees an unexplained sentinel. | L2 | **OPEN** · ✅ I VERIFIED | `sed -n '1513p' purity.rs` → `rust_caller_span!()`; `sed -n '168,169p'` → the justification, naming this fn; `grep -rn 'rune:conformare'` over target 2 → **0**. Closed by a rune at the site citing `:168-169` and `:1494-1500` |
| **2P1 ★** | purgare | `clause.rs:68-71` | ⭐ **A RUNE WHOSE REASON IS FALSE.** `rune:purgare(trait-contract)` on `Accumulate.var` says *"current consumers only walk `from`"* — but `validate/mod.rs:561-562` destructures `Accumulate { var, from, .. }` and does `out.push(var)`, feeding the freeze-time trapped-bind wall. The field is LIVE; the rune and its `#[allow(dead_code)]` describe a world that no longer exists. | L2 | **OPEN** · ✅ I VERIFIED | `sed -n '561,562p' validate/mod.rs` → `out.push(var)`. Closed by deleting the rune + attribute; the field stays |
| **2P2** | purgare | `clause.rs:72-76` | `Accumulate.acc_form` IS genuinely dead — every destructuring site discards it via `..`. ⚠ Here the rune's **reason is TRUE** (*"fire reads acc-form off the AccumulateNode"* — confirmed: `kernel/arm.rs:415,925` read `node_named_ast(node, "acc-form")`, never this parse field). Only the **category** is wrong: `trait-contract` on a plain enum field that no trait bound mandates. | L2 | **OPEN** · ✅ I VERIFIED | `grep -rn 'acc_form' src/ --include=*.rs \| grep -v clause.rs` → all hits are `kernel/arm.rs` locals reading the compiled node. Closed by recategorising |
| **2P3** | purgare | `matcher.rs:484-568`, `:843` | `sym: Option<&SymbolTable>` threaded through `alpha_match_inner`/`_local`/`_seeded`/`_opts` → `eval_clauses`/`eval_clause` is **never `None`** at any of its 8 call sites (4 production, 4 in `kernel/tests/`). The doc concedes *"no caller lacks one today"* — honest, but not a rune. ⛔ **See my qualification below: `resolve_operand`'s own `sym` IS reached with `None`, from a different caller.** | L2 | **OPEN** · ✅ I VERIFIED, with a correction | zero `None` call sites in the alpha_match family; **but** `eval_insert.rs:290` passes `None` to `resolve_operand` directly. Closed by collapsing the MATCHER family only, or a `rune:purgare(safety-margin)` |
| **2P4** | purgare | `export.rs:2286-2300` | `let agg = match export { … }; let _ = agg;` — bound, immediately discarded, never read. The refusal is entirely done inside the match arms. Reader tax, no behaviour. | L2 | **OPEN** · ⚠ ward-reported | closed by a `matches!`-guarded early return with no binding |
| **2P5** | purgare | `eval_test.rs:72-73` | `rune:purgare(trait-contract)` on `eval_test_core`'s `env` parameter. All 5 call sites pass a fresh `Environment::new()` — a genuine constant-parameter, correctly spotted. But `eval_test_core` is a plain fn, not a trait impl; the category does not fit the doctrine's own taxonomy. | L2 | **OPEN** · ⚠ ward-reported | closed by recategorising to `future-fixture` |
| **2P6** | purgare | `wat/rete/factbag.wat:103-115` | `:wat::rete::factbag::count-of` is DEAD — a full-name grep across `.wat` and `.rs` returns the definition and nothing else. No Rust dispatch arm, no wat caller. ⚠ The ward was careful to exclude `:sq::count-of` / `:t118b::count-of`, unrelated namespaces that a substring grep would have swept in. | L2 | **OPEN** · ✅ I VERIFIED | `grep -rn 'factbag::count-of' --include=*.wat --include=*.rs .` → **1 hit, the defn**. Closed by deleting it, or a `rune:purgare` if held for the file's rung-3 seal |
| **2P7 ★** | purgare | `wat/rete/compile.wat:263-266` | ⭐ **A RECORD THAT PROMISES A DIAGNOSTIC IT NEVER GIVES.** `AxisViolation` declares `head`, `axis`, `span`, and `purity.rs:2055-2070` populates all three with real values on every construction. Only `head` is ever read. The record's own doc says a caller can *"report as well as the substrate can"* — but `axis-violation-message`'s four arms use `head` alone, so the span is computed, carried across the boundary, and dropped. | L2 | **OPEN** · ✅ I VERIFIED | `AxisViolation/head` → **4** reads; `AxisViolation/axis` → **0**; `AxisViolation/span` → **0**. Closed by wiring the 4 arms to use `span`, or dropping the fields |
| **2S1 ★** | solvere | `matcher.rs:733-741` + `compiled_cond.rs:1403-1412` | ⭐ **THE `CmpKind → bool` TABLE IS HAND-WRITTEN TWICE.** Six arms, same semantics, in the interpreted path and the compiled path. `clause.rs:119-126` defines `CmpKind` and implements **no method on it**; there is no `cmp_holds` anywhere. ⚠ The compiled site's own doc reasons about drift one level too shallow: *"`compare_values` is REUSED from `matcher.rs`… so an ordering definition can never drift"* — true, and it protects the `Ordering` computation while leaving the dispatch table **on top of it** duplicated. | L2 · structural, low blast radius | **OPEN** · ✅ I VERIFIED | both bodies read: 6 arms each, `?`-propagation vs `matches!`; `grep -rn 'impl CmpKind\|fn cmp_holds\|fn holds' src/rete/` → **0**. Closed by one `cmp_holds` in `clause.rs` beside the type |
| **2S2 ★★** | solvere | `export.rs:751-754` vs `:834-839`, `:1106`; ten `pack_X`/`unpack_X` pairs | ⭐⭐ **ONE FILE HOLDS BOTH ENDS OF THE LADDER.** `pack_expr` is a bare `match` with no catch-all — a new variant **cannot compile** without its arm. `unpack_expr` matches a runtime **string tag** with `other => Err(malformed(…))`, so the same new variant compiles clean and fails at runtime. Ten pairs, same asymmetry. ⛔ **The file DIAGNOSES ITSELF** — `:751-754` says the packer is *"the one whose exhaustiveness the compiler enforces for you… Its inverse cannot get that guarantee"* — and names the mitigant as a **test corpus a human must remember to extend**. | L2 · structural | **OPEN** · ✅ I VERIFIED | `pack_expr` has no catch-all; `unpack_expr:1106` is `other => Err(...)`; `:838` names the corpus as the catcher. Closed by a per-variant table or a macro emitting both arms |
| **2S3** | solvere | `wat/rete/compile.wat:1079-1100` + `:1103-1124` | `compile-rule` and `compile-query` run the identical pipeline — `sort-lhs` → `CondFoldAcc` → `foldl compile-condition` → destructure → build terminal → `assoc` → `wire-parents` → bump `next-id` — differing only in the RHS fence and the terminal node type. ⛔ **Self-diagnosed:** the comment at `:1102` reads *"compile-query — same LHS fold as compile-rule; terminal is a QueryNode."* Named, never extracted. | L2 · structural | **OPEN** · ✅ I VERIFIED | `sed -n '1102p' compile.wat` → the comment, verbatim. Closed by one `compile-terminal` helper parameterised by the terminal constructor |
| **2X1 ★** | excusare | `compiled_cond.rs:245-246` | ⭐ **AN EXEMPTION WHOSE OWN COMMENT DOCUMENTS THE CHANGE THAT MADE IT INERT.** `#[allow(clippy::too_many_arguments)]` sits on `from_parts`, which takes **exactly 7 parameters** — I counted them. Clippy's default `too-many-arguments-threshold` is 7 and the lint fires only when the count **exceeds** it; `clippy.toml` sets no override (read in full — it configures only `ignore-interior-mutability`). And the line directly above the attribute reads: *"7 args since A3 (was 8: two arrays became the zip)."* The refactor that comment records is what took it below the threshold; the suppression stayed. | L2 · **STALE-GUARD (candidate)** | **OPEN** · ✅ I VERIFIED the arithmetic; ⚠ lint liveness UNVERIFIED by design | 7 params confirmed; `cat clippy.toml` → no threshold key. ⛔ **Settle it by MUTATION, not by reading:** delete the `#[allow]`, run `cargo clippy --all-targets --release -- -D warnings`. Silent → STALE-GUARD, remove it. Fires → HOLDS, and the threshold is not what we think |
| **2X2 ★★** | excusare | `clause.rs:72-76` | ⭐⭐ **TWO WARDS, ONE RUNE, TWO VERDICTS — the second such split of this cast.** `purgare` rowed this (2P2) as mis-categorised: `trait-contract` on a plain enum field no trait bound mandates. `excusare` — whose entire remit is weighing exemptions — returns **HOLDS**, and gives its ground explicitly: category fit is *"a check another ward can decide"* and therefore outside its remit; its own question is only whether the reason earns the `#[allow(dead_code)]`, and it independently confirmed the reason is TRUE (`acc_form` written once at `clause.rs:354`, never read as this field; fire reads `node_named_ast(node, "acc-form")` in `kernel/arm.rs`). **Both stand as returned.** `vigilia` forbids me re-classifying a child's verdict — so this is a decision for the builder. | 2P2 says mis-categorised · 2X1 says HOLDS | **OPEN — NEEDS A DECISION** · ✅ I VERIFIED both readings | the reason's truth and the category's fit are **separable**, and the two wards each judged a different one. Closed when the builder rules whether a true reason under a wrong category is a defect |
| **2T1 ★** | struere | `where_tree.rs:271-272` vs `:299` | ⭐ **THE DOC NAMES A DOWNGRADE THE CODE DOES NOT DO.** `walk`'s header: *"The moment the walk takes a wildcard **or a range edge** — a guard, not a proof — everything below it is `maybe`."* The wildcard arm (`:316`) passes `false` — matches. The range arm's `Some(true)` (`:299`) passes **`proven` unchanged**. ⚠ The code is nonetheless SAFE, for a reason the doc never states: the sole caller gates on `proven.contains(&tid) && sink.where_tree.is_pure_cmp(tid)` (`kernel/fire/mod.rs:2284`), and for a pure-cmp id `DimCon` permits one constraint per dim, so a held range edge really does prove its dim. **Half the contract lives in another file.** | **L1** (ward's severity, passed through) | **OPEN** · ✅ I VERIFIED | `:299` → `walk(child, …, proven, …)`; `:316` → `walk(wc, …, false, …)`; `:301` (`None`) → `false`. ⛔ **The hazard is a plausible "fix":** forcing `proven=false` on any range edge would match the doc and silently kill the pure-cmp fast path for every range-typed clause — a pessimization no test would redden. Closed by stating the real invariant on `walk` |
| **2T2 ★★** | struere | `compiled_cond.rs:1567-1614`; doc at `:95-96` and `:1538-1539` | ⭐⭐ **A TEST NAMED `every_op_variant_lands_in_core_or_driver` THAT DOES NOT COVER EVERY OP VARIANT.** `lands()` (`:1550-1565`) is a genuine compiler-enforced exhaustive match over `Op`'s **8** variants and classifies `Bind \| Eval → Driver`. The test drives it from a **hand-typed array of 7** — `Op::Eval` is absent — and then asserts `driver.len() == 1, "driver must be exactly Bind"`. **That assertion passes only because of the omission.** Three sources now disagree: the module doc (`:95-96`, *"Driver = slot population (`Bind` only)"*), the `Lands` doc (`:1538-1539`, same), and `lands()` itself (`Bind \| Eval`). The test freezes the two stale ones. | L2 (ward's severity, passed through) | **OPEN** · ✅ I VERIFIED | `Op` has 8 variants (`enum Op` at `:102`, bounded read); array lists 7; `:1557` maps `Bind \| Eval → Driver`; `:1608-1612` asserts `driver.len() == 1`. ⛔ **See my note — completing the array REDDENS the test, and the red points at the wrong file.** Closed by deriving `variants` exhaustively and fixing both stale docs |

## Verified by the orchestrator — target 2

- **2C1** — CONFIRMED, and it is the *good* kind of L2: a file that already contains its own correct
  sentence. `matcher.rs:981-985` accepts four literal kinds and falls to `_ => None`;
  `validate/mod.rs:720-725` correctly omits Rational/BigInt/Nil; `:748` says it precisely. Only the
  doc comment at `:709` generalises to "a literal". Not vacuous — all three unhandled variants
  really exist (`hash.rs:189,201,218`).

⛔ **AND THE WARD CORRECTED MY BRIEF — THE ERROR WAS MINE, AND IT IS ONE OF MY OWN NAMED FAILURES.**
I handed it a list of `Mirrors` claims and wrote: *"That grep returned 7 lines; the 7th is a
continuation."* **It is not.** There are **seven distinct claims**, and the one I omitted is
`src/rete/expr_ir/eval.rs:1191` (`TupleNew` mirrors `eval_tuple_ctor`) — a real, checkable spec claim
in a file I never quoted. The cause: I ran the grep piped through `head -6`, read the total as 7, and
*inferred* the seventh was a wrap-around without ever looking at it. That is exactly
`[[a-truncating-pager-makes-absence-unfalsifiable]]` and `[[never-cite-a-symbol-you-have-not-grepped]]`,
committed inside a brief whose entire purpose was to hand a ward verified ground.

⭐ **What caught it was the brief's own defensive clause** — *"derive your own list and say if you find
more or fewer than I did."* Without that sentence the ward would have audited my six, found them
sound, and the seventh would have gone unread with nobody the wiser. **Keep that clause in every
brief that hands a ward a pre-measured list.** A measurement handed down is a claim, and the cheapest
way to test it is to ask the worker to re-derive it and report the delta.

⭐ **The ward also went BEYOND its list, correctly.** It swept the five `.wat` spec files for
mirroring language and found an eighth claim (`compile.wat:1076`); it spot-checked a lowercase
`mirrors` at `compiled_cond.rs:132` that its own grep pattern would have missed; and it read
`clause.rs:25`'s topology claim against `compile.wat:364-650`. **All eight hold.** For a ward that
produced this cast's first L1 on target 1 by reading exactly this kind of claim, eight-for-eight is
a real statement about the compile side — not silence.

- **2F1** — CONFIRMED. `purity.rs:1513` is `Err(AxisViolation::at(crate::rust_caller_span!(), path,
  axis))`; the justification at `:168-169` is real, on point, and names `classify_native_fn`
  explicitly. Zero `rune:conformare` exist anywhere in target 2. So the finding is **not** that the
  exception is wrong — it is that the exception's warrant is unreachable from the site and
  unfindable by machine.

⭐⭐ **AND THE HEADLINE IS THE CONFORMANCE, NOT THE FINDING — this is the ward answering the question
its target-1 cast could not.** On target 1 `conformare` reported, correctly, *"this target defines no
error type of its own."* That answer is what exposed the scope hole and widened target 2. Cast on the
ground it pointed at, it found **four** error-type families where my brief named three — it located
`LowerError` (`expr_ir/mod.rs:188`), `ReteDefnCheckError` (`purity.rs:1667`) and `AxisViolation`
(`purity.rs:174`) beyond the three I handed it — and **all four independently use Pattern A**: span
on the outer struct, kind enum carrying variant data, span mandatory at construction. I verified the
principal: `ReteCheckError { span: Span, kind: ReteCheckErrorKind }`, and `error.rs:432` names the
pattern itself. **Zero `impl From<`** in the whole 20-file Rust surface, so there is no conversion
boundary that could drop a span. **Zero `Span::unknown()`.**

That is a real statement about the compile side, and only a cast ward could make it: the wrong shape
here is not merely discouraged, it is **uncompilable**, and it got that way independently in four
places.

⭐ **The ward's scope discipline is worth copying.** It excluded five candidates *with stated
reasons* rather than padding: `reachability.rs`'s `Verdict`/`DefectKind` (a calibration ledger, never
in a `Result` a caller sees), `step_payload.rs`'s `Result<_, String>` (the string is an in-band
explain-trace VALUE, not a raised diagnostic — a rendering concern its own spell excludes), and
`EvalBreak`/`RuntimeError` (defined outside the target; read only to resolve threading). It also
declined to flag `matcher.rs:854` discarding a `LowerError` via `.ok()`, correctly naming that a
`solvere` question about error-recovery strategy rather than a span-shape defect. **A ward that
refuses findings outside its own concern is what makes its in-concern findings worth crediting.**

- **2P1 ★ / 2P2** — CONFIRMED, **and the pair is sharper than either row alone.** Two runes sit six
  lines apart on two fields of one enum variant, both categorised `trait-contract`, and they fail in
  *opposite* ways:
  · **`var`** — the rune's REASON is **false**. It says *"current consumers only walk `from`"*;
    `validate/mod.rs:561-562` reads `var` and pushes it into the trapped-bind wall. The field is
    live, the suppression is unneeded, and the sentence has been wrong since whenever that consumer
    landed.
  · **`acc_form`** — the rune's REASON is **true**. It says fire reads acc-form off the compiled
    `AccumulateNode`, and it does: `kernel/arm.rs:415` and `:925` call
    `node_named_ast(node, "acc-form")`. The field really is dead. Only the CATEGORY is wrong.
  ⭐ **That is the distinction the ward earned its keep on** — a lazier sweep flags both as "stale
  runes" or clears both as "explained", and either way one of the two is mis-served. A rune has two
  separable parts, and they rot independently.

- **2P3** — CONFIRMED **with a correction I am adding, because acting on the row as written could
  break the build.** The ward's claim is exact for the family it names: zero `None` call sites across
  `alpha_match_inner`/`_local`/`_seeded`/`_opts` and `eval_clauses`/`eval_clause`. **But
  `resolve_operand` — the terminus of that same chain — IS called with `None`, from
  `eval_insert.rs:290**` (`resolve_operand(arg, &[], &[], bindings, None)`). ⚠ And that is not
  incidental: `conferre` cited *that exact call* as the evidence for its own adjudication of the
  `rhs_operand_can_never_resolve` mirroring claim — the `None` is what makes a Keyword genuinely
  unresolvable on the RHS path. **So the `Option` is vacuous in the matcher family and load-bearing
  at the terminus.** A future hand collapsing "the whole chain" to `&SymbolTable` would delete a
  live distinction two wards depend on. Collapse the matcher family only, or rune it.

- **2P6** — CONFIRMED, and the ward's method is why I credit it. A substring grep for `count-of`
  returns hits in `:sq::` and `:t118b::` namespaces — scratch probes and type-system tests. It
  excluded them by **full-name** match and said so. My own re-run of
  `grep -rn 'factbag::count-of'` returns exactly **one** line: the definition.
  `[[a-throwaway-sweep-is-an-instrument]]` — this is what anchoring one looks like.

- **2P7 ★** — CONFIRMED and it is the best of the seven. `AxisViolation/head` is read **4** times
  (`compile.wat:332,339,346,359`); `AxisViolation/axis` and `AxisViolation/span` are read **zero**
  times anywhere in `wat/`, `wat-scripts/`, `tests/` or `src/`. The Rust side builds both on every
  construction. So a real source location is computed, carried across the language boundary, stored
  in a record whose doc promises the caller can *"report as well as the substrate can"* — and then
  never read. ⚠ **This is the same family as target 1's F1 and `conformare`'s whole concern**: a
  diagnostic that cannot name the user's line. There the span was discarded at the boundary; here it
  is carried faithfully and dropped by the consumer. Different mechanism, same loss.

⭐ **purgare came back CLEAN on two of its five sweep groups and said which** — `purity.rs`,
`reachability.rs`, `step_payload.rs` (group C) and `vocabulary.rs`, `where_tree.rs`,
`expr_ir/{eval,mod}.rs` (group D). It also cleared the **108-row `RETE_OPS` table** explicitly, which
was the scope correction I most worried it would mishandle: a table row without a caller is not dead
when the table IS the language's operator surface. And it honoured the other correction — it did not
flag `reachability.rs` for lacking callers, having read its DISCONFIRMING-PROBE header first.

## Cast log — target 2

| target | cast at | wards mustered | returned | still to cast | L1 | L2 |
|---|---|---|---|---|---|---|
| 2 · `src/rete/**` minus `kernel/` + `wat/rete*.wat` (25 files, 23,886 lines) | 2026-09-07 | 14 read-only + `experiri` sequenced separately | **6** — conferre · conformare · purgare · solvere · excusare · struere | intueri · sequi · temperare · exigere · cernere · probare · perspicere, then **`experiri`** (serialized, it DRIVES), then **`circumspicere` LAST** | **1** | 15 |

- **2S1 ★** — CONFIRMED, **and it pairs with `conferre` in a way neither ward could see alone.**
  `conferre` read these exact two bodies this cast (its claim #3) and adjudicated them **TRUE — no
  observable divergence**. `solvere` read the same two bodies and says they are **two hand-written
  copies with no shared function**. *Both are right, and together they are the finding:* the copies
  agree **today**, which is precisely why nothing has ever gone red, and there is no mechanism that
  makes them agree tomorrow. A spec-fidelity ward can only report the current state of a
  duplication; only the structural ward can say it is a duplication at all.
  ⚠ The one real difference between the copies is already known and benign: `?`-propagation
  (a `None` short-circuits `eval_clause`) vs `matches!` (a `None` becomes `false`). `conferre`
  checked it and found both produce "clause fails" for `Lt/Gt/Le/Ge`. **That is the drift surface
  with one foot already on it.**

- **2S2 ★★** — CONFIRMED, and it is the sharpest row of target 2 so far, because **one file holds
  both ends of the extirpare ladder and says so.**
  · `pack_expr` sits at the TOP rung — a bare `match` with no catch-all, so a new `Expr` variant
    **has no way to be written down** without its arm. The wrong shape is uncompilable.
  · `unpack_expr` sits at the BOTTOM — a runtime string tag with `other => Err(malformed(…))`, so
    the same new variant compiles clean and degrades at runtime.
  · The file's own doc names the asymmetry exactly (*"the one whose exhaustiveness the compiler
    enforces for you… Its inverse cannot get that guarantee"*) and nominates the mitigant: a test
    corpus, plus the instruction that *"a new variant belongs in their corpus in the same change."*
  ⛔ **That mitigant is a CONVENTION guarding a check.** The tests fire only if a human remembers to
  extend the corpus — which is rung 1 of the ladder, protecting a rung-3 sibling in the same file.
  The doc is honest, accurate, and dated; what it is not is a cure. ⚠ **And this is the sharpest
  form of the arc's signature shape yet**: not a fix that never reached a sibling path, but a fix
  that reached one HALF of a pair and structurally cannot reach the other in its present form.

- **2S3** — CONFIRMED. `compile.wat:1102` reads, verbatim: *"compile-query — same LHS fold as
  compile-rule; terminal is a QueryNode."* The duplication is named in a comment directly above the
  duplicate.

⛔ **TWO OF THE THREE ARE SELF-DIAGNOSED, AND THAT IS THE PATTERN WORTH NAMING.** 2S2's file says
*"Its inverse cannot get that guarantee"*; 2S3's comment says *"same LHS fold as compile-rule"*.
Neither is a case of nobody noticing. **In both, noticing is where it stopped** — the observation was
written down, correctly and durably, and then served as the record that the thing was understood
rather than as the trigger to fix it. An accurate comment naming a defect is evidence the defect is
known; it is not evidence it is bounded. `[[an-accurate-comment-can-be-a-defects-alibi]]` — and here
it recurs twice in one ward's return.

⭐ **`solvere` also cleared the axis the cast most expected to be tangled, and named its method.** It
grepped `classify_rete_clause` and found every consumer — `matcher.rs`, `compiled_cond.rs`,
`alpha_tree.rs`, `step_payload.rs`, `validate/{mod,typing}.rs`, `kernel/{stratify,arm}.rs` — routing
through **one door**. The "four walkers over one grammar" worry I put in the brief is, on the
clause-classification axis, already cured. It then found the real duplication one level down, in the
comparison table those walkers each evaluate. **And it declined `where_tree.rs`'s range family with
a reason**: that code is the *product* of a prior consolidation whose own doc records the fifth
hand-match it replaced. A ward that can tell a cure from a defect is worth casting.

- **2X1 ★** — CONFIRMED on the arithmetic, and the ward's restraint is what makes it citable.
  `from_parts` takes `ops, zip, n_slots, seed_reads, fact_bind, span, slot_names` — **7**.
  `clippy.toml` configures only `ignore-interior-mutability`; there is no threshold key. And the
  comment at `:245` is the tell, in the exemption's own hand: *"7 args since A3 (was 8: two arrays
  became the zip)."*
  ⭐ **It refused to assert what it could not run.** Told READ-ONLY, it did not run clippy, and said
  in its own report that the lint's inertness is *unverified* — asserting only the arithmetic and
  the absence of an override. That is why the row can carry a mutation as its re-derivation instead
  of a claim. `[[a-reading-cannot-see-an-execution-defect]]` — the reading is sound and the reading
  is not the proof.

- **2X2 ★★** — CONFIRMED, and it is **the second ward-disagreement of this cast.** X3 on target 1
  was three wards on one rune; this is two wards on one rune, and the axis of disagreement is
  cleaner: **a rune has a REASON and a CATEGORY, and they are separately judgeable.** `purgare`
  judged the category (wrong for a plain enum field). `excusare` judged the reason (true, and
  independently re-verified) and explicitly declined the category as another ward's business. Both
  are correct on their own axis. Whether a true reason under a wrong category is a defect is not
  mine to rule — `vigilia` forbids the aggregator re-classifying a child.
  ⚠ Note this makes **2P1/2P2 a matched pair once more**, from a second direction: on `var` the two
  wards AGREE (excusare independently reached ILLEGITIMATE-AT-BIRTH), on `acc_form` they SPLIT. The
  rune whose reason is false draws unanimity; the rune whose reason is true does not.

⛔⛔ **AND EXCUSARE CORRECTED MY ENUMERATION — IN BOTH DIRECTIONS. THAT IS TWO FOR TWO.**
I handed it "33 runes, 5 allows". Both were wrong:
· **My 33 was Rust-only.** The wat spec half carries one more — `wat/rete.wat:524`,
  `rune:exigere(scope-affirmative)` — and my target list explicitly includes those five files.
  Re-derived: **33 Rust + 1 wat = 34.**
· **My 5 allows were 4.** My pattern `#\[allow\(` matched a **prose mention inside a doc comment**:
  `validate/mod.rs:392` reads *"an alternative here was an `#[allow(clippy::too_many_arguments)]`,
  which silences the signal"* — a note about what they deliberately did NOT do. ⚠ Precision, because
  the ward slightly overstated: it named *two* prose sites, but only that one matched my pattern
  (`typing.rs:44` writes `` `#[allow]` `` with no paren). The substantive correction — **4 real
  attributes** — is exactly right.

**Every pre-measured list I have handed a ward this cast has contained an error.** `conferre` found
one (a seventh `Mirrors` claim I called a continuation without looking). `excusare` found two.
Three errors, two lists, one session — and in all three cases the *only* thing that surfaced them
was the clause instructing the ward to re-derive and report the delta.
⭐ **That clause is now the most load-bearing sentence in the casting procedure**, and it is cheap:
one sentence buys an independent check on every number the orchestrator hands down. A measurement
handed to a worker is a claim wearing a measurement's clothes
(`[[a-throwaway-sweep-is-an-instrument]]`, `[[an-example-in-a-brief-is-a-claim-too]]`).

⭐ **Its method deserves the credit too: it weighed mechanically, not for plausibility.** All 16
`cited-name-absent` runes were checked by grepping the repo for each cited name and confirming
**zero code positions** — the rune's exact claim. It found the repo's own gates behind several
categories (`rete_citation_resolves`, `no_loose_string_assert`, `retired_name_justified`,
`no_unknown_ward_rune`) and used their source as corroboration, noting correctly that reading a
gate is not running it.
⚠ **And it surfaced a gap nobody asked about:** `docs/CONVENTIONS.md` carries closed-set vocabulary
tables for `sequi`, `perspicere`, `purgare` and `excusare` — but **`solvere`, `exigere` and
`temperare` have no closed-set gate anywhere in this tree**. Each of those three is in use in this
target. That is the same shape `sequi` had before its 2026-08-25 incident, and it is a finding about
the guard rather than about the code.

- **2T1 ★** — CONFIRMED on all four arms. `:299` (`Some(true)`) passes `proven` through; `:301`
  (`None`) passes `false`; `:316` (wildcard) passes `false`. So the doc's *"or a range edge"* is
  false for exactly the held-range case, and true for the other two. ⚠ **The ward did the harder
  half**: rather than stopping at "doc wrong", it traced WHY the code is sound anyway — the caller's
  second gate at `kernel/fire/mod.rs:2284` — and named that the real contract is split across two
  files. **The hazard it identifies is the sharp part and I want it on the record:** a maintainer
  reading only this doc would "fix" `:299` to force `proven=false`, matching the stated contract and
  silently killing the pure-cmp fast path for every range-typed clause. That is a pessimization, not
  a wrong answer — **no test would go red.** A doc that invites a correct-looking change into a
  silent regression is worse than one that merely mumbles.

- **2T2 ★★** — CONFIRMED, **and it is sharper than the ward graded it.**
  · `Op` has exactly **8** variants — I anchored this, because my first extraction was WRONG: an
    unbounded `awk` swept in `OperandLowering`'s variants from `:633` and reported 11. Re-run with
    the enum's real bounds: `Bind, BindCheck, Cmp, SeedCmp, Eval, Or, Not, Fail`.
    `[[a-throwaway-sweep-is-an-instrument]]`, again, in the middle of verifying a finding *about* a
    miscount.
  · The array lists **7**. `Op::Eval` is absent. `lands()` maps `Bind | Eval → Driver`. The test
    asserts `driver.len() == 1` **and** `matches!(driver[0], Op::Bind { .. })`.
  · So the test's two assertions encode *"Driver is exactly Bind"* — which the live classifier
    directly contradicts — and they pass **only** because the input omits the counter-example.
  ⛔ **THIS IS NOT MERELY VACUOUS; IT IS A TRAP.** The obvious repair — add `Op::Eval` to the array,
  completing the coverage the test's NAME already claims — makes it go **RED**. And the red does not
  point at the array: it points at `driver.len() == 1`, i.e. at the live, correct `lands()`. A
  maintainer following the failure would be led to "fix" the classifier back to `Bind`-only, which
  is the stale taxonomy two doc comments still assert. **The fixture actively defends the wrong
  answer against its own repair.**
  ⚠ The ward graded this **L2** and `vigilia` forbids me re-classifying a child's verdict, so L2
  stands as returned. My reading is that a test whose *name* claims coverage it does not have, and
  whose assertions pin a taxonomy its own subject contradicts, is closer to L1. **That disagreement
  is the builder's to settle, not mine.**

⭐ **AND THE CLEAN HALF IS A REAL RESULT ABOUT THIS TARGET.** `struere` swept every
`panic!`/`.unwrap()`/`.expect()` in all 25 files, classified each against its file's `#[cfg(test)]`
boundary, and found **ZERO production sites reachable from user rule text** — against target 1's
**eleven** join-key `panic!` sites, which were its own sharpest row there (T1). The two non-test
hits (`export.rs:1985,1997`) are same-map lookups where the key was just collected from the map.
It also traced ~30 `-> bool` classifiers to their callers hunting the `ClassPlan::observe` shape it
found on target 1 — **none found.** Two of its target-1 signature defects are simply absent here,
and it said so with the method. That is the answer to *"prove the others don't find anything."*
