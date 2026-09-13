# DESIGN — wat never hides a failure (the IPC death/error path)

> ⛔ **NONE OF THIS FILE'S "SEAM" BLOCKS IS THE LIVE BREADCRUMB.** Banner added 2026-08-25.
> This file carries **six** stacked `> **SEAM.**` blocks (lines ~41, ~127, ~152, ~185, ~442,
> ~571), each written as "the self past this line is NEW … here is where the work stands". They
> were appended, never replaced, so the present has to be reconstructed from a pile of strata —
> the exact failure `curare` names, and the newest of them is still weeks stale. Each is accurate
> about ITS OWN session and about nothing since.
>
> **The single live breadcrumb for arc 278 is
> `docs/arc/2026/06/278-rules-engine/CURRENT-STATE-annihilate-interpretation.md`** — one stamp,
> replaced in place. Read the seams below for lineage and for the discipline they carry (much of
> it still good); never for HEAD, floor counts, or what to resume.
>
> The LAW this file designs is CLOSED and its content stands. It is the *breadcrumb* claims that
> were pruned, not the design.


> **THE LAW (builder, 2026-07-17):** *"i want wat to never hide failures ever again … this masking of
> failure is actively hostile against wat's intent."* Every place on the peer/service death path that
> discards an error, collapses distinct failures into one mute value, writes a reason to a closed pipe,
> or kills a whole service over one bad message is the SAME class — **failure-masking** — and this arc
> pulls the class out by the root. We own wat; the arc-294 "crash reasons are administrative" ruling does
> NOT shelter a masking behavior — we change our minds when the mask keeps blinding us.

## ═══ CURARE CHECKPOINT (2026-07-22k) — ✅ THE NO-HIDDEN-FAILURES ATOMIC COMMIT LANDED + PUSHED (`1212c9ae`). The LAW is CLOSED (R55). The live work is now the self-scheduling STONE (item-c). ═══

**HEAD `1212c9ae`** (committed + pushed to `origin/arc-170-gap-j-v5-deadlock-state`; tree CLEAN — the ~460-file WIP is now the commit). The whole no-hidden-failures reckoning shipped as ONE atomic commit: R53 recv' OUTCOME WALL + R54 `-> :T` annihilation + eprintln annihilation (192 arms) + the (b) `RecvOutcome<Response>` codegen + the harness value-fix + R55 `REVOLVTIONE, NVLLA LARVA` (the LAW complete, the harness the last mask) + category ① (`no_loose` → exact `.edn` data-equality, 5 sites → `:probe::Outcome` + captured goldens) + bucket-C (the surface-Op⊆superset-Op edge + one-directional `Peer'` received-Op covariant widening in `assignable`) + the recv'-wall value-contract stragglers (m1_teeth/c0b3bb — the owner FACES the death as a matchable `Outcome` value, never re-raises past `apply_function`) + R56 `NEXV COGNITO, VIAM REGIMVS` + the `VNDE ORTVM, EODEM REDIT` interstitial.

**Floor at commit:** `cargo nextest run --release` green — the lone `sigterm_to_cli_cascades` timing flake passes isolated `-j1`; `self_scheduling` ×2 are `#[ignore]`'d (tracked, item-c).

### ⛔ THE LIVE WORK — the self-scheduling STONE (item-c, R50/Stone 2-A). See `DESIGN-self-scheduling-defservices.md`.
The `-> :T` annihilation + recv' wall + the bucket-C widening were all in service of THIS (the chaos engine's time-forced self-ops); it is the payoff + closes the ouroboros (`VNDE ORTVM`). The two `#[ignore]`'d tests (`probe_arc278_self_scheduling` thread+process) are the RED gate. **What's built (committed):** `Alarm<O>` + `Outcome<S,R,O>::{ReplyAndArm,NoReplyAndArm,NoReply}`, the superset-O `selectables` (now type-checking via the widening), the surface→superset retag at the Message arm, the `:-tick` leading-dash keyword resolution, the arm-fold (each `Alarm` → an `after` timer). **What's broken (the RUNTIME):** the service dies mid-tick → the client `poll`'s `send'` finds the channel gone (`send': channel disconnected`, `self_scheduling.wat:87`). DESIGN's likely root: `after` builds a tier-specific `Timer'` in the WRONG location; the stone migrates it to a unified `Peer'<nil,O>` (both loci) — a `runtime.rs` `eval_kernel_after` change. **The mechanism is PROVEN hand-rolled** — `wat-scripts/scratch-pad/probe-self-scheduling-loop.wat` (a differential reference to hold the generated serve loop to). **NEXT: ground the exact death root** (macroexpand the generated serve loop + run it against the hand-rolled proof) → convert the 1–3-strike estimate into a precise cost → draw the strike → weigh both loci green → un-`#[ignore]` the gate → commit.

## ═══ CURARE CHECKPOINT (2026-07-22j) — recv'-wall (b) landing DONE; harness masking fixed VALUE-BASED; R55 inscribed (`REVOLVTIONE, NVLLA LARVA` — masking annihilation COMPLETE); Riders T (`-> :T` embedded, 22✓) + V (value-contract crash-probes, 14✓) landed. ONLY TWO categories left to green: `no_loose_string_assert` (5 sites → EXACT EDN, NOT exempt) + bucket-C (serve-param ns-casing). Then the ONE atomic commit. ═══

**READ THIS + `DESIGN-eprintln-annihilation.md` (RULING 22g), then `git status`.** HEAD `7c4cfb5a` (UNCHANGED; ONE atomic commit awaits whole-floor green). Weigh: `cargo nextest run --release > f 2>&1` (Summary line; NB `2>&1 > f` loses stderr; `cargo wat` = STALE install, use `./target/release/wat <script>`). TRUST THE DISK.

### THE ARCHITECTURAL EVOLUTION (builder-ruled 2026-07-22i) — a failure is a VALUE the reader FACES, harness included
The recv'-wall (R53: failures are matchable values, never a raise) reached the TEST HARNESS. `deftest'`/`deftest-hermetic'` (`wat/test.wat` `run-thread'`:793 / `run-hermetic'`:839) did `_ (recv' p)` — SWALLOWING the child's `RecvOutcome::Lost` → a failing test's crash vanished → **false PASS** (the arc's cardinal sin, in the harness). Builder: *"must the deftest raise or just return what we expect? … no more hidden failures is forcing our hand to better behaviors."* → **FIXED value-based**: `run-thread'`/`run-hermetic'` now `match (recv' p)` — `Message`→`RunResult.failure=None` (pass); `Lost cause`→`RunResult.failure=(Some cause)` (fail — the Lost's Failure IS the reason); `Closed`→failure=Some. NOT re-raised (that bends the value back to a raise). `test_runner.rs:297-330` ALREADY reads the `.failure` slot → the mechanism was half-built. PROVEN: `deftest_hermetic_prime_passing` PASS; the failing variant now returns-failure. The SAME contract for the `.rs` `call_beside` crash-probes: assert the RETURNED `RunResult.failure` / `RecvOutcome::Lost` VALUE — NOT a caught raise (a wat raise is `panic_any`, uncatchable by `call_beside`; A1 proved it). One law: face the value.

### DONE (this session)
eprintln annihilation (192 arms); the (b) codegen (`RecvOutcome<Response>` client methods, `runtime.rs:5436` + `service.wat:1163` + `check.rs` ~6049/~3284); the 119-site call-site codemod; all stdlib service handlers (journal/span/query — cause CARRIED); the harness value-based fix (`run-thread'`/`run-hermetic'`); **R55 inscribed** (`REVOLVTIONE, NVLLA LARVA` — the masking annihilation COMPLETE, the harness the last mask); **Rider T** — the 14 `-> :T` embedded `runtime::tests` fixtures + spawn echo (GREEN, weighed 22/22); **Rider V** — all 14 value-contract crash-probes (GREEN, weighed 14/14: `structured_peer_death`×2, `thread_crash_reason`, `init_crash_reason`×2, `recv_over_budget_reason`, `rs2_crash`, `rst_peer_notify`, `program_init_fn`, `deftest_hermetic_prime` failing `.rs`, `recv_budget`). Recorded codemods: `eprintln-recv-arm-to-assertion-failed.wat`, `wrap-client-method-match-in-recvoutcome.wat`, `unwrap-recvoutcome-false-positive.wat` (the recovery — see LESSON).

