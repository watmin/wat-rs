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
