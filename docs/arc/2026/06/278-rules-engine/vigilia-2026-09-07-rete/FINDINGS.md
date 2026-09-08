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
| **2I1 ★★** | intueri | `vocabulary.rs:104-105`, `:1845`, `:1846`, `:1853` | ⭐⭐ **FOUR NUMBERS DESCRIBE ONE ARRAY, IN ONE FILE, AND ONLY ONE IS GATED.** `NAMING_RULE_EXCEPTIONS` is claimed as **nine** by the module header, **eleven** by the test's own doc comment, **fourteen** by that test's function NAME one line below it, and **14** by its assertion one line below that. The assertion is the only mechanically enforced one, and it is the true one. ⛔ **The doc comment disagrees with its own test's name TWO LINES AWAY.** | **L1** (ward's severity, passed through) | **OPEN** · ✅ I VERIFIED | `sed -n '104,105p'` → *"Nine rows total"*; `:1845` → *"exactly the eleven rows"*; `:1846` → `fn naming_rule_exceptions_are_exactly_the_documented_fourteen`; `:1853` → `assert_eq!(…len(), 14)`. Closed by deleting both prose numbers in favour of *"count enforced by the gate below"* |
| **2I2** | intueri | `export.rs:2274` | `import_export`'s doc says *"Its 194 lines are phase COUNT rather than depth."* The body spans `:2278-2535` — **258 lines**. The qualitative claim it supports (9 phases, brace nesting peaks at 3) is still true; only the number rotted, by 64. | **L1** (ward's severity, passed through) | **OPEN** · ✅ I VERIFIED | body end located by brace-match at `:2535`; 2535−2277 = 258. Closed by dropping the number — the point stands without it |
| **2I3 ★** | intueri | `purity.rs:1811` | ⭐ **A CYCLE DETECTOR WHOSE HEADER CALLS ITSELF PURITY — IN THE FILE WHERE "PURE" IS THE LOAD-BEARING TERM.** `walk_rete_defn_callees`'s first line: *"Walk a rete definition's callees for **purity**, returning the first **impure** one."* It classifies no purity; it is a gray/black DFS cycle detector, called only from `rete_defn_cycle`. ⚠ Two neighbouring comments in the same file say the opposite explicitly — `:1803` *"this walk is a LOAD refusal, not a fifth axis"* and `:1782` *"cycle is a second question (#87 recursion), not a fifth fence axis."* And the doc's OWN second paragraph correctly describes the gray/black colouring. **The header contradicts its own body-description.** | L2 | **OPEN** · ✅ I VERIFIED | `:1811` uses "purity"/"impure"; `:1813-1815` describes cycle-safe colouring; `:1782` and `:1803` both name cycle-≠-purity. Closed by naming the recursion cycle in the first line |
| **2M1 ★★** | temperare | `where_tree.rs:84` vs `alpha_tree.rs:56` | ⭐⭐ **TWO FILES, THE SAME TYPE-ALIAS NAME, ONE FIXED WITH A MEASURED STONE AND ONE NOT.** Both declare `type EqChildren` for a discrimination tree's equality fan-out. `alpha_tree.rs:56` is `FxHashMap`, carrying a measurement at `:66-67`: *"FxHash — SipHash of the field `Value` 40k times was the I−G walk (`DESIGN-STONE-alpha-tree-fxhash`)."* `where_tree.rs:84` is plain `std` `HashMap`, and the file does not import `rustc_hash` at all. ⚠ **This one IS on the fire path** — 2T1 established that `walk`'s `proven` flag gates the pure-cmp fast path at `kernel/fire/mod.rs:2284`. Dimension: **n tokens** × tree depth, every round. | L2 · fire-dimension | **OPEN** · ✅ I VERIFIED | `grep -n 'rustc_hash\|FxHashMap' where_tree.rs` → **0**; same in `alpha_tree.rs` → `:46,56,150`. Closed by swapping `:84` and the maps feeding it, matching the sibling |
| **2M2** | temperare | `eval_insert.rs:148`, `:170`, `:172` | `build_insert_fact` performs the identical `sym.types().and_then(\|t\| t.get(type_keyword))` twice — once to bind `names`, once for `field_names`. Control only reaches `:170` when `:148` already returned `Some(Aggregate(_))`, so the `_ => Arc::new(Vec::new())` fallback at `:172` is **unreachable**. Dimension: n calls to the interpreter/differential door — not production fire. | L2 | **OPEN** · ⚠ ward-reported | closed by `let field_names = names.clone();` and deleting the dead arm |
| **2M3** | temperare | `purity.rs:1371-1394`; call sites `compile.wat:450,589,742` | Each fence calls `pure?`/`deterministic?`/`total?`/`primitive?` as four primitives, each doing its own single-axis `classify_expr`. The axis loop sits **inside** one recursive descent (`:1387`), so 4 calls = 4 full tree walks where 1 would do — and `apply_rete_defn_contracts` already uses `&Axis::ALL` for the same walk. ⚠ The ward checked this against the builder's 2026-08-05 ruling at `compile.wat:273-290` and found it does **not** conflict: that ruling forbids *skipping* an axis, not *sharing a traversal*. | L2 · freeze-time | **OPEN** · ⚠ ward-reported | closed by one `&Axis::ALL` call per fence yielding four booleans |
| **2M4** | temperare | `wat/rete/compile.wat:1038` + `:1061` | `sort-lhs` calls `uses-result?` once per condition in the `independent` guard and again in the complementary `rest` guard — two recursive `ast-qvars` walks per condition for one boolean. Dimension: n LHS conditions per rule, doubled, freeze-time. | L2 · freeze-time | **OPEN** · ⚠ ward-reported | closed by one fold producing a three-way bucket verdict |
| **2M5** | temperare | `wat/rete/acc.wat:107-122` | `acc::distinct` is `foldl` + `PersistentVector/contains?` — a linear scan before every `conj`, so O(n²) in elements gathered. ⚠ **Oracle-only**: `kernel/arm.rs:285` recognises the name and dispatches a native `AccFold::Distinct`, so native fire never runs this body. It is real cost on every floor run, in the oracle's interpreted accumulate pass. | L2 · oracle-dimension | **OPEN** · ⚠ ward-reported | closed by a seen-`PersistentMap` instead of `contains?` |
| **2M6** | temperare | `wat/rete.wat:539` | `render-dag`'s outer `foldl` does `(string::concat acc line)` on a growing accumulator — potentially O(n²) in output length. ⚠ The ward checked whether the adjacent `rune:exigere(scope-affirmative)` at `:524-527` covers it: **it does not** — that rune protects the fixed-depth nested-concat that builds one `line`, not the outer accumulator. Diagnostic renderer, lowest priority. | L3 | **OPEN** · ⚠ ward-reported | closed by a rope/joiner, or left as-is with a rune |
| **2E1** | exigere | `wat/rete/factbag.wat:7` | *"Doors, all under `:wat::rete::factbag::` — the whitelist a future rung-3 seal **will name**:"* — a future-work promise naming **no tracker**. *"rung-3"* is a repo-wide phase label, not an arc number. ⚠ **The finding stands on the comment's own text** — it names no arc, checkable in the file itself. The ward's supporting evidence (a companion doc calling a sibling rung-3 item *"neither scoped nor scheduled"*) is `docs/*.md` and therefore **corroboration, not authority**, under today's ruling. | **L1** (ward's severity, passed through) | **OPEN** · ✅ I VERIFIED | `sed -n '7p' wat/rete/factbag.wat` → the phrase, verbatim; the line cites no arc. Closed by formalising the whitelist now, or a `rune:exigere(attested-arc)` naming a real arc |
| **2N1 ★★** | cernere | `expr_ir/eval.rs:59` and `:905` | ⭐⭐ **THE SAME CLASS AS TARGET 1's N1, TWICE MORE — AND IT IS NOW A CLASS, NOT A SITE.** Two user-facing `MalformedForm` errors carry `head: ":wat::rete::exec_value"` and `head: ":wat::rete::apply_op"`. Both are **the Rust functions' own names**, verbatim — `fn exec_value` at `:69`, `fn apply_op` at `:890`, each ~10 lines from its own error site. Both snake_case inside a namespace where every real name is kebab-case. Both resolve **nowhere**: one repo-wide hit each, their own construction site. ⭐ **And the same file demonstrates BOTH correct alternatives** — a real FQDN at `:772` (`":wat::rete::core::match"`) and a plain non-namespaced label at `:1295` (`"compiled-exec"`) for exactly this kind of internal guard. | L2 | **OPEN** · ✅ I VERIFIED | `grep -rn ':wat::rete::exec_value'` → **1**; `apply_op` → **1**; `grep -n 'fn exec_value\|fn apply_op'` → same file, same neighbourhood. Closed by adopting `:1295`'s non-namespaced form at both sites |
| **2N2** | cernere | `clause.rs:551` | `":wat::rete::core::vector::="` appears in `unrelated_heads_are_not_constraints`'s list of heads that must NOT classify. No `RETE_OPS` row has ever borne that name (checked against the full 79-row extraction) and it appears nowhere else in the tree. ⚠ **The ward graded its own confidence LOWER here and said why**: the name is used correctly as a *negative probe*, which is what `cernere`'s `spell-probe` rune category exists for — it simply carries no rune. Test-only data, never user-visible. | L3 | **OPEN** · ⚠ ward-reported | closed by a `rune:cernere(spell-probe)` naming the test, or by using a real inadmissible head |
| **2R1 ★★** | probare | `matcher.rs:114-131` | ⭐⭐ **"THREE INDEPENDENT SITES" — THERE ARE FOUR, AND THE FOURTH IS NAMED NOWHERE.** The doc justifies `enum_variant_ctor`'s `(enum, variant, arity)` return shape by *"the three callers"* and says so three times (`:115`, `:124`, `:127`). Actual callers: `purity.rs:976`, `expr_ir/mod.rs:1087`, `validate/mod.rs:1056`, **and `validate/typing.rs:335`** — the last resolving `Unit`/`Tagged` for a diagnostic classification, a fourth purpose the doc does not mention. ⚠ **The ward proposed a cause worth keeping**: this likely postdates `partire`'s 2026-08-30 split of `validate.rs` into `mod`/`typing`/`error` — the split created a caller in the new file that the doc in the *other* file never absorbed. | **L1** (ward's severity, passed through) | **OPEN** · ✅ I VERIFIED | `grep -rn 'enum_variant_ctor(' src/` → 4 callers of `matcher::enum_variant_ctor` (`check.rs`'s `literal_enum_variant_ctor` is a different, local fn — correctly excluded). `grep tests/lint/` → **no gate pins this count.** Closed by correcting the number and naming the fourth |
| **2R2** | probare | `validate/mod.rs:66-67` | *"Full coverage is each CALLER's job, and **both callers** now enforce it"* — naming `eval_kwargs_construct` (runtime.rs) and `validate_rule_when_and_reorder_then`. There are **three** production call sites: `runtime.rs:18965`, `check.rs:13115`, `validate/mod.rs:1284`. ⚠ **Weaker than 2R1 and I am saying so**: the doc DOES mention `check.rs`'s `infer_kwargs_construct_check` — but as *enforcement backing the runtime caller*, not as a caller of this helper. The ward read `check.rs:13095-13118` and found it a **direct, independent call**. So the call-site count is wrong; whether the doc's own taxonomy makes "both" defensible is arguable. | L2 | **OPEN** · ✅ I VERIFIED | 3 production sites + 2 `#[cfg(test)]` in-file (`:1664`, `:1675`, correctly excluded); no gate pins it |
| **2V1 ★★** | perspicere | `compiled_cond.rs:791-792`; `alpha_tree.rs:58` vs `:228`; `expr_ir/eval.rs:1469` | ⭐⭐ **THREE SITES SPELL BY HAND A TYPE THE FILE ALREADY NAMES — AND ONE OF THEM IS THE FUNCTION'S OWN RETURN TYPE.** `invert_slot_names` returns `expr_ir::SlotNames` (= `Box<[SlotName]>`, `SlotName = Option<Arc<str>>`) and its **very next line** declares `let mut out: Vec<Option<Arc<str>>>` — re-spelling the element type of its own signature. `alpha_tree.rs:58` declares `type AlphaWildcard = Option<Arc<AlphaDiscNode>>` (used twice) while `:228` writes it longhand 170 lines below. `expr_ir/eval.rs:1469` re-spells `compiled_cond::SlotFrame` in a doc comment that **already states the equivalence in prose** two lines above. | L2 | **OPEN** · ✅ I VERIFIED | `sed -n '58p;228p' alpha_tree.rs` → alias then longhand; `sed -n '790,792p' compiled_cond.rs` → `-> SlotNames` then `Vec<Option<Arc<str>>>`; `expr_ir/mod.rs:167-168` → the alias pair. Closed by three substitutions, no new nouns |
| **2V2** | perspicere | `export.rs:721`; `expr_ir/mod.rs:833`, `:163` | The `(field-name, slot-index)` pair — `(Arc<str>, u16)` — is spelled longhand at three sites across two modules with **no alias anywhere**, while `expr_ir/mod.rs:167-168` already hosts a `Slot*` family it would sit beside. ⭐ **The ward applied its own rule-4 bar before recommending**: 3 uses across 2 modules clears "reused, not read-once". | L2 | **OPEN** · ⚠ ward-reported | closed by `type FieldSlot = (Arc<str>, u16);` beside `SlotName` |
| **2V3** | perspicere | `wat/rete.wat:32`, recurring at `:429` | The wat-side mint. `Token.matches` inlines `(PersistentVector :- [(Tuple :- [Record i64])])` in a `defrecord` field — 2 levels of bracket nesting, the analog the ward derived for a language with no angle brackets. The inner `(Tuple :- [Record i64])` is **one alpha-hit**; the file's own comment at `:419` says *"each tuple is (sfact, alpha-id)"*, and the identical shape recurs at `:429` as a lambda parameter. ⚠ **Confirmed NOT read-once by the twin**, and no `typealias :wat::rete::*` covers it (all six checked). | L2 | **OPEN** · ⚠ ward-reported | closed by a `:wat::rete::AlphaHit` typealias used at both sites |
| **2Y1 ★★** | experiri | `reachability.rs:83-85`; harness at `docs/arc/2026/06/278-rules-engine/harness-experiri/` | ⭐⭐ **A CAST-LEVEL L1: THE `:then` COLUMN IS UNTESTED, AND THE INSTRUMENT TO TEST IT WAS BUILT AND SHELVED.** The live ledger sweeps `InlineConstraint` + `WhereFence` (79/79 rows, all fire) and `AccHeadFold` (1/1). The **fourth** position — `:then`, which the vocabulary's own module doc declares admissible for every rete verb — is excluded with the reason *"remains deliberately unmodelled… an un-calibrated position would add a column of findings nobody can trust."* ⛔ **That reason is falsified by the repo itself:** a calibrated harness for exactly this position exists at `harness-experiri/positions-3-4.rs.txt`, was driven 2026-08-30, and found **two real L1s since fixed**. It is saved as `.rs.txt` — the README says *"so no tooling mistakes it for a live module"* — and *"asserts nothing."* The ward reproduced its calibration clean today and drove 4 of 79 rows at `:then`, all firing. **The exclusion defers to a harness already built; `experiri`'s rune rule requires a reason that DISPOSES.** | **L1 (cast-level)** | **OPEN** · ✅ I VERIFIED | `README.md:19` → *"Saved as `.rs.txt` so no tooling mistakes it for a live module"*; `:132` → *"`positions-3-4.rs.txt` asserts nothing"*. Closed by converting the shelved sweep to an assertion, or by a reason that disposes |
| **2W1 ★★★** | circumspicere | claim: `expr_ir/mod.rs:14-19` · code: `mod.rs:255-262`, `:374,431,473`; `eval.rs:201` · **the sibling cure: `export.rs:350-370`** | ⭐⭐⭐ **THE ONE EXPRESSION CORE PROMISES "TOTAL OR IT REFUSES" AND HAS NO DEPTH GUARD AT ALL.** The header: *"`lower` IS TOTAL OR IT REFUSES… `exec` therefore raises only on VALUES … **never on shape**. A refusal that belongs at compile time and lands at fire time is a defect in this file."* But `lower_expr`/`lower_list`/`lower_hof_callee` mutually recurse over `WatAST` with **no depth counter** — `LowerCx` has four fields and none is a depth — and `exec` recurses the resulting tree on every row of every fire. The parser feeding it has no guard either. ⛔ **AND THE CURE EXISTS ONE FILE OVER, MEASURED**: `MAX_IMPORT_DEPTH` records that *"the same 20,000-deep Export was ACCEPTED on a 256 MiB thread and killed a 2 MiB one with `fatal runtime error: stack overflow, aborting` — an abort, not a panic, so nothing catches it."* An abort is neither of the two outcomes the header calls exhaustive. | **L1** | **OPEN** · ✅ I VERIFIED | `sed -n '14,19p' expr_ir/mod.rs` → the claim verbatim; `:255-262` → `sym, slots, next, hof_fn_pos`, no depth; `grep -cE 'depth\|recursion_limit\|stacker'` over both lowering files → **0, 0**; `grep -cE 'depth\|recursion\|MAX_' src/parser.rs` → **0**. Closed by threading `depth` through `LowerCx` by `MAX_IMPORT_DEPTH`'s own method — measure the corpus max and the abort window, then cap with headroom |
| **2W2** | circumspicere | `wat/rete.wat:329-352` vs `export.rs:308-370` | The one wat-facing spec of `Export` documents the record field by field — including behavioural notes like *"Import refuses a miss"* — and says **nothing** about `MAX_IMPORT_NODES` (10,000) or `MAX_IMPORT_DEPTH` (300), the two walls `:wat::rete::import` enforces, nor that the build is quadratic beneath them. A wat author reading the only spec that documents this record cannot learn the ceilings exist without reading Rust. | L2 | **OPEN** · ⚠ ward-reported | closed by naming the two walls in the `Export` doc block |

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
| 2 · `src/rete/**` minus `kernel/` + `wat/rete*.wat` (25 files, 23,886 lines) | 2026-09-07 | 14 read-only + `experiri` sequenced separately | **15 of 15 — COMPLETE** — conferre · conformare · purgare · solvere · excusare · struere · intueri · sequi **CLEAN** · temperare · exigere · cernere · probare · perspicere · experiri **DROVE** · circumspicere | perspicere, **`circumspicere` LAST** | **7** | 26 (+2 L3, +1 ward-split) |

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

- **2I1 ★★** — CONFIRMED, all four. This is target 1's *"five counts of one population"* recurring —
  but **worse, and in a way that is worse for a precise reason.** There the five figures lived in
  five different wards' reports and no finding turned on any of them. Here **four figures describe
  one array inside one file**, three of them are prose and one is a gate — and the prose numbers
  rotted at *different rates*, so they do not even agree with each other. The failure is visible in
  two adjacent lines: `:1845`'s doc comment says *"eleven"* while `:1846`'s function name says
  *"fourteen"*.
  ⭐ **And the gated number is the right one** — `assert_eq!(…len(), 14)` passes on the green floor,
  so the array really has 14. That is the lesson in miniature: **the figure that recomputes is
  correct; every figure a human retyped rotted.** `[[a-right-number-vouches-for-a-wrong-label]]` —
  here the right number vouches for three wrong ones sitting above it.

- **2I2** — CONFIRMED. `:2274` claims 194; the body runs `:2278-2535` = **258**. Note what did NOT
  rot: the claim it *supports* — nine phases, brace nesting peaking at 3 — is still accurate. Only
  the number moved. A qualitative claim outlived the quantitative one propping it up.

- **2I3 ★** — CONFIRMED, and the internal contradiction is the sharp part. `walk_rete_defn_callees`'s
  **first line** says purity; its **own next paragraph** correctly describes gray/black cycle
  colouring; and two other comments in the same file (`:1782`, `:1803`) explicitly rule that cycle
  detection is *"a second question (#87 recursion), not a fifth fence axis."* So the file as a whole
  knows the distinction and states it twice — one header reached for the wrong noun anyway. In
  `purity.rs`, where `pure?` is a precisely-scoped axis term, that misdirects a reader to the wrong
  bug class.

⭐⭐ **TWO WARDS LANDED ON THE SAME SITE THROUGH DIFFERENT LENSES — AGAIN.** `intueri`'s fourth
finding is `compiled_cond.rs`'s `every_op_variant_lands_in_core_or_driver`, already rowed as **2T2**
by `struere`. **I have NOT double-rowed it**; I record the second reading here because the two are
complementary and neither is the other:
· `struere` (craft): the fixture is a hand-typed array of 7 against an 8-variant enum, so the
  assertion is vacuous — and completing it reddens the test against the *live* classifier.
· `intueri` (communication): the assertion's **message text** — *"driver must be exactly Bind"* —
  contradicts `lands()`'s own doc **two lines above it**, which says `Eval` is `Bind`'s sibling and
  lands identically. The message overclaims a coverage the test does not have.
One ward found the hole; the other found that the hole is *narrated* as a guarantee. This is the
third such convergence of the vigilia (after I1/T5 and N1/F2 on target 1), and every one has come
from casting the full guard rather than a chosen roster.

⚠ **`intueri` also disclosed a process failure inside its own cast, unprompted**: one of its three
forks *"mistook itself for a coordinator and briefly delegated out of scope"*, and the ward
disregarded that side work, using only the fork's own direct findings — which it then re-derived
against the live file itself before reporting. **A ward that reports its own contamination and says
what it discarded is a ward whose clean findings are worth more**, not less.

⭐ It also re-derived the handed line count and reported **delta 0** — the first of my measurements
this cast to survive a re-derivation intact.

⛔⛔ **AND I COMMITTED THE DEFECT `2I1` DESCRIBES, IN THE COMMIT THAT ROWED IT.**
Rowing *"four numbers describe one array and only one is gated"*, I then typed **16 L2** into the
cast log by hand. My verification `awk` said **14**. The truth is **15** (+1 ward-split row whose
severity cell holds a disagreement rather than a level). **Three numbers, one population, within one
commit.**

Both wrong numbers have causes worth keeping:
· **16** was a hand-carry — I incremented my previous figure instead of recounting. Exactly what the
  module header in `vocabulary.rs` did.
· **14** was an *instrument* failure, and a subtle one: `awk -F'|'` splits a markdown table on the
  same character the table uses, so row **2T2** — whose finding text contains an escaped `\|` inside
  a code span — shifted every field after it and put prose in the severity column. **The counting
  tool and the data shared a delimiter.**
  `[[a-throwaway-sweep-is-an-instrument]]` — this is the third instrument error of this session, and
  the second inside a verification of a finding *about* miscounting.

The count is now re-derived per row, by matching the severity LABEL rather than by field position:
**19 rows = 3 L1 + 15 L2 + 1 ward-split.** ⚠ **The lesson `2I1` states is the cure for this
paragraph:** a number that recomputes is right; a number retyped rots. The cast log should carry a
command, not a total — and until it does, this note is the evidence that its totals cannot be
trusted on sight.

⭐⭐ **`sequi` RETURNS CLEAN ON TARGET 2 — AND IT IS THE FIRST WARD TO USE THE CORRECTED VOCABULARY.**
No rows. The value is entirely in what it says it looked at, and **I re-ran every grep it reported**:

| its claim | my re-run |
|---|---|
| `set!` in the 5 wat files | **0** |
| `Atom` in the 5 wat files | **0** |
| `Mutex`/`RwLock` in the 20 `.rs` | **0** |
| `Atomic` in the 20 `.rs` | **0** |
| `thread_local!` in the 20 `.rs` | **1** (`eval.rs:87`) |
| `unsafe` in the 20 `.rs` | **0** |
| `rune:sequi` in the target | **2** |

Every figure reproduces exactly. **This is the first return this cast where a handed measurement and a
re-derived one agreed on every line** — and it is the ward's own numbers, not mine, which is the point:
a CLEAN whose method is reproducible is evidence; a CLEAN without one is silence.

⭐ **It also closed the corrected-clause loop.** The convergence clause was rewritten this session
after `conformare` and `perspicere` both ended divergent reports with the word CONVERGED. `sequi` is
the first ward cast under the new wording, and it ended with **CLEAN**, unambiguously, having found
nothing. The fix works.

**The two `rune:sequi` sites I handed it — both stand, and I verified both readings:**
· **`EXEC_ARENA`** (`eval.rs:85`) — `with_exec_frame` zeroes `*slot = None` across `[0,len)` on entry,
  and a live nested borrow takes an `Err(_)` arm that allocates a **fresh** `vec![None; len]` rather
  than aliasing. Nothing survives one call into another. A reused allocation, not accumulated state.
· **`KINDS`** (`eval.rs:896`) — `RETE_OPS.iter().map(…).collect()` inside `get_or_init`. Derived once
  from a compile-time constant, never mutated.

⚠ **AND IT DECLINED TO MANUFACTURE A THIRD WARD-SPLIT, WHICH IS THE RIGHT CALL.** It observed that
`EXEC_ARENA`'s category `ambient-context` *"undersells what it actually is"* — the rune's own prose
(*"so … do not allocate per token"*) describes a `performance-counter` case, and that category
requires a cited measurement this rune does not carry. `excusare` passed the same rune as HOLDS this
cast. **`sequi` did not row it**, naming it *"a labeling/taxonomy nit, not a composition break"* —
its concern is whether the chain holds, and it does. **That is a ward refusing a finding outside its
own remit**, the same discipline `conformare` showed declining a `.ok()` discard as a `solvere`
question. Recorded here as a nit, not as 2X2's sibling.

⭐ **One fact it established that is worth keeping beyond this cast:** `reachability.rs` — 2,180 lines
— is `#[cfg(test)]`-gated via an `include!` at `mod.rs:86-89`. **It never compiles into the
production binary.** I confirmed it. That is the second independent reason to leave it alone (the
first being its DISCONFIRMING-PROBE header), and it means its size does not bear on the compile
side's shipped surface at all.

⛔⛔ **`temperare` CORRECTED MY BRIEF ON A LOAD-BEARING FACT — MY FIFTH HANDED-DOWN ERROR, AND THE
FIRST THAT WOULD HAVE MIS-SIZED FINDINGS RATHER THAN MISCOUNTED THEM.**
I wrote into the brief that `matcher.rs`'s `eval_clause`/`resolve_operand` family *"IS reached at fire
time… That file is the exception to the compile-side-is-cold rule and deserves your closest
reading."* **It is not.** I verified the ward's correction myself:
· `mod.rs:56` says it outright: *"Native authority is compiled exec; `alpha_match_inner` is the
  oracle / differential."*
· `compiled_cond.rs:960-962`: *"On a real fire `match:calls` still reads zero because the round
  loop's step 1 is this function."*
· Every caller of `alpha_match_inner`/`eval_alpha_match`: `matcher.rs:327` (internal),
  `kernel/tests/alpha_discrimination.rs` ×4 (target 4), and `runtime.rs:5645` — the wat-visible
  `:wat::rete::alpha-match` primitive. **None is the automatic per-fact fire loop.**

⚠ **The cost of this error would have been silent.** The four earlier corrections were counts — wrong
numbers that a ward re-derived. This one was a *reachability* claim, and it sets the DIMENSION every
temperare finding is scored on. Had the ward trusted me, up to six rows would have shipped labelled
"n facts" when the honest label is "n oracle/differential calls" — findings inflated by orders of
magnitude, each looking urgent, none checkable without redoing the reachability work. **A wrong
number is visible; a wrong denominator is not.**

- **2M1 ★★** — CONFIRMED, and it is the arc's signature shape in its cleanest form yet. **Both files
  declare a type alias with the SAME NAME, `EqChildren`, for the same job** — a discrimination
  tree's equality fan-out. `alpha_tree.rs:56` is `FxHashMap` and carries the measurement in its own
  doc: *"SipHash of the field `Value` 40k times was the I−G walk (`DESIGN-STONE-alpha-tree-fxhash`)."*
  `where_tree.rs:84` is `std::collections::HashMap`, and `grep -n 'rustc_hash\|FxHashMap'` over that
  file returns **zero** — it never even imports the type. A fix that was profiled, named a stone, and
  shipped on one tree did not reach its twin.
  ⭐ **And the ward established reachability by citing ANOTHER WARD'S ROW**: 2T1 (`struere`) proved
  `walk`'s `proven` flag gates the pure-cmp fast path at `kernel/fire/mod.rs:2284`, which is what
  makes this a fire-dimension finding rather than a freeze-time one. Two wards composing — one
  established the path is live, the other found what is slow on it.

⭐ **It also refused to over-claim in three separate places, and each refusal is load-bearering:**
· **2M5** — it checked whether native fire runs `acc::distinct` at all, found `kernel/arm.rs:285`
  dispatches a native `AccFold::Distinct` by name, and labelled the finding **oracle-dimension**
  rather than letting an O(n²) read as a production hazard.
· **2M6** — it checked whether the adjacent `rune:exigere` already excused the concat growth and
  found it does not, saying so explicitly rather than assuming either way.
· **2M3** — it checked its own recommendation against a **builder's ruling** (`compile.wat:273-290`,
  2026-08-05: the four axes are *"NOT collapsed even where one arguably implies another"*) and drew
  the distinction that saves it: that ruling forbids *skipping an axis*, not *sharing a traversal*.
  **A ward that reads the standing rulings before proposing a change is the one whose proposals can
  be acted on.**