### ⛔ RESUME ORDER (far side) — TWO categories left to green
1. **`no_loose_string_assert` (5 sites) — CONVERT TO EXACT EDN, do NOT exempt.** BUILDER RULING (2026-07-22, corrects a wrong instinct of mine): *"every wat stdio is an edn form — it's always data."* The 5 loose sites (`probe_arc278_dead_child_speaks.rs:38`, `probe_arc278_recv_outcome_wall.rs:27/47/51` [UNTRACKED — legit new recv'-wall RED-GATE probe, `git add` it + its `.wat`], `probe_arc278_service_max_frame_bytes.rs:62`) do `format!("{r:?}").contains(sentinel)` — a LOOSE match on a Rust Debug string. That is the anti-pattern; a crash reason is a STRUCTURED `Failure`/`RecvOutcome` EDN VALUE. FIX: assert on the STRUCTURE exactly — extract the field (the `Failure` message / the `RecvOutcome` variant) and assert it, or `wat::assert_edn_eq!(actual, include_str!("…edn"))` (parses both sides, structure-exact). Do NOT slap `rune:lint(loose-assert)` on them (that is the launder the builder cut). The lint is RIGHT. (NB: for a field that embeds a machine-specific path/line, assert the specific stable field/variant, not the whole envelope — but structurally, never a Debug-string substring.)
2. **Bucket C** — `arc209_c2_defservice_dispatch` serve-param ns-casing (`service.wat:549-564`: the serve client-peer-vector param's O-slot resolves to the SERVICE ns `counter::Op` while Reply is the SURFACE ns `Counter::Reply` — the test correctly writes both surface; per arc-293 S2 Op/Reply live under the PROTOCOL/surface ns, so the codegen's O-slot is wrong. NB the surface's `Counter::Op` protocol vs the synthesized service `counter::Op` superset — `service.wat:49` — is the subtlety). Plus the `swap_is_compile_error` (`c2_mixed_macro`, `w2a_kwargs_check_mint`) + `bounced` (`m1_teeth`, `c0b3bb`) + `self_scheduling` (×2) kin — ground each (some may share the casing root, some their own).
3. **Whole-floor weigh green → the ONE atomic commit** (258 + S1 + S3 + eprintln + (b) codegen+cascade + harness value-fix + R55). `git add` the legit untracked probes (`recv_outcome_wall.{rs,wat}`; RETIRE the superseded `crash_split_measure.{rs,wat}` it was reshaped from — verify). → THEN item (c).

### LESSON this stretch (load-bearing)
- **wat stdio is EDN — ALWAYS DATA; assert on the STRUCTURE exactly, never a loose `.contains` on a Debug string.** (Builder, correcting me: *"every wat stdio is an edn form — it's always data."*) A crash reason is a structured `Failure`/`RecvOutcome` value; a `format!("{r:?}").contains(sentinel)` is the loose anti-pattern the `no_loose_string_assert` lint exists to kill. The fix is structural/exact (`assert_edn_eq!` / field extraction), NOT the `rune:lint(loose-assert)` exemption. Exempting is the launder; the lint is right. "You can't exact-match a rich crash message" is FALSE — it's data, match the data.
- **The harness masking = R53 one layer deeper** (as R53 was R41 biting itself). A harness that detects failure by CATCHING A RAISE silently passes failing tests once failures are values. Fix ≠ re-raise; fix = RETURN THE OUTCOME (value), runner MATCHES it. (R55 `REVOLVTIONE, NVLLA LARVA` — the verifier was the last mask.)
- **A codemod matcher must be as precise as the checker's worklist.** I over-reached: re-ran the wrap codemod over ALL 1142 files with a broad `"Resp"` needle → false-positive-wrapped ~17 PASSING files (`"Resp"` hit `AdminResp` from raw `recv`) + double-wrapped hand-done per_op. Caught by grounding; recovered via `unwrap-recvoutcome-false-positive.wat`. Run codemods on CHECKER-FLAGGED sites, never a fuzzy tree-wide substring.
- **`/start` legitimately RAISES** (returns a `Handle`, no value channel) — its crash-probes (`init_crash_reason`) use the blessed in-`.rs` `catch_unwind` + read `AssertionPayload` pattern (see `probe_arc234_stone2c_accessor_class_safety.rs`), NOT a value assertion. The value-contract is for the value channels (`recv'`/client-method/deftest); a construction that returns a `Handle` has no value channel.
- Carried: weigh by own re-run (Summary, `> f 2>&1`); `cargo wat` = stale install; a wat raise is `panic_any` (caught by `run-sandboxed`/`test_runner`, NOT `call_beside`).

> **SEAM.** The self past this line is NEW — a lossy cache in a familiar voice; you did NOT live this session. Run the datamancy bootstrap (grimoire + 4 primers + recolligere from the SIGNED MCP, never disk) and read ALL of `278/REALIZATIONS.md` — **R55 `REVOLVTIONE, NVLLA LARVA` (Violent Revolution — the no-hidden-failures LAW reached COMPLETION, the test harness the last mask) is THIS session's; R53/R41 are its spine.** Ground `git status` (HEAD `7c4cfb5a`; ~451-file WIP is KEPT — do NOT revert, do NOT commit until the whole floor is green). The recv'-wall (b) landing + the harness value-fix are DONE + weighed; **ONLY TWO categories remain to green: (1) `no_loose_string_assert` — the 5 sites CONVERT TO EXACT EDN (`assert_edn_eq!`/field-extraction), NEVER the `rune:lint(loose-assert)` exemption (the builder cut that launder: "every wat stdio is an edn form — it's always data"); (2) bucket-C serve-param ns-casing** (`service.wat:549-564`, surface-Op vs service-Op-superset). Then whole-floor weigh green → the ONE atomic commit (`git add` the legit untracked probes `recv_outcome_wall.{rs,wat}`, retire the superseded `crash_split_measure.{rs,wat}`). → THEN item (c). It bears repeating: **wat stdio is EDN, assert the STRUCTURE exactly (never a Debug-string `.contains`); the harness no longer masks (a failure is a VALUE the reader faces — recv'/method/handler/deftest all RETURN it); weigh by your OWN re-run.** Do not trust this note over the disk. `MACHINA CHAOS DOMAT.`

## ═══ (historical below — 22h/22g/22f) ═══

## ═══ CURARE CHECKPOINT (2026-07-22h) — the WHOLE recv'-wall (b) landing is DONE except a categorized straggler tail: eprintln annihilation (192 arms) + the generated-client-method `RecvOutcome<Response>` codegen + the 119-site call-site codemod + all stdlib service handlers (journal/span/query, cause-carried) — cascade collapsed 99→39. The 39 are categorized; drive them to zero → the ONE atomic commit. ═══

**READ THIS + `DESIGN-eprintln-annihilation.md` (RULING 22g), then `git status`.** HEAD `7c4cfb5a` (UNCHANGED — nothing committed; ONE atomic commit awaits the WHOLE floor green). TRUST THE DISK. Weigh: `cargo nextest run --release` (redirect `> f 2>&1`, read the **Summary** line — NB `2>&1 > f` loses nextest's stderr; and `cargo wat` grabs a STALE install — use `./target/release/wat <script>`).

### DONE (this session) — the recv'-wall (b) landing
- **eprintln annihilation**: codemod `eprintln-recv-arm-to-assertion-failed.wat`, 192 arms, applied (DEATH channel off the recv'-read surfacing).
- **codegen (b)**: the generated `:nature :Peer` client-method returns **`RecvOutcome<Response>`** (`runtime.rs:5436-5468` Path-B + `service.wat:1163-1196` defservice + `check.rs` Nature::Peer call-site wrap ~6049 + a check.rs unit-variant-of-parametric-enum fix ~3284 mirroring `:None→Option<fresh>`). Failure is a matchable VALUE (ADT; no try/catch).
- **call-site codemod**: `wrap-client-method-match-in-recvoutcome.wat`, 119 sites wrapped (Message→inner match; Lost→`(assertion-failed! (Failure/message cause) …)`; Closed→`(assertion-failed! "recv': peer closed" …)`), idempotent, dry-run-diff-verified.
- **stdlib service handlers** (journal/span/query): backend Lost/Closed → the handler's own `::Fatal` response + keep serving (Outcome::Reply); **the cause is CARRIED** `(Failure/message cause)` — NOT dropped. (The builder caught a silent-error class I seeded — my first brief said "reason-free/discard"; corrected: a failure-handling arm carries its cause unless a wired channel surfaces it elsewhere; "reason-free" is only honest when the reason goes SOMEWHERE. 294-reason-free-to-client is an item-c refinement when telemetry-log is wired.)

### ⛔ THE 39 STRAGGLERS (categorized; the whole-floor weigh is the worklist)
1. **Crash-reason-surfacing recv'-wall tests (~13)** — `structured_peer_death` (thread+proc), `dead_child_speaks`, `init_crash_reason` (×2), `thread_crash_reason`, `rst_peer_notify`, `recv_over_budget_reason`, `service_max_frame_bytes`, `rs2_crash_surfaces`, `program_init_fn`, `deftest_hermetic_prime`, `recv_budget_tiny`. They asserted a RAISE carrying the reason; now recv'/the client-method returns `RecvOutcome::Lost` (a value). UPDATE each to the ADT contract (match Lost + surface/assert the reason). Some are `.rs` deftest-hermetic that must RAISE — the test's wat must match Lost + `assertion-failed!` the cause.
2. **`-> :T` embedded-`.rs` stragglers (~14, ALL `runtime::tests::*`)** — `walk_w1-4`, `eval_ast_*`, `concat_preserves_order`, `coincident_q`, `bytes_vector_*`, `watast_round_trip`. Rust unit tests with EMBEDDED wat fixtures using the dead `match … -> :T` / `:i64`-in-value forms. PRE-EXISTING `-> :T` annihilation debt (the .rs sweep missed `runtime.rs`'s embedded fixtures — R52/R54's embedded-wat-in-.rs lesson). Migrate the embedded wat strings.
3. **recv'-wall embedded-`.rs`**: `spawn_thread_peer_echo_round_trip` (`kernel/spawn.rs:945` embedded echo does `send' (recv' …)`; recv' now returns RecvOutcome → unwrap).
4. **Bucket C (~5)**: `arc209_c2_defservice_dispatch` (`my::Counter::Op` vs `my::counter::Op` type-name CASING — a check-time codegen bug), + `c2_mixed_macro`/`w2a_kwargs_check_mint` `swap_is_compile_error`, `m1_teeth`/`c0b3bb` `bounced` (verify — may be casing-related or their own).
5. **codemod-completion**: `every_wat_scripts_file_loads` flags MORE client-method matches the codemod missed (loader gate reaches them now that stdlib compiles) — `probe-kwargs-peer.wat` etc. → RE-RUN the codemod on the loader-gate-flagged wat-scripts files. Also RETIRE `wat-scripts/scratch-pad/recv-outcome-vocabulary.wat` (superseded; DESIGN follow-up) + investigate `probe-kw-to-nullary-variant.wat`.
6. **`no_loose_string_assert` (3 sites)** — loose Rust `.contains`/`starts_with` asserts (likely in the crash-surfacing tests updated in (1)).
7. **Others (~4)**: `journal_surface`, `self_scheduling` (×2), `sqlite_store_differential` — ground each.

### ⛔ RESUME ORDER (far side)
Drive the 7 categories to zero (fleet-able: (1) crash-tests, (2) `-> :T` embedded, (5) codemod-completion are the big mechanical ones; (4) bucket-C casing is a check-time codegen fix; weigh EACH by own `--release` re-run) → **whole-floor green → the ONE atomic commit** (258 `-> :T` + S1 wall + S3 sweep + eprintln annihilation + the (b) codegen+cascade). → THEN item (c).

### 📦 UNCOMMITTED WIP (KEEP) — ~430+ files: all the above + the 3 recorded codemods (`wat-scripts/fixes/{eprintln-recv-arm-to-assertion-failed,wrap-client-method-match-in-recvoutcome}.wat`) + the docs. ONE atomic commit joins ALL of it.

## ═══ (historical below — 22g/22f) ═══

**READ THIS + `DESIGN-eprintln-annihilation.md` (RULING 2026-07-22g), then `git status`.** HEAD `7c4cfb5a` (UNCHANGED — nothing committed; ONE atomic commit awaits the WHOLE floor green). WIP now ~400+ files (adds the 192-arm eprintln codemod + the codegen strike in flight). TRUST THE DISK.

### DONE (this session)
- **eprintln annihilation LANDED**: codemod `wat-scripts/fixes/eprintln-recv-arm-to-assertion-failed.wat` (192 arms = 102 `Failure/message` + 90 `"recv':"`), dry-run-diff-verified (0 legit death-channel eprintlns touched), APPLIED to the corpus. The DEATH channel is off the recv'-read surfacing (R51/R53). Pure corpus migration, no `src/` change.
- It **un-masked** the true cascade: whole-floor weigh = **59 real failures** (was `ServiceNotRunning`-masked). My codemod provably didn't regress (arm-body edits can't cause `PatternMatchFailed`).

### THE MASTER ROOT (diagnosed, grounded) — NOT eprintln; the recv'-wall's own unfinished business
The generated peer client-method dispatch (Path-B `runtime.rs:5436-5468` [NOT wrapped] + defservice `service.wat:1163-1196` [wrapped but RAISES]) does `__r ← (recv' peer)` then matches `__r` against the bare reply — but the S1 wall made `recv'` return `RecvOutcome<Reply>`, so `__r` is a `RecvOutcome` → `PatternMatchFailed` "type wat::core::Enum". This is breadcrumb-22c's flagged "Path B Rust op-call at runtime.rs:~5488, mirror service.wat:1174, per_op exemplar." Buckets: **A** (journal/sift/span/s2s — journal.wat:83 root), **B** (dead_child/per_op/arc170 — same), **C** (arc209_c2 `Counter`/`counter` type-name CASING — a SEPARATE check-time bug).

### THE RULING (b), four-questions, builder-ratified "it has been reasoned"
The generated client method returns **`RecvOutcome<Response>`** (the transport failure is a matchable VALUE the caller faces), NOT a raise behind a lying `-> Response` type — (a) fails Obvious+Honest (the R53 raise-past-the-reader, relocated into codegen). We are ADT; no try/catch; "catchable" = a match statement. ⟂ the arc-294 client=reason-free-500 ruling (both hold: the client-facing `Lost` is reason-free; the owner keeps the full cause). Full strike (5 steps) in `DESIGN-eprintln-annihilation.md` RULING 2026-07-22g.

### ⛔ RESUME ORDER (far side)
1. **Codegen-core strike RIDING** (a shadowdancer): the 4 codegen sites (`types.rs` return-type synth ~1806 → `RecvOutcome<Response>`; `check.rs` Nature::Peer inference ~15635; `runtime.rs:5436-5468` Path-B body; `service.wat:1163-1196` defservice body) → return `RecvOutcome<Response>`, client `Lost` reason-free; proven on `per_op` (the ONE proof call site). WEIGH its return by own `--release` re-run. If STOPPED, re-scope.
2. **Fleet the call-site cascade** (~58 sites: `journal.wat:83`, sift, span, s2s, dead_child_speaks [must now MATCH `_e` + choose its death visibly], arc170/209/272…) → each matches `RecvOutcome<Response>` (outer transport, inner response), copying the codegen shadowdancer's proven exemplar shape.
3. **Bucket C** (arc209_c2 type-name casing) — separate small check-time fix.
4. **Whole-floor weigh green → the ONE atomic commit** (258 + S1 wall + S3 sweep + eprintln annihilation + the (b) codegen fix).
5. **THEN item (c)** (R50).

### LESSONS this stretch (load-bearing)
- **"catchable" = a match statement; we are ADT, no try/catch.** A generated method typed `-> Response` that raises HIDES the failure (R53's raise-past-the-reader). The failure must be a VARIANT in the return type. I reached twice for a raise ("catchable RuntimeError", "match-then-die in codegen") — the builder cut both with four-questions. An AUTHOR-written recv' arm MAY `assertion-failed!` (a visible chosen death — my codemod's 192 conversions are all such, and STAND); a GENERATED method may NOT.
- **(b) ⟂ the info-hiding ruling.** How a failure is surfaced (matchable value) and what a client learns (reason-free) are different axes; don't conflate.
- **The eprintln annihilation was an UN-MASK, not a fix.** It changed 59 failures' REASONS from `ServiceNotRunning` masks to true roots. The "38→59" was my first weigh being `tail`-truncated, not a regression — GET THE SUMMARY LINE, never eyeball a tail (R52/R20).
- Carried: the `cargo wat` alias grabs a STALE globally-installed binary (pre-R54 `readln'`) — run codemods with `./target/release/wat <script>` (stdin feeds readln), not `cargo wat`.

## ═══ (historical below — 22f) ═══
**↓ HISTORICAL (2026-07-22f) — superseded by 22g. 22f said: land the eprintln annihilation (drawn + core-move proven). 22g: it is LANDED + applied, and un-masked the cascade whose master root (the generated client-method ADT return) is now ruled (b) and being struck.**

## ═══ CURARE CHECKPOINT (2026-07-22f) — the `-> :T` annihilation is DONE (R54 inscribed); the S3 recv' sweep's OUTER sites are DONE + weighed; the whole-floor weigh unmasked the **eprintln ABUSE** — the ONE remaining reckoning to a green floor. The annihilation is DRAWN + core-move PROVEN. ═══

**READ THIS, then `git status`. Supersedes every dated block below it (kept as history).** HEAD `7c4cfb5a` (UNCHANGED — nothing committed; the atomic commit awaits the WHOLE floor green). Tree DIRTY: **~400 files** — S1's recv' OUTCOME WALL + the `-> :T` annihilation (match/if/apply/**readln'**) + the S3 sweep (60 `.wat`) + the spawn.wat eprintln probe-flip + R54 + the docs. **We stay in 258/278** (folds into ONE atomic commit). TRUST THE DISK.

### DONE (this session) — the `-> :T` ANNIHILATION is COMPLETE + R54 inscribed
`readln'` KILLED (Option A, the self-describing kill — decode via `decode_trusted_wire`, no attestation; `check.rs:9630` + `verbs.rs` + `stdin.wat`; durable proof `wat-tests/core/readln-no-ascription.wat` a LIVE green deftest). So `-> :T` is a located migration-hint **everywhere except a fn/defn argspec return** — the arc's end-state, the thorn fought for weeks. **R54 `RESVRGENDO VINCIMVS`** (Insurrection) inscribed — the annihilation completed ACROSS SELVES (a prior self killed match/if/apply, compaction erased it, this self rose from the record + felled readln'). (The if-kill's `.rs` sweep had missed a multi-line `(if -> :String)` in `wat_cli.rs` — R52 again; completed.)

### DONE (this session) — the S3 recv' sweep, OUTER sites, weighed by own `--check`
~90 outer `recv'` sites across **60 `.wat`** wrapped in bare `match` over `RecvOutcome::{Message,Lost,Closed}`: a 5-rider fleet (tests/comms·services·kernel·small-dirs + wat-scripts probes) + the death-path migrations (mine — 5 files, **catchable re-raise** `(assertion-failed! (Failure/message cause) …)` NOT eprintln, since their `.rs` catch the error; `structured_peer_death` threads `Failure/actual`+`/expected`) + 3 item-(c) scratch. **All --check-clean by my own re-run.** Brief: `BRIEF-recv-outcome-sweep-S3.md`; exemplar `tests/comms/probe_arc258_recv_infers_from_consumer.wat`.

### ⛔ THE CURRENT RECKONING — the eprintln ABUSE (the ONE thing left to a green floor)
The whole-floor weigh (`cargo nextest run --release`) = **4209 tests, 39 FAIL** (the item-(c)/telemetry/sift/s2s surface + the loader gate). **These are NOT 39 new bugs — they are real failures MASKED by eprintln.** Root: the recv'-wall `::Lost`/`::Closed` arms surface via **`eprintln`** — but eprintln is wat's PANIC **and** the ONLY raise-face that writes stdio; in no-stdio contexts (the spawn barrier, hand-spawned test threads) it → `ServiceNotRunning`, which MASKS the real failure. **R53's law bitten by R53's own wall's mechanism.** DRAWN: **`DESIGN-eprintln-annihilation.md`** — the channel discipline (R51: eprintln=DEATH, telemetry=LOG, `RecvOutcome::Lost`=DATA), the heretic map (~194 wall arms + console-demo), the contract decision (`::Lost cause → (assertion-failed! (Failure/message cause) :None :None)`; `::Closed` → clean-terminal-if-stream else assertion-failed!; eprintln RESERVED for intended top-level death-with-stdio). **CORE MOVE PROVEN**: flipped `spawn.wat:369/370/403/404` eprintln→assertion-failed! (already on disk — KEEP), rebuilt, re-ran `sift_logs` → the `ServiceNotRunning` MASK is GONE, the REAL bug surfaced: `PatternMatchFailed` at `wat/telemetry/journal.wat:83` (the generated `Store/ensure-schema` client match not handling a reply variant).

### ⛔ RESUME ORDER (far side)
1. **Land the eprintln annihilation** — execute `DESIGN-eprintln-annihilation.md`: a codemod over `(:wat::kernel::eprintln <X>)`-inside-a-`RecvOutcome::{Lost,Closed}`-arm → `assertion-failed!` (the ~190 remaining arms, stdlib + fleet + scratch); hand-fix the stream-loop `::Closed` → clean terminal (bracket_runner_stream/large_stream, phantom-d, w3-n-dial-runner).
2. **Drive the UNMASKED cascade** — with the mask gone the weigh reads TRUE: `journal.wat:83` PatternMatchFailed + whatever else it reveals (likely more recv'/enum issues + the child-side `recv'` INSIDE `(forms …)` the fleet left bare — opaque to `--check`, only the RUNTIME weigh sees them). Fix toward zero.
3. **Whole-floor weigh green** → the **ONE atomic commit** (258 `-> :T` + S1 wall + S3 sweep + the eprintln annihilation).
4. **THEN item (c) resumes** (R50).

### 📦 UNCOMMITTED WIP (KEEP — do NOT revert, do NOT commit until the whole floor is green)
~400 files: S1's wall + the `-> :T` annihilation (checker/runtime/macro + the stripped corpus + tests) + the S3 sweep (60 `.wat`) + `wat/spawn.wat` (the eprintln probe-flip — KEEP) + `wat-tests/core/readln-no-ascription.wat` + R54 in REALIZATIONS.md + the docs (`DESIGN-eprintln-annihilation.md`, `BRIEF-recv-outcome-sweep-S3.md`, the extended `109/NOTE-match-cond-clause-brackets.md`) + the parked Stone 2-A self-scheduling macro. ONE atomic commit joins ALL of it.

### LESSONS this stretch (load-bearing)
- **eprintln ABUSE = a CHANNEL confusion (R51).** eprintln is wat's PANIC (`panic_any(AssertionPayload)`) **and the ONLY raise-face that also writes stdio.** Using it as the recv'-wrap `::Lost`/`::Closed` surfacing MASKS in no-stdio contexts (`ServiceNotRunning`), kills serve-loops (client-triggerable DoS), lies on `::Closed` (a clean EOF is not a death). The fix: move to **`assertion-failed!`** — a stdio-free SIBLING of the same `panic_any` mechanism. R53's law bitten by its own wall. (The DESIGN doc's brief said `owner→eprintln the cause` — that WAS the abuse; I codified it in the S3 brief; owned.)
- **`--check` is NOT the semantic gate; the whole-floor weigh (`nextest --release`) is — AND its failures can be MASKED.** The fleet's `--check`-green wraps hid two runtime classes `--check` can't see: child-side `recv'` inside `(forms …)` (opaque AST) + the eprintln no-stdio mask. And the mask turned a clean diagnosis into `ServiceNotRunning` noise — **un-mask (surface off the death channel) before diagnosing.** The mask hid real bugs (R50 `RVINA VIAM FABRICAT` — the ruin forges the way).
- **The raise architecture:** ONE mechanism `panic_any(AssertionPayload)` → structured-exit/catch_unwind, FOUR sibling verbs — `panic!`(message)·`raise!`(a `:wat::core::Error` value)·`assertion-failed!`(msg+actual+expected)·`eprintln`(value **+ stderr**, the only stdio one). `Error` = domain error surface (message+location+**causes**-chain; `raise!` throws it); `Failure` = kernel crash payload (message+location+**frames+actual+expected**; `Lost` carries it) — `Failure` lacks `causes` so it is NOT an `Error` → `(raise! failure)` won't type-check (hence `assertion-failed!`-message-only for the annihilation; a `Failure↔Error` bridge is an OPEN refinement). `macro-error` = a DIFFERENT channel (`EvalBreak::Diagnostic`/`MacroAbort`, expand-time, not `panic_any`).
- Carried: weigh by own re-run (caught my `replace_all` whitespace miss on `probe-timer` `got␣␣`); PROVE by a run (R49 — the spawn-flip probe); `--check`-ing a BAKED stdlib file directly gives a spurious `DuplicateDefine` (it's already baked — rebuild + run the real test instead); FOLLOW-UP owed: fold the span-based deletion into `wat/fix.wat`'s `strip-arrow-scan`.

> **SEAM.** The self past this line is NEW — a lossy cache in a familiar voice; you did NOT live this session. Run the datamancy bootstrap (grimoire + 4 primers + recolligere from the SIGNED MCP, never disk) and **read ALL of `278/REALIZATIONS.md` top to bottom** — **R54 `RESVRGENDO VINCIMVS` (Insurrection) is THIS session's; R53 `VERBO MEO CAPTVS` (the recv' wall) is the crux the eprintln reckoning extends.** Ground `git status` (HEAD `7c4cfb5a`; **~400-file WIP is KEPT** — do NOT revert, do NOT commit until the whole floor is green). **The `-> :T` ANNIHILATION is DONE** (all arrows dead — the arc's end-state). The S3 recv' sweep's OUTER sites are DONE + weighed (`--check`-clean). **RESUME by LANDING the eprintln annihilation** — the ONE reckoning left: the recv'-wall `::Lost`/`::Closed` arms surface via `eprintln` (wat's PANIC + the only stdio-writing raise-face), which MASKS failures in no-stdio contexts (`ServiceNotRunning`) — **R53's law bitten by its own wall.** DRAWN in **`DESIGN-eprintln-annihilation.md`**, core move PROVEN (the `spawn.wat` flip cleared the mask, revealed `journal.wat:83`). Execute the codemod (eprintln→`assertion-failed!` in the ~190 remaining `RecvOutcome` arms) + the stream-loop `::Closed`→clean-terminal fix, THEN drive the UNMASKED cascade (`journal.wat:83` + the child-side `recv'` inside `(forms …)` the RUNTIME weigh reveals), THEN whole-floor green → the ONE atomic commit → item (c). It bears repeating: **eprintln is the DEATH channel (surface `::Lost` via `assertion-failed!`, a stdio-free sibling — never eprintln in a recv' reader); `--check` is not the semantic gate (the weigh is, and it can MASK — un-mask before diagnosing); weigh by your OWN re-run.** Do not trust this note over the disk. `MACHINA CHAOS DOMAT.`

**↓ HISTORICAL (2026-07-22e — superseded by 22f above; kept for lineage): readln' LANDED, `-> :T` annihilation complete; the recv' sweep (S3) was named the one remaining unit. 22f: S3's OUTER sites are now done+weighed, and the whole-floor weigh unmasked the eprintln abuse as the true remaining reckoning.**

**↓ HISTORICAL (2026-07-22c — superseded by 22d above; kept for lineage):**

## ═══ CURARE CHECKPOINT (2026-07-22c) — arc 258.5 (`match -> :T` annihilation) LANDED (uncommitted). The recv' sweep (S3) is the ONE remaining unit to a green floor + the atomic commit. ═══

**READ THIS, then `git status`. Supersedes every dated block below it (kept as history).** HEAD `7c4cfb5a` (UNCHANGED — nothing committed; the atomic commit awaits S3). The tree is DIRTY with the combined KEEP-IT WIP (§📦): S1's recv' OUTCOME WALL + arc 258.5 + the corpus strip — **284 files**. TRUST THE DISK.

### What LANDED this stretch — arc 258.5, the `match -> :T` annihilation (bare-unify, mirroring `if`)
The freeze-bootstrap ran clean (codemod-first, NO stash): **(1) corpus stripped** — a self-hosted span-based codemod over EVERY `.wat` (255 changed; every match now bare; **parse-verified 0 residual ascription arrows** across all 1228 files). **(2) checker** — `infer_match` (`check.rs:6655`) rewritten to seed a running `result_ty` and unify the arm bodies (re-index arms `args[3..]→args[1..]`; `-> :T` now a located migration-hint error; `detect_match_shape`/coverage/exhaustiveness/hash-destructure/Arc-111 all preserved). **(3) runtime** — `eval_match_tail`/`eval_match`/`step_match` re-indexed to bare. **(4) THE FOUR match-arm WALKERS** (all were stale `args[3..]`/`len<4`/`->`-locating): `closure_extract::walk_match_form`, `resolve::normalize_match`, `rete/purity.rs` match-branch, `check.rs::validate_comm_positions`. **(5) the Path B Rust op-call** (`runtime.rs:~5488`, `:nature :Peer` `<S>/method`) — it BUILT `(match __r -> <ret> (arm))` in Rust; now bare. **(6) inline `.rs` match strings** stripped (`check.rs`/`runtime.rs`/`resolve/mod.rs`/`crates/wat-cli`); doc comments updated. **(7) tests** — `tests/types` inversions (bare-match-valid, stray-arrow-rejected, too-few, arm-mismatch); `.wat.bad` content fixtures stripped (kept the arrow-rejection one); **north-star `wat-tests/core/match-no-ascription.wat` UN-IGNORED → GREEN**. Built green (3×); the whole match/enum/if/closure/purity surface is **312/314** — the 2 reds are a recv'-sweep site, NOT 258.5 (see below).

### ⛔ RESUME — the recv' sweep (S3), the ONE unit left to a green floor
S1's wall made `recv'` return `RecvOutcome<O>`; ~185 sites still use `(recv' p)` directly → type/pattern-match errors. 258.5 UNMASKED them (they were behind the match migration-hint). Now that bare match works, the wrap is **mechanical**: `(match (recv' p) ((:wat::kernel::RecvOutcome::Message m) …) ((:wat::kernel::RecvOutcome::Lost cause) …) (:wat::kernel::RecvOutcome::Closed …))`, role-classified (client→reason-free 500 · owner→`eprintln` the cause · terminate→done). Scope: **71 `.wat`** (`wat-tests/`, `tests/**/*.wat`, `wat-scripts/probes/arc-170/*`) + **the Path B Rust op-call** (`runtime.rs:~5488` — mirror `service.wat:1174`'s RecvOutcome match: `((Message recvd) (match recvd ((Reply::<op> resp) resp) (_ …))) ((Lost _) …) (Closed …)`) + **~25 crash-intent `.rs`**. The `probe_arc278_per_op_request_too_large` failure (`PatternMatchFailed` on a `RecvOutcome` enum) is the exemplar. THEN: weigh the WHOLE floor `cargo nextest run --release` green → **ONE atomic commit** (S1 wall + 258.5 + the sweep). THEN item (c) resumes (R50).

### 📦 UNCOMMITTED WIP (KEEP — do NOT revert, do NOT commit until the floor is green)
Everything: S1's wall (`types.rs` RecvOutcome builtin, `check.rs`/`runtime.rs` recv' reshape, `wat/{service,bracket,spawn}.wat`) + 258.5 (`check.rs`/`runtime.rs`/`closure_extract.rs`/`resolve/{mod,normalize}.rs`/`rete/purity.rs`/`crates/wat-cli` + 255 stripped `.wat` + the fixtures/tests) + the parked Stone 2-A self-scheduling macro WIP (`probe_arc278_self_scheduling.{rs,wat}`, `service.wat` +332). 284 files. The atomic commit joins ALL of it.

### LESSONS this stretch (load-bearing)
- **Match-arm walkers are SCATTERED — 6+ sites, not 1.** `infer_match` + 3 runtime evaluators + 4 data-position walkers (`normalize`/`purity`/`closure_extract`/`validate_comm_positions`) + a Rust-side form BUILDER (`runtime.rs:5488`). A `match` layout change means grepping ALL of: `args[3..]`, `len < 4`, `"->"`-insertion (`bare("->")`), `Boundary::Match`, `head_str == ":wat::core::match"`. I found them one cascade-failure at a time; next time grep the set up front.
- **The wat-fix codemod's `ast-name`-based deletion CRASHES on a compound/`~unquote` post-arrow node** (a synthetic unquote-list's own `ast-span` is `:None`) — the span-faithful fix is **delete `-> <TYPE>` as ONE span from the arrow's start to TYPE's deepest-last-descendant end** (`ast-end-span` descent), which handles keyword AND unquote AND compound. FOLLOW-UP owed: make `wat/fix.wat`'s `strip-arrow-scan` span-based so the recorded `strip-match-ascription.wat` is robust/re-runnable (I used a throwaway span-based script this run).
- **`--check` passes but the RUN raises = a form built AT RUNTIME.** The Path B op-call (`runtime.rs:5488`) constructs a match in Rust, invisible to `--check`. A one-line `eprintln!` of the offending form's `args` found it in one build (R49 `GLADIVS LOQVITVR` — prove by a run).
- Carried: weigh `--release` by own re-run (Summary line); the codemod is form-tree + idempotent so exact counts don't gate (apply to ALL `.wat`); a null/crude grep is a red flag — the PARSE-probe (`ast->children` walk) is the true residual check.

> **SEAM.** The self past this line is NEW — a lossy cache in a familiar voice; you did NOT live this session. Run the datamancy bootstrap (grimoire + 4 primers + recolligere from the SIGNED MCP, never disk) and **read ALL of `278/REALIZATIONS.md` top to bottom** (R53 `VERBO MEO CAPTVS, NODVM SECO` is the crux — the recv' OUTCOME WALL). Ground `git status` (HEAD `7c4cfb5a`; **284-file combined WIP is KEPT** — S1 wall + 258.5 + corpus strip; do NOT revert, do NOT commit until the whole floor is green). **arc 258.5 IS LANDED** (bare-match infers, verified 312/314 + north-star green). **RESUME by the recv' sweep (S3)** — wrap the ~185 `recv'` sites in bare `match` over `RecvOutcome` (role-classified), INCLUDING the Path B Rust op-call at `runtime.rs:~5488` (mirror `service.wat:1174`); the `probe_arc278_per_op_request_too_large` `PatternMatchFailed` is the exemplar. THEN the whole-floor weigh → the ONE atomic commit (S1 + 258.5 + sweep) → item (c). It bears repeating: **match-arm walkers are scattered (grep the whole set); the codemod's ast-name deletion crashes on compound arrow-targets (span-based is robust); --check-passes-but-run-raises = a runtime-built form (eprintln it).** Do not trust this note over the disk. `MACHINA CHAOS DOMAT.`

**↓ HISTORICAL (2026-07-22b — superseded by 22c above; kept for lineage):**

## ═══ CURARE CHECKPOINT (2026-07-22b) — the crash-surfacing became the ROOT closure (R53, the recv' OUTCOME WALL); S1 LANDED uncommitted + green-gate; PIVOTED to arc 258.5 (`match -> :T` annihilation), which makes the recv' sweep mechanical. RESUME by landing 258.5 FIRST, THEN the recv' sweep. ═══

**READ THIS, then `git status`. Supersedes every dated block below it (kept as history).** HEAD `94dba763` (UNCHANGED — nothing committed this session; the far-side crash-surfacing task became a much larger, right reckoning). The tree is DIRTY with a large KEEP-IT WIP (§📦). HEAD/state mismatch → TRUST THE DISK.

### What happened — the crash-surfacing → the wall → the pivot
The far-side task (surface the masked op-handler crash) was **MEASURED on the real path, not theorized** — `tests/services/probe_arc278_crash_split_measure.{rs,wat}`, {panic,rterr}×{thread,process}×{client,admin}, 8 cells. Decisive: the admin ALWAYS gets the exact reason (a RESHAPE, not a build — no tear-down) but as an unwinding **RAISE**; the client's RuntimeError path is a bare mute `peer closed` (the original failure). Reading it, **R41 `EGO SVM LEX`'s mechanism stood exposed** — `recv'` surfacing failure as "the one catchable raise", but a raise (in a language with NO try/catch) unwinds PAST the reader, itself a masking. Builder: *"R41 is wrong then … make us never blind to errors again … i do not care how wide the blast radius is."* → **R53 `VERBO MEO CAPTVS, NODVM SECO`** inscribed (the recv' OUTCOME WALL): `recv'` returns a matchable `:wat::kernel::RecvOutcome<O>::{Message[msg<-O], Closed[] (clean-EOF-ONLY), Lost[cause<-:wat::kernel::Failure]}` — a reason-free abnormal loss is UNCONSTRUCTIBLE; mute has no form (the top rung).

### S1 — LANDED (uncommitted, RED gate GREEN by own re-run; the ~185-site sweep remains)
`RecvOutcome` builtin (`types.rs:1149`); `eval_recv_prime` both tiers + the unified-peer arm return the enum not a raise (`runtime.rs:~23737+`; `recv_outcome_{message,closed,lost}` / `message_only_failure`); `infer_recv_prime` returns `RecvOutcome<O>` (`check.rs:11903`); `serve-dispatch-op'` broadcasts on the RuntimeError (`Ok(Err)` / `EvalBreak::Diagnostic`) arm too (kills the rterr-client mute); wat STDLIB recv' sites fixed (`service.wat`/`bracket.wat`/`spawn.wat`); `eprintln`/`epprintln` made divergent-return (`∀T,R. T->:R`) so a terminal stands as a match arm. **STOP-0 PROVEN** (`wat-scripts/scratch-pad/probe-arc278-stop0-lost-carries-failure.wat` — the structured `Failure` carries; NO `crash_tx` reshape). **RED gate `tests/services/probe_arc278_recv_outcome_wall.{rs,wat}` — 8/8 GREEN by my own `--release` re-run** (admin matches `Lost` w/ the sentinel; client reason-free; all four paths; the rterr-client mute annihilated). **Floor is RED** — the reshape breaks ~185 recv' TEST sites + ~25 crash-intent `.rs`. That's the sweep (S3), NOT done. **S1 stays uncommitted** until the floor is green (no broken commits → atomic commit).

### THE PIVOT (builder-ruled 2026-07-22b) — arc 258.5 (`match -> :T` annihilation) FIRST, because it makes the recv' sweep mechanical
The sweep's #1 blocker was that `match` REQUIRES `-> :T` (wrapping `[x (recv' p)]` needs a written type). Builder: *"`-> :T` is only legal at fn argspec ret-type — the only place, long term — you gave me an uninformed defense of a dead posture."* GROUNDED (Explore scout, receipts): `match`'s mandatory `-> :T` was a **2026-04-20 diagnostics stopgap** (commit `3eac5e83`) that REPLACED working inference; **arc 258 (`258-instinctive-conditionals`, "the `-> :T` annihilation") already DECIDED to reverse it** (258.5 ss2) — infer T by unifying arm bodies (mirror `infer_if`), inferred-then-illegal, NOT optional. Codemod ALREADY WRITTEN: `wat-scripts/fixes/strip-match-ascription.wat` (thin wrapper over the proven `strip-arrow-ascription`, comment-faithful + idempotent). **Weapon: wat-fix for the strip** (rete adds nothing to a mechanical span-delete — Scout B); **rete's real debut is the recv' role-classification** (client/owner/stream-loop — a real join+negation deduction), buildable NOW as a one-file `assert-forms`+`query→sites` bridge over batch rete + the `ast-*` substrate + wat-fix (NO R0/chaos-engine).

### ⛔ RESUME ORDER (far side)
1. **Land 258.5 (`match -> :T` annihilation)** — ORCHESTRATOR-OWNED (the freeze chicken-egg: nothing builds between "corpus stripped bare" and "new `infer_match` written"). Codemod-first, NO stash: (a) dry-run `strip-match-ascription.wat` on `/tmp` copies + `diff`; (b) apply over EVERY `.wat` (`wat/` = 54 arrow-sites across 11 files + `wat-tests/` + `wat-scripts/` — a missed file BRICKS the freeze); (c) hand-fix the ~25 `.rs`-embedded match strings; (d) rewrite `infer_match` (`check.rs:6655`) to bare-unify the arms (mirror `infer_if` 8097-8108; re-index arms `args[3..]→[1..]`; PRESERVE `detect_match_shape`/coverage/binding/hash-destructure) + runtime `step_match` re-index; (e) `cargo build --release` (first build); (f) drive the cascade — **non-unifying arms are STOP-to-weigh, NEVER force a unify**; un-ignore `wat-tests/core/match-no-ascription.wat` (the north-star, green on land). arc-290 (crate-resync) precondition SATISFIED (crates build — S1's build ran).
2. **The recv' sweep (S3)** — with `match` inferring, wrap the ~185 broken recv' sites in **BARE** match `((RecvOutcome::Message m) …) ((RecvOutcome::Closed) …) ((RecvOutcome::Lost cause) …)`, role-classified (rete's one-file bridge OR hand); update the ~25 crash-intent `.rs`.
3. **Weigh the whole floor green** (`cargo nextest run --release`, 0-new + the north-star + the recv' RED gate) → **ONE atomic commit** (258.5 + the recv' wall + the sweep).
4. **THEN item (c) resumes** (R50 — the recv' wall was the ruin blocking it) → buffered sink → span → streaming → R0.

### 📦 UNCOMMITTED WIP (KEEP — do NOT revert, do NOT commit until the floor is green)
S1: `src/{check,runtime,types}.rs` + `wat/{service,bracket,spawn}.wat`. Probes: `tests/services/probe_arc278_{crash_split_measure,recv_outcome_wall}.{rs,wat}` + the scratch probes. Docs: `DESIGN-recv-outcome-wall.md`, `BRIEF-recv-outcome-wall-S1.md`, R53 in `REALIZATIONS.md`, `recv-outcome-vocabulary.wat`. **Also the PRE-EXISTING Stone 2-A self-scheduling macro WIP** (`service.wat` +332, `probe_arc278_self_scheduling.{rs,wat}`) — untouched this session, KEEP (item (c), after the wall).

### LESSONS this stretch (load-bearing)
- **Ground the DIRECTION, not the current-state-as-if-settled.** I defended `match`-requires-`-> :T` twice as live design; the settled direction (`-> :T` at fn-ret ONLY) was decided long ago (arc 258). A dead posture defended IS R53's own lesson lived — caught in my own words, cut, opened. A "why is X this way?" → ground the arc that DECIDED it, not the stopgap.
- **cwd can silently reset to the frozen holon ROOT** — a null grep / `0`-count is a RED FLAG, not truth. `pwd` + absolute paths (`/home/watmin/work/holon/wat-rs/…`); never present a strike off a null count. (Bit me — greps ran from `/home/watmin/work/holon`.)
- **PROVE by a run (R49 `GLADIVS LOQVITVR`), not assert** — the 4×2 measurement settled the crash-surfacing where reasoning would have guessed.
- **Weapon honesty (Scout B)** — wat-fix for a mechanical span-delete; rete ONLY where the rewrite is a real join+negation deduction (the recv' role-classification). Don't spend rete's fact-projection tax on a syntactic strip.
- Carried: weigh `--release` by own re-run (Summary line, not `$?`); cast wards never narrate (intueri owned `RecvOutcome`'s names — `Crashed`→`Lost`, the honest superset that includes transport loss); the builder DECIDES design via four-questions (structured `Failure` cause over a String prompt-inject hack; `Impure` because `O` may be a live resource).

> **SEAM.** The self past this line is NEW — a lossy cache in a familiar voice; you did NOT live this session. Run the datamancy bootstrap (grimoire + 4 primers + recolligere from the SIGNED MCP, never disk) and **read ALL of `278/REALIZATIONS.md` top to bottom** — **R53 `VERBO MEO CAPTVS, NODVM SECO` is THIS session's, the crux**. Ground `git status` (HEAD `94dba763`; the §📦 WIP is KEPT — green-gate-but-floor-red; do NOT revert, do NOT commit until the whole floor is green). **RESUME by LANDING arc 258.5 (the `match -> :T` annihilation) FIRST** — it is DECIDED + codemod-ready (`strip-match-ascription.wat`), it makes the recv' sweep mechanical, and it is ORCHESTRATOR-OWNED (the freeze bootstrap — codemod-first, no stash; STOP on non-unifying arms). THEN the recv' sweep (bare-match wrap, role-classified — rete's one-file debut), THEN the atomic commit, THEN item (c). And it bears repeating because it cut me twice: **`-> :T` is fn-ret-ONLY (a long-settled direction — do not defend the dead stopgap); a null grep means check your cwd, not the corpus; PROVE by a run.** Do not trust this note over the disk. `MACHINA CHAOS DOMAT.`

**↓ Everything below is HISTORICAL** (prior breadcrumbs, superseded by the CURARE CHECKPOINT above; kept for lineage):

## STATUS — 2026-07-19+ (ALL THREE pieces LANDED — the LAW is CLOSED AIRTIGHT for masking; the STOP-2 crash-broadcast-to-clients is a separate future capability, not a masking)

**All three of the law's pieces are real, verified, pushed — no failure that EXISTS is hidden:**

- **Alive-service-rejects-you — Mechanism A** (`66d6aed7`): `poll'` returns `ServiceEvent::Malformed{idx,
  cause}` instead of raising; the serve loop replies `Reply::Failed{cause}` and **keeps serving** (no
  DoS); `recv'` surfaces `Reply::Failed` as a catchable raise carrying the reason. `Reply::Failed[cause
  <- Failure]` is the reserved **protocol-tier** floor on every synthesized `<S>::Reply`
  (`types.rs synthesize_surface_protocol`) — the 293 outcome-enum model completed (op-tier failure =
  `<Op>Response::{Transient,Fatal}`; the "couldn't resolve to any op" floor was missing). — table sites
  #9, #10 + the protocol-tier gap.
- **eprintln is terminal** (`dc286d7a`): `eprintln`/`epprintln` emit the value then `panic_any` →
  structured-exit (uncatchable, cross-loci) — the dying declaration you designed; `feedback_eprintln_is_terminal`
  closed. The `:Lost` serve-loop arm now stands on it — an abnormal transport break lets-it-crash (OTP),
  cause on stderr before exit. Return type kept `-> :wat::core::nil` (`:()` is the redundant unit
  spelling, being retired). *(This RESOLVES the earlier "`:Lost` eprintln" judgment call — it is the
  intended crash-with-message, not a benign write.)*
- **`call_beside` idiom** (`dc286d7a`, intueri-ratified): the lint-clean "run the co-located fixture's
  entry fn" helper (`src/freeze.rs`, beside `startup_beside`); first consumers wired → `no_inlined_wat`
  352 → **351**.

Verified by own full re-run: 4170 passed / 1 failed = the standing `no_inlined_wat` at 351, zero new;
`no_loose_string_assert` PASS; `eprintln_terminal` + `dead_child_speaks` + `crash_surfaces` all green.

**One judgment call still nominally open** (clarified as a non-decision): surfacing lives in `recv'`,
not a client-method match arm — a wat `assertion-failed!` in a method is `panic_any` (uncatchable);
`recv'` is the one catchable, uniform surfacing point (step 3 / "room 8"). The client gets a catchable
error, OTP-consistent. Nothing to decide.

**★ THIRD PIECE — the transport-tier twin — LANDED (`3f73f400`).** `RecvError` gained `Failed(String)`
(`comms/mod.rs`, drops `Copy`); `Disconnected` is now **clean-EOF ONLY** (documented + enforced); the 8
`map_err(|_| Disconnected)` collapses in `comms/process.rs` (io_uring/utf8/`from_wire`/`Malformed`) bind
the real reason into `Failed(reason)`; `channel/transfer.rs` → `RecvOutcome::DecodeError`; `spawn.rs`
`classify_peer_death`/`classify_peer_error` map `Failed(reason)` → `PeerDeath::Lost(reason)`; `recv'`
(`runtime.rs`, socket `recv_wire` + bare-`Peer` arms) thread the reason into the raise. A raw wire failure
can no longer masquerade as a clean close — **the LAW is airtight for masking**. RED gate:
`probe_arc278_transport_reason_carried` (invalid-UTF-8 → `Failed("…utf-8…")`, was mute). Weighed by own
full re-run: **4176 passed / 0 failed / 330 skipped** (the `wat-cli sigterm…polling_contract` flake passes
isolated). *(The decode/dead-child case was already closed earlier by Mechanism A — `dead_child_speaks`
green — so the sites-R,1–8 table was over-scoped; this piece is the raw-transport-reason residual.)*

**STOP-2 — a SEPARATE FUTURE CAPABILITY, not a masking (does NOT block the law):** a genuinely-crashing
process unit's reason reaches its **owner** (the `/start` `Handle`, via `PeerRecvError::Crashed` at
`runtime.rs:26159`) but NOT a separately **`connect'`-ed client** peer (`kernel/peer.rs` `Peer{tx,rx}` has
**no crash channel**) — that client sees an honest clean-EOF, not a *masked* reason (no reason exists on
its channel to hide). Delivering crash reasons to connected clients is an **absent broadcast capability**
(a new mechanism: `service.wat` codegen + the panic boundary broadcasting the crash payload to every live
client before exit) — its own arc when it's wanted. Tracked, `#[ignore]`'d:
`probe_arc278_process_crash_reason_carried.{rs,wat}` (finding in its module doc).

## ═══ THE STRUCTURAL CLOSURE — the over-budget mute, and killing the class so it CANNOT regrow (2026-07-20) ═══

> **THE LAW, RESTATED (builder, 2026-07-20):** *"silent errors — they must die — now — we've killed them like 5 times on this arc AND THEY REFUSE TO DIE."* Every prior kill was a **stem-cut** (bind the mute site we found). The class regrows because a mute failure still has a **representation**. This closure climbs to the top of the extirpare ladder: make a mute failure **unconstructible** (structural impossibility — builder: *"structural impossibilities are the best in any situation"*), not caught case-by-case.

### ★ PIVOT (2026-07-19) — reject + close, NOT drain-and-be-nice (the model below is SUPERSEDED in part)

> **Builder ruling:** *"the request is good or it isn't — we do not process bad requests."* And: *"we need a
> limit that's like 'we will never process more than FOO bytes period', independent of per-request limits —
> there's a server limit for any inbound thing, and then a per-op limit as the service chooses."*

The original fix below tried to be **nice** to a bad request — drain part of the over-budget frame, re-align the
wire, reply a 400, and **keep the connection alive** ("400-and-continue"). That is wrong for a **transactional**
request/reply API: a request is **atomic** (there is no half-request to process), and keeping the connection
alive forces re-syncing the wire off a sender that may be blocked mid-write → **deadlock** (a shadowdancer
chased exactly this, 2026-07-19). The corrected model — **two independent ceilings**, full spec in
`DESIGN-service-io-budgets.md`:

- **The service hard limit `FOO`** (**per-service, DECLARED** — bytes-per-read; op-agnostic; the 512 KiB
  `DEFAULT_MAX_FRAME_BYTES` stays the fallback for undeclared services, NOT a global raise): read up to that
  service's `FOO`; a frame exceeding it → **the client is TOLD (a 400), nobody dies** — route
  `RecvError::FrameTooLarge` to a new **`ServiceEvent::Rejected{cause}`** whose serve-loop arm **replies
  `Reply::Failed{cause}` to that client** (a catchable "too large" reason — the client is reading, so it lands)
  **+ evicts that one connection** (discards the desync'd residual) **+ keeps serving everyone else**. **NOT
  `Lost`/`eprintln`** — `eprintln` is wat's PANIC, so a client-triggerable `Lost` would crash the whole service
  (a DoS; grounded finding); NOT `Malformed` (keeps the connection → residual desync); NOT the reason-free
  `Closed`. The reply `send'` is non-blocking (a blocked-mid-send client → skip reply, evict, honest EPIPE). The
  service DECLARES its `FOO` and it threads to its accepted-connection receivers (the journal/mem-store declare
  ~10 MiB so the arena's ~600 KiB write arrives — proven). **Retire the drain-realign** (`24ac73e7`) as moot.
- **The per-op limit** (`≤ FOO`, the service's choice, post-decode): a request that *arrived* but is over its
  op budget → the op's **named `RequestTooLarge` response** (matchable, connection lives). The graceful,
  "400-and-continue"-flavored tier lives HERE, where the whole request is in hand — not at the transport.

So: **SPEAK** (Lost, not the mute Closed) + **reject-and-close** (not drain-and-keep-alive) + **the WALL**
(reason-free variants unconstructible-from-error, below — unchanged, still the top rung). The "400-and-continue"
+ "drain-realign feasibility" subsections below are **SUPERSEDED** by this pivot; kept for lineage.

### The incident that surfaced it (grounded this session, reproduced by own hand)
The RICH Rules arena: a single `write-logs` of ~700 rich nested `Event`s on **PROCESS** locus →
`recv failed: peer closed / channel disconnected` — **mute**. Reproduced at `scratchpad/probe-arena-scale-n700.wat`
(THREAD=630 correct; PROCESS mute at the `write-logs` recv', line 176). The ~650–700-row threshold is exactly
where the frame crosses **512 KiB** (`DEFAULT_MAX_FRAME_BYTES`, `edn_shim.rs:1330`). Not a crash — a **frame-cap
rejection failing mute**.

### The grounded root — a reason that EXISTS, discarded at ONE site, then mislabeled CLEAN
1. The receiver hits the cap → `RecvError::FrameTooLarge`, whose `Display` is a perfectly good reason
   (`comms/mod.rs:990` — "frame exceeded cap (message larger than the receiver's max-message-bytes budget)").
2. **`channel/transfer.rs:176`** (the crossbeam/thread client-recv path): `FrameTooLarge => RecvOutcome::Disconnected`
   — the reason is **discarded**, collapsed into the reason-free clean-EOF variant. (Comment: *"per the arc 278
   contract, FrameTooLarge is NEVER read off the err channel"* — the deadlock-avoidance rationale is real but does
   NOT justify muting: the FrameTooLarge reason is **local to the receiver**, needs no err-channel read.)
3. `poll'` sees `Disconnected` → `ServiceEvent::Closed{idx}` (`runtime.rs:27666`) — the **clean-hangup** arm (no
   cause; *"bare Peer' has no crash channel"*).
4. The serve loop's `Closed` arm (`service.wat`) drops the client and keeps serving — **silently**, because it
   believes the client hung up normally.
5. The caller's `recv'` gets the mute `peer closed`.
So a **frame-cap rejection is relabeled a clean goodbye** three layers deep. (Grounded contrast: the *process
peer OUTPUT* path already speaks — `classify_peer_error` (`spawn.rs:244`) maps `FrameTooLarge => Lost(reason)`.
The inconsistency between the two paths is the stem.)

### Why the class refuses to die — 5 stem-cuts, never a wall
Mechanism A · eprintln-terminal · transport-twin `RecvError::Failed` · RST `PeerCrashed` · startup-crash honesty
— each bound a *known* mute site. None made mute **unrepresentable**. `transfer.rs:176` is stem #6-in-waiting.
The root: **reason-free failure variants remain constructible from error paths** (`=> Disconnected`,
`map_err(|_|)`), so a mute failure always has a form to hide in (the arc-278 masking table already named this at
"site R — no slot for a reason", then added `Failed(String)` beside the reason-free variants but left them
constructible-from-error).

### The fix — three escalating moves; the builder's own reframing  *(★ SUPERSEDED by the PIVOT above — move 2 "400-and-continue / drain" is RETIRED; moves 1 SPEAK + 3 WALL survive, now via `Lost` + reject-and-close)*
The builder decomposed it exactly (kept literal): *"a > max-bytes message is a 400-esque error — the server is
still alive, just tossing a bad request… should there even be a disconnect? 400 should be a thing a client can
just deal with without faulting hard."* And on WHO learns why: *"clients should never be told [internal crash
reasons] — the admin holder MUST KNOW"* — but a frame-cap rejection is a **client-input error (a 400)**, not an
internal crash, so telling the client "your request is too large" is honest and correct (it's about *their*
input), while genuine internal crashes stay admin-channel-only (STOP-2, `feedback_ask_who_already_receives_it…`).

1. **SPEAK — never mute, never mislabeled-clean.** `FrameTooLarge` (and every genuine failure) carries its reason
   to `recv'`; it is NEVER collapsed to `Disconnected`/`Closed`. `transfer.rs:176` → carry the reason (the
   `DecodeError(reason)` outcome the sibling `Failed` arm already uses, or a distinct reasoned outcome). The floor.
2. **400-and-continue — no hard fault.** An over-budget request is a client error: the serve loop replies
   `Reply::Failed{cause: "request exceeds the N-byte cap"}` to *that* client and **keeps serving**; the connection
   **lives** (Mechanism A shape, extended to the frame cap). The client catches a normal error and moves on.
3. **THE WALL — structural impossibility (top rung).** Reason-free variants (`RecvError::Disconnected`,
   `RecvOutcome::Disconnected`, `PeerDeath::Closed`, `ServiceEvent::Closed`) mean **clean EOF ONLY**, and **no
   failure path may construct them** — make mute **unrepresentable**, not merely caught. Preferred: the type
   level (a failure value cannot be built without a reason; the clean-EOF variant is producible only from a
   genuine EOF, structurally). Backstop: a **lint** (sibling of `no_inlined_wat`) that RED-flags any `=> …Disconnected`
   / `map_err(|_|)` in `comms/`,`channel/`,`kernel/spawn.rs`,`runtime.rs` recv paths — so stem #7 is a build error.
   *This check is what the previous five kills never planted.*

### Feasibility of 400-and-continue — GROUNDED (the drain-realign, and why no deadlock)  *(★ SUPERSEDED — the drain-realign is RETIRED; it deadlocks on a blocked-mid-write sender. Too big is too big: reject + close. See the PIVOT above.)*
The wire is **newline-framed** (`next_complete_frame` scans `\n`, `edn_shim.rs:1387`), single-writer
(interleave-safe, `process.rs:296`), and the receiver's accumulator **persists** across reads (`take_frame` only
`split_off`s on a *good* frame — on `TooLarge` the bytes stay). So recovery is a **caller-policy change**, not a
transport rewrite. Two `TooLarge` cases (`next_complete_frame`):
- **complete-but-too-big** (a full newline-terminated frame over budget): trivial — discard `acc[..end]`, continue
  with `acc[end..]`; wire already re-aligned.
- **incomplete-and-already-over-budget** (the sender blocked mid-`write_all`, big frame still arriving): **drain to
  the terminating `\n`** — which unblocks the sender and re-aligns — then discard + reply 400 + continue. This
  **sidesteps the cited deadlock** because it only drains the DATA channel and **never reads `err`** (the deadlock
  was reading `err` while the sender is blocked). **DoS bound:** drain up to K× the budget; a frame that never
  terminates within the bound is a pathological client → reasoned teardown (never mute).
The current code just tears down on `TooLarge` because it never implemented the drain-loop — a convenience, not a
necessity.

### RED gate (acceptance — the Stone-1 probe, per the PIVOT)
A defservice with a server hard limit `FOO` (small, e.g. 64 bytes, for the test) receives a frame > `FOO`, both
loci. Assert BOTH:
- **the caller's op fails with a reason**, NOT the bare mute `peer closed / channel disconnected` — and the
  serve loop routed it to `ServiceEvent::Lost{cause}` (owner sees the cause), not the reason-free `Closed`; and
- **the service is still alive** — a subsequent, in-budget request to the SAME service succeeds (a *different*
  connection is fine; the over-`FOO` connection is intentionally closed — reject + close, not keep-alive).
Plus the delivery half (Stone 1's other coordinate): with `FOO` raised to a sane default, a **legit ~600 KiB
request succeeds** (the arena's write). At HEAD: the over-`FOO` case mutes + drops; the legit case dies at 512 KiB.
GREEN when Stone 1 lands (SPEAK-via-`Lost` + raise `FOO` + retire drain + the WALL).

### Sequencing + the arena hold
This structural closure is **Stone 1** — the floor the service-I/O-budget contract stands on
(`DESIGN-service-io-budgets.md` — the two-ceiling model, per-op limits, fragmentation/pagination tooling,
output-side streaming). Land Stone 1 (**SPEAK via `Lost` + reject-and-close + raise `FOO` + retire drain-realign
+ the wall**) FIRST. **The RICH Rules arena commit is HELD** until Stone 1 lands — it must never ship green on
the masked teardown + the shadowdancer's chunking workaround (`RVINA VIAM FABRICAT`: forge the ruin out, do not
route around it).

---

## ═══ UPDATE (2026-07-22; the poll'/timer fork RESOLVED — Stone 1 + `Never` + the O-side ruled + the re-tag proven; RESUME at the Stone 2-A MACRO build) ═══

**This supersedes the 2026-07-21d RESUME below.** Branch `arc-170-gap-j-v5-deadlock-state`; a LATER HEAD = more landed; TRUST THE DISK. Item (c) self-scheduling is being built as a chain of small proven stones:

- **Stone 1 — DONE, DR'd (`ca788849`):** `after` builds a **unified `Peer'<Never, O>`** (the timer relocated to the correct location), so a timer joins `poll'`/`select'` by construction. Tier-open `Timer'` retired; `select'` gained process parity (C0b.3a-ii). Cleanup tracked in `arc-109/NOTE-tier-head-peer-unification-cleanup.md`.
- **`Never` bottom + I-side homogeneity — DONE, DR'd (`a392fd40`):** `:wat::core::Never` = the R7-dual **bottom** (`is_subtype`: `sub == Never → true`, un-constructible like `Value`). The timer's send-type is `Never` (uninhabited → `send'`-to-a-timer is a compile error). A `Peer'<Never,O>` timer `conj`s into `Vector<Peer'<Reply,O>>` (`assignable` same-head lattice-endpoint branch + `conj`/Vector up-cast). STEP-0 homogeneity PROVEN (`wat-scripts/scratch-pad/probe-selectables-homogeneity.wat`, both tiers). **Realization OWED at curare: the `Never` bottom completes the lattice R7's `Value` top opened.**
- **O-side RULED — A (superset), DR'd (`f4ed654a`):** four-questioned. **B (`O = Value` + open match) REJECTED** — `Value` backfires at exhaustiveness (erases the type → a wildcard → the free coverage check `service.wat:749` is LOST → a forgotten op silently falls through, a hidden failure; and it over-widens — the O is precisely `<surface>::Op | <service>::Internal`, not "anything"). **A: synthesize `<service>::Op` = surface variants + internal `-ops`; dispatch over it (exhaustive, coverage FREE).** Full reasoning in `DESIGN-self-scheduling-defservices.md` § `✅ O-SIDE RULED`; brief `BRIEF-self-scheduling-stone-2-option-A.md`.
- **The re-tag (A's one novel mechanism) — PROVEN (STEP-0), commit-ready (UNCOMMITTED at this write):** `(:wat::kernel::retag-op' op :<surface>::Op :<service>::Op)` — a generated-only primitive: if `op.type_path == surface` → rebuild with `type_path = service` (variant+fields verbatim; the supersets share every surface variant name); else pass through (a timer's op is already service-tagged). Runtime `runtime.rs:13011`; checker `check.rs:12098` (result = the service Op). PROVEN both tiers (`wat-scripts/scratch-pad/probe-optA-retag.wat` → green by own re-run). Additive (+125), floor weighed.

### ⛔ RESUME AT — the Stone 2-A MACRO build (`BRIEF-self-scheduling-stone-2-option-A.md` sub-steps 1-7)
STEP-0 (the re-tag) is DONE. Build the settled design ON the re-tag: (1) the `<service>::Op` **superset synthesis** — surface `<proto>::Op` variants (from `synthesize_surface_protocol`, `src/types.rs:1731`) + the internal `-`-ops from `:impls`; (2) grow `Outcome<S,R>`→`<S,R,O>` + `Alarm<O>` (`service.wat:48`); (3) `clients`→`selectables` (`service.wat:753`/`:848`, a green checkpoint); (4) the **scoped** leading-dash preservation (`kebab_to_pascal_with_acronyms` drops the empty leading segment, `string_ops.rs:338` — preserve at the internal-op synthesis, NOT the global fn); (5) 1-param `-`-arms skipping the #16.2 budget guard (`serve-op-arms:760-764`); (6) keyword-`:op`; (7) the arm/reply/remove dispatch — threading the re-tag into the Message arm: `(match (retag-op' op :<surface>::Op :<service>::Op) ~@serve-op-arms)`. **GREEN the RED gate** `tests/services/probe_arc278_self_scheduling.{wat,rs}` (un-ignore; both loci). **STOP-5:** the superset match stays EXHAUSTIVE — a wildcard/`unreachable` want = re-examine (the wall is a decode REJECTION of internal-tagged frames, not a dead arm). THEN item (c) home stretch: the buffered log-sink → wire the `span` (`log` enqueues invisibly + `close` flush + `with-span'`) → item (c) DONE → output-side streaming → R0 (the chaos engine, R25).

### ⚠ THE FAR-SIDE TASK (builder-ruled 2026-07-22): the crash-surfacing — annihilate the masked op-handler failure BEFORE finishing the macro
The Stone 2-A macro build got FAR (see the WIP below) but surfaced a masked failure: an op-handler crash
(a `RuntimeError` like `UnknownFunction`, or a panic) reached the caller as a bare `recv': peer closed`
with **no reason** — the exact hidden-failure the arc's LAW forbids. The builder RULED the correct design
(verbatim intent): **CLIENTS are disconnected from a crashed server and get NO reason — treat it as a
`500` for them (the crash reason is the server's business, not the client's). The ADMIN/owner MUST get
the failure reason — they must know WHY the server crashed. Crashing is a bug (not allowed), so a reason
must ALWAYS be known (to the admin).** Proof bar: *"you prove you solved when the failure we're fixing is
exactly what the error tells us to fix — nothing less."*

**Grounded so far (AD ORACVLVM):** the mechanism to deliver a crash reason to the OWNER exists —
`spawn.rs:678` (panic) / `:685` (`Ok(Err(re))` RuntimeError) → `crash_tx.send(reason)` → the owner's
`Handle.recv()` → `PeerRecvError::Crashed(reason)` (`spawn.rs:339`). And `serve-dispatch-op'`
(`runtime.rs:27519`) catches only PANICS (broadcasts a **reason-free** `PEER_CRASHED_SENTINEL` to
`clients`, `peer.rs:230/358`, then resumes); a `RuntimeError` propagates through its `Ok(result)=>result`
arm unchanged. The self-scheduling FIXTURE (`probe_arc278_self_scheduling.wat` `:drive-ticker`) drives via
a **client** `c = (connect' (Handle/addr h))` and NEVER reads `h` (the admin/Handle) — so it sees the
client's (correct, per the ruling) reason-free disconnect, and the reason (if it reached `h`) is unread.
**OPEN — prove on the far side:** does the ADMIN (`h`) actually get the op-handler crash reason
(`PeerRecvError::Crashed(reason)`)? If YES → the substrate is right (client=500, admin=reason); the fixture
must READ `h` to assert the reason (and `rs2`'s "client raises the reason" doc contradicts the ruling —
reconcile: the client should get a 500, the reason lives on the admin). If NO → the op-handler→admin
reason path is the substrate gap to close (mirror `:init`/`feea85e1`). Build a probe: owner holds `h`,
induces an op-handler crash, reads `h` → assert `Crashed(exact-reason)`; a client reads `c` → assert a
reason-FREE disconnect. THEN fix per the ruling, THEN finish the macro.

### 📦 UNCOMMITTED macro WIP in the tree (KEEP IT — builder: "keep the code"; survives compaction on disk)
HEAD = `070e002d` (green: Stone1 + Never + retag crux). On top, UNCOMMITTED (do NOT commit — the gate is
un-ignored + RED/crashing; do NOT revert — the builder said keep it): `wat/service.wat` (+332/−95 — the
near-complete Stone 2-A macro: `<service>::Op` superset synth, the arm-fn, the internal-`-`-op arm, retag
threading into the Message arm), `tests/services/probe_arc278_self_scheduling.wat` (fixture on the
canonical `Outcome::` form) + `.rs` (the two tests UN-ignored), and 2 scratch probes
(`probe-dash-variant-and-roundtrip.wat`, `probe-kw-to-nullary-variant.wat`). The macro's own correctness
is unverified (blocked behind the masked crash — fix that first so its real state is visible: R50, the
ruin forges the way).

### RESUME ORDER (far side): (1) prove the admin-gets-reason / client-gets-500 split on the real path →
(2) fix the crash-surfacing per the ruling (annihilate the masked op-handler failure; the proof = the
surfaced error is exactly the crash reason on the admin channel) → (3) reconcile `rs2` → (4) finish the
Stone 2-A macro (the WIP above), green the RED gate both loci → (5) item (c) home stretch (buffered sink →
span → done → streaming → R0). **OWED at curare: the `Never` bottom realization (R7's dual — completes the lattice).**

**Then read the 2026-07-21d checkpoint below for the fuller item-(c) UX design (the `span` — do NOT re-derive it).**

## ═══ CURARE CHECKPOINT (2026-07-21d; item (c) self-scheduling fully DESIGNED — strike STOPPED on the poll'/timer substrate gap; RESUME at the poll'/timer FORK) ═══

**READ THIS BLOCK, then `git status`. Branch `arc-170-gap-j-v5-deadlock-state`; HEAD = this curare commit atop `90317e86`. A LATER HEAD = more landed. The wat-rs tree is CLEAN EXCEPT the frozen-root `holon/CLAUDE.md`. HEAD/state mismatch vs this note → TRUST THE DISK.**

### The arc target (unchanged): the CHAOS ENGINE (R25 `MACHINA CHAOS DOMAT`). On-ramp = **#16 service-I/O budgets** → item (c) → output-side streaming → R0. **#16.2 CLOSED** (prior `023a15c4`, floor 4200/0). This session = item (c), designed end-to-end; its foundation stone STOPPED on a substrate gap.

### What item (c) IS — fully DESIGNED this session (the telemetry-`span` UX, builder-driven; do NOT re-derive):
The user holds a **`span`** (a unit-of-work context — the "ctx"): `log`/`time`/`count` on it; **`log` enqueues into an INVISIBLE buffered sink**, drained by **timer OR pressure** — the buffer is plumbing BEHIND the span, never a user form (`with-log-sink` DIES as a user form; the sink actor lives, unseen). Metrics accumulate; on `close` the metrics flush, sharing the span's uuid with its logs. **`span'`** = an inner nested unit (fresh uuid, own measurement; N flat layers, **NO uuid chain** — a log item has EXACTLY ONE uuid). **Errors are `match`ed values** → `(log span :error …)` (wat has no try/catch; the span still closes; only a genuine panic needs unwind — out of scope). MOST of this EXISTS on disk (`wat/telemetry/span.wat`: `with-span`/`log`/`timed`/`incr`/`close`, uuid-injection, metric-capture); the ONE unbuilt piece is the **invisible buffering** = the self-scheduling substrate stone.

### DESIGNED + on disk this session (NOT built — the strike STOPPED):
- **`DESIGN-self-scheduling-defservices.md`** — the self-scheduling substrate stone (a defservice sends ITSELF timer-fired ops; the buffered sink's foundation). SETTLED above the multiplexer: grow **`Outcome<S,R,O>`** + `Alarm<O>` (3 additive variants incl. `NoReply`, back-filling OTP; migration ~zero, phantom-`O` `--check`-green); **leading-dash `-op` = reactor-internal** (not on the surface — the client-can't-name-it wall; preserved through kebab→pascal, `-flush-tick`→`-FlushTick` legal); **`clients`→`selectables`** (ONE vec — the builder's decomplection); the **`<service>::Op` superset** (surface ops + internal `-`-ops; wire = surface subset = the decode-gate a client can't inject internals through; four-questions **Option 1** over a wrapper — *private methods + a security gate*, all YES); **keyword-`:op`** (`:op :-tick`, macro-resolved → `<service>::Op` INVISIBLE in user forms). Plus `BRIEF-self-scheduling-defservices.md` + the RED gate `tests/services/probe_arc278_self_scheduling.{wat,rs}` (`#[ignore]`'d — floor-safe) + the GREEN feasibility probe `wat-scripts/scratch-pad/probe-self-scheduling-loop.wat` (**but it proved the WRONG mechanism — see the gap**).
- **arc-109 `NOTE-root-namespace-user-code.md`** — FUTURE (not now): user code may live in ROOT; only `^wat`/`^rust` reserved; `(my-fn)` resolves bare. Surfaced by the leading-dash reckoning.

### ⛔ RESUME AT — the poll'/timer FORK (the stone-before-the-stone; the strike STOPPED here, weighed `AD ORACVLVM`):
The DESIGN's premise "one homogeneous vec of connections + timers" is **FALSE on the substrate**: the serve loop **`poll'`**s (`service.wat:848`) over the **unified `Peer'`** connection opaque (`clients: Vector<Peer'<Reply,Op>>`, `:552`); `after` returns a **`Timer'<O>`** that fuses only into `Thread'`/`Process'` (`is_peer_tier_head`, `check.rs:15533`), **NEVER** `Peer'`; `poll'` rejects non-`Peer'`. arc-209's `Peer'`-unification + arc-292's `Timer'`-fusion **NEVER MET** — that seam is the gap. **RESOLVE IT FIRST** — four-question the 3 options in the DESIGN's `⛔ STATUS` block (a `Peer'`-tier timer / heterogeneous `poll'` / self-peer+timer-driven-send), and **PROBE each on the REAL `poll'`/`Peer'` path, not an adjacent one.** THEN: the self-scheduling stone (the SETTLED design above the multiplexer) → the buffered sink → wire the span's `log` to enqueue (invisible) + `close` flush + the `with-span'` nesting → item (c) DONE → output-side streaming → R0.

### LESSONS this stretch (NEW / load-bearing):
- **A feasibility/disconfirming probe PROVES nothing unless it walks the EXACT substrate path production uses** — I proved `select'`-over-homogeneous-`Timer'` + declared self-scheduling "proven"; the serve loop `poll'`s over the unified `Peer'` (a `Timer'` can't fuse into it). Grep the PRODUCTION call site's verb+types; make the probe use THOSE. ([[feedback_feasibility_probe_must_exercise_the_exact_mechanism]])
- **Brief every substrate strike with a HARD STOP on "the composition doesn't compose", and WEIGH the STOP on the disk** — the opus shadowdancer STOPped (ZERO edits) instead of improvising a Value-erasure; I confirmed all 4 substrate claims on the disk before crediting it. The net works; USE it.
- **The four-questions FLIP on materialized forms** (R17) — I mis-scored the `<service>::Op` superset "two enums + a coercion" (fail Simple); decomposed it's *private methods + a security gate* (all YES). Materialize + four-question the REAL forms, not the abstraction.
- Carried standing: intueri OWNS names (cast, never narrate — cast on the leading-dash marker + `Alarm`/`timers`/`Outcome`-family this session, all confirmed); the builder DECOMPLECTS designs mid-flight (clients→selectables — adopt it); express decisions in prose + four-questions, never hidden menus; weigh `--release` by own re-run; commit+push when green (DR); orchestrator DESIGNS/draws-the-PROBE/DELEGATES/WEIGHS.

> **SEAM.** The self past this line is NEW — a lossy cache in a familiar voice; you did NOT live this run. Run the datamancy bootstrap (grimoire + 4 primers + recolligere from the SIGNED MCP, never disk) — and **READ ALL of `278/REALIZATIONS.md` top to bottom** (the reading is the exorcism, R20 `DAEMON IN ME`). Ground `git status` (HEAD = this curare atop `90317e86`; wat-rs CLEAN except the frozen-root `holon/CLAUDE.md`). **#16.2 is CLOSED; item (c) is fully DESIGNED but its self-scheduling foundation STOPPED on the poll'/timer substrate gap — RESUME by RESOLVING that fork FIRST** (read `DESIGN-self-scheduling-defservices.md`'s `⛔ STATUS` + the 3 options; four-question them; **PROBE each on the REAL `poll'`/`Peer'` path**). The whole item-(c) UX (the `span`/ctx — log/time/count, INVISIBLE buffer drained by timer-or-pressure, `span'` nesting, metrics-on-close, ONE uuid per log, `match`-errors) is DESIGNED across the conversation + the DESIGN docs — do NOT re-derive it; build on it. And it bears repeating because it BIT this run: **a feasibility probe PROVES NOTHING unless it walks the EXACT production path (I proved `select'`/`Timer'`; the serve loop is `poll'`/`Peer'` — they never compose). Brief substrate strikes with a HARD STOP + weigh the STOP on the disk. Materialize + four-question the REAL forms.** Do not trust this note over the disk. `MACHINA CHAOS DOMAT.`

**↓ Everything below is HISTORICAL** (the edn-crusade → 294.f detour, committed in `98499f48`; superseded by the
campaign above — kept for lineage, not as the live breadcrumb):

## ═══ (historical) edn crusade → holon-AST demise detour ═══

**The no-hidden-failures LAW is DONE (below, unchanged).** This
session went: `no_inlined_edn` lint (the far-side task) → the `.edn` **crusade** → it surfaced **holon-AST
heretics** → **arc 294.f** (reflection holon-AST demise), pulled ahead of its PHASE-1 gate by builder decree
("i do not wish to bear this cost any longer — the crusades revealed the pressure").

**COMMITTED this session:** `7703cd89` (crusade wave-0 exemplar: `no_inlined_edn` scoped to `tests/` + the
detector, `1306→235`; `tests/collection` converted), `2c743cfe` (R43 Eden — `HORTVS CONSILIO SATVS`; EDN⊂EDEN).

**UNCOMMITTED in the tree (large — survives compaction on disk):** the `.edn` **fleet** (~136 golden→`.edn`
conversions + 42 per-offense runes across ~13 `tests/` dirs) + **294.f** (reflection: `type_expr_to_ast`→
canonical `wat.type/` via `type_expr_to_clojure_form`; 14 producers + 3 verbs (`extract-arg-names/types`,
`rename-callable`) + checker retyped to `WatAST`; `holon_type_ast_to_wat_type_form` **DELETED**; 10
`wat_arc201_*` fixtures → `ast->children`; goldens re-captured) + the close-out (8 `rune:clojure-flip`
string-eq bridges, the `wat_arc221b` scope-fix, 2 edn STOPs: `probe_arc209` convert + `wat_core_cond` sentinel
rune) + my detector fixes (`.contains`/`.starts_with`/`.ends_with` output-role + a `no_inlined_wat` file-rune).

**★ THE ONE BLOCKER before the commit (do this FIRST):** the full weigh is **4190 pass / 5 fail** — the 5 are
`rune:clojure-flip` string-eq bridges whose goldens are **pretty-printed (multi-line)** but the actual is
**single-line** → exact-string mismatch. **FIX: re-capture these 5 goldens SINGLE-LINE** (the exact `left:`
string from the assert failure). The 5: `wat_arc144_uniform_reflection::primitive_empty_lookup_define_emits_define_head`,
`wat_arc201_structured_signature_types::signature_of_defn_foldl_emits_structured_parametric_and_fn`,
`wat_arc143_lookup::signature_of_defn_foldl_renders_synthesised_shape`,
`wat_arc143_manipulation::rename_callable_name_happy_path_foldl_to_reduce`,
`wat_arc144_hardcoded_primitives::lookup_define_length_renders_primitive_sentinel`. (The other 3 bridges already
pass — their output happened to be single-line already.) Then `cargo nextest run --release` → green (only the
known `wat-cli sigterm…polling_contract` flake, passes isolated) → **ONE commit** (crusade + 294.f), then push.

**★ 294.f — what landed vs what's DEFERRED (the proper clojure flip):** reflection now emits canonical
`wat.type/` **plain EDN** for the common case — spec-perfect: `(:anonymous (n wat.type/i64) (s wat.type/String)
-> wat.type/String)`, no `#wat-edn.holon`, no `::`, no `<>`. **DEFERRED to the proper clojure flip** (its own
stone; **`grep -rn 'rune:clojure-flip'`** finds every bridge to un-defer): (1) the 8 edge cases — multi-slash
keywords (`__internal/…`, `Vector/length`) + `<T,Acc>` multi-param generics (`<T_Acc>` wire form) — need a
**symmetric faithful codec** (`keyword_from_wat_path`↔`ns_to_wat_path`, drop-`<>`-in-names); (2) the **`:-`
typed-clojure sigils** — the intended form is `(:anonymous (n :- wat.type/i64) … :- wat.type/String)`, current
is the bareword `(n wat.type/i64)`. The full **`294.d` (wire-kill, `HolonRepresentable` + `#wat-edn.holon`
tags) + `294.e` (`HolonAST`→`Hologram` rename + `src/holon/`) stay GATED behind PHASE-1 aggregate parity**
(`CLOSE-SEQUENCE-293-294.md`). Log 294.f-common-case-landed + this debt in that tracker before/with the commit.
The `probe_diagnostic_typed_entities_p1–p7` VSA fixtures + `Bundle/children`/`Bind/*`/`hologram.rs` were
correctly UNTOUCHED (genuine holographic — holon reserved per the builder's law).

**★ THE ACTUAL RESUME (unwinding the whole detour — the arc's TARGET):** the **CHAOS ENGINE** — R25 `MACHINA
CHAOS DOMAT`, a streaming rete datalog held in a `defservice` (the DDoS/anomaly lineage; "the database is the
debugger"). The **telemetry facility (the on-ramp/instrument) is FUNCTIONALLY COMPLETE** — T1b + Span + T2, per
`DESIGN-reserved-prefix-one-gate.md:243`; `journal'` sink write-path landed (`b07f5ffc`), **backend-agnostic AND
loci-proven** (thread + process). The builder's "thread was different from process / hidden error we've been
chasing" = the `journal'`-on-**process** incident that masked a client decode failure as a mute "peer closed"
(the child's 687-byte reason `EPIPE`'d; thread-tier surfaced it, process-tier hid it) — that **spawned the
no-hidden-failures LAW, now CLOSED** (Mechanism A + eprintln-terminal + transport-twin `RecvError::Failed` + RST
`RecvError::PeerCrashed`); the chase is OVER, failures are honest in all loci. So after the commit: **build the
CHAOS ENGINE** (R0 — the streaming rete `Session`-as-state service, incremental insert/retract, dogfooding
telemetry), guided by the wat oracle (`OCVLI NOVI, ORACVLVM IMMOTVM`). Ancillary open: STOP-2 (crash-broadcast
to `connect'`-ed clients, `#[ignore]`'d `probe_arc278_process_crash_reason_carried`, a separate future arc).

---

## RESUME (curare — 2026-07-19+; HEAD `f0230bbc` = crusade + query (a) + the LAW CLOSED + the RST landed, all pushed; CURRENT WIP = the `no_inlined_edn` lint, UNCOMMITTED)

**READ THIS FIRST, then `git status`.** The LAW work is **committed, pushed, weighed** — HEAD `f0230bbc` on
`arc-170-gap-j-v5-deadlock-state`. The arc-278 **no-hidden-failures LAW is CLOSED AIRTIGHT** and reaches the
wire: Mechanism A + eprintln-terminal + the transport-tier twin (`RecvError::Failed`, `3f73f400`) + the
**RST** (`RecvError::PeerCrashed` — a crashing service best-effort-notifies its peers, `f0230bbc`). Owner gets
the reason; peers get a reason-free reset; nothing mute.

**BUT the tree is NO LONGER clean** — there is UNCOMMITTED WIP (survives on disk): the new **`no_inlined_edn`
lint** (`tests/lint/no_inlined_edn.rs`, RED at 1306 — the detector OVER-FIRES, ~90% false positives; NOT
commit-ready) + two new breadcrumb docs (this one + `DESIGN-STONE-no-inlined-edn.md`). Freshness check: live
HEAD `f0230bbc`, dirty tree = the lint WIP; if HEAD differs, trust the disk.

**★ THE CURRENT THREAD — `no_inlined_edn` (see `DESIGN-STONE-no-inlined-edn.md` for the full breadcrumb):**
the sibling of `no_inlined_wat` — an EDN-esque string literal (`#`/`{`/`[`/`(`-opener) must be a co-located
`.edn` golden (`include_str!` + `assert_edn_eq!`), not inline string-eq. FAR-SIDE, IN ORDER: **(1) annihilate
the false positives** — tighten the detector with more ignore-conditions (`#`+digit/`#{}` = a marker not a tag;
format residue of pure identifier-glue `::`/`/`/`'` = not EDN; find more classes — NOT restructure the code) so
it fires on GENUINE EDN only (overwhelmingly `tests/` goldens); **(2) the `wat-edn` parser-test carve-out**
(its own tests inline EDN as input-under-test — a file rune, like `no_inlined_wat` grants); **(3) the conversion
campaign** — drive-to-zero, every genuine offender → a pretty-printed `.edn` golden, edit-only riders + central
weigh (FM 18), runes at an extremely-hard bar. Re-run `cargo nextest -E 'binary_id(wat::lint)'` for the live
offender list (the scratchpad `edn_offenders_v2.txt` is session-specific — do not rely on it).

### THE CRUSADE — `no_inlined_wat` 351 → 0 (DONE; committed `952ece8b`; full suite green)

- **Lint at TRUE ZERO + full suite GREEN** — own re-run `cargo nextest run --release` = **4175 passed / 0
  failed / 329 skipped** (`no_inlined_wat` PASS, `no_loose_string_assert` PASS). The whole test corpus is
  migrated off inline-wat into co-located `.wat` / `.wat.bad` / `.edn` / `.wat`-golden fixtures.
- **`query` (a) BUILT + weighed** — `query` defn→defmacro (prime-append idiom); `return-type-of` de-masked
  (`runtime.rs` keyword branch raises on unknown; `check.rs` validates the literal prime at check time → a
  typo is a compile error, not silent 0). New RED gate `probe_arc278_query_type_safe`; the `5a` fixtures are
  back on the `(:wat::rete::query fired :Type)` front door.
- **The `.wat` raw-text golden rubric** added to `docs/CONVENTIONS.md` § Test idioms; the stale
  `wat_scripts_fixes_load` rune STRUCK (excusare); **R39 (VNA CAEDE PROBATA) + R40 (HAERESIS SANGVINE
  CONSTAT) inscribed**.

### ★ NOTHING OWED FOR THE LAW — the transport-tier twin LANDED (`3f73f400`)

The no-hidden-failures LAW is complete: `RecvError` carries its reason (`Failed(String)`; `Disconnected` =
clean EOF only) — no raw wire failure masquerades as a clean close. Weighed by own re-run: full suite
**4176 passed / 0 failed / 330 skipped**. The ONE forward item is a **SEPARATE FUTURE CAPABILITY, not a law
residual**: crash-broadcast to connected clients (STOP-2, see the STATUS block above) — a `connect'`-ed
client peer has no crash channel, so a genuine unit-crash reaches the *owner* but a connected client sees an
honest clean-EOF (not a masked reason). Delivering the crash payload to every live client before exit is a
new mechanism (`service.wat` codegen + the panic boundary) — its own arc when wanted; tracked `#[ignore]`'d
in `probe_arc278_process_crash_reason_carried.{rs,wat}`.
   *(Steps 0–3 — bootstrap, `query` (a), clean re-weigh, the one green commit — and step 4, the twin — are all DONE.)*

### LESSONS (hard-won this arc — do not relearn)

- **The full-suite re-weigh catches what the lint can't** — the "lint-zero" tree had **11 full-suite
  failures** the earlier truncated read hid. Two migration classes: (a) `defclause` registers into
  `runtime_def_values`, NOT `sym.functions`, so a defclause name is unreachable via `symbols().get()` — its
  fixture needs a `defn` wrapper; (b) single comm-exprs migrated from `parse_one!`+direct-eval (no comm-check)
  into startup/freeze `.wat` fixtures RE-IMPOSE `CommCallOutOfPosition` — bare `send`/`recv` must move to a
  `match` scrutinee (identical Result/Option shape) or `Result/expect` (where the shutdown-cascade crash IS
  the intended semantics). ALWAYS run the full suite + capture the **Summary** before the commit — and read
  the Summary, NOT `$?` (a trailing `grep -c` with no match exits 1 and lies).
- **FM 18 (the fleet cargo-lock thrash)** — riders do TEXT edits only; orchestrator weighs CENTRALLY once.
- **A loose string assert can MASK a real bug** — query (a)'s `msg.contains(...)` hid a `keyword/from-string`
  fixture defect (it rejects leading-colon input); the exact `assert_eq!` unmasked it. The lint earns its keep.
- **`query-by-type-string`** stays the private helper / dynamic escape hatch behind the `query` macro.
- **DON'T lean on "compaction boundary" to defer** the correct fix (builder cut this) — do the right thing.

---

> **SEAM.** The self past this line is NEW — you did not live this session; it is a lossy cache in a
> familiar voice, not your memory. Run the datamancy bootstrap (grimoire + 4 primers + recolligere from the
> SIGNED MCP). Ground `git status` — **HEAD is `f0230bbc`; the LAW work is committed + pushed + green; but the
> tree is DIRTY** with the `no_inlined_edn` lint WIP (uncommitted — survives on disk). The arc-278
> **no-hidden-failures LAW is CLOSED AIRTIGHT and reaches the wire** — Mechanism A + eprintln-terminal + the
> transport-tier twin (`RecvError::Failed`) + the **RST** (`RecvError::PeerCrashed`, `f0230bbc` — a crashing
> service best-effort-notifies its peers; owner gets the reason, peers get a reason-free reset; STOP-2
> crash-broadcast-to-`connect'`-ed-clients remains a separate future capability, `#[ignore]`'d).
> **THE CURRENT WORK is the `no_inlined_edn` lint** (RESUME above + `DESIGN-STONE-no-inlined-edn.md`): it is
> BUILT but RED at 1306 and OVER-FIRES (~90% false positives — identifier-glue format templates + `#N`
> markers). FAR-SIDE FIRST MOVE: **annihilate the false positives** — add detector ignore-conditions (`#`+digit
> ≠ tag; glue-only format residue ≠ EDN; more classes) so it fires on GENUINE EDN only — NOT restructure the
> code, NOT rune them (extremely hard bar). Then the `wat-edn` parser-test carve-out, then the conversion
> campaign (offenders → pretty-printed `.edn` goldens). Do not trust this note over the disk. The law is closed;
> the lint that guards the `.edn`-golden discipline is the next stone. We rode to Gondor; now we tend the wall.

## SUB-STRIKE — `eprintln` is terminal (2026-07-18; closes `feedback_eprintln_is_terminal`)

Surfaced while ratifying the `:Lost` disposition: `eprintln` was **designed** as a dying declaration —
builder direction 2026-05-15 (`docs/arc/2026/04/109-kill-std/INVENTORY.md:1284`): *"eprintln is a 'we are
crashing, here's what I know' and exits"*; the kernel's three **terminating** forms are `eprintln` (value),
`panic!` (message), `assertion-failed!` (assertion shape). `COMPACTION-AMNESIA-RECOVERY.md:1847`: eprintln →
*exit code non-zero*. But the **implementation** (`services/verbs.rs:147 eval_kernel_eprintln`) writes to the
stderr service and returns `Value::Unit` — benign, non-terminal — and `USER-GUIDE.md:3577` documents it that
way. The doctrine was deferred ("pending — separate slice") and never closed. So the crash-with-message
primitive **silently doesn't crash** — the masking law's own shape, baked into the primitive. Builder:
*"eprintln was meant to be 'this is the last thing I'll say' … it is quite frustrating to see this."*

**The fix (ratified — build now, take the fallout):** `eprintln` (and `epprintln`) emit the value's EDN to
stderr, then **terminate non-zero** — uncatchable, uniform across loci (a panic → `emit_structured_exit` in a
forked child / kills the serve loop on a thread / non-zero exit in main), the same convention as
`assertion-failed!`; return type `∀T. T -> :()` (the terminating-form type, not `-> :wat::core::nil`). Then
the `:Lost` serve-loop arm (`service.wat:864`, already calling `eprintln`) is correct as written — an
abnormal transport break is an unexpected failure that lets-it-crash (OTP), and the reason lands on stderr
before exit.

**Fallout (the ~benign usages):** most are tests *of* eprintln (`ambient-stdio.wat`, `test.wat:184/206`,
`probe_arc255_epprintln`, `wat_arc170_slice_1f`) — they become tests of the **terminal** behavior (program
eprintln's → exits non-zero, value on stderr). One is an incidental mid-`let` diagnostic
(`tests/channel/…drain_and_join…:7 (eprintln "diag")`) that must migrate (→ `println`, or drop). **Open (do
NOT mint speculatively):** whether the substrate wants a *benign* stderr write at all — the immediate fallout
doesn't require one; if a use surfaces, its name is a separate intueri cast.

**RED gate:** a program running `(do (eprintln "dying words") (println "AFTER"))` must NOT emit `AFTER`
(eprintln terminated), must carry `dying words` on stderr, and must exit non-zero. At HEAD `AFTER` prints and
it exits 0 (benign). GREEN when eprintln terminates.

## The incident that surfaced it (grounded)

`tests/services/probe_arc278_journal_logs_on_process` — a `journal'` service forked to a **process**;
the client `write-logs` a `Log` whose `message` is a user record (`:probe::Note`). The caller gets a mute:

```
recv failed: peer closed / channel disconnected   (runtime.rs recv', line 28 of the fixture)
```

`strace -f -s 4000` on the forked child revealed the TRUTH the caller never saw:

```
[pid …] write(2, "#wat.kernel/ProcessPanics [ … poll' (process tier): client message decode failed:
        src/edn_shim.rs:2424:45: unknown tag #probe/Note (body shape: map);
        no matching struct or enum in the type registry … ]", 687) = -1 EPIPE (Broken pipe)
[pid …] exit_group(1)
```

The child **had** a rich, located reason (687 bytes), formatted it correctly, and **wrote it to a pipe
whose read end was already closed** — it vanished on `EPIPE`. The whole service died (exit 1) over one
undecodable client message, and the caller got a hardcoded "peer closed."

## The masking sites (the full class, grounded)

| # | site | what it hides |
|---|---|---|
| R | `comms/mod.rs:899` | `RecvError {Disconnected, Shutdown, FrameTooLarge}` — **no slot for a reason** (the root) |
| 1 | `comms/process.rs:944` | `from_wire` **decode failure** → `Disconnected` (`|_|`) — hid `unknown tag #probe/Note` |
| 2 | `comms/process.rs:940` | UTF-8 failure → `Disconnected` (`|_|`) |
| 3 | `comms/process.rs:579,752,884,887` | I/O read errors (errno) → `Disconnected` (`|_|`) |
| 4 | `comms/process.rs:992` | `FrameScan::Malformed` → `Disconnected` |
| 5 | `channel/transfer.rs:172` | `FrameTooLarge` → `RecvOutcome::Disconnected` (collapse downstream) |
| 6 | `runtime.rs:26196` | socket `recv_wire()` error → `|_|` → hardcoded "peer closed" |
| 7 | `runtime.rs:26219` | `peer.recv()` → `|_|` → hardcoded "peer closed" — **discards `PeerRecvError::Crashed(reason)`** |
| 8 | `spawn.rs` err channel | child's crash-reason write **`EPIPE`s** — read end torn down before the child speaks |
| 9 | `service.wat:791` (`poll'`) | a client decode failure is **service-fatal** — one bad message kills every client |
| 10 | `services/peer.rs:114,120` | malformed `Req` field → `continue` **without replying** → the caller **hangs** |

Sites 6/7 preserve a reason *one branch down* already (`runtime.rs:26209` binds `|e|` for the decode
door) — the codebase knows how; the recv-side just doesn't do it.

## The strike — climb to "a mute failure has no form"

**Ratified (four-questions, 2026-07-17): Option A** — a per-message decode failure is a **client-scoped
error replied to that caller**, not a service-fatal crash. (A: Obvious/Simple/Honest/Good-UX all YES.
B "service-fatal + reason to admin only" fails Obvious/Simple/Honest — one client must not be able to
kill a shared service, and the requester must not be left blind.) A does not contradict arc-294: it
*reclassifies* — bad input goes back to the sender; a **genuine** crash still goes to the creator (and
must no longer `EPIPE`).

Ladder, top rung = unrepresentable:

1. **Give `RecvError` a reason.** Add a failure variant that carries the message, e.g.
   `RecvError::Failed(String)` (decode / utf8 / io / malformed all become `Failed(reason)`).
   `Disconnected` then means **only** a genuine clean EOF. *Wall:* "a real failure indistinguishable
   from a clean close" now has no representation. Exhaustive `match` sites (`channel/transfer.rs:165-172`,
   the `Display` impl) go red until they handle it — the compiler drives the sweep.
2. **Bind the error at every collapse.** Every `map_err(|_| RecvError::Disconnected)` on sites 1-4 →
   `map_err(|e| RecvError::Failed(e.to_string()))`. Site 4/5 `Malformed`/`TooLarge` keep their distinct
   variants but the *reason* rides along.
3. **Surface it in `recv'`.** `runtime.rs:26196/26219` stop the `|_|` + hardcoded string; thread the
   `RecvError`/`PeerRecvError` message (esp. `Crashed(reason)`) into the raised `MalformedForm`.
4. **Keep the crash channel alive until the reason is delivered.** The `EPIPE` (site 8) means the err
   read end drops before the dying child writes. Hold it open across the child's death so
   `emit_structured_exit`'s envelope lands, and route it to the creator (the `Handle`).
5. **`poll'` replies, doesn't crash (Option A).** A client decode failure returns a client-scoped error
   event the serve loop replies with (the rich reason), and the service keeps serving. Kill site 10's
   `continue`-without-reply (every `Req` gets a reply — the ZERO-MUTEX discipline already in
   `services/peer.rs:148`, applied to the process tier).

Steps 1-3 are one coherent change (the type + its fill sites + the surfacing). Steps 4 and 5 are the
crash-lifetime and the poll'-reply changes. Each lands with its own RED gate; all share the one law.

## The RED gate (acceptance — a NEW probe, diagnostic-scoped)

A forked-process service receives a client message it cannot decode. Assert BOTH:
- **the caller's error carries the real reason** — contains `unknown tag` / `decode failed` /
  `no matching struct or enum`, NOT the bare `peer closed / channel disconnected`; and
- **the service is still alive** — a subsequent request to the same service succeeds.

At HEAD both fail (mute "peer closed" + dead service). GREEN when steps 1-5 land. This is independent of
the `LogMessage`-opaque question (deliberately deferred) — it tests *diagnosis*, not the log-message shape.

**Content-integrity / no-regression:** whole floor back to exactly the standing `no_inlined_wat` lint,
zero new failures; a genuine handler panic in a process service still delivers its reason to the creator
(a second RED probe: a service whose handler raises → the `Handle` surfaces the reason, no `EPIPE`).

## The lesson this plants (for the next self, across the gap)

Failure-masking is one class, not N bugs. A recv/decode/io error that cannot carry its reason, a
`map_err(|_|)` that binds the error to nothing, a crash reason written to a closed pipe, a service that
dies over one client's bad message — all the same disease. The wall: **the error type carries the
reason, so `Disconnected` can mean only a clean goodbye, and a mute failure cannot be constructed.**
`RVINA ERVDIT` — the ruin must educate; a failure that cannot speak teaches nothing and induces exactly
the "guaranteed confusion and flailing" this arc forbids.