⚠ **And it declined to score a seventh** (`factbag.wat`'s three O(n) scans) because the only caller
it could find lives in `wat/rete/oracle/**` — **outside this target's file list** — so whether the
aggregate is O(n²) cannot be settled from files in scope. It named it for whoever scopes the oracle
files next rather than claiming it. ⛔ **That is a `peragrare` cell in waiting**: a cost whose
population lives on the other side of a target boundary.

**Rune verdict — the only `rune:temperare` in the target, and it is judged WEAK.** `purity.rs:1782`,
`simplicity-win`, reason *"cycle is a second question (#87 recursion), not a fifth fence axis."* The
category's own format rule requires citing a **cost ceiling**; this reason gives an
architectural-separation argument with no date, no measurement, no ceiling. Measured against the
model form the ward itself named on target 1 (`fire/mod.rs:494` — dated, names its floor run, carries
`206 / 245583 = 0.084%`), it does not reach the bar. ⚠ The ward asked for a better-formed rune rather
than a fix, since what it excuses is human-bounded and freeze-time — the proportionate call.

- **2E1** — CONFIRMED, and modest by design. The line promises a *"whitelist a future rung-3 seal
  will name"* and names nothing. ⚠ **I am recording one qualification the ward could not**: its
  supporting citation — a companion doc saying a sibling rung-3 item is *"neither scoped nor
  scheduled"* — is a `docs/*.md` file, and **today's ruling forbids a doc as authority**. The row
  does not need it: the comment cites no arc, and that is a fact about the comment. The doc is
  colour. ⭐ **This is the docs ruling biting one cast after it was made**, exactly as intended — the
  finding survives because its primary evidence is in the file under audit.

⭐⭐ **`exigere`'s REAL PRODUCT HERE IS THE DISMISSAL LIST, AND IT IS WHY THE ONE FINDING IS
CREDIBLE.** It ran nine distinct grep families and read **32 hits** on `defer|punt|stub|placeholder|
for now|eventually` alone — then dismissed every one with a reason and the quote. The dismissals are
the evidence that the sweep was real:
· `compiled_cond.rs:1378` — *"AFFIRMATIVELY CUT, not deferred (T7's close, 2026-08-25)"* — the
  codebase explicitly renouncing a deferral, which a naive grep reads as one.
· `purity.rs:1758` — *"no unordered value left for a future hand to forget to sort"* — **a negation**.
· `matcher.rs:230` — `cond-has-deferred-constraint?` — **domain vocabulary**, not a promise.
· `wat/rete/compile.wat:998` — *"Clara defers accumulators…"* — **a different engine**.
· `expr_ir/eval.rs:1370` and `compile.wat:598` — the falsified-belief-in-place pattern, which the
  brief warned it about and which it correctly exempted.
**Every one of those would have been a false L1 from a grep alone.**

⭐ **And it flagged its own judgment call rather than burying it.** `factbag.wat:19` sits *twelve
lines* from the finding, on the same "rung-3" concept: *"`no_raw_factbag_access.rs` is the seal
today. When rung 3 arrives, that gate is deleted — the deletion is the proof."* It read that as
**exempt** — a self-obsoleting contract stated in the present tense whose own deletion is the
objective trigger — and then said so explicitly *because* of its proximity to a genuine finding.
A ward that shows you the line it nearly flagged, and why it didn't, is one whose flags mean more.

⭐ **Both `rune:exigere` sites verified ON DISK, and I re-checked both.** `purity.rs:22`'s
`attested-arc` names arc 255 — `docs/arc/2026/06/255-builtin-registry/` exists **and** the specific
`NOTE-purity-is-definition-time-queryable-metadata.md` it cites exists inside it.
`wat/rete.wat:524`'s `scope-affirmative` names arc 278, which exists, and gives a checkable
substrate cause (the arc-277 auto-fix is bare-symbol-only and structurally cannot reach a compound
case). **The spell's hardest rule is that a rune naming a nonexistent arc FAILS; both pass.**

⚠ **Zero TODO/FIXME/XXX/HACK in target 2, re-derived twice by the ward and once by me.** It explicitly
did NOT inherit target 1's zero — *"target 1's zero did not carry over; this target's zero was earned
separately."* That is the second time this ward has refused to inherit a number, and both times it
was right to.

⭐⭐ **2N1 PROMOTES N1 FROM A SITE TO A CLASS, AND THE CLASS NOW SPANS BOTH TARGETS.** Five error
sites, three names, two files, one shape:

| name | sites | file | target |
|---|---|---|---|
| `:wat::rete::to_transient` | 3 | `kernel/session.rs:754,981,1159` | 1 (N1) |
| `:wat::rete::exec_value` | 1 | `expr_ir/eval.rs:59` | 2 |
| `:wat::rete::apply_op` | 1 | `expr_ir/eval.rs:905` | 2 |

**Every one is the Rust function's own name, verbatim, a few lines from its own error site** — I
confirmed `fn exec_value` at `:69`, `fn apply_op` at `:890`, `fn to_transient` at
`kernel/session.rs:1142`. Every one is snake_case inside a namespace where **every** resolving name
is kebab-case. Every one resolves nowhere — one repo-wide hit each.

⛔ **This is the extirpare ladder's bottom rung, exactly.** The convention *"an error head names a
wat form"* is held by nothing: no gate, no type, no check. When an author needs a `head` and the
nearest name to hand is the function they are standing in, the wrong thing is the easiest thing to
write. Three separate authors, two files, five sites — that is not carelessness, it is **a shape the
substrate makes writeable**.

⭐ **And the cure is already in the same file as two of the sites.** `expr_ir/eval.rs` uses a real
FQDN where one applies (`:772`, `":wat::rete::core::match"`) and a **plain, non-namespaced label**
where the failure is internal (`:1295`, `"compiled-exec"` — *"compiled apply cannot dispatch kind…"*).
The file already knows both correct answers; two sites invented a third. **A gate asserting that any
`head:` string beginning `:wat::rete::` resolves to an authority would make this class
unwriteable** — and `rete_names_in_wat_scripts_resolve.rs` already contains the resolver to do it.

⛔⛔ **AND CERNERE CORRECTED MY COUNT — MY SIXTH, AND THE SECOND WITH THE IDENTICAL ROOT CAUSE.**
I handed it *"`RETE_OPS`, 108 rows"*. The real count is **79**.
· My grep was `grep -c 'rete_name'` — which matched the module doc's **prose about** `rete_name`
  (`:69`, `:83`, `:102`), the **struct field declaration** (`:278`), and inline comments (`:567`,
  `:904`) alongside the actual rows. 29 of my 108 were the identifier being *discussed*, not *used*.
· The ward's `grep -c 'rete_name: ":wat::rete::'` returns 79, and — the anchor that settles it —
  `tests/lint/rete_names_in_wat_scripts_resolve.rs:709` carries its own measured comment:
  *"Measured 2026-09-01: 79 `RETE_OPS` rows, 328 attested names."*
⚠ **This is the SAME failure as my `#[allow]` miscount** (`excusare` found 4 real attributes where I
counted 5, because my pattern matched an allow **named inside a doc comment** explaining what they
deliberately did *not* do). **In a codebase this comment-dense, a grep for an identifier matches the
prose about it as readily as its uses — and here the prose outnumbered the uses 29 to nothing.**
Six handed-down errors, six caught by wards that re-derived; two of the six share this one cause.

⭐ **The registry itself is clean, and that is a real result.** The ward extracted all 79
`(rete_name, core_name)` pairs and confirmed every distinct `core_name` — 67 of them — is
independently attested **outside** `vocabulary.rs`. The question this target uniquely allowed
(*"does the table everything trusts contain a phantom row?"*) is answered: **no.**

⭐⭐ **2R1/2R2 MAKE THE FALSE-CALLER-COUNT A CLASS TOO — the second class this cast has promoted.**
`probare` found `outcome.rs`'s *"three callers … at `fire/rules.rs:425`"* on target 1 (rowed R1;
the citation pointed at a data literal and there were four callers). Here it found the same shape
twice more:

| claim | site | claimed | actual | gated? |
|---|---|---|---|---|
| `fire_fixpoint_delta_armed` callers | `kernel/outcome.rs:22-25` | 3 (+ a wrong `file:line`) | 4 | no |
| `enum_variant_ctor` callers | `matcher.rs:114-131` | 3 | **4** | no |
| `reorder_kwargs_by_field_name` callers | `validate/mod.rs:66` | 2 | **3** | no |

**Three caller-count claims across two targets; all three wrong; none gated.** ⛔ And the cure exists
and is already in use: `rete_header_claims_are_asserted.rs` carries an arm named
`the_termination_verifier_still_has_exactly_one_call_site` — **a caller-count claim, mechanically
held.** The mechanism is built, proven, and applied to exactly one claim. Three others sit outside
it. That is the arc's signature shape for the *thirteenth* time, and here it is unusually cheap to
close: the gate's existing arm is a template.

- **2R1** — CONFIRMED, and unambiguous. The doc says "three" three separate times and the fourth
  caller (`validate/typing.rs:335`) is named in none of them. ⭐ **The ward's proposed cause is worth
  keeping**: `partire`'s 2026-08-30 split of `validate.rs` into `mod`/`typing`/`error` created a
  caller in the new file, and the doc lives in `matcher.rs` — **a different file entirely**. A
  refactor verified per-file cannot see a count that lives elsewhere.
  `[[an-item-level-move-drops-what-is-not-an-item]]`.
- **2R2** — CONFIRMED on the count, **and I have marked it weaker than 2R1 deliberately.** The doc
  does mention `check.rs`'s `infer_kwargs_construct_check` — but casts it as *enforcement backing the
  runtime caller*, not as a third caller of this helper. The ward read `check.rs:13095-13118` and
  found a direct, independent call. The number is wrong; the taxonomy is arguable. Same treatment I
  gave R1 on target 1, where the citation was wrong and the count defensible — **the two halves of a
  claim rot separately and must be graded separately.**

⭐⭐ **AND THE JUDGMENT I REFUSED TO PRE-DECIDE CAME BACK ANSWERED, WITH COUNTS.** `reachability.rs`
is 2,180 lines, `#[cfg(test)]`-gated, and declares itself a DISCONFIRMING PROBE. I put the question
to the ward without a hint. **Verdict: substance, not description** — and I re-derived every number:
**20 `#[test]` fns, 54 assertions, 42 function definitions** across 1,186 code lines. The assertion
it cited as non-vacuous is real (`:505` — *"the two call sites must render DIFFERENT programs"*,
an `assert_ne!` between two independently synthesized programs, not a liveness check).
Its reasoning is the part that makes the verdict usable: the 855 comment lines are **annotation on a
working calibration harness**, each `⛔` block sitting beside the code that closes the risk it names
— *"an instrument that has not reproduced a known answer is not an instrument"* at `:26-34`,
immediately followed by the four calibration cells that pin exactly that. **A ward that answers an
open question with counts rather than an impression is one whose answer can be re-checked.**

⭐ It also honoured the ratio correction exactly as on target 1: it ran the table over all 25 files,
**marked every exemption rather than dropping the row**, and flagged the single row whose raw number
would mislead (`mod.rs` at 0.29:1 — a module index where the count of `pub(crate) mod` lines is the
right denominator). **Zero described-or-hollow forms** anywhere in the target, and it caught that the
only `stub` hits are a *filename citation* (`BRIEF-the-f64-surface-is-a-stub.md`) and prose about a
different concept — the prose-vs-thing trap that has now bitten my own greps twice.

- **2V1 ★★** — CONFIRMED, all three, and the `compiled_cond.rs` one is the sharpest instance of
  "reuse before inventing" this vigilia has produced. `:791` reads `-> crate::rete::expr_ir::SlotNames`
  and `:792` reads `let mut out: Vec<Option<Arc<str>>>`. **The alias is not a sibling in another
  module — it is the type the function is declared to return**, one line above, already imported.
  `alpha_tree.rs` is the same shape with 170 lines of distance: `type AlphaWildcard` at `:58`, used
  twice, and `:228` writing it out.

⭐⭐ **AND THE EqBuckets SECOND READING STRENGTHENS 2M1 — from a different ward, on a site it was only
invited to comment on.** `temperare` rowed `EqChildren` because `alpha_tree.rs` uses `FxHashMap`
(with a measured stone) and `where_tree.rs` uses `std` `HashMap`. `perspicere` looked at the same twin
files and found a **second** identically-named alias: `type EqBuckets = HashMap<Value, Vec<i64>>` at
`alpha_tree.rs:53` **and** `where_tree.rs:41` — byte-identical, `std` on both sides, **no divergence.**
⛔ **That is evidence about 2M1's nature, not just a DRY note.** The two files declare two aliases
with the same two names; one pair diverged and one did not. If the twins were deliberately different
engines you would expect divergence in both. **The `EqBuckets` match is the control that makes the
`EqChildren` mismatch read as an accident** — a fix applied to one file and not carried to the other —
rather than a considered choice. `perspicere` was invited to give a second reading and gave one that
changes how the first row should be read.

⭐⭐ **IT CONFIRMED MY MEASUREMENT EXACTLY — AND THEN FOUND WHAT I HAD MISSED.** *"Same 121 raw hits
across the same 16 files"*; my disclosed 18 comment-line hits confirmed at ~19. **This is the first
handed-down measurement of this cast to survive re-derivation on every figure** — and it survived
because I disclosed the pattern's contamination alongside the number instead of just the number.
But the ward did not stop at agreeing:
· **A fourth false-positive category I never named** — `Option<Fact<'_>>`, `Vec<WrapperBind<'a>>`,
  `Option<AlphaPattern<'_>>`: the second `<` is a **lifetime**, not a hidden noun. The noun is already
  at the surface. It judged these not-flagged, correctly.
· ⛔ **And it demolished my framing of `reachability.rs`.** I called its 27 hits *"the largest bucket
  AND the least consequential."* Wrong on the second half in a way that matters: the ward read all 27
  and found them **100% wat-DSL text inside Rust string literals** — `<-` bind arrows and `<12`/`<18`
  format specifiers. **Not one real Rust generic.** It is not a low-value bucket; it is an **empty**
  one. A ward that reads what a count contains, rather than weighing the count, is the only kind that
  can tell those apart.

⭐ **The runes: 3 of 3 clear, and the two `read-once` claims were CHECKED, not assumed.** Both
`purity.rs:2587` and `:2593` assert their type appears exactly once; the ward grepped each exact type
string whole-repo and got **one hit each**. Same discipline it applied to target 1's seven, same
outcome. ⚠ It also noted both sit inside a `#[cfg(test)]` block, so the stakes were low either way —
saying so rather than inflating the result.

⭐ **And it declined 20 sites in one group with a stated bar.** The `Result<_, EvalBreak>` idiom is
attested 35× in `export.rs` alone; minting per-`T` aliases is precisely the "dozen aliases nobody
reuses" its own rule 4 forbids. It flagged the *one* thing that would help — a generic
`type EvalResult<T> = Result<T, EvalBreak>` covering all 35 at once — and then explicitly refused to
recommend it, calling it a crate-wide convention change rather than a per-site mint. **Naming the
larger fix and declining to smuggle it in under a narrower ward is the restraint that keeps a cast's
recommendations actionable.**

⭐⭐ **`experiri` IS THE ONLY WARD OF THIS VIGILIA THAT EXECUTED, AND ITS RESULT SPLITS CLEANLY IN
TWO: what it drove is spotless, and what nobody drives is the finding.**

**Driven, and clean — 163 cells, zero cell-level findings:**
· 79 rows × {`InlineConstraint`, `WhereFence`} = 158 cells, **every one fires and discriminates**;
  both of the ledger's exclusion lists (`NOT_YET_GENERABLE`, `COMPILED_EXECUTOR_CANNOT_RUN`) are
  **empty**. · `AccHeadFold`: 1 eligible row, fires. · `:then`: 4 rows sampled fresh, all fire and
  agree with their other-position verdicts. **Zero asymmetric declarations.** For the ward whose
  entire reason to exist is finding a declared surface that cannot be driven, that is a strong
  statement about `RETE_OPS`.

**The finding is the column nobody sweeps** — and the ward earned it by refusing an exemption.
`experiri`'s rune rule says a `position-not-modelled` reason must **dispose**, not defer, and that
this category **exempts no cell**. The ledger's stated reason — *"an un-calibrated position would
add a column of findings nobody can trust"* — is a good reason to have waited and **not** a
disposal, because the calibration it says is missing **exists**: I read `README.md:19` (*"Saved as
`.rs.txt` so no tooling mistakes it for a live module"*) and `:132` (*"`positions-3-4.rs.txt`
asserts nothing"*). A built, calibrated, twice-productive harness, deliberately shelved so tooling
cannot load it, and never converted to an assertion. ⛔ **The reason defers to work already done.**

⚠ **AND I VERIFIED THE THING I HAVE A MEMORY ABOUT.** The ward left **nine `.wat` files** in
`wat-scripts/scratch-pad/experiri-then/` — a tree two lint gates walk on every floor run. It said it
re-ran both. **I ran them myself** rather than take that:
`Summary [ 136.552s] 2 tests run: 2 passed, 5497 skipped`, zero `FAIL` anywhere in the log, both
gates green by name. `[[a-file-landing-in-a-gated-tree-needs-that-gate-run]]` — I have pushed a red
floor from a "docs" commit that added one `.wat`; this time the gate ran before the commit.
**The files are committed, not left untracked**: they are the reproduction evidence for 2Y1, they
pass the gates, and an untracked `.wat` under a walked tree makes my floor and a fresh clone's
disagree.

⚠ **The header's row count is stale, and the ward reported the delta without re-filing it.**
`reachability.rs:5` says *"a 74-row x N-kind ledger"*; `RETE_OPS` measures **79** — the ward derived
it three independent ways (`rete_name:` rows, `ReteOp {` openings, and `operands_for`'s own
`27 special + 52 uniform = 79`), matching the lint gate's own measured comment. **It correctly
classified this as the same stale-count class already rowed as 2I1 rather than claiming a new
finding.** That is the prior-art discipline working at the end of a long cast.

⭐ **One curiosity it recorded without inflating.** Law A (a `:then` admits only `:wat::rete::` ops)
holds — but `:wat::core::>` is refused by a **totality** check while `:wat::core::not` is refused by
the **rete-primitive** check. Two controls, two different refusal mechanisms, both correct. The ward
called it *"a curiosity, not a bypass"* and moved on. **A ward that can tell an interesting fact from
a finding is one whose findings are worth reading.**

- **2W1 ★★★** — CONFIRMED on every coordinate, and it is **the strongest single finding of target 2.**
  The claim at `:14-19` is verbatim as quoted. `LowerCx` carries exactly four fields — `sym`,
  `slots`, `next`, `hof_fn_pos` — **and no depth.** `grep -cE 'depth|recursion_limit|stacker'` over
  `expr_ir/mod.rs` and `expr_ir/eval.rs` returns **0 and 0**. `src/parser.rs`, which feeds them,
  returns **0**.
  ⛔⛔ **AND THE SIBLING CURE IS A MODEL ONE, WHICH IS WHAT MAKES THIS THE ARC'S SIGNATURE SHAPE
  AGAIN — the fourteenth time.** `export.rs:350-370` is not a guessed constant; it is the standard
  this codebase sets for a measured bound, and it says so: *"**MEASURED, not chosen for
  roundness.**"* It records the defect it answers (*"`import` had no depth criterion at all"*), the
  driving that found it (*"the same 20,000-deep Export was ACCEPTED on a 256 MiB thread and killed a
  2 MiB one with `fatal runtime error: stack overflow, aborting` — an abort, not a panic, so nothing
  catches it. Acceptance was a property of the importing THREAD."*), **both numbers behind the
  choice** (3, the measured corpus maximum, instrumented; 3,000–5,000, the observed abort window),
  the arithmetic (*"300 is 3 × 100 headroom … and one tenth of the low end"*), and the rule for
  changing it (*"Raise it only with a new measurement"*).
  **A defect found, driven, measured to two numbers, cured with documented headroom, and shipped on
  `import` — and the sibling recursive descent that EVERY rete expression compiles and runs through
  never received it.** The header of that very file calls a compile-time refusal landing at fire time
  *"a defect in this file"*; an uncatchable process abort is worse than either outcome it names.

⭐⭐ **AND THE WARD CHECKED THE TARGET-1 SHAPE AND REPORTED THAT IT DID *NOT* RECUR — which is the
harder, more useful answer.** I pointed it at the `MAX_IMPORT_NODES` surface because target 1's
sharpest L1 was a qualification that existed in one file and **never travelled** to the sites that
shipped the claim. It checked exactly that here and found **the discipline HELD**: the quadratic-build
justification is stated consistently at three sites inside `export.rs` — module header, the constants'
own docs, and the runtime refusal — and the node check runs *before* any unpack, so the refusal costs
nothing. **It then found the real gap adjacent to it** (2W2: the qualification never reached the wat
*spec*). A ward that can say "the shape you sent me after is not here, and here is what is" is worth
more than one that finds what it was pointed at.

⭐⭐ **IT ALSO RESOLVED MY WEAK MEASUREMENT — IN THE DIRECTION OF NO FINDING.** I handed it a
gate-coverage count with its weakness disclosed: eleven of the 25 files are *named* by no gate under
`tests/lint/`, **but being named is not being covered**, and I said so and told it the check was the
finding either way. It did the check — reading each gate's `collect_rs`/`root()` rather than its
names — and found **13 broad gates recurse `src/rete` or `["src","tests","crates"]` wholesale**, so
every one of the 25 files is swept. Its verdict: *"the measurement's own warning was correct to raise
the question; the answer is 'covered,' not 'gap.'"*
⛔ **That is the disclosure practice paying in the OTHER direction.** Six of my handed-down numbers
were wrong and wards caught them. This one was *incomplete rather than wrong*, and disclosing how it
was incomplete is what stopped it becoming a seventh error — a false negative-space finding against
eleven files that are, in fact, covered.

⚠ **Zero `rune:circumspicere` in target 2, re-derived.** Target 1 had exactly one, adjudicated valid.
So **no surface on the compile side has ever been declared an accepted-by-design edge** — which is
not the same as there being none, and the ward said so rather than reading the zero as health.

---

# TARGET 3 — `wat-scripts/perf/grid/` (148 files, 16,616 lines — re-derived 2026-09-08 by `find wat-scripts/perf/grid -type f`)

Muster derived in `README.md` with measured triggers. Returns land verbatim in `reports-target3/`.

| id | ward | site | finding | sev | status | re-derivation |
|---|---|---|---|---|---|---|
| **3P1 ★★★** | peragrare | the grid's 5-axis grid; census at `wat-scripts/perf/grid/peragrare-census.sh` | ⭐⭐⭐ **THE CORPUS PROVES EVERY MECHANISM ALONE AND NO TWO TOGETHER.** All three founding defects that birthed this ward are now closed **as isolated axes** — `userfn-head`, `retract-multiplicity`, `accum-over-derived` each have a fixture and a mutation proof. But **no fixture combines any two of them.** 5 of 108 cells are empty *and* carry a live compound hypothesis: a user-fn head whose LHS accumulates over a type the same ruleset derives; a duplicate-retract feeding a **leading** accumulate; a leading accumulate whose `:from` is itself derived; a positive consumer downstream of a **leading** gate; a duplicate-retract of the accumulate's own source. **Each cure was proven only where the other pressure is absent.** | **L1** ×5 | **OPEN** · ✅ I VERIFIED the census, its anchor and its arithmetic | `bash peragrare-census.sh` → 108 cells, 9 visited, 99 empty, 16 members; `--pins` → all three PASS, moving pin **8 of 8** with the spell's own denominator `(2-1)+(2-1)+(3-1)+(3-1)+(3-1)`; `--verify` → *"all 16 fixtures' mechanical facts agree with the table"*. Closed cell by cell, each with a fixture |
| **3S1 ★★** | solvere | `run-axis.sh:277-280` vs `check-grid-three-way.sh:233-241` | ⭐⭐ **ONE DECODE, TWO COPIES, AND THEY HAVE ALREADY DIVERGED.** Both extract fields from the `#grid/Result` wire line. `check-grid-three-way.sh` uses a parameterised `extract()` carrying a **`(?<=[ {])` lookbehind**; `run-axis.sh` hand-writes four inline `grep -oP` calls **without it**. The guarded copy's own comment names the hazard it defends: *"`:oracle-derived` does NOT contain `:derived` … but a future `:spec-derived` would, and the match count below is what refuses an ambiguous line."* **The drift is not hypothetical — it is present.** | L2 · incidental but live | **OPEN** · ✅ I VERIFIED | `sed -n '277,280p' run-axis.sh` → no lookbehind; `:235` of the sibling → `(?<=[ {])`. Closed by one shared extractor carrying the guard |
| **3S2 ★** | solvere | `check-spec-native.sh:28-35` + `check-query-compat.sh:50-55` | ⭐ **`rewrite_to_spec()` DUPLICATED VERBATIM — INCLUDING ITS COMMENT, WORD FOR WORD.** Both run the identical `perl -pe 's/:wat::rete::fire-rules(?!\$oracle)(?!-)/…/g'` and carry the same two-line justification; only local-var naming differs. ⛔ **This encodes domain logic, and the class has already bitten**: `run-axis.sh:179-180` records *"The 2026-08-20 skip was a no-op because it still matched `fire-rules-spec` after the `$oracle` rename."* A verb-naming convention encoded in two places, after it silently broke once when encoded in one. | L2 · incidental, higher-risk | **OPEN** · ✅ I VERIFIED | both bodies read; comments byte-identical; `sed -n '179,180p' run-axis.sh` → the prior incident |
| **3S3** | solvere | 11 `gen-*.sh`, two variants | The Clara **harness** — session build, JIT warmup, timing block, `#grid/Result` print — is byte-identical across 8 generators (simple timing) and again across 3 (extended, adding `:insert-ns`/`:fire-ns`/`:query-ns`/`:protocol-ns`), with **no shared source**: `grep -ho 'source …' gen-*.sh` → empty. ⚠ The per-axis *workload* is genuinely eleven different programs and is **not** the finding; the harness around it is. | L2 · incidental | **OPEN** · ⚠ ward-reported | closed by one sourced `gen-lib.sh` carrying the harness |
| **3S4** | solvere | `compare-grids.sh:39-60`; `check-grid-speed.sh:57-60` | `#grid/Verdict` field extraction done twice in **two different techniques** — awk `match`/`substr` in one, four `sed -E` one-liners in the other — over the same line shape, with no field unique to either consumer. | L2 · incidental | **OPEN** · ⚠ ward-reported | one shared verdict-reader |
| **3S5** | solvere | `check-where-shapes.sh:68-84`; `check-query-compat.sh:25-44`; `check-grid-three-way.sh:98-112` | JDK discovery (`PATH → JAVA_HOME → $HOME/opt/jdk-*`) triplicated; the two function forms differ only in whitespace and statement splitting — **the signature of independent retyping** — and the convention is documented twice in near-identical header lines. | L2 · incidental | **OPEN** · ⚠ ward-reported | one sourced `find_java` |
| **3S6** | solvere | `check-spec-native.sh:15-23`; `check-query-compat.sh:15-23`; `check-grid-three-way.sh:60-68`; `check-where-shapes.sh:55-66`; `run-axis.sh:45-62` | `WAT_BIN`/`GRID_DIR`/`REPO_ROOT` discovery-and-validation repeated **5×**, byte-identical but for the echoed script name; its rationale comment duplicated in long and short forms. ⚠ **Only `run-axis.sh` carries the freshness wall** — the other four read the same binary for the same kind of measurement with no such protection, which is itself evidence the copies are not kept in sync. | L2 · incidental | **OPEN** · ⚠ ward-reported | extract the 5-line core; leave `run-axis.sh`'s freshness wall local |
| **3C1 ★★** | conferre | contract: `CLARA-TRANSLATIONS.md` (422 lines) · code: `where-or-conditions.clj:2,45` + 15 sibling twins | ⭐⭐ **THE CONTRACT NEVER MENTIONS `:or` — AND 16 OF 43 TWINS CARRY A DEFENSIVE COLLAPSE FOR IT.** Clara's `:or` compiles to independent activation paths, so a rule fires its RHS **once per matching disjunct**; `where-or-conditions.clj:2` says so outright — *"Clara insert!s twice when both arms match"* — and compensates with `(count (set …))`. **16 twins carry that collapse; their `.wat` partners use a raw `length`.** The contract's stated job is *"any semantic caveat that could make an accuracy/speed differential misleading"* — and ⛔ **I verified the word `:or` does not appear in it at all.** | L2 (ward: Medium) | **OPEN** · ✅ I VERIFIED | `grep -n ':or\b' CLARA-TRANSLATIONS.md` → **empty**; `grep -lE '\(count \(set ' *.clj \| wc -l` → **16**. Closed by an `:or` entry in the contract |
| **3C2** | conferre | `CLARA-TRANSLATIONS.md:363` | ⚠ **A DANGLING SELF-CITATION THAT IS THE SOLE JUSTIFICATION FOR A DECISION.** The line strikes A10's original "no twin needed" reasoning on the authority of *"Rule 4 of this document (mirror the OPERATION, not the vocabulary)"*. **There is no rule list in this document** — the ward read all 422 lines and all three commits of its history; I re-checked the current file: `Rule [0-9]` occurs **exactly once**, at `:363`, the citation itself. | L2 · aside | **OPEN** · ✅ I VERIFIED | `grep -coE 'Rule [0-9]' CLARA-TRANSLATIONS.md` → **1**, which is the citation. ⚠ **Filed as an observation, not a conferre finding** — the ward noted it has only ONE coordinate, and its spell requires two. Closed by writing the rule, or citing what actually holds |
| **3G1 ★** | purgare | `run-all.sh:139` | ⭐ **A DIAGNOSTIC THAT PROMISES AN EXIT CODE AND PRINTS A CONSTANT.** `if ! bash run-axis.sh …; then echo "… FAILED (rc=$?)"` — inside the `then` branch of a **negated** test, bash has already collapsed `$?` to `0`. The message names the failing axis's exit code and can only ever print `rc=0`. ⚠ **The propagation itself is correct** — `rc=1` and the final `exit $rc` are unaffected; only the printed number is dead. ⛔ Note the interaction: `mora` cited this exact line as evidence the sweep *"fails loud"*. **It does — the failure is loud and the number in it is meaningless.** | L2 | **OPEN** · ✅ I VERIFIED by mutation | `f() { return 3; }; if ! f; then echo $?; fi` → **0**; bare `f` → **3**. Closed by capturing `$?` in the `else` arm |

| **3I1 ★★** | intueri | `run-all.sh:46` vs `neg-consumer.wat:38-43` | ⭐⭐ **A COMMENT THAT LICENSES DISMISSING THE EXACT SIGNAL ITS AXIS EXISTS TO RAISE.** `run-all.sh:46` ends the `neg-consumer` dial entry with *"RED until task #94 is closed."* The axis's own header says the opposite in terms: *"THIS AXIS FOUND AND THEN CLOSED task #94 … Fixed in `ff581b6f` … It must now read `:accuracy :match` on ALL THREE columns; **any MISMATCH is that regression returning**."* ⛔ **The two files disagree about what a MISMATCH MEANS** — the driver says *expected*, the axis says *regression*. This is not a stale date; it is a standing pre-blessing of a red, in a repo whose `CLAUDE.md` bans the category by name (*"⛔ THERE IS NO SUCH THING AS A KNOWN FLAKE. A RED IS A RED"*). | **L1** (⚠ `exigere`: **dismissed**) | **OPEN** · ✅ I VERIFIED · ⚠⚠ **3X1 — TWO WARDS, ONE SITE, OPPOSITE DISPOSITIONS** | ⚠⚠ **`exigere` REACHED THIS EXACT LINE AND DISMISSED IT** (`reports-target3/exigere.md:38`, item 14). It ran the *same* cross-check against `neg-consumer.wat:38`, reached the *same* fact — *"**Task #94 is closed**"* — and disposed of it as *"retrospective narration … **Dismissed as historical context.**"* `intueri` reached the same line and returned **L1**. Same research, opposite verdicts. ⛔ **I may not re-classify either** (`vigilia` forbids the aggregator that), so both stand and this is the THIRD decision the builder owes, with **X3** and **2X2**. ⭐ **BUT MY OWN READ IS A THIRD FRAMING, AND IT IS THE OPERATIVE ONE — NEITHER WARD ASKED THIS QUESTION.** `exigere` asked *is #94 closed?* (yes → history). `intueri` asked *is the comment stale?* (yes → L1). **Neither asked what the comment tells a reader to DO when the axis mismatches.** And *"RED **until** task #94 is closed"* is not retrospective narration — it is **present-tense and forward-looking**, a standing claim that the axis is red NOW and will remain so UNTIL a condition is met. A reader who hits a `neg-consumer` MISMATCH is told by the driver to expect it and by the axis that it is a regression returning. **The grammar is the evidence, and it is checkable independently of either ward's lens.** | `sed -n 46p run-all.sh` → the phrase; `sed -n '38,43p' neg-consumer.wat` → the contradiction; `git log -1 ff581b6f` → **2026-08-13 12:20:31**, *"#94 CLOSED"*; `git log -S'RED until task #94' -- run-all.sh` → written at `839d02a30`, **2026-08-13 11:16:51 — 63 minutes BEFORE the fix landed.** Closed by deleting the clause or replacing it with the axis's own reading |
| **3I2 ★** | intueri | `CLARA-TRANSLATIONS.md:418` + the table at `:405-415` | ⭐ **THE CONTRACT'S "SUMMARY FOR THE ORCHESTRATOR" OMITS THE AXIS WITH THE FULLEST ENTRY.** The closing sentence claims Clara's source *"fully grounded all six forms above."* `grep -n '^## A'` returns **8** axis sections (A2, A3, A5, **A9**, A6, A7, A8, A10) and the summary table immediately above the sentence has **7** rows — **A9 is in neither the count nor the table**, despite holding the document's most detailed section (`:160-201`, a full worked bug story). ⚠ The table's own heading is *"Summary for the orchestrator"*: the axis with the richest caveat is invisible to precisely the reader the table names. Companion to **3C1** — same document, same failure mode, different omission. | L2 | **OPEN** · ✅ I VERIFIED | `grep -n '^## A' CLARA-TRANSLATIONS.md` → 8; table rows at `:407-415` → 7, no A9; `sed -n '417,419p'` → *"all six forms"*. Closed by an A9 row + a count that is not hand-maintained |
| **3I3** | intueri | `REMAINING-CLARA-MOUTHS.md` (title vs `:10,22,34,47,57,69,78,88`) | **A FILE TITLED "REMAINING" WHOSE ENTIRE CONTENT IS A CLOSURE RECORD.** All 7 numbered items read `— DONE (where-*)`; the closing header at `:88` is literally `## This list is empty.` ⚠⚠ **TWO CORRECTIONS TO THE WARD, BOTH MINE.** (1) ⛔ **Its cited support is FALSE** — it claimed `check-where-shapes.sh:52` and `check-spec-native.sh` point at this file "as a locked reference"; `:52` actually points at `check-spec-native.sh`, and `grep -rn REMAINING-CLARA-MOUTHS wat-scripts/` returns **nothing**. Nothing in the corpus names it. (2) ⭐ **But the real evidence is stronger than the one it reached for:** the live breadcrumb carries a standing countermeasure at `CURRENT-STATE-annihilate-interpretation.md:1250` — *"expressivity is closed; **do not reopen it as 'the next mouth'**"*. **A prior self had to write a permanent warning into the recovery path against this filename's own promise.** That is the cost, already paid. | L2 | **OPEN** · ✅ I VERIFIED, and strengthened | `grep -c '— DONE' REMAINING-CLARA-MOUTHS.md` → 7; `sed -n 88p` → `## This list is empty.`; `sed -n 52p check-where-shapes.sh` → names `check-spec-native.sh`, not this file. ⚠ **`exigere` reached this same file and DISMISSED it** (`reports-target3/exigere.md:42`, item 16) — **no conflict**: a closure record is correctly *not* a deferral, and the name is still a broken promise. Two lenses, one site, both right |
| **3I4** | intueri | `peragrare-census.sh:32` — ⛔ **MY OWN FILE** | **A PLACEHOLDER DATE I NEVER RESOLVED.** The line reads *"the `userfn-head.wat` cure, closed **2026-09-0x**"*. `userfn-head.wat` landed at `caeef4793`, **2026-09-07** — one `git log` away, and the same file writes *2026-09-07* precisely elsewhere. ⚠ Recorded because the census is **my** contribution to this corpus, committed mid-vigilia, and the brief told the ward to audit it with no special standing. It did. | L2 | **OPEN** · ✅ I VERIFIED | `sed -n 32p peragrare-census.sh` → `2026-09-0x`; `git log --format='%h %ad' --date=short -- userfn-head.wat \| tail -1` → `caeef4793 2026-09-07`. Closed by writing the date |

| **3T1 ★★** | struere | `run-axis.sh:220-222,283,305,311,382` → `run-all.sh:137-140` | ⭐⭐ **THE HEADLINE SIGNAL NEVER REACHES THE EXIT CODE.** `ACCURACY`/`ORACLE_ACCURACY`/`PORT_ACCURACY` init to `:match` and are set to `:MISMATCH` on divergence — each branch only `echo`s to stderr. **Not one of `run-axis.sh`'s nine `exit` statements is reachable from a MISMATCH branch** (they are all setup/crash paths); the loop's last statement is the successful `#grid/Verdict` echo, so the script exits 0 on a wrong answer. `run-all.sh`'s only per-axis failure signal is that exit status. ⚠⚠ **SEVERITY CORRECTED BY ME, L1 → L2, AND THE WARD CITED THE REFUTATION ITSELF WITHOUT SEEING IT.** It called `check-grid-speed.sh` a *contrast*; that file is `run-all.sh`'s **CI caller** (`:51`, job at `ci.yml:199`), and it parses the Verdict lines for `:accuracy != match` → `fail=1` → `exit 1` (`:63-68`, `:93`), with a non-vacuity floor at `:86-90`. **That is the ward's own proposed cure #2, already implemented.** So the signal is gated; what is true is narrower: **the safety lives in each caller remembering to parse stdout**, and `run-all.sh` — which its own header advertises as the sweep entry point — does not. ⛔ Interaction: **3G1** recorded *"the propagation itself is correct — `rc=1` and the final `exit $rc` are unaffected"* and **`mora` cited `run-all.sh:139` as evidence the sweep "fails loud"**. Both stand, and both now carry a qualifier neither knew: that channel can fire on a crash and **never on a wrong answer**. | L2 (ward: L1) | **OPEN** · ✅ I VERIFIED, severity corrected | `grep -n '\bexit\b' run-axis.sh` → 9 hits, none in a MISMATCH branch; `sed -n '137,143p' run-all.sh` → exit-status only; `sed -n '51p;63,68p;86,93p' check-grid-speed.sh` → the stdout gate + the vacuity floor; `grep -n 'check-grid-speed' .github/workflows/ci.yml` → `:199`. Closed by `run-axis.sh` exiting nonzero on any non-`:match`, which costs nothing and removes the per-caller obligation |
| **3T2 ★** | struere | `run-all.sh:23` vs the whole file | ⭐ **A HEADER PROMISING OUTPUT THE SCRIPT NEVER PRODUCES.** `:23` — *"Emits every `#grid/Verdict` line from every axis, **then a summary tally on stderr**."* **There is no tally anywhere in the file.** A grep for `tally\|summary\|total\|count\|seen` over all 143 lines returns exactly two hits: the promise itself at `:23`, and the word *count* inside an unrelated `fanout` dial comment at `:56`. Nothing counts `:match` vs `:MISMATCH`, nothing counts axes attempted vs completed. ⚠ Note the pairing with **3T1**: the one output that would have made a bare `run-all.sh` legible to a human — a tally naming how many cells mismatched — is the output the header promises and the body omits. | **L1** | **OPEN** · ✅ I VERIFIED | `grep -n 'tally\|summary\|total\|count\|seen' run-all.sh` → `:23` (the promise) and `:56` (unrelated). Closed by writing the tally — the Verdict lines are already in hand in the same loop — or by deleting the clause |
| **3T3** | struere | 19 of 38 `where-*.wat`; worst `where-exists.wat` (18×), `where-or-conditions.wat` (14×); cure already present at `where-shapes.wat:248-268` | **THE CORPUS ALREADY CONTAINS THE EXTRACTION AND 19 FILES DO NOT USE IT.** Each scenario hand-threads `compile-all` → `insert` → `fire-rules` with its own full `match` arms for `MayNotTerminate`, `MemoryCeilingExceeded`, `RoundCapExceeded`. `where-shapes.wat:248` defines `:wsh::run-row [row] -> String` doing that pipeline **once**, and `:user::main` at `:263-266` just `foldl`s it. A change to the compile/fire contract means editing up to 18 sites in one file. ⭐ **The ward disclosed this as a MEASUREMENT rather than a citation, and named the token it counted** — so I could re-derive it, and it lands exactly: 38 files, 19 with >1, distribution `19×1, 3×2, 1×3, 1×4, 4×5, 4×6, 4×8, 1×14, 1×18`. **That disclosure is the practice this vigilia has been paying to learn.** | L2 | **OPEN** · ⚠ ward-reported (site read by ward; the COUNT re-derived by me) | `grep -c 'CompileOutcome::Compiled' where-*.wat` → distribution above, identical; `sed -n '248,250p;263,266p' where-shapes.wat` → the helper and its `foldl` both present. Closed file by file, starting with the two worst |
| **3T4** | struere | `peragrare-census.sh:171,191,205,212` vs `:133,:241` — ⛔ **MY OWN FILE** | **THE SAME SIX VARIABLE NAMES ARE `local` IN TWO FUNCTIONS AND LEAKED IN TWO OTHERS, IN ONE FILE.** `do_verify` (`local fail=0`) and `cell_count` (`local qH..qN n=0`) both then run `while read -r name H R A L N` **without declaring those six local** — so they become script globals. `verify_fixture:133` and `test_move:241` declare the identical names `local`. The discipline is known in the file and applied inconsistently. Harmless today only because the `case` dispatch runs exactly one entry point per invocation. | L2 | **OPEN** · ✅ I VERIFIED | `sed -n '168,171p;189,191p;132,133p;239,241p' peragrare-census.sh` → two declare, two do not, same names. Closed by adding the `local` lines |
| **3T5** | struere | `peragrare-census.sh:239-246` — ⛔ **MY OWN FILE** | **A SELF-TEST THAT MUTATES THE FILE'S DECLARED SOURCE OF TRUTH AND PUTS IT BACK.** `TABLE` (`:108`) is documented as the hand-derived, cited judgment the whole census rests on. `test_move` does `TABLE_SAVE="$TABLE"` → append a synthetic row → `cell_count` → restore. `cell_count` is honest (it only reads `$TABLE`), so the test reaches *around* an interface that is already clean. ⭐ **One beat further than the ward went: `TABLE_SAVE` is not `local` either** — the save/restore dance leaks its own scratch variable, which is 3T4's defect appearing inside 3T5's cure. | L2 | **OPEN** · ✅ I VERIFIED, and extended | `sed -n '239,248p' peragrare-census.sh` → the save/mutate/restore, and `TABLE_SAVE` carries no `local`. Closed by making `cell_count` take the table as an argument, which deletes both the dance and the leak |

| **3Q1** | sequi | `check-where-shapes.sh:90-133,128,139,149,158`; `check-query-compat.sh:67-125,123,129,132,141`; `check-spec-native.sh:37-74,69,78,84,93` | **THREE CALL SITES THAT READ AS PREDICATES AND ALSO MUTATE A TALLY.** Each is written `check_X "$stem" \|\| FAILED=1` — a pure boolean at the call site. Each body ALSO does `ROWS_TOTAL=$(( ROWS_TOTAL + wn ))`, and the script's final summary prints `"$ROWS_TOTAL rows"` as its evidence. In bash the call site is the only visible signature, so this is the spell's *hidden state via global counter* with `fetch_add` replaced by arithmetic and no `local`. ⚠ **The number on disk today is CORRECT** — the ward checked: the summary prints only when `FAILED -eq 0`, so every call contributed. The defect is that **nothing enforces the coupling**: a reordered loop, an added `continue`, or a retried stem would silently produce a wrong coverage claim with no type, lint or assertion to catch it. | L2 | **OPEN** · ✅ I VERIFIED all 9 cited lines | each of the 12 line numbers read individually — all hold verbatim. `find_java`'s `export` was weighed by the ward and correctly dismissed as host-idiom (bash has no other channel). Closed by echoing the count on stdout and accumulating at the call site — domain state, so per the spell **no rune is available** |
| **3Q2 ★★** | ⛔ **orchestrator, found while verifying 3Q1** | same three files: `check-where-shapes.sh:152` vs `:158`; `check-query-compat.sh:135` vs `:141`; `check-spec-native.sh:87` vs `:93` | ⭐⭐ **THE OUTER NON-VACUITY IS GATED AND THE INNER ONE IS NOT — IN THREE SCRIPTS, WHILE A FOURTH IN THE SAME DIRECTORY NAMES THE PRINCIPLE.** All three guard `if [ "$PAIRS" -eq 0 ]` — *did we discover any stems?* **None guards `ROWS_TOTAL`.** So `PAIRS=5, ROWS_TOTAL=0` **passes green**, printing `"5 pair(s), **0 rows** — wat == Clara on every shape"`. Five pairs each comparing nothing, and the success claim is vacuously true with the zero printed in the message. ⛔ **`check-grid-speed.sh:85-90` guards exactly this class by name** — *"only $seen verdict(s) … **A short sweep is a gate that cannot fail, not a green one**"*, `exit 2`. The principle is written down, in this directory, by this author, and three sibling gates do not apply it to the number they themselves print as coverage. | **L1** | **OPEN** · ✅ I VERIFIED | `grep -n 'ROWS_TOTAL\|PAIRS'` on each of the three → `PAIRS -eq 0` guard present, no `ROWS_TOTAL` guard anywhere; `sed -n '85,90p' check-grid-speed.sh` → the named counter-example. Closed by a `ROWS_TOTAL -eq 0` floor in each, mirroring the sibling |

| **3M1 ★★** | temperare | `run-axis.sh:236-239` (wat, guarded) vs `:263-265` (Clara, bare); consumed at `:368`, `:376` | ⭐⭐ **THE TWO ARMS OF A WALL-CLOCK COMPARISON DO NOT PAY THE SAME HARNESS TAX.** The wat bracket is `WAT_W0=$(date +%s%N)` → **`guard "$WAT_BIN"`** → `WAT_W1`. `guard` is `systemd-run --user --scope --quiet -p MemoryMax=… -- timeout …` (`:155-157`), or a `( ulimit -v …; exec timeout … )` subshell on the fallback (`:164`). **A D-Bus transient-scope creation sits INSIDE the measured window.** The Clara bracket is `CLARA_W0` → bare `clojure -Sdeps … -M -m` → `CLARA_W1` — **no guard, no `timeout`.** `WALL_RATIO` (`:368`) and `FIRE_SHARE` (`:376`) are both computed from that asymmetric pair, and the file's own header sells them as load-bearing: *"An engine can win the fire and lose the program, and collapsing them hides exactly that."* ⭐ **MY ADDITION — WHICH WAY IT LIES, AND IT MATTERS:** `WALL_RATIO = CLARA_WALL/WAT_WALL` and `FIRE_SHARE = WAT_MEAN/WAT_WALL`. An inflated `WAT_WALL` **deflates both**, so **every distortion runs AGAINST this project's own engine.** The defect cannot manufacture a flattering result — only a pessimistic one. What it CAN manufacture is a false *"the fire is only N% of the program"*, sending someone to hunt overhead that is the measurement harness itself. | L2 (secondary field; the GATED verdict is clean — see below) | **OPEN** · ✅ I VERIFIED, and determined the direction | `sed -n '154,166p;236,239p;263,265p;366,368p;376p' run-axis.sh` — guard inside the wat bracket, absent from Clara's, both derived fields reading `WAT_WALL`. Closed by wrapping Clara in the same guard, or by starting the wat wall timer INSIDE the scope |
| **3M2 ★★** | temperare | `check-where-shapes.sh:22` vs `:103`+`:149`; cited at `check-grid-three-way.sh:29-32` | ⭐⭐ **A HEADER SAYS THE JVM TAX IS PAID *ONCE NO MATTER HOW LARGE THE CORPUS GROWS*. IT IS PAID 38 TIMES — AND THE ONE FILE THAT CITES THE CLAIM IS THE ONLY FILE THAT IMPLEMENTS IT.** `:22` — *"Here the JVM tax is paid ONCE no matter how large the corpus grows. Measured at 6 rows: wat 0.22 s + Clara 3.7 s."* But `clojure -Sdeps … -M "$clj"` sits at `:103` **inside `check_pair()`**, called once per stem at `:149`. **I counted 38 pairs with a `.clj` twin.** 38 cold JVM boots, in a CI job (`ci.yml:248`), under a header promising one. ⛔ **AND IT PROPAGATED.** `check-grid-three-way.sh:29-32` cites `check-where-shapes.sh:18-23` **by `file:line`, as the measurement** — *"3.7 s for all thirty-eight in one JVM. **The same applies here**"* — and then correctly stages every axis into ONE temp dir and drives a SINGLE JVM (`:198-223`). **The correct implementation justifies itself by citing a claim that is false about the file it points at.** `check-query-compat.sh:90` has the identical shape at 3 stems (`ci.yml:254`). | **L1** | **OPEN** · ✅ I VERIFIED | `sed -n '17,23p;103p;149p' check-where-shapes.sh`; pair count derived by loop → **38**; `sed -n '29,32p;198,205p' check-grid-three-way.sh` → the citation and the real batching. ⚠ **ARCHAEOLOGY CORRECTED:** the ward said `c8b062e64` refactored *"into 38-and-growing discovered pairs"* — at that commit there was still exactly **1** `where-*.wat` (`git ls-tree -r c8b062e64`). It introduced the per-pair *mechanism* (making the claim false in principle); the corpus grew to 38 later (making it false in magnitude). `git log -S'JVM tax is paid ONCE'` → introduced at `30d15b441`, where the corpus was 1 file and the claim was TRUE. The header hunk is untouched by `c8b062e64` — confirmed. Closed by batching (the sibling's `drive.clj` is the pattern) and by correcting both headers |

| **3F1 ★★** | conformare | `run-axis.sh:264` vs `:227-228` and `:236`/`:253` | ⭐⭐ **THE FILE NAMES THIS EXACT BUG, FIXES IT ON ONE SIDE, AND STILL HAS IT ON THE OTHER.** `:227-228` reads *"stderr is CAPTURED, not discarded: `2>/dev/null` made a wat-side failure loud but **REASONLESS** — you learned the axis produced nothing and never why."* The wat side duly captures to `WAT_ERR` (`:236`) and `cat`s it on failure (`:253`). **28 lines later the Clara side is `clojure … 2>/dev/null`** (`:264`), and its failure branch (`:267-271`) echoes `$CLARA_OUT` — **stdout only**. A JVM exception, a stack trace, a deps-resolution error: gone. ⛔ **3 of the 4 Clara-launch sites in this directory capture-and-surface** (`check-query-compat.sh:90`, `check-where-shapes.sh:103`, `check-grid-three-way.sh:218`); the outlier is the one every `run-all.sh` sweep goes through and the one `check-grid-speed.sh:51` runs in CI. | **L1** | **OPEN** · ✅ I VERIFIED | `sed -n '226,229p;236p;253p;264p;267,271p' run-axis.sh`; `grep -n 'clojure -Sdeps' *.sh` → exactly 4 sites, one outlier. ⚠ **I NEARLY OVERSTATED THIS AND STOPPED MYSELF:** `:235` ends *"Now it does, **both sides**"* — which reads like a false completeness claim, but belongs to the **wall-clock** paragraph (`:231-235`), where it is **TRUE** (`WAT_W0/W1` and `CLARA_W0/W1` both exist). Two adjacent paragraphs, nearly read as one. Closed by threading `2>"$CLARA_ERR"` exactly as `WAT_ERR` is threaded |
| **3F2 ★★** | conformare | all **54** `.wat`; representative `retract-multiplicity.wat:68`; counter-example `where-collection.wat:260-262` | ⭐⭐ **EVERY AXIS DESTRUCTURES ITS FAILURE DIAGNOSTICS AND THROWS THEM AWAY FOR A STATIC STRING — AND THE SAME LINE SHOWS IT KNOWS BETTER.** `((FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (assertion-failed! "fire-rules: session memory ceiling exceeded" None None))`. If that arm ever fires, the user gets the sentence and **no limit, no usage, no round count**. ⭐ **The proof is inside the same `match`:** the sibling arm `((FireOutcome::Fired __fired) __fired)` binds AND uses its value. One arm consumes, the others discard. And `where-collection.wat:260-262` builds a dynamic message with `String/concat` + `i64::to-string`, so the capability is present in the corpus. ⛔ **THE POPULATION IS LARGER THAN THE WARD FOUND — I ENUMERATED IT: FOUR arm families across THREE outcome enums, each in 54/54 files**, not three. It reported `FireOutcome::MemoryCeilingExceeded`, `FireOutcome::RoundCapExceeded`, `CompileOutcome::MayNotTerminate`; it **missed `InsertOutcome::MemoryCeilingExceeded __limit __used __count`**, which I found by stripping arm heads and seeing 158 occurrences of `__limit` survive. | **L1** | **OPEN** · ✅ I VERIFIED, and enumerated the missed family | `grep -l` per variant → **54/54 × 4 families**. Token test: strip every `(…Outcome::X __a __b)` head, then `__rounds`/`__cap`/`__still`/`__rule`/`__fact-type` → **0 remaining** (bound, never consumed), while `__fired` → **186** (consumed). ⚠ `InsertOutcome`'s static-string form is in **51** of 54 — three files differ and I did not chase which. This is a 4-family corpus rewrite = **the `wat-fix` codemod's exact shape**, per this repo's own doctrine |
| **3F3** | conformare | `peragrare-census.sh:189-198`, `:269`; also `:170` vs `:200-201` — ⛔ **MY OWN FILE** | **A QUERY THAT RETURNS `0` FOR BOTH *EMPTY* AND *MALFORMED*, AND SAYS SO IN ITS OWN COMMENT.** `cell_count` is documented `# H R A L N -> population of that cell (0 if none / invalid)` and the `--cell` entry point validates nothing against `H_VALUES`…`N_VALUES` (`:183-187`). A real-but-empty cell and a typo'd axis value both print `0`, exit `0`, indistinguishable. **The comment self-documents the ambiguity instead of resolving it** — `[[a-catch-all-holds-two-facts]]`, written by me, in my own introspection tool. ⚠ Same file: `do_report` calls `check_membership` bare while `do_verify` calls it guarded — two authorship patterns for one failure. | L2 | **OPEN** · ✅ I VERIFIED | `sed -n '189,198p;269p;170p;200,201p' peragrare-census.sh`. Closed by validating the five arguments against the declared value sets and exiting nonzero on an unknown one |
| **3R1 ★** | probare | `compare-grids.sh:6` | ⭐ **A COUNT THAT IS WRONG BY 12 AND OPENS THE FILE.** *"**Thirteen** `GRID-native-vs-clara-*.txt` files sit in this directory and NOTHING read them against…"* — `ls GRID-native-vs-clara-*.txt \| wc -l` → **25**. ⚠ **ARCHAEOLOGY NOT REPRODUCED:** the ward reported 19 existing at the introducing commit `d9fb1b88f`; `git ls-tree -r d9fb1b88f -- wat-scripts/perf/grid` returns **0** matches for me, so the naming or path differed then and I could not confirm it was wrong-when-written. **The live claim is false regardless, and that is what I row.** | L2 | **OPEN** · ✅ I VERIFIED the live claim; ⚠ archaeology NOT reproduced | `sed -n 6p compare-grids.sh` vs `ls … \| wc -l` → 13 vs **25**. Closed by deriving the count or dropping it |
| **3R2 ★** | probare | `check-where-shapes.sh:14` and `:29` | ⭐ **"THE OTHER NINE AXES" — THERE ARE ELEVEN.** Both lines say *nine*; `ls gen-*.sh \| wc -l` → **11**. The ward traced it to `30d15b4410` (2026-08-01) when nine was exact; `gen-neg-consumer.sh` (08-13) and `gen-leading-exists.sh` (08-24) landed after and neither back-filled the comment. ⛔ **This is the SECOND stale claim in this one file** — `:22`'s *"paid ONCE"* is already `3M2`. A file that explains the instrument's economics has two false numbers in its first 30 lines. | L2 | **OPEN** · ✅ I VERIFIED | `sed -n '14p;29p' check-where-shapes.sh` → *nine*, twice; `ls gen-*.sh \| wc -l` → **11** |
| **3R3 ★★** | probare | `check-grid-three-way.sh:16-17` | ⭐⭐ **THE CLAIM MY OWN MUSTER FLAGGED — AND I HAD SUSPECTED THE WRONG POPULATION ENTIRELY.** *"every one of the **47** recorded `GRID-*.txt` was produced under `GRID_SKIP_ORACLE=1` (**0 of 47** carry `:oracle-accuracy`)."* I flagged this in the muster as *"0 of 47 — **against 54 `.wat` axes**"*. ⛔ **47 was never about the axes.** It counts recorded RESULT files, and it spans **two directories**: 29 here + 21 in `docs/arc/2026/06/278-rules-engine/` = **50** today, 47 when written. ⭐ **The ratio still holds** — `grep -l ':oracle-accuracy'` across all 50 → **0**. So: numerator TRUE and re-derived, **denominator stale by 3**, and the line **never discloses that its population lives partly outside this directory** — which is why my own suspicion pointed at the wrong set. | L2 | **OPEN** · ✅ I VERIFIED, including my own error | `ls GRID-*.txt \| wc -l` → 29 here, 21 there = **50**; `grep -l ':oracle-accuracy'` over all 50 → **0**. Closed by deriving the denominator and naming both directories |

| **3V1 ★** | perspicere | `where-collection.wat:102,287,290,291`; precedent at `wat/rete.wat:326-327` | ⭐ **THE SUBSYSTEM UNDER AUDIT ALREADY ALIASED THIS EXACT SHAPE — THE GRID AXIS DID NOT.** Four sites, one file, one type: `(:wat::core::PersistentVector :- [(:wat::core::PersistentVector :- [:wat::core::i64])])` — a field of `:wc::Item`, a `defn` return, and an inner `fn`'s accumulator and return. ⭐⭐ **THE WARD FOUND A PRECEDENT I DID NOT KNOW ABOUT AND IT IS IN `wat/rete.wat` ITSELF:** `:wat::rete::ClassFields` is byte-for-byte the same 2-level `PersistentVector`-of-`PersistentVector` nesting, differing only in element type (`String` vs `i64`), **already given a name by the rete stdlib this whole vigilia is auditing.** So this is not a suggestion — it is an inconsistency with the subsystem's own settled practice. The domain noun is already spoken at both sites (`grid`, `:wc::build-grid`); only the type lacks it. ⚠ Ward committed to **mint the typealias**, arguing all three rune categories out: not `read-once` (4 uses), not `mumble-alias` (`Grid` is one clean word), not `intentional-structure` (no site needs the raw nesting). It also settled `typealias` vs `defalias` by reading both across the stdlib — `defalias` aliases **callables** (`dissoc`→`HashMap/dissoc`), `typealias` aliases **types**. | L2 | **OPEN** · ✅ I VERIFIED | `sed -n '326,327p' wat/rete.wat` → the precedent, exact; `grep -hn defalias wat/*.wat` → every use aliases a callable. Closed by `(:wat::core::typealias :wc::Grid …)` beside the `:wc::Item` defrecord, used at the four sites |

| **3W1 ★★★** | circumspicere | `check-grid-speed.sh:5-12,42-46` · `GRID-native-vs-clara-2026-08-27T07-15-56Z.txt` · `ci.yml:175-178,224-227` | ⭐⭐⭐ **THE CI SPEED FLOORS WERE MEASURED ON A JDK NOBODY RECORDED, AND CI NOW PINS A JDK THAT WAS NEVER CHECKED AGAINST THEM.** The `FLOOR` array is derived, by its own comment, *"from the recorded 33-cell grid"* — a file which, like **all 29 recorded grids**, names **zero** JDK/Temurin/Clojure version (`grep -il 'jdk\|temurin\|openjdk\|java '` over all 29 → **0**). It is a manual local capture (`bbeb1997d`), taken through the undocumented `PATH → JAVA_HOME → $HOME/opt/jdk-*` discovery. CI pins `distribution: temurin`, `java-version: "21"`. **Nothing has ever re-derived the floors under the JDK CI actually runs.** ⭐⭐ **AND THE SHAPE IS SHARPER THAN THE WARD PUT IT — THIS COMMENT WAS WRITTEN BY A PRIOR `circumspicere` CAST.** `:5` opens *"`circumspicere` found the speed half runs in no CI job"*, and `:7` records that the old excuse *"EXPIRED on 2026-08-27 when the `parity` job added Temurin 21."* **The Temurin pin and the floor-defining grid date to the same window, and the cure that predecessor prompted is what opened this gap.** `[[a-cure-can-carry-the-defect-one-level-down]]`. | **L1** | **OPEN** · ✅ I VERIFIED | `sed -n '5,14p' check-grid-speed.sh` → the citation and the EXPIRED note; `grep -il` over all 29 grids → **0** name a JDK; `grep -A3 setup-java ci.yml` → temurin 21, twice, **and no `cache:` line**. Closed by adding a JDK line to the provenance header every capture already carries (*"Box: … Code: …"*) and re-deriving the floors once under Temurin 21 |
| **3W2 ★** | circumspicere | `check-query-compat.sh:46`, `check-where-shapes.sh:86`, `check-grid-three-way.sh:115`, `run-axis.sh:46` | ⭐ **THE REFERENCE ENGINE'S VERSION IS A BARE STRING, COPIED FOUR TIMES, AND NOTHING ASSERTS THEY AGREE.** All four carry `com.cerner/clara-rules {:mvn/version "0.24.0"}`. **The entire three-way comparison rests on all four gates running the same reference engine** — no test asserts it, and no `deps.edn` or lockfile exists anywhere in the target or `ci.yml`. ⚠ **Deliberately NOT solvere's angle**, and the ward checked that itself by re-reading `solvere.md`: `3S5` covers the triplicated *JDK-discovery code* as duplication; **the version-coordinate identity is untouched.** ⭐ The ward's own closing line is the point: *"they agree today, which is exactly why nothing would catch it if they stopped."* | L2 | **OPEN** · ✅ I VERIFIED | `grep -n 'clara-rules' *.sh` → exactly 4, all `0.24.0`. Closed by a grep-assertion lint in the shape of the existing `every_parity_script_is_invoked` gate |
| **3W3** | circumspicere | `ci.yml:175-178`, `:224-227` vs `:84-85`; claim at `CLARA-TRANSLATIONS.md:12-13` | **BOTH CLARA CI JOBS RE-RESOLVE THE DEPENDENCY FROM MAVEN CENTRAL, UNCACHED, IN PARALLEL.** `actions/setup-java@v4` is used **without its opt-in `cache:` parameter** in both jobs; the only cache in the workflow is `Cargo cache` on the Rust side. The two jobs carry no `needs:`, so each pays full Maven resolution independently. Nothing beyond the bare coordinate pins the artifact — no checksum, no vendored jar, no lockfile — **while `CLARA-TRANSLATIONS.md:12-13` calls it "the pinned jar"** as though that established reproducibility. ⚠ **The ward proposed this may be legitimately `accepted-by-design`** (a released Maven coordinate is immutable in practice) — **but no rune anywhere says so**, which is consistent with the zero-rune fact seven wards have now measured. | L2 | **OPEN** · ⚠ ward-reported (`ci.yml` JDK pin + absent `cache:` ✅ verified by me) | `grep -n -A3 setup-java .github/workflows/ci.yml` → two jobs, temurin 21, **no `cache:`**. Closed either by `cache: maven` on both steps, or by writing the rune that says the coordinate is trusted and where that bound is documented |
| **3W4** | circumspicere | `CLARA-TRANSLATIONS.md:17` | **A GROUNDING CLAIM CITING A PATH WITH SOMEONE ELSE'S USERNAME IN IT.** The line says `clara-tools/` *"(at `/home/watmin/work/holon/clara-tools`, a separate sibling git repo) was checked"*. **That path does not exist** — this machine's user is `john`. The claim it supports is not load-bearing elsewhere, so severity is low, but **the specific grounding can never be re-verified by anyone.** ⚠⚠ **THIS IS THE THIRD DEFECT IN THIS ONE 422-LINE FILE, EACH FOUND BY A DIFFERENT WARD** — `3C1` (`:or` never mentioned), `3C2` (`:363`, a phantom "Rule 4"), `3I2` (`:418`, "all six forms" against 8), and now `:17`. **`conferre` and `intueri` both read this file in full and neither flagged the path.** | L2 | **OPEN** · ✅ I VERIFIED | `sed -n 17p CLARA-TRANSLATIONS.md` → the path, verbatim; `ls -d /home/watmin` → *No such file or directory* |

## Verified by the orchestrator — target 3

⭐⭐ **`circumspicere` REFUTED THE LEAD I THOUGHT WAS ITS SHARPEST, AND THE REFUTATION IS CORRECT.** I handed it three measured leads. Lead C proposed that a cold Maven cache would put a network fetch inside the Clara wall-clock bracket and corrupt the comparison — noting it would skew **opposite** to `3M1`'s `guard` asymmetry, with nothing recording which state the machine was in. **It does not hold.** The ward traced `run-axis.sh:263-265` and established that bracket feeds only `:wall-ratio`, and **`check-grid-speed.sh` never reads that field** — it parses `:ratio`, which comes from the self-timed fire-only figure inside each `#grid/Result`. So a cold fetch skews a number nothing gates on. **I marked the lead verify-or-refute and it was refuted; that is the framing working, not a wasted lead.** Leads A and B both held.

⚠ **AND IT READ ALL FOURTEEN PRIOR WARD REPORTS RATHER THAN TRUSTING MY COVERAGE TABLE.** Its report says so explicitly — *"to precisely rule out overlap rather than trust the summary table alone"* — and that is how it established that `3W2` is not `solvere`'s ground: it re-read `solvere.md` and confirmed `3S5` covers the triplicated JDK-discovery **code** while the version-coordinate **identity** is untouched. **A ward that audits the orchestrator's own coverage map before believing it is doing exactly what this vigilia has spent thirteen corrections learning to ask for.**

⭐⭐⭐ **`cernere` RETURNED CLEAN — AND ITS REAL PRODUCT IS A CORRECTION TO THIS REPO'S OWN `CLAUDE.md`.**
**⚠ THIS IS FOR THE BUILDER; IT IS A DOCTRINE ITEM, NOT A TARGET-3 ROW, AND IT IS DELIBERATELY NOT ROWED.**

`wat-rs/CLAUDE.md` warns that names outside `:wat::rete::` go unresolved in *"a `def` body nothing forces."* I handed that over as **a claim from a document, to test — not as fact.** The ward read the resolver instead of the prose, and the framing is **narrower than the mechanism**. I re-verified all three citations myself:

1. `src/resolve/walk.rs:264-267` — `is_resolvable_call_head` returns `true` for **any** reserved-prefix head, unconditionally. Its own comment says so: *"A wrong name under those prefixes (e.g. `:wat::holon::Bogus`) fails DOWNSTREAM at runtime or lowering … leaf-level validation is the type checker's concern."*
2. `src/resolve/quote.rs:14-46` — `check_quasiquote_template` resolves call heads **only inside `unquote` escapes**; *"Everything else in the template is data and must not be descended into."* So a record or field name written literally inside a quasiquote is never visited — **`def` vs `defn` is irrelevant to it.**
3. `RESERVED_PREFIXES` includes bare `:wat::`, covering every sub-namespace.

⛔ **AND THE CONSEQUENCE IS THE SHARP PART.** This corpus contains **zero raw `def`s** — the ward grepped and I accept it — so **the scenario `CLAUDE.md` names is ABSENT here**, while the real vulnerable shape is **PRESENT in 14 of the 54 files** that build rules through quasiquote templates. **A reader following the doctrine would check for `def` bodies, find none, and conclude the corpus is safe.** The ward hand-checked every quasiquote-embedded record and field name in those 14 files against its own `defrecord` — field-for-field and arg-order-for-arg-order — and every one is correctly spelled. **So: the hole is real, the doctrine's account of it misdirects, and this corpus does not currently exploit it.**

⭐ **The CLEAN is itself load-bearing, and it was traced against a spec I verified is authoritative:** `git -C ~/work/holon/clara describe --tags --exact-match HEAD` → **`clara-rules-0.24.0`**, working tree clean, exactly the version pinned at `check-grid-three-way.sh:115`. Every Clara form across 43 `.clj` **and the 11 Clara programs embedded in `gen-*.sh` heredocs** traced to a real, correctly-aritied form. Two judgment calls worth keeping: it investigated `clojure.string/join` called fully-qualified without a local `:require` in 5 files and correctly declined it (`clara.rules` transitively loads the namespace) — *"real form, fragile idiom, not a phantom"*; and it caught **its own automated pass mis-flagging ~50 local heads**, then confirmed each definition site by grep before reporting. **A CLEAN that names what it checked and what it nearly got wrong is evidence; a bare CLEAN is silence.**

⚠ **A THIRTEENTH FIGURE CORRECTED, AND THIS ONE IS WRONG IN BOTH DIRECTIONS.** I gave *"`clojure.string/` in 7 files."* The ward's literal-slash regex gives **6**; the true count of files where `clojure.string` is in play is **8**, because `where-shapes.clj` and `where-string.clj` require it as `:as str` and no slash-grep can see that. **My 7 was not a rounding error — it sat between two different right answers to two different questions.**

⚠ **A TWELFTH HANDED-DOWN FIGURE CORRECTED — AND THE DISCLOSURE IS WHY IT COULD BE.** I gave `perspicere` *"44 lines carrying two or more `:- `"*. It measured **43**, re-derived twice by two methods, and said it could not reproduce 44 by any method. **It is right and I can show why my number was wrong:** `grep -h ':- .*:- ' *.wat` → **44**; `awk '{n=gsub(/ :- /,"&"); if(n>=2) print}' *.wat` → **43**. My pattern did not require the leading space on the first occurrence, so it admitted one line carrying only a single real ` :- ` token. **The stricter token count is the right one.**

⭐⭐ **AND IT REFINED MY TRANSLATION IN A WAY I HAD MISSED ENTIRELY.** I told it the real form was *`:-` inside another `:-`*. That is right but incomplete: a line can carry two ` :- ` occurrences that are **siblings** — a parameter type and a return type on one signature line — rather than **nested**. Of the 43 lines: **22 comments, 21 code**; of the 21 code lines, **17 are sibling shallow types and only 4 are genuinely nested.** Without that distinction the finding would have been reported as 21 sites. **The refinement reproduces my "4" exactly, which is what makes it a correction rather than a new claim.** Together with `sequi`'s 37→36/41 and `probare`'s density verdict, that is three briefs in a row where a figure handed over WITH its derivation came back sharper instead of simply adopted.

⛔⛔ **`probare` CONTRADICTED THE VERDICT I SUGGESTED IN ITS OWN BRIEF, AND IT IS RIGHT.** I wrote that the measured comment fractions *"suggest this corpus will come back **substance-rich**"*. It measured, both ways, and the answer is **MIXED**: counting non-comment non-blank lines gives `.sh` **1.81:1**, `.clj` **2.10:1**, `.wat` **2.09:1**; counting the spell's stricter Lisp form-starts gives `.wat` **1.43:1** and `.clj` **1.21:1**. Every figure lands inside the spell's **1:1–3:1 mixed** band, none in the >3:1 substance-rich band. It also supplied the blank-line counts (161/551/816) my figures had omitted — the denominators I handed over included blanks in neither numerator. **A ward that adopts the orchestrator's expected verdict is worth nothing; this one measured and said no.**

⚠ **AND IT CORRECTED MY SUSPICION, NOT JUST MY NUMBER (see 3R3).** The muster block flagged `"0 of 47"` as needing re-derivation *because there are 54 `.wat` axes* — implying 47 was an undercount of the axis corpus. **47 has nothing to do with axes.** It counts recorded `GRID-*.txt` result files across two directories. I flagged the right line for the wrong reason, and only a ward told to derive the true value rather than confirm my hunch would have found that.

⭐⭐ **THE GATED VERDICT IS CLEAN, AND THIS IS THE FIRST TIME ANYONE CHECKED.** `temperare` was told to rank measurement-corrupting work above merely-slow work, and to **state explicitly with line numbers if the timed regions are tight**, because that is a positive result about the instrument nobody had established. It did. Across **16 sized `.wat` axes** (6 read in full, 10 grepped at the binding line) `:native-ns` binds to exactly the interval between the timer immediately before `(:wat::rete::fire-rules staged)` and the one immediately after — nothing else inside. `fanout.wat:130-168` is the strongest case: it carries **five additional timer pairs** for a diagnostic side-channel print and deliberately keeps every one of them OUT of `:native-ns`. All 11 `gen-*.sh` build and insert the session **before** `t0`, warm the JIT with 3 untimed dry fires, and bind `:clara-ns` to the fire-only delta. **So `:ratio`/`:winner` — the verdict `check-grid-speed.sh` actually gates on — is uncontaminated**, and `3M1`'s asymmetry reaches only the secondary wall/fire-share pair. Two wards in a row have now tried to break this instrument's numbers and failed; that is worth as much as a finding.

⭐⭐ **A ★-FLAGGED HYPOTHESIS OF MINE WAS REFUTED, LINE BY LINE, AND THAT IS A RESULT.** I told `sequi` I believed `run-axis.sh`'s ~20 interpolated globals were its richest surface, and asked whether any value from a previous SIZE or RUN could survive into a `#grid/Verdict` line — *"a wrong number that would look completely plausible"*. **It cannot.** The nine accumulators reset at `:215-223`, which I confirmed is the first thing executed inside `for SIZE in "$@"` after only the `mktemp`/generate preamble; every remaining interpolated variable is unconditionally recomputed at `:332-381`; and the two early exits the theory had to clear (`:255`, `:271`) are hard `exit 1`s that never reach `:382`. **The reset discipline is correct and deliberate, and the instrument is more trustworthy than I assumed.** ⚠ The ward also declined to file a `HAS_ORACLE` wrinkle (`:223` reset per SIZE, set at `:303` inside the RUN loop, read at `:379`) as *theoretical* — I agree with the declination and record it here so it is not lost: it needs a binary that emits `:oracle-derived` on one run and not another of the same size, which nothing suggests happens.

⚠ **AND IT CORRECTED MY VARIABLE COUNT — the second handed-down figure this cast to be re-derived rather than adopted.** I gave *37 distinct* `GRID_*`/`WAT_*`/`CLARA_*`/`ORACLE_*`/`JAVA_*` names **and disclosed that it was a raw grep which had not separated definitions from uses**. The ward measured **41 raw token lines** (`$WAT_BIN` and `${WAT_BIN` counting separately) collapsing to **36 distinct names**, and reported the delta instead of adopting either. ⭐ **Disclosure worked again** — that is now twice in this vigilia that a figure handed over WITH its known contamination came back corrected and usable, against eleven that were handed over bare and were simply wrong.

⭐⭐⭐ **THE FACT THAT REFRAMES THE WHOLE GRID, AND I HAD IT WRONG UNTIL THE CENSUS RAN: THE
THREE-WAY INSTRUMENT COMPARES 16 OF THE 54 AXES.** `check-grid-three-way.sh:126` carries
`case "$stem" in where-*) continue ;; esac` — and `ls where-*.wat | wc -l` is **38**. 54 − 38 = 16.
I have referred to "the grid" as 54 axes throughout this vigilia; the arc's flagship correctness
instrument runs over **16**. ⚠ **This is NOT a finding**: the 38 are an *honest exclusion* handed
to two **named** siblings (`check-where-shapes.sh`, `check-query-compat.sh`), which is exactly the
distinction `peragrare`'s step 4 demands between *never offered* and *silently dropped* — and
`dropped` measured **0**. But every coverage intuition about this instrument has to be rebuilt on
16, not 54.

- **3P1 ★★★** — CONFIRMED, and I ran the census myself rather than reading its output.
  · `bash peragrare-census.sh` → 108 cells (2×2×3×3×3, the arithmetic closes), 9 visited, 99 empty,
    16 members. · `--pins` → **populated PASS** (5 where 5 expected), **empty PASS** (`H=lambda`, a
    value no axis takes — correctly a *value* and not a forbidden *combination*), **moving PASS,
    8 of 8**, with the denominator the spell specifies. · `--verify` → *"all 16 fixtures' mechanical
    facts agree with the table."*
  ⭐ **And the ward disclosed the one thing that would have made this a rumour.** Its coordinates are
  **hand-derived, not computed** — it built a mechanical classifier, found it misread this corpus's
  *second* authoring idiom (bare let-bound symbols vs inline quoted forms) and produced false
  userfn heads, and **refused to ship it**. The spell's own clause covers this: *"Where a fixture's
  coordinate is read literally off it rather than computed, this pin is trivially satisfied."*
  Hand-assignment is the read case. It then built `--verify` so a future corpus edit breaks the
  table **loudly**. That is the honest disposition of a limitation, not a workaround for one.

⛔⛔ **AND I NEARLY ROWED TWO FALSE FINDINGS IN ONE VERIFICATION CHAIN — BOTH FROM GREPPING A NAME.**
1. The ward's aside says *"no Rust test drives `check-grid-three-way.sh` at all."* My
   `grep -rl 'check-grid-three-way' tests/` returned **0**, and I began treating the arc's flagship
   differential as ungated.
2. I then found `tests/lint/every_parity_script_is_invoked.rs` and started to row *that* gate for
   not covering it.
**Both were wrong, for the same reason.** The gate's own header says: *"**Discovery, not a list.**
The directory is WALKED. A list cannot notice what was never added to it — which is the whole
defect."* It globs `check-*.sh` under the grid, so a covered script is **never named** — my grep was
measuring mentions, not coverage. And *"What counts as invoked: named by the CI workflow, **or** by a
Rust test. Both are real invocation paths… the question is whether SOMETHING runs it, not which
thing."* `check-grid-three-way.sh` is invoked at **`.github/workflows/ci.yml:262`**.
⚠ **This is the THIRD distinct instance this session of one error class**: my `#[allow]` count
matched a prose mention; my `RETE_OPS` count matched 29 prose mentions of `rete_name`; and now two
name-greps mistook "not mentioned" for "not covered." **A grep for a name answers a question about
names.** The ward was right to file its note as *"not a peragrare finding — an instrument-robustness
aside"*, and that framing is why it did no damage: the residue that survives is narrow and real —
the *"BOTH is a hard failure"* guard is live code that has never been **mutation-proved**.

⭐ **9 of 9 visited cells READ, zero hollow** — and the evidence is specific rather than asserted:
`userfn-head.wat`'s witness carries both Rate and Out *"so a witness carrying Out alone cannot tell
'Out dropped' from 'nothing derived'"*, which reads two axes at once; `accum-lead-rule-cascade.wat`
asserts count-**constancy** precisely to catch a round-count leak. **The corpus's fixtures assert on
the axes they occupy.** The gap is entirely in what nobody staged, not in what was staged weakly.

⚠ **One number I declined to adjudicate rather than get wrong an eighth time.**
`check-grid-three-way.sh:17` says *"0 of 47 carry `:oracle-accuracy`"*; the ward re-derived **0 of
29** (corpus rotation). My own quick `grep -c` found **11 `.wat` mentioning** `:oracle-accuracy` —
but that is a **different population** (sources mentioning a field, not recorded outputs carrying
it), and it is the exact mention-vs-thing trap above. **I am recording all three numbers and
adjudicating none**, because settling it needs the recorded-output corpus enumerated, which nobody
has done.

⭐⭐ **`mora` RETURNS CLEAN — the only ward in this vigilia whose trigger fires on target 3 and
NOWHERE ELSE.** It measured **0** on the fire path and **0** on the compile side, both recorded as
facts about those targets. Here it fires on exactly two lines, and its verdict is that **its own
discipline does not reach them.** No row is filed.

**I re-ran all three of its greps: zero `sleep` in the 20 shell scripts, zero readiness-snapshot
surface, exactly two `timeout` sites — both in `run-axis.sh`.** No delta.

⛔ **The judgment is the result, and it is argued rather than asserted.** Both sites are the shell
`timeout(1)` used as a kill-guard, one paired with `ulimit -v` in the same breath. The ward walked
the four questions and landed on *out of domain*: **a coordination wait needs an event to wait FOR,
and what is being bounded here is the ABSENCE of a terminating event** — a hang or a runaway
allocation — which no select can wait on. I confirmed the guard's provenance: `run-axis.sh:124-137`
is headed **"THE BLAST DOOR"** and records the incident that produced it — *"2026-07-30:
`run-axis.sh node-share …` **CRASHED THE BUILDER'S MACHINE**. The N=50 point consumed the box's
43 GiB of available RAM and the OOM killer took the desktop."*

⭐ **And it refused a rune rather than force one.** Its spell offers four categories
(`calibration`/`external-api`/`no-kernel`/`no-reactor`) and it judged that **none fits a
runaway-process safety net**, saying that forcing one *"would misuse the taxonomy rather than honour
it."* A ward that declines an exemption it is entitled to reach for is worth more than one that
files a tidy rune.

⭐ **It also did the work reading 3 demanded and reported the timeout path fails LOUD.** `WAT_RC` is
captured outside `set -e` so `pipefail` cannot swallow it; RC `124` and `137` get distinct
diagnostics; a missing result line hard-exits 1 rather than feeding a partial sample into the
mean/spread; and `run-all.sh:139-144` propagates the failure with a nonzero sweep exit. **The
instrument holds itself to the doctrine its sibling states** — *"unreadable is a HARD FAILURE — never
a skip."*

⚠ **The honest boundary it named and deliberately did NOT file** is the most useful sentence in the
return: `WAT_RC` is consulted only inside the `-z "$WAT_LINE"` branch, so a process killed *after*
flushing a complete result line but before exiting would keep its result, never have its RC
inspected, and fold the hung time into `:wat-wall-ms`. It has **no evidence that binary hangs
post-print**, said so, and classified it as *"an RC-scoping completeness question"* rather than a
mora finding. **Naming the limit of what you confirmed, and refusing to file it as what you did not,
is the discipline this whole vigilia runs on.**

⛔ **AND IT CORRECTED A COUNT OF MINE IN A WAY THE OTHER EIGHT WERE NOT: I INVALIDATED IT MYSELF.**
I handed it *"19 `.sh`"*. It measured **20**. Both are right — **I committed
`peragrare-census.sh` into that very directory two commits earlier**, so the population I quoted was
accurate when measured and stale by my own hand by the time it was read. ⚠ This is a *new* failure
mode, distinct from the eight prose-vs-thing and reachability errors before it: **a measurement can
be invalidated by the measurer's own subsequent action.** A handed-down number needs the commit it
was taken at, not just the command.

⭐⭐ **`exigere` RETURNS CLEAN ON TARGET 3 — AND THE "REAL POPULATION" I HANDED IT WAS NOT ONE.**
I briefed it that this was *"the first target in this vigilia where your trigger has a real
population"* — **7** TODO-family hits, against zero on both prior targets. It re-derived and split
the number:

    grep -rniE '\b(TODO|FIXME|XXX|HACK)\b'   → 7
    grep -rniE '\b(TODO|FIXME|HACK)\b'       → 0     ← XXX excluded

**All seven are `XXX`, and every one is `"XXX"` as a deliberately-nonexistent location code** in a
query axis's missing-loc row — I read them: `(count (query s temps-at :?loc "XXX"))`. The identifiers
`at-xxx` / `params-xxx` are derived from that literal. ⛔ **So the grid's true TODO-family count is
ZERO, the same as targets 1 and 2** — and my "7" was a pattern matching **domain data**, not a
marker. `[[a-throwaway-sweep-is-an-instrument]]` in yet another form: not prose-about-the-thing this
time, but **data-that-looks-like-the-thing.**

⛔⛔ **AND I INVALIDATED MY OWN LINE COUNT WITH THE SAME COMMIT THAT INVALIDATED THE `.sh` COUNT.**
I handed it *"16,345 lines"*; it measured **16,616** and reported the delta rather than adopting
mine. `peragrare-census.sh` — which I committed into that directory two commits earlier — is
**exactly 271 lines**, and 16,345 + 271 = **16,616**. So one commit of mine invalidated **two**
handed-down numbers, and **two different wards caught them independently** (`mora` the `.sh` count,
`exigere` the line count). ⚠ That is the strongest possible confirmation of the failure mode the
`mora` note names: **a measurement carries the state of the tree at the moment it was taken, and the
measurer is one of the things that can change it.**

⭐ **The dismissal list is again the product, and one dismissal is a small masterpiece.**
`REMAINING-CLARA-MOUTHS.md` — a file whose *title* is "Remaining" — turns out to have all seven of
its numbered items marked `— DONE (where-*)` and to close with `## This list is empty.` /
`2026-08-17: items 1–7 locked.` **A backlog file whose entire content is a closure record.** A
title-level grep would have filed it; reading it dismissed it. Others in the same shape:
`run-all.sh:46`'s *"RED until task #94 is closed"* cross-checked against `neg-consumer.wat:38`'s
*"★ THIS AXIS FOUND AND THEN CLOSED task #94 … Fixed in ff581b6f"*; two "defers" that belong to
**Clara's** evaluation ordering, not ours; and a *"not yet built"* that is a comparative statement
about Clara having no per-round index at all.

⭐⭐ **AND IT ANSWERED THE COLLISION I SET UP BETWEEN TWO WARDS — with a result, not a hedge.**
I told it `peragrare` had cast on this same corpus and that a fixture header's coverage matrix is
*"exactly your quarry AND exactly `peragrare`'s"*, then asked which shape it found. It searched for
the open-cell form (`not yet covered`, `NEXT AXIS`, `not visited`) and got **zero hits**: every
matrix header here uses only **"covered"** (past-tense, closed) and **"THIS AXIS"** (naming the
present file). ⛔ **So the two wards do not collide on this corpus, and the reason is documented in
the corpus itself**: `peragrare`'s 5 unvisited cells are a *structural absence its census found by
walking the grid*, not prose promising to fill them. **No coverage note in this directory is written
as a promise.** That is a real fact about how this corpus documents itself, and it took casting both
wards to establish it.

- **3S1 ★★ / 3S2 ★** — CONFIRMED, and these two are qualitatively different from the other four
  because **the risk each names has already materialised once.**
  · **3S1: the drift is present, not predicted.** `run-axis.sh:277` is
    `grep -oP ':derived\s+(?:#wat\.core/PersistentVector\s+)?\K\[[^]]*\]'`; the sibling's `extract()`
    at `check-grid-three-way.sh:235` is the same decode **plus `(?<=[ {])`**. Two copies of one
    wire-format reader, one hardened, one not — and the hardened one's comment names exactly what it
    defends: *"a future `:spec-derived` would [contain `:derived`], and the match count below is what
    refuses an ambiguous line."* **The unguarded copy is not wrong today; it is wrong on the day the
    field set grows, and only one of the two will notice.**
  · **3S2: the class has already produced a silent no-op in this very directory.**
    `run-axis.sh:179-180` records it verbatim: *"The 2026-08-20 skip was a no-op because it still
    matched `fire-rules-spec` after the `$oracle` rename."* And the duplicated `rewrite_to_spec()`
    encodes that same verb-naming convention **twice**, comment and all, word for word.
    `[[an-accurate-comment-can-be-a-defects-alibi]]` in its sharpest form: the incident is written
    down, the lesson is not applied to the copy sitting one file away.

⭐ **`solvere` read all 20 scripts in FULL and said so — no sampling.** That matters for a
duplication ward: its spell's own rule is that a finding naming one instance of an N-fold copy is a
fraction of a finding, and **Findings 1, 4, 5 and 6 each enumerate EVERY copy** (11, 3, 2 and 5
sites). It also ran the disconfirming grep first — `grep -ho 'source …' gen-*.sh` → **empty** —
establishing there is no shared helper before claiming duplication, rather than inferring it.

⭐⭐ **AND IT REFUSED THE FINDING I EXPLICITLY WARNED IT AWAY FROM, then found the real one under it.**
I told it: *"Do not flag 'there are eleven generators' as a finding. Eleven perf axes need eleven
Clara programs; that is the design."* It agreed in its own words — *"the per-axis workload stays in
each `gen-<axis>.sh` (that part is genuinely eleven different programs — not a finding)"* — and then
isolated what actually IS duplicated: the **harness** around the workload, byte-identical in two
variants (8 files + 3 files). **Separating the eleven-different-things from the one-thing-eleven-times
inside the same eleven files is the whole discipline**, and a ward that reports the file count would
have missed it.

⚠ **All six are judged INCIDENTAL, and that judgment is load-bearing rather than a hedge.** Its
spell's categories for a *legitimate* braid are `load-bearing-coupling`, `irreducible-tangle` and
`historical-shape`; it found **none of the six qualifies for any of them**, and no `rune:solvere`
exists in the 20 scripts to pre-excuse them. Incidental means *nothing forces these copies* — which
is what makes them fixable, and what makes 3S1's already-present divergence a warning rather than a
cost of doing business.

⚠ **One observation of its own worth keeping, which it drew from the copies rather than the code:**
of `WAT_BIN`'s five sites, **only `run-axis.sh` carries the freshness wall** — the other four read
the same binary for the same kind of measurement with no such protection. It reads that asymmetry as
*"evidence the copies aren't being kept in sync as the convention evolves."* ⛔ **That is the same
shape this session already met at the top of the tree**: I ran a stale `~/.cargo/bin/wat` earlier
today, and only caught it before casting `experiri` because the lesson had been written down. **Four
of these five scripts have no equivalent of that lesson at all.**

- **3C1 ★★** — CONFIRMED, **and it is STRONGER than the ward stated.** It reported that the contract
  *"discusses `:or` zero times as a Clara-vs-wat semantic point."* I ran `grep -n ':or\b'` over all
  422 lines: **the token does not appear in the file at all.** Not under-discussed — **absent.**
  Meanwhile `grep -lE '\(count \(set '` returns **16** of the 43 twins, exactly as reported.
  ⛔ **The asymmetry is the finding.** The contract documents 8 axes in depth, and — the ward's
  sharpest observation — **only ONE of those 8 has a static `.clj` in this directory at all**; the
  other seven are `gen-*.sh` axes with no twin to check against. So the document is thorough about
  axes it cannot be checked on, and silent on the idiom that reaches 16 of the twins it can.
  ⚠ **And the code is right.** This is not a live accuracy bug — the 16 sites already compensate.
  The defect is that a rider authoring a NEW `:or`-shaped twin, using the contract as its stated
  grounding source, would not learn that Clara doubles the insert.
  `[[a-briefs-read-list-vouches-for-what-it-points-at]]` — here the corpus's own translation
  authority is the thing that vouches, and the gap is what it does not say.

- **3C2** — CONFIRMED, and delicious. `CLARA-TRANSLATIONS.md:363` strikes a prior decision on the
  authority of *"Rule 4 of this document."* I ran `grep -coE 'Rule [0-9]'` over the file: **1** —
  and that one occurrence **is the citation itself.** There is no Rule 1, no Rule 4, no rule list;
  the ward checked all three commits of the file's history and found none ever existed. **A document
  citing a rule of its own that it does not contain**, as the sole justification for treating its
  "most important axis" as resolved.
  ⭐ **The ward filed it as an aside rather than a finding, and was right to.** Its spell requires
  **two coordinates** — spec and code — and this has only one: it is a spec-internal broken
  citation, not a spec/code divergence. **A ward that declines to promote a good catch into a
  category it does not fit is one whose categories still mean something.** I have rowed it as an
  observation, in its own words.

⭐ **AND ITS SAMPLING RULE IS THE MODEL FOR A CORPUS TOO LARGE TO READ.** I told it 43 pairs is more
than one cast can read properly, and to state a rule rather than skim. It read **8 pairs in full**,
chose them by a stated three-part rule (axes the contract documents that actually have a static twin;
the newest additions post-dating the contract's last edit; the highest-risk shapes named in the
cast), **grep-verified the pattern it found across all 43**, and then listed by name the ~35 pairs it
did NOT read in full. ⭐ **It also reported the clean half:** all 8 pairs read in full were
*"structurally faithful rule-for-rule"*, two with exemplary self-documented deviations — including
`userfn-head`'s own fidelity guard, *"if mk-rate ever computes, this file… must be rewritten."*
**A sample whose rule is stated and whose exclusions are named is evidence; a larger sample without
one is not.**

- **3G1 ★** — CONFIRMED **by running the mutation myself**, not by reading. `f() { return 3; }` then
  `if ! f; then echo $?; fi` prints **0**; the same `f` un-negated prints **3**. So `run-all.sh:139`
  can only ever emit `rc=0`.
  ⛔ **And it lands on a line another ward vouched for.** `mora` read this exact site an hour ago and
  cited it as evidence the sweep *"does not swallow"* a killed axis — *"`rc=1`, propagating the
  failure rather than reading as a quiet gap."* **Both wards are right, and neither alone is the
  whole truth:** the propagation is correct and the diagnostic beside it is a constant. `mora` was
  reading the control flow; `purgare` was reading the value. **A line can be load-bearing and carry a
  dead field at the same time**, and it took two wards with different questions to see both.

⭐⭐ **AND ITS METHOD IS THE ANSWER TO THE FAILURE I HAVE MADE FOUR TIMES THIS SESSION.** I warned it
that a name-grep here cannot see coverage-by-walk or consumption-by-argument, and that I had nearly
handed it a false finding (28 of 29 `GRID-*.txt` "unreferenced" — they are `compare-grids.sh`'s `$1`
and `$2`). It answered with **six distinct query kinds**, named: by name; **by discovery-walk**
(reading the actual glob code in three scripts and reconciling *bidirectionally*); by call-site in
sibling scripts (building the real invocation chain `check-grid-speed → run-all → run-axis → gen-*`);
by function name against call sites; **by independent glob-diff** (`comm` over stems, re-deriving the
exactly-one-of rule from scratch rather than trusting my citation); and by **mutation**.
**Six queries where I used one.** Its clean result — every script alive, all 54 axes covered by at
least one instrument, no orphaned function or env var — is worth something *because* of that, and
would have been worth nothing without it.

# TARGET 4 — `tests/rete/` + `src/rete/kernel/tests/` (**306 files, 38,058 lines** — measured 2026-09-08; the tracker said *264 / ~36k*)

Muster derived in `README.md` with measured triggers. Returns land verbatim in `reports-target4/`.

| id | ward | site | finding | sev | status | re-derivation |
|---|---|---|---|---|---|---|
| **4X1 ★** | excusare | `src/rete/kernel/tests/fanout_cost.rs:841-842` | ⭐ **THE ONLY LIVE EXTERNAL SUPPRESSION IN 306 FILES, AND IT PLEADS NOTHING.** `#[allow(unused_variables)]` with **no reason string**, over `let child_tax: f64 = …` — a real computation (two census names × a calibration factor) that **`grep -c 'child_tax'` finds exactly ONCE in the whole file: its own definition.** Computed and discarded. ⭐⭐ **AND THE COMMENT DIRECTLY ABOVE IT SAYS WHAT IT WAS FOR, IN THE IMPERATIVE:** *"18.992 → 11.524 ms, wall 24.491 → 16.690. **Subtract the children's tax from the parent, or the biggest number in the table is the instrument.**"* Nothing subtracts anything. The sibling `top_sum`, defined immediately after, **is** printed. So the `#[allow]` silences the compiler reporting exactly the gap the comment describes — a bare suppression over an unfinished intention. Per the spell's own default, a suppression with no reason is **ILLEGITIMATE-AT-BIRTH**. | **L1** | **OPEN** · ✅ I VERIFIED | `sed -n '839,844p' fanout_cost.rs` → the comment, the bare `#[allow]`, the binding; `grep -c 'child_tax'` → **1**. Closed by printing it in the phase table (the comment's evident intent) or deleting the computation — either way the `#[allow]` goes |

| **4L1 ★★** | complectens | `src/rete/kernel/tests/mod.rs:483-594` (`render_phase_table`) vs its only proof at `rank_and_instrument.rs:260-272` | ⭐⭐ **THE ONE TEST NAMED AS THIS HELPER'S PROOF CANNOT REACH THE ARITHMETIC — AND ITS NAME PROMISES TWO ARMS.** `render_phase_table` does instrument-subtraction (`net_of`, `total_min`, `total_net`, per-phase %, a *BELOW ITS OWN INSTRUMENT* flag), is called from **4 sites**, and **its own doc records TWO past arithmetic bugs**: it *"returned `sum/xs.len()`"* — a mean — under a **MINIMUM**-labelled header until arc 278 C1, and *"an earlier version of this table report 124% coverage"* from double-counting parent+child. The sole dedicated test is `render_phase_table_proves_missing_phase_and_zero_total`. It feeds a census **missing the required phase**, which panics **before** the arithmetic runs. ⭐⭐⭐ **AND I READ THE WHOLE TEST: IT IS THREE STATEMENTS. THERE IS NO ZERO-TOTAL ARM.** The setup passes `\|_, _\| 0` — a zero total — and *also* omits the required phase, so **the first condition short-circuits the second and the name's second promise can never be observed.** `[[a-tests-setup-can-void-its-own-assertion]]`, `[[one-mutation-cannot-prove-a-multi-arm-gate]]`. The four callers only `println!` the table; `node_share_cost.rs:970-996` asserts on raw `ns_of(…)` *"on the DATA, not the rendered text"*. **The part with a written history of being wrong has no proof at all.** | L2 (ward); ⭐ **the name is an L1 shape and that is mine** | **OPEN** · ✅ I VERIFIED, and strengthened | `sed -n '260,272p' rank_and_instrument.rs` → `catch_unwind` + one `assert!(boom.is_err(), "missing required phase must panic")` + `}`. Closed by a case that drives the arithmetic on fixed synthetic rows and asserts a known net/percentage |
| **4L2** | complectens | `node_share_cost.rs:98-99` (rune) vs 7 un-runed siblings | **THE EXEMPTION WAS WRITTEN ONCE FOR AN IDIOM THAT APPEARS EIGHT TIMES.** `rune:complectens(inline-fixtures)` justifies a 718-line, ~23-outer-binding interleaved-timing test. **Seven sibling files carry the structurally identical idiom at comparable scale with no rune**: `accum_alpha_cost.rs` (301/291/285/169-line tests, 23 outer bindings confirmed on `:74`), `accum_cost.rs` (7 tests, 256→145), `gather_probe_cost.rs` (257/177/158), `fanout_cost.rs`, `cascade_cost.rs`, `harvest_cost.rs`, `strat_cost.rs`. ⚠ **The ward read three end-to-end and judged them NOT violations** — same legitimate shape, four questions all pass, every phase variable carries its own named non-vacuity assert. **The finding is the inconsistency, not the shape**: a reader landing on an un-runed sibling has no signal its form was already adjudicated. ⛔ Pairs with `excusare`'s independent verdict that this same rune **HOLDS** — so the cure is to extend it, never to strike it. | L3 | **OPEN** · ⚠ ward-reported | `grep -n 'rune:complectens' ` → 2 in target, 1 of them this one. Closed by runing the seven, or by hoisting the reason to a shared module doc |
| **4L3** | complectens | `src/rete/kernel/tests/alpha_discrimination.rs:38-49` | **A 5-STEP FIXTURE HELPER WITH FOUR CONSUMERS AND NO PROOF OF ITS OWN.** `alpha_tree_fixture_50_100()` composes `fire_cascade` → `to_transient` → `sorted_node_ids` → `build_alpha_index` → `AlphaTree::build`, consumed at `:67,139,244,362`. ⚠ **The ward argued its own finding down and I keep that reasoning**: each `.expect(…)` carries a distinct message so a break does narrow, and the helper composes already-tested library primitives rather than introducing logic — the spell's proof-need axis is logic complexity, not call count. Recorded because it is the only helper-without-sibling-test the mechanical pass found outside the cost-benchmark family. | L3 | **OPEN** · ⚠ ward-reported | `grep -n 'alpha_tree_fixture_50_100' alpha_discrimination.rs` → 1 def, 4 uses, 0 tests |

| **4O1 ★** | vocare | `probe_arc278_import_fold_key.rs:33,180,185,246,295,341` vs `probe_arc278_export.rs:136,177,254,462,500,556,570` | ⭐ **SIX TESTS DO THE IDENTICAL BYPASS A SIBLING FILE RUNES SEVEN TIMES OVER.** All six reach into `Value::Aggregate { a.names, a.fields }`, rebuild the `Export` via `AggregateValue::record(…)` to tamper a `:sum` fold key on the wire, then re-enter through the public `:wat::rete::import`. `export.rs` carries `rune:vocare(vantage-bypass-test)` at **7** sites for exactly this maneuver, with the reason *"host Aggregate.fields poke; wat has no Export setter."* The ward **verified that fact independently** — no wat-level Export setter exists — so the state is **not caller-producible**, which is what makes it a vantage-bypass needing a rune rather than an exempt defect-fixture. ⭐ **The bypass shape is FULLY ENUMERATED, not sampled:** only **4 files in the whole target** touch `Value::Aggregate`/`AggregateValue::record`, two runed and two not. **The tests are load-bearing and correct; the gap is the annotation.** | L2 | **OPEN** · ✅ I VERIFIED the enumeration | `grep -rl 'Value::Aggregate\|AggregateValue::record' tests/rete/*.rs` → exactly 4: `export.rs`, `compiled_where_ops.rs` (both runed), `import_fold_key.rs`, `import_accounting.rs` (neither). Closed by adding the rune its sibling already carries |
| **4O2** | vocare | `probe_arc278_import_accounting.rs:131,176` | **TWO SHAPES IN ONE FILE, AND THE SECOND IS A RUST UNIT TEST LIVING IN THE INTEGRATION CRATE.** `import_refuses_a_node_count_past_the_cap` is the same `Value::Aggregate` poke as `4O1`. But `an_origin_already_filed_is_never_re_based` calls `wat::alloc_counter::mark_session_origin_at`, `thread_bytes`, `session_bytes` **directly — zero wat evaluation, no Session or Export value in play at all.** ⭐ **Its own docstring states the justification almost in rune format:** *"The two `.wat` arms above cannot see this — each files its key exactly once — so this arm has its own probe or it has none."* The file numbers all three as *Arms 1/2/3* of one probe, so the file reads as consumer-vantage throughout while Arm 3 is a Rust-internals unit test. ⚠ **The cure has two forms and the second is better:** rune it, **or relocate it into `src/rete/export.rs`'s own unit-test module — where `vocare`'s `src/*.rs` architectural exemption would cover it outright, with no rune needed.** | L2 | **OPEN** · ⚠ ward-reported | `sed -n '176p' probe_arc278_import_accounting.rs`. Closed by the rune or the relocation |
| **4V1 ★★** | perspicere | `accum_alpha_cost.rs:242,272,586,1161,1191,1295,1308,1350` + `gather_probe_cost.rs:999` vs `src/rete/kernel/session.rs:176` | ⭐⭐ **THE PRODUCTION CODE ALREADY NAMED THIS NOUN AND THE TESTS RE-SPELL IT NINE TIMES.** `HashMap<i64, Vec<u8>>` appears raw at 9 test sites. `session.rs:176` defines `pub(crate) type BindOnlyFields = HashMap<i64, Vec<u8>>;` — **used in 5 production files**. ⛔ **And the test variables are literally named `bind_only` / `bind_only_prod` / `bo` — the production field's own name — while importing the alias ZERO times.** The reader is asked to re-assemble, nine times, a noun the substrate has already minted one directory up. ⚠ **Second time this ward has found exactly this on this vigilia**: on target 3 it found `wat/rete.wat:326-327` had already aliased the shape a grid axis re-spelled. **The cure is reuse, not invention.** | L2 | **OPEN** · ✅ I VERIFIED | `grep -n 'type BindOnlyFields' session.rs` → `:176`; `grep -rl BindOnlyFields src/` → **5**; `grep -rn BindOnlyFields tests/rete src/rete/kernel/tests` → **0**; raw respellings → **9** |
| **4V2** | perspicere | `gather_probe_cost.rs:552,553,688,695` vs its own runed `:499,509,527,650,661`; also `accum_cost.rs:495-496,513-514,531-532` | **THE RUNE COVERS FIVE SITES AND MISSES FOUR OF THE SAME SHAPE IN THE SAME FILE.** `rune:perspicere(read-once) — gather microbench index; not a domain noun` sits at 5 sites; **4 structurally identical `FxHashMap<K, Vec<usize>>` microbench indexes in the same file carry none.** ⭐ **This is an inconsistency, not an open design question** — `excusare` independently validated the verdict for this exact shape in this exact file as *rune, not alias*, and the file's own precedent rejected reuse even where production aliases `GatherUnary`/`GatherNary` would technically fit. Three more of the shape recur unruned at `accum_cost.rs`. | L3 | **OPEN** · ⚠ ward-reported | closed by adding the matching rune to the 4 (+3) missing sites |
| **4V3** | perspicere | `probe_arc278_rete_defn_recurse.rs:165` | **THE DEEPEST GENUINE NESTING IN THE TARGET — 4 LEVELS.** `let mut runs: Vec<(Option<i32>, Vec<u8>, Vec<u8>)>` — a per-run outcome triple (exit code, stdout, stderr), built once and consumed once inside a single test whose subject is process-blame determinism, not this collection's shape. ⭐ **The ward argued AGAINST minting and I keep its reasoning**: a name here would be single-use, which is precisely what the spell says the `read-once` rune is for rather than an alias. No sibling alias exists. | L3 | **OPEN** · ⚠ ward-reported | `sed -n 165p` → the type. Closed by `rune:perspicere(read-once)` |

| **4G1 ★★** | purgare | `tests/rete/datamancer.src.wat` (whole file); consumer at `probe_arc278_rete_edn.rs:19` | ⭐⭐ **A CHECKED-IN GOLDEN WHOSE GENERATOR NOTHING RUNS AND NOTHING CHECKS.** Of 144 `.wat` in the target, **143 are reached** — 69 by co-location, 76 by explicit literal. **One is reached by nothing:** `datamancer.src.wat`. I grepped the entire repo for its name and got **exactly one hit — its own header**, a comment giving the manual regen command. ⛔ **And no gate covers it either:** the two `.wat`-must-load walks cover `wat-scripts/` and `docs/arc` — **`tests/` is in neither** — and the `.wat.bad` walk only takes `.bad`. **So this file is checked by nothing at all, not even a parse check.** ⭐⭐⭐ **The consequence is the finding, not the deadness.** `:135` says *"Writes tests/rete/datamancer.rete.edn. Invoked only by the wat CLI, never by probes."* It is the declared SOURCE for a checked-in `.edn` that a live test **does** consume. **So the artifact is alive and its producer is unchecked** — an edit here, or a compiler semantics change, silently desyncs the golden from its source and nothing notices. ⚠ It fits none of the four rune categories; the ward named a fifth shape: *"manual regenerator for a checked-in golden, no drift check."* | L2 | **OPEN** · ✅ I VERIFIED | `grep -rn 'datamancer.src.wat' .` → **1 hit, its own header**; `sed -n 135p` → the self-aware line; `grep -n 'collect_wat(Path::new' tests/lint/*.rs` → `"wat-scripts"` and `"docs/arc"`, **never `tests`**. ⭐ Closure: a regen-and-diff gate beats a rune — it makes the desync impossible rather than merely declared |

| **4P1 ★★★** | peragrare | `tests/lint/every_wat_bad_fixture_actually_fails.rs:242-301`; census at `tests/lint/peragrare-bad-census.sh` | ⭐⭐⭐ **THE GATE'S OWN ADVERTISED SELF-CLEARING MECHANISM HAS NEVER FIRED — 6 OF ITS 7 EXEMPTION STATES ARE UNVISITED.** The instrument's exemption path is a **7-state machine** (`absent` · `bad-category` · `short-reason` · `no-owner-field` · `owner-absent` · `owner-live` · `owner-ignored`). **Exactly ONE state has ever been driven by a real fixture** — `owner-ignored`, 3 members. The other six are empty, each with a named defect hypothesis; **all six are L1**. ⛔ **The sharpest is `(clean, owner-live)`**, because the gate's header sells precisely that branch as its reason for existing: *"That makes the exemption **self-clearing** — the day arc 255 lands and the test is un-ignored, this gate goes RED… A rune whose owner passes is not an exemption, it is a stale note."* **No fixture has ever transitioned `Ignored → Live` while this test ran**, so a bug in the upward attribute scan (`:190-203`) would let a stale exemption survive silently. ⭐ 7 further cells are exempt `cell-unconstructible`: the `continue` at `:244-246` skips every declaration read when a fixture errs, so `(err, ⟨any state⟩)` has no reachable control-flow path. Grid **14 = read 1 + hollow 0 + empty 6 + exempt 7**. | **L1** ×6 | **OPEN** · ✅ I VERIFIED, including re-running the census myself | I ran `bash tests/lint/peragrare-bad-census.sh --pins` → **all three pass, moving pin 7 of 7**; the full report → **268 members = 265 `(err,n/a)` + 3 `(clean,owner-ignored)`**, the other six clean cells **0**. `grep -rl 'rune:lint(' $(find … -name '*.wat.bad')` → exactly the 3 named. `sed -n '49,55p'` → the self-clearing promise, verbatim |

| **4D1 ★★★** | secare | `src/rete/kernel/arm.rs:727-728,908`; read at `tests/arm_lease.rs` (19 sites, 5 tests) + `tests/cascade_cost.rs` (3 sites) | ⭐⭐⭐ **A PROCESS-GLOBAL COUNTER DRIVES EXACT-EQUALITY ASSERTIONS IN SIX TESTS, AND ITS RACE-FREEDOM RESTS ENTIRELY ON RUNNER BEHAVIOUR NO TEST FILE STATES.** `ARM_BUILDS: AtomicUsize` is `fetch_add`ed at `:908` and read `Relaxed` by `assert_eq!(after, before)`, `assert_eq!(after, before + 1)` and *"must increment ARM_BUILDS exactly once"* across two files in the shared lib-test binary. **Any other test's `fire-rules` bumping it mid-window silently breaks all of them.** The counter's only rune is `rune:sequi(performance-counter)` at `:727`, which addresses production **domain-purity** — *"carries no domain data"* — **not cross-test isolation**; a sibling ward passed it under that different concern. ⛔⛔ **AND THIS REPO HAS ALREADY RULED ON THIS EXACT SHAPE, IN THIS SAME ARC.** `BRIEF-signal-tests-cannot-race.md` records process-global `AtomicBool`s racing across `#[test]`s and the builder's verdict verbatim: *"the tests **must never have a race, period**… **Not 'green under our runner.' Race-free by construction.**"* That same doc names the trap: *"That reasoning is **true for nextest and false for `cargo test`**."* **`ARM_BUILDS` is that trap, unfixed and undeclared.** | **L1** | **OPEN** · ✅ I VERIFIED | `sed -n '727,728p' arm.rs` → the counter and its `sequi` rune; `grep -rc ARM_BUILDS src/rete/kernel/tests/*.rs` → `arm_lease.rs:19`, `cascade_cost.rs:3`; `grep -n 'never have a race\|Race-free by construction\|true for nextest and false' BRIEF-signal-tests-cannot-race.md` → all three, verbatim. ⚠ **No `rune:secare` category fits** *"safe only because the runner forks per test"* — the sibling doc's own conclusion was a structural wall, not a rune |
| **4R1 ★** | probare | `src/rete/kernel/tests/where_tree_branch_differential.rs:50-54` (+ `:324`, `:428`); `termination_verdict.rs:94-95` | ⭐ **TWO HEADLINE FRACTIONS WHOSE POPULATION LIVES OUTSIDE THE TREE THAT STATES THEM.** `:50-54` reads *"Measured over **this corpus**, 2026-09-04: **115** fixtures fire the branch pair; 39 reach obligation 1 and **34** reach obligation 2… **528** skipped pairs… keeps **34 of 115**."* ⛔ *"This corpus"* is **not this tree**: `walk_corpus()` at `:428` does `let dir = grid_dir()`, defined at `:324` as **`wat-scripts/perf/grid/`** — **target 3's corpus, read by target 4's code.** Neither target's cast knew, and no reader inside `src/rete/kernel/tests/` can check any of those four numbers. Same shape at `termination_verdict.rs:94`: *"371 of 381 corpus rules"* against a fixture corpus of **three synthetic namespaces, one rule each** — `381` appears nowhere in either tree. ⚠ **Neither is called FALSE** — the ward filed both **unverifiable-as-stated within scope**, which is the honest verdict. **The defect is the undisclosed cross-tree population, not the arithmetic.** | L2 | **OPEN** · ✅ I VERIFIED | `sed -n '50,54p'` → the claim; `grep -n 'fn grid_dir'` → `:324`; `:428` → `let dir = grid_dir()`. Closed by naming the corpus's tree in the claim |

## Verified by the orchestrator — target 4

⛔⛔⛔ **`secare` CORRECTED MY PROCESS-STRUCTURE RULING, AND THE CORRECTION CAME FROM THE ONE INSTRUCTION I GOT RIGHT.** I told it `src/rete/kernel/tests/` is *"ONE lib-test binary whose ~125 tests share a process and run concurrently"* — a compilation fact I had verified — and framed the whole cast on it. **It is not the operative mechanism.** `.config/nextest.toml:3-4` reads: *"**each test runs in its OWN forked process**, so the arc-170 execve global-state leak … cannot cross tests."* **Isolation is per-`#[test]`, not per binary**, uniformly across all 613. ⭐⭐ **And that sentence is a COMMENT, not a config key — which is exactly why my grep for `threads|test-groups|serial|max-threads` returned nothing.** I had written *"I did not read that file in full. Read it. If it constrains threading in a way my grep missed, that fact changes this cast and is worth more than any finding."* **It did, it does, and the ward found it corroborated four more times in-repo.**

⭐⭐ **AND THE FINDING SURVIVES THE CORRECTION — SHARPER, NOT WEAKER.** The race cannot fire under the runner that ships. It fires under **`cargo test`**, and the repo has already had that exact argument and settled it: *"Not 'green under our runner.' Race-free by construction."* **So `4D1` is not a hypothetical — it is a standing builder ruling with an unremediated instance.**

⭐⭐⭐ **TWO WARDS INDEPENDENTLY CONFIRMED 613 AND NAMED THE SAME TWO PROSE LINES.** `secare` and `probare` each re-derived the test count, each got **615 raw**, and each identified the identical pair — `probe_arc278_P4c_native_retraction.rs:18` and `probe_arc278_P4a_native_fire_rules.rs:16`, both `//` comments *about* what a `#[test]` needs. **613 confirmed three ways** (`complectens`, and now these two independently), and `vocare`'s 615 is definitively the outlier. ⭐ **The trap this vigilia has recorded six times reproduced itself on my own headline number, and two wards caught it without being told which lines.**

⭐ **`probare` RETURNED FOUR VERIFIED-TRUE CLAIMS AND 21 REAL COMMIT HASHES, WHICH IS THE HALF OF ITS JOB THAT USUALLY GOES UNREPORTED.** It settled `86091edf7` (4 call sites, diff matches) and a `git blame`-confirmed narration at `gather_probe_cost.rs:1042` where a comment claims `g` was *"the one mean among six minima"* that a named commit missed — confirmed exactly, including that the fix landed one day later in a different commit. **21 of 21 sampled hashes resolve and topically match.** It also caught **two of its own grep false positives** — `2097268` and `545259536` are byte counts from allocation measurements, not hashes — and excluded them rather than reporting them.

⭐⭐ **AND THE TRAP-WARNING I GAVE IT WORKED, IN THE DIRECTION THAT COSTS SOMETHING.** I warned that its quarry *is* comments, so *"a comment describing a past state is not asserting a present one — distinguish an assertion from a narration before you call it false."* It met a claim that looks false — *"a guard at slot 3 of 5 failed while the same guard at slot 4 of 5 worked"* — checked the file's own rules (`:when` lists of 4, 4, 6, 4; **none of 5**), and **declined to call it false**, filing it as likely narration of an out-of-scope bug report instead. **Refusing a finding it could have claimed is the harder half of the discipline.**

⭐⭐⭐ **`peragrare` REFUSED MY FRAMING ON ITS OWN SPELL'S GROUND, AND IT WAS RIGHT.** I built its brief around what I called the ⭐⭐ tension: *the comparison reads ONE BIT while **129 of 268** fixtures name an intended error kind in a comment it cannot read.* **It declined to make that an axis**, and quoted the rule that forbids it: an axis must be something the comparison **could come out differently along**, and *"changing which error kind fires, while staying an `Err`, cannot flip this comparison's verdict (still `.is_err()` = true)."* **Admitting it would have been exactly the domain-colour mis-cast the spell opens by warning against** — the one that *"yields a permanent, flattering, meaningless green."* ⭐ **And the axis it derived instead is better than mine**: the exemption **state machine**, 7 branches read out of `:251-301`, of which **one** has ever been driven. My observation survives as an observation; its axis is the finding.

⭐⭐ **I RE-RAN THE CENSUS MYSELF RATHER THAN TAKING THE PINS ON REPORT.** `tests/lint/peragrare-bad-census.sh` is now committed. Under my hand: **populated pin 3, empty pin 0, moving pin 7 of 7** — and the moving pin is the one the spell says earns the census, because this cast's coordinates are **computed**, not read off the fixture. The full report gives **268 = 265 + 3** and six zero cells. **The arithmetic closes at every level**: grid 2×7 = 14 = 1+0+6+7; findings 6 = 6+0+0+0+0; empty 6 = 6 filed + 0 unfiled; the moving pin's denominator (2−1)+(7−1) = 7.

⚠ **IT ALSO DISCLOSED THE ONE THING ITS CENSUS CANNOT SEE, AND THAT DISCLOSURE IS LOAD-BEARING.** AXIS1 (`err` vs `clean`) **cannot be measured without executing `startup_from_file`**, which a read-only cast forbids — so it is **inferred** from declaration presence. A truly-clean-but-undeclared fixture would misclassify as `err`, **which is exactly the `(clean, absent)` cell it filed as unvisited.** The census names this limitation in its own header, so the next reader inherits the caveat with the number. **A census that states what it cannot see is worth more than one that quietly cannot.**

⭐ **AND IT ARGUED ITS OWN CENSUS PATH RATHER THAN TAKING MINE.** I proposed `tests/rete/`; it placed the script at `tests/lint/` because **only 19 of the 268 members sit under `tests/rete/`** — the corpus is repo-wide, so *"beside the corpus"* has no single directory home, and `tests/lint/` is where this repo already keeps corpus-wide walk-and-declare gates. ✅ **Before committing it I confirmed the new `.sh` is inert to every gate**: `every_walking_gate_declares_non_vacuity` scopes to *"a `tests/lint/*.rs` file"*, and `every_parity_script_is_invoked` walks `wat-scripts/perf/grid` for `check-*.sh` — wrong directory and wrong prefix. `[[a-file-landing-in-a-gated-tree-needs-that-gate-run]]`.

⭐⭐⭐ **THE FIXTURE-LIVENESS QUESTION IS ANSWERED — AND IT IS A NEAR-PERFECT POSITIVE.** This is the question I twice measured wrongly (*85/59 named*, then re-aimed but left open), and `purgare` closed it by enumerating **every** reach mechanism instead of grepping for names: **`.wat.bad` 19/19 alive and DOUBLY reached** — the walk-gate takes all 19 regardless of naming, *and* every one is separately named by an explicit literal for a located-error assertion, so there are **zero walk-only stragglers**. **`.edn` 23/23 alive**, each by explicit literal, verified file-by-file with the arithmetic shown (2+3+4+1+4+4+4+1 = 23) — the `__variant` suffix is why a same-stem check finds zero, exactly as the brief warned. **`.wat` 143/144.** ⭐ **A corpus of 186 fixtures with exactly one gap is a strong result about this corpus, and nobody had established it.**

⭐ **AND IT FOUND AN EIGHTH REACH MECHANISM I HAD NOT LISTED** — `Command::new` with a `grid_dir()` of `wat-scripts/perf/grid` — then **correctly ruled it out of scope**, because those two files drive a different corpus entirely. Finding a mechanism and then declining to count it is the harder half.

⚠ **A SEVENTEENTH FIGURE OF MINE CORRECTED, SAME SHAPE AS ALL THE OTHERS.** I gave *"`startup_beside(file!())` — 7 files, 40 call sites."* The files were right; the call sites are **32**. The other 8 hits are `use wat::freeze::{…, startup_beside}` import lines and one doc comment. **A grep for a function name counts its import.**

⭐ **THE WARD CAUGHT ITS OWN INSTRUMENT MID-CAST, AND THE CAUSE IS THIS CORPUS'S OWN CONVENTION.** Its orphan-scan flagged ~100 candidates; six were false, all in `probe_arc278_export.rs`, because **a `// rune:vocare(…)` comment sits between `#[test]` and the fn signature** and broke its attribute-tracking. It corrected the script and confirmed all six are real tests. ⛔ **That is worth keeping as a fact about the corpus:** the runes this vigilia has spent two casts weighing are physically interposed between an attribute and its item, and any future scanner that pairs the two must expect them.

⛔ **I DOUBTED ONE OF ITS CITATIONS AND I WAS WRONG.** It quoted `docs_wat_loads_or_declares_why_not.rs:7` as saying *"It walks `wat-scripts/` **only**"*, which looked like a misattribution — that file walks `docs/arc`, as I had measured myself. I checked: `:7` is a `//!` doc-comment **about its sibling gate** `wat_scripts_fixes_load.rs`, and the ward quoted it accurately. **After sixteen corrections running the other way, this one is mine** — a citation is two claims, and I checked the wrong one.

⭐⭐⭐ **A CLASS, PROMOTED — FOUR WARDS, FOUR RUNE VOCABULARIES, ONE STRUCTURAL DEFECT: A RUNE WRITTEN FOR SOME SITES OF AN IDENTICAL IDIOM AND NOT ITS SIBLINGS.** This is the shape of target 4 and no single ward could have seen it:
- `complectens` **4L2** — `rune:complectens(inline-fixtures)` on **1 of 8** structurally identical cost-benchmark files.
- `perspicere` **4V2** — `rune:perspicere(read-once)` on **5 of 9** identical microbench indexes **in one file**, plus 3 more elsewhere.
- `vocare` **4O1/4O2** — `rune:vocare(vantage-bypass-test)` on `export.rs`'s **7** Aggregate-poke sites but on **neither** of the two sibling files doing the identical poke.

⛔ **AND `excusare` INDEPENDENTLY RULED EVERY ONE OF THOSE RUNES *HOLDS*.** So in all three cases **the cure is to EXTEND the exemption, never to strike it** — the annotated sites are right and the unannotated ones are the same thing undocumented. A future reader landing on an un-runed sibling has no signal its form was already adjudicated. **Three of the five target-4 rows are this one class.**

⛔⛔ **I TOLD `perspicere` ITS LITERAL TRIGGER WORKED HERE. IT DID NOT — 72 HITS, ZERO GENUINE.** My brief said, as a contrast with the previous target's 475-hit noise, *"here you can use the trigger as written."* The ward read every hit and refuted it: my pattern `<[^<>]*<[^<>]*<` demands **THREE** unbroken `<` — **stricter than the spell's own stated "2 or more"** — and the 36 `.rs` matches were embedded `.wat` source inside string literals, doc-comments quoting wat syntax, and **`println!` alignment specifiers**. I confirmed by eye: the sample reads `{:<2}\| {:<6}\| {:<`. **Not one was a type.** It built the correct pattern itself — two adjacent `IDENT<…IDENT<` — and found **55 lines across 17 files**, then read all 55. ⭐ **The irony is exact: I quoted the previous target's 100%-noise trigger as the cautionary contrast, and handed over the same failure from a different cause.**

⚠ **TWO WARDS DISAGREED ON THE TEST COUNT AND I ADJUDICATED IT.** `complectens` reported **613**; `vocare` reported **615** (and 618 raw, correctly identifying 3 as prose inside `.wat` comments). `grep -rhE '^\s*#\[test\]'` over `.rs` only → **613**. **`complectens` is right; `vocare` is off by two.** Both wards measured and disclosed, which is what made the disagreement visible and settleable.

⛔ **AND MY OWN ARCHITECTURE FIGURE WAS WRONG — FOR A FOURTH VARIANT OF THE SAME TRAP.** I handed down *"99 of 100 `.rs` use `startup_beside` or `call_beside_value`."* `vocare` re-derived **65 of 99**. The cause is mine and it is new: I ran `grep -rl 'startup_beside' tests/rete` over the **whole directory**, so **28 `.wat` files mentioning the verb in comments counted as `.rs` usage** — 35 vs the true **7**. ⭐ **The ward's correction improved the conclusion rather than weakening it:** the other ~34 files are not a gap but use adjacent, equally consumer-facing drivers — `call_beside`, `Command::new` against the compiled binary (exactly what a CLI user does), `startup_from_file` + `apply_function`. **Its verdict stands: the corpus's vantage is right by architecture**, and it proved it further by confirming **no test imports a deep internal module path** — every non-`freeze`/`runtime` import is an embedder-facing surface.

⭐ **A HANDOFF `vocare` CORRECTLY DECLINED TO CLAIM.** `probe_arc278_49_one_core_covers_the_surfaces.rs` self-admittedly *"models the shapes locally and cannot be held against the real type"* because `Op` is `pub(crate)`. The ward ruled that a **vacuity** concern, not a vantage breach — *"it never reaches past an interface; it never reaches an interface at all"* — and left it. **That is `peragrare`/`complectens` ground and it is now named for whoever casts them here.**

⭐⭐⭐ **THE 257-FILE QUESTION IS ANSWERED WITH A STRUCTURAL PROOF, NOT AN OPINION.** I handed `complectens` the fact that **257 of 286 files share the `probe_arc278_` prefix** explicitly as a QUESTION — its spell names multi-file stepping stones as an L2, and this could have been the largest instance ever found. **It is not, and the reason is airtight: each file is a separate cargo integration-test binary, so one probe CANNOT call into another's functions — Rust's compilation model rules it out.** `grep -l '^mod \|include!\|#\[path'` across all 257 returns nothing; cross-references between them are prose pointers only. Bodies are uniformly 5-10 lines — the opposite shape of a stepping-stone family. **The numbering is arc 278's build order (data-model → compile → alpha-match → insert → … → accumulate), not one proof cut into pieces.** Framing it as a question rather than a finding is what got a proof back instead of a confirmation.

⭐⭐ **AND IT CHARACTERISED THE CORPUS'S ARCHITECTURE, WHICH NOBODY HAD DONE — THEN RESOLVED A QUESTION I HAD LEFT OPEN.** Only **5 of 144 `.wat`** carry a `:wat::test::deftest` at all; the other **139 are fixture-only** — world and rule data whose correctness is proven by a sibling `.rs`. ⛔⛔ **That directly re-specifies the drive-coverage question in the target-4 muster.** I had measured *85 of 144 `.wat` named in some `.rs`, 59 not* and flagged it as a QUESTION because a name-grep cannot see a path built by concatenation. **It cannot, and here is the exact mechanism**: `src/freeze.rs:987` `startup_beside(caller_rs)` derives `<stem>.wat` **from the calling `.rs` file's own name** — its doc says *"Rename-safe — rename the probe and the derived path follows."* **So a fixture is reached by name-DERIVATION, and being named in the source is not the property that matters.** ⚠ **But this does not close the question, it re-aims it** — I checked, and only **73 of 144** `.wat` have a same-stem `.rs` sibling, so at least three reach-mechanisms are in play (`startup_beside` same-stem, `call_beside_value` with an explicit name, and whatever reaches the rest). **`peragrare`/`purgare` now inherit a named mechanism instead of a bad query — which is a better handoff than either "59 undriven" (false) or "all covered" (unproven).**

⚠ **ONE FIGURE OF THE WARD'S I COULD NOT REPRODUCE AS STATED, AND IT IS ESSENTIALLY RIGHT.** It reported *"95 of the 100 `.rs` call `startup_beside(file!())`"*; `grep -rl 'startup_beside' tests/rete` gives me **35**. The gap is a second verb: `call_beside_value` accounts for **64** more, and 35 + 64 = 99 of 100. **Its claim holds under the right query and my first one was too narrow** — the same shape of error this vigilia has now recorded fifteen times, this time mine and caught in verification rather than by a ward.

⭐ **A POSITIVE THE WARD ESTABLISHED AND I AM KEEPING:** the two largest `tests/rete/` tests — `grid_axes_run_and_derive_nonvacuously` and `every_grid_axis_native_matches_its_oracle` — are **hardened exemplars of the vacuity discipline**, not new instances of it. The first asserts the EXACT SET of on-disk axis stems against a hardcoded list precisely so *"an empty glob or a moved directory fails loudly instead of passing vacuously."* The class-cure from `complectens`'s own earlier cast is visibly working.

⭐⭐⭐ **80 OF 81 EXEMPTIONS HOLD, AND THAT IS THE HEADLINE — NOT THE ONE STRIKE.** `excusare` weighed the largest exemption population of the vigilia (targets 1/2/3 carried 65 / 36 / **zero**). It weighed all 45 live exemptions exhaustively, all 9 `no-inlined-wat` (exceeding its instruction), and **27 of 52** `loose-assert` by a stated rule — *every instance in the two largest carrier files plus at least one from each of the other 13, covering every distinct reason-shape.* **A single strike out of 81 is a well-tended corpus, and the evidence for that is the 80.**

⛔⛔ **A FOURTEENTH HANDED-DOWN FIGURE CORRECTED — AND IT IS THE SAME SHAPE I HAD ALREADY CORRECTED, IN THE SAME BRIEF.** Before casting, I caught that my *"5 `#[ignore]`"* were all **prose discussing** `#[ignore]` (two of them recording its removal) and wrote that correction into the brief as a warning about my other numbers. **The ward then found three MORE instances of that exact shape that I had missed**: one `vocare` hit is a doc-comment *quoting* a rune living in `pass_semantics.rs`; the sole `struere` hit quotes a rune living at `src/rete/kernel/fire/acc.rs:81` — **outside the target**; and both `red-by-design` hits describe a rune in a `docs/` `.wat` that, by the same prose, **has already been struck**. Live population: **106, not 109.** ⭐ **The lesson is sharper than "disclose your contamination": disclosing ONE instance taught the ward the SHAPE, and it found three more of the same class in the same population.** I verified both phantom sites myself — `probe_arc278_import_fold_key.rs:5` and `probe_arc278_match_arm_is_not_a_call.rs:37` are `//!` lines quoting runes that live elsewhere.

⭐⭐ **THE 37 SIBLING-WARD RUNES ARE LEGITIMATE, AND THE REASONING IS THE PRODUCT.** These were the cast's real question: 23 `vocare` + 12 `perspicere` + 2 `complectens` are exemptions from wards that muster on **this same target** — potentially pre-emptive skips of findings a sibling was about to raise. Every one earns it by a check performed **from the code**, not from the annotator's word: each `vocare` site reaches implementer-only state behind a rule with a **verified empty `:rhs`** (so no wat-level query mouth could ever see the result) or a value wat has no constructor for. ⭐ **And the `pass_semantics.rs` quartet does better than legitimate — each rune NAMES ITS OWN COVERAGE GAP and points at the sibling test that closes it.** That is the opposite of a pre-emptive skip: an exemption that argues for itself and then gets checked.

⭐⭐⭐ **ONE RUNE CATEGORY IN THIS REPO IS RE-CHECKED BY A GATE ON EVERY RUN, WHICH IS THE ANSWER TO EXCUSARE'S OWN CENTRAL PROBLEM.** The spell's thesis is that *"a checker never re-judges its own suppressions"* — once written, a rune is trusted until a human looks. **`tests/lint/census_name_read_by_a_cost_test_is_emitted.rs:653` defines `every_census_name_retired_rune_names_a_name_the_engine_no_longer_emits`** — a gate that re-derives the claim of every `rune:lint(census-name-retired)` on every floor run. I verified the function exists at that line. **That is the top of the extirpare ladder applied to exemptions: the rune cannot rot undetected.** It is worth naming as the pattern the other six rune categories do not have.

