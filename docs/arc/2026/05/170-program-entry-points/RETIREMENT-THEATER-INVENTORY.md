# Retirement-theater inventory — arc 170 audit findings

**Audit date:** 2026-05-11
**Surveyor:** Explore agent (read-only sweep across src/, wat/, wat-tests/, crates/, tests/, docs/ excluding docs/arc/, examples/, README.md, .claude/skills/)
**Total lies identified:** 48 (Bucket A + Bucket B per FM 14)
**Excluded from audit:** let* (handled by parallel sweep; see `SCORE-SLICE-3-LET-STAR-PURGE.md` at `daa973d`)

User direction 2026-05-11 mid-let-star-purge:
> *"i am quite disappointed to find let* still being used 'in fall through' --- that shit is .... i am lost for words ... my anger is intense...."*
> *"go find what else is lingering from our claimed victories"*

This inventory is the result. Each retirement we have INSCRIBED is audited against disk truth. Lies are surfaced for systematic cleanup.

---

## Bucket classification (per FM 14)

- **Bucket A** — live identifier using legacy name as concept → RENAME / DELETE
- **Bucket B** — stale comment text claiming behavior that doesn't hold post-retirement → UPDATE
- **Bucket C** — historical retirement context comment → KEEP (records what changed and when)
- **Bucket D** — orphaned scaffolding per arc 113 precedent (variant + Display + walker firing kept as reintroduction surface) → KEEP

---

## Discipline gap — three failure modes the audit surfaced

The pattern is NOT that walker arms go unarmed while Rust-side identifiers rot. Arc 163 re-armed every walker; the audit verified all 14 `BareLegacy*` variants in `CheckError` enum still fire. Substrate enforcement machinery is intact.

The actual pattern has three modes:

### Mode 1 — Documentation rot
When a concept is retired, user-facing documentation is not updated. Worst offender: `:wat::console::*` — fully annihilated from substrate, but 12+ live-looking references across 6 doc files (USER-GUIDE, CONVENTIONS, ZERO-MUTEX, CIRCUIT, CLOJURE-ROSETTA, WAT-CHEATSHEET). New users hit cold `UnknownFunction` cliffs.

### Mode 2 — Stale function-level docstrings
`eval_fn` (runtime.rs:4233) + `infer_fn` (check.rs:9897) both claim "`:wat::core::lambda` (retired fall-through) routes here." FALSE — lambda dispatch arm was removed; walker fires fatal first. These docstrings actively mislead future implementers.

### Mode 3 — File-path time-warp
Directory `wat/std/` does not exist. README:501, 658-676 (ascii directory tree), USER-GUIDE, CONVENTIONS, ZERO-MUTEX, wat-tests/README all reference paths under `wat/std/`. README's ascii tree is fiction.

---

## Inventory by retirement

### arc 105c — `:wat::kernel::run-sandboxed-ast` / `run-sandboxed-hermetic-ast`

**Status:** Substrate Rust correctly retired. wat-side defines in `wat/kernel/{sandbox,hermetic}.wat` are stdlib-baked replacements. Active check arms correct.

**Surface lies (7 hits, all Bucket B file-path/verb-name):**
- `src/check.rs:10884-10889` — comment says "lives in `wat/std/hermetic.wat`" — file at `wat/kernel/hermetic.wat`. Also says "atop fork-program-ast + **wait-child**" — hermetic.wat no longer uses wait-child. Bucket B.
- `docs/USER-GUIDE.md:3471` — "wat stdlib define in `wat/std/sandbox.wat`" — file at `wat/kernel/sandbox.wat`. Bucket B.
- `docs/USER-GUIDE.md:3472` — "wat stdlib define in `wat/std/hermetic.wat`" — same. Bucket B.
- `wat-tests/README.md:80,86` — "See `wat/std/hermetic.wat`" — same. Bucket B.
- `README.md:236` — "`wat/std/hermetic.wat` on top of `:wat::kernel::fork-with-forms`" — triple wrong: path, verb (`fork-with-forms` does not exist anywhere; should be `fork-program-ast`). Bucket B.
- `README.md:238` — "fork-with-forms, wait-child" — phantom verb. Bucket B.
- `README.md:98` — "`fork-with-forms` + `wait-child`" — phantom verb. Bucket B.

**Recommended action:** doc/comment sweep. No substrate work.

---

### arc 170 slice 2 — `:wat::kernel::fork-program-ast` / `fork-program` / `spawn-program` / `spawn-program-ast`

**Status:** PARTIALLY RETIRED — by design. Walker fires on USER code (`BareLegacyForkProgram` + `BareLegacySpawnProgram` at check.rs:2186-2194). Runtime dispatch arms still live (`runtime.rs:3878-3884`) because stdlib still calls them. Documented as intentional; slice 4 destructively retires.

**Lingering items (7 hits — 1 substrate, 6 doc):**

*Substrate (Bucket A):*
- `src/fork.rs:258-290` — `pub fn eval_kernel_wait_child(...)` — dead Rust function. No dispatch, no type registration, zero callers. runtime.rs:3898-3902 promises slice 5 removes it. **Folds into Slice 4 destructive reap.**

*Substrate (Bucket C, acceptable forward refs):*
- `src/runtime.rs:3878-3884` — live dispatch arms needed by stdlib. KEEP until slice 4.
- `src/spawn.rs:251` — "spawn-program-ast retires in arc 170 slice 2; until then..." — accurate.
- `src/stdlib.rs:148-149` — "spawn-program-ast which slice 4 destructively retires" — accurate.
- `src/fork.rs:699-720` — `eval_kernel_fork_program` still dispatched.

*Docs (Bucket B):*
- `docs/USER-GUIDE.md:398-419` — documents `:wat::kernel::fork-program` / `spawn-program` as current patterns without noting walker.
- `docs/USER-GUIDE.md:585` — tier 3 kernel list includes all four. No walker note.
- `docs/USER-GUIDE.md:3475-3478` — reference table lists without walker note.
- `docs/CLOJURE-ROSETTA.md:146,312` — code examples using `:wat::kernel::spawn-program` as idiomatic.
- `docs/WAT-CHEATSHEET.md:195` — verb-signature table lists as live.

**Recommended action:** doc sweep adding "(walker fires; use Process/Thread substrate)" notes; the dead Rust function ships with Slice 4 destructive reap.

---

### arc 162 — `:wat::core::lambda` Rust-side identifiers

**Status:** Sweep mostly complete. Walker re-armed at arc 163 fires `BareLegacyLambda` correctly. Remaining issues are stale comments + 2 substrate docstring lies.

**Substrate (2 Bucket B docstring lies):**
- `src/runtime.rs:4231-4235` (`eval_fn` docstring) — "Dispatch arms for both `:wat::core::fn` (canonical) and `:wat::core::lambda` (retired fall-through) route here." **FALSE.** Lambda dispatch arm removed (runtime.rs:3280-3283 explicitly notes retirement). Nothing routes lambda to `eval_fn` — walker fires fatal first.
- `src/check.rs:9897-9901` (`infer_fn` docstring) — same false claim.

**Docs (7 Bucket B hits):**
- `docs/USER-GUIDE.md:2818` — "Anonymous lambdas render as `<lambda@<file>:<line>:<col>>`" — FALSE. Actual format in runtime.rs:14532 is `<fn@{}>`.
- `docs/USER-GUIDE.md:1919-1920` — "Each Thread is an OS thread running the body **lambda**" — concept now exclusively "fn".
- `docs/USER-GUIDE.md:3412` — spawn-thread table: "body is a **lambda**..." — should say "fn".
- `docs/USER-GUIDE.md:2392` — code example uses `:wat::core::lambda` directly.
- `docs/USER-GUIDE.md:584` — tier 2 list says "define, **lambda**, let, match..." — should be "fn".
- `crates/wat-edn/docs/IPC-BRIDGE.md:150,341` — two code examples using `:wat::core::lambda` actively. Would fire BareLegacyLambda.
- `.claude/skills/complectens/SKILL.md:293-294` — two snippets using `:wat::core::lambda` as normative.
- `README.md:158` — test file name `wat_spawn_lambda` listed. File is now `tests/wat_spawn_fn.rs`.

**Recommended action:** sweep substrate docstrings + doc references. Walker stays armed.

---

### arc 153 — `:wat::core::unit` → `:wat::core::nil`

**Status:** Clean. Walker re-armed at arc 163. No Bucket A/B hits.

---

### arc 109 § kill-std — `:wat::std::stream::*` namespace flatten

**Status:** Walker `BareLegacyStreamPath` fires correctly. Doc rot extensive.

**Docs (5 Bucket B hits):**
- `docs/USER-GUIDE.md:586` — "`stream combinators (:wat::std::stream::*)`" as if active.
- `docs/USER-GUIDE.md:2052, 2092-2099, 2112, 2142` — entire streaming section uses `:wat::std::stream::*` throughout.
- `docs/USER-GUIDE.md:3487-3495` — reference table lists all stream verbs under `:wat::std::stream::`.
- `docs/CONVENTIONS.md:642-644` — typealias table with `:wat::std::stream::Stream<T>`, `ChunkStep<T>`, `KeyedChunkStep<K,T>` and wrong file path `wat/std/stream.wat`. Triple wrong: namespace + file path + inner type (`ProgramHandle<()>` vs actual `Thread<nil,nil>`).
- `docs/SUBSTRATE-AS-TEACHER.md:225` — historical context mention (Bucket C, not a lie).

**Recommended action:** doc sweep replacing `:wat::std::stream::*` → `:wat::stream::*`; correct typealias table.

---

### arc 109 § kill-std — `:wat::console::*` (FULLY RETIRED in slice 1f-η)

**Status:** **MOST ACUTE.** Substrate fully annihilated:
- No dispatch arms (slice 1f-η deletion confirmed)
- No type registrations
- **No walker — no diagnostic safety net**
- No `BareLegacyConsole*` variant

Replacement: ambient `:wat::kernel::println` / `:wat::kernel::eprintln` / `:wat::kernel::readln` (registered check.rs:12877-12900; implemented thread_io.rs). Any user writing `(:wat::console::*)` hits cold `UnknownFunction`.

**Docs (12 Bucket A hits — substantive shape change required):**
- `docs/USER-GUIDE.md:586,743,896,908,909,1876,2359,2365,2470,3021,3027,3423` — 11 references to `:wat::console` verbs and types as if live, including the full §11 Stdio "Console gateway" section.
- `docs/CONVENTIONS.md:428,586,645` — references to `:wat::console::spawn`, exempt list, type table.
- `docs/CIRCUIT.md:30` — code example with `:wat::console::spawn`.
- `docs/ZERO-MUTEX.md:188,313` — references to `:wat::console` as active gateway.
- `docs/CLOJURE-ROSETTA.md:213,215` — code example with `:wat::console::Console` and `println!`.
- `docs/WAT-CHEATSHEET.md:93` — code example with `:wat::console::log`.

**Recommended action:** **PHASE G-CONSOLE — NEXT IN QUEUE.**
1. Mint `BareLegacyConsolePath` walker (variant + Display + Diagnostic + walker firing) — fulfills the "every retired form fires friendly diagnostic" invariant
2. Sweep all 12 doc hits to teach new ambient `:wat::kernel::println` / `:wat::kernel::eprintln` / `(:wat::kernel::readln -> :T)` surface

The shape change is substantive (service/struct with methods → ambient verbs taking EDN values), not 1:1 text replacement.

---

### arc 109 § kill-std — `wat/std/` directory (fictional)

**Status:** Directory does not exist. Docs reference it as if it does.

**File-path lies (6 Bucket B hits):**
- `README.md:501` — "Every file under `wat/std/`..." — directory does not exist.
- `README.md:658-662` — directory tree shows `wat/std/` with `stream.wat`, `hermetic.wat`, `test.wat`, `service/Console.wat` — all wrong locations; Console.wat deleted.
- `README.md:675-676` — `wat-tests/std/` path claim — does not exist (files at `wat-tests/` root).
- `docs/ZERO-MUTEX.md:313` — "Reference: `wat-rs/wat/std/service/Console.wat`" — file deleted.

**Recommended action:** doc sweep; correct paths to current layout (`wat/kernel/` + `wat/` root).

---

### arc 155 — `:wat::core::lambda` user-facing keyword

**Status:** Clean at user-facing level. Walker fires correctly. Doc-level issues captured under arc 162 above.

---

### arc 114 — spawn's R retirement

**Status:** Clean. Poisoned type registrations in check.rs + retirement entries in special_forms.rs route correctly. No live runtime arms for bare verbs. `Thread/join-result` / `Process/join-result` are the live replacements.

---

### Retirement scaffolding audit — `BareLegacy*` variants

All 14 variants enumerated. Push counts verified active for every one:

| Variant | Push count | Status |
|---|---|---|
| `BareLegacyPrimitive` | 1 | Active |
| `BareLegacyUnitType` | 1 | Active |
| `BareLegacyUnitName` | 1 | Active (arc 163) |
| `BareLegacyLetStar` | 1 | Active (arc 163) |
| `BareLegacyLambda` | 1 | Active (arc 163) |
| `BareLegacyLowercaseFn` | 1 | Active (arc 163) |
| `BareLegacyContainerHead` | 1 | Active |
| `BareLegacyStreamPath` | 1 | Active |
| `BareLegacyTelemetryServicePath` | 1 | Active |
| `BareLegacyLruCacheServicePath` | 2 | Active |
| `BareLegacyKernelQueuePath` | 1 | Active |
| `BareLegacyMainSignature` | 1 | Active |
| `BareLegacyForkProgram` | 1 | Active |
| `BareLegacySpawnProgram` | 1 | Active |

**No orphaned variants.** **Missing variant: `BareLegacyConsolePath`** (the discipline gap that lets console source cliff into cold UnknownFunction).

---

## Summary table

| Retirement | Bucket A | Bucket B | Total |
|---|---|---|---|
| arc 105c — sandboxed-ast paths / fork-with-forms phantom | 0 | 7 | 7 |
| arc 170 slice 2 — fork/spawn-program | 1 (dead Rust → Slice 4) | 6 | 7 |
| arc 162 — lambda Rust-side + docstring lies | 0 | 9 | 9 |
| arc 153 — unit→nil | 0 | 0 | 0 |
| arc 109 — stream namespace | 0 | 5 | 5 |
| arc 109 — `:wat::console::*` (MOST ACUTE) | 12 | 0 | 12 |
| arc 109 — `wat/std/` phantom paths | 0 | 6 | 6 |
| arc 155 — lambda surface | 0 | 0 | 0 |
| arc 114 — spawn R | 0 | 0 | 0 |
| Stale fall-through comments (eval_fn/infer_fn) | 0 | 2 | 2 |
| **Totals** | **13** | **35** | **48** |

---

## Priority queue

**Three-phase sequencing** (user direction 2026-05-12, updated post-V4):

### Phase 1 — Drain the lies ✅ COMPLETE

All four retirement-theater purge slices shipped. 48 audit findings drained from user-facing surfaces; substrate scaffolding intact per arc 113 precedent.

1. **Phase G-console** ✅ shipped (`b4ea6a4`) — minted `BareLegacyConsolePath` walker + swept 10 files
2. **Phase G-stream** ✅ shipped (`2b8c253`) — sweep `:wat::std::stream::*` doc rot
3. **Phase G-lambda-docstrings** ✅ shipped (`b174bfc`) — fixed `eval_fn` / `infer_fn` substrate docstring lies + 9 doc files
4. **Phase G-wat-std-paths** ✅ shipped (`8a424a2`) — drained `wat/std/` phantom paths + `fork-with-forms` phantom verb

### Phase 2a — Gap-closure prerequisites (current focus)

**Arc 170 does NOT resume forward work until all 4 gap slices close** (user direction 2026-05-13). Path A substrate-correctness + Path E isolation-contract both required.

5. **Gap F-1** (running, `c6f8d2e`) — struct/enum accessor pregen in top-level `do`/`let` splice. Mirror of Gap C V2 / D / E pattern. Substrate enabler for V4-shape's failure pattern 1.
6. **Gap F-3** — closure type-registry inheritance to spawn-process child. Substrate enabler for V4-shape's failure pattern 3.
7. **Gap F-2** — resolver quote-awareness (forms-quoted data opacity). Substrate enabler for V4-shape's failure pattern 2.
8. **Gap G** — Path E macro shape: `deftest-hermetic` puts prelude INSIDE the closure (strict isolation contract; nothing modifies parent's frozen world; substrate-enforced via spawn-process boundary). Enforcement probes prove no cross-test prelude leakage.

**The four gaps surface from Phase E V4's blocked attempt** (commit `f2de549` SCORE). V4 attempted single-shape (Path A) deftest rewrite; 16 failures revealed:
- 3 substrate gaps (F-1, F-2, F-3) — Path A enablers
- 1 architectural gap (G) — Path E enables strict isolation as a distinct user-choice contract

After Phase 2a closes (all 4 gaps), arc 170 resumes Phase 2b.

### Phase 2b — Resume arc 170 forward work

9. **Phase E V5** — deftest macro Path A shape rewrite. After Gap F substrate closes, V4's target shape works.
10. **Phase F** — retire `:wat::kernel::run-sandboxed-*` substrate verbs (deftest + deftest-hermetic now off the verb; `run-ast` + `run-hermetic-ast` wrappers + wat/kernel/hermetic.wat audit)
11. **Slice 4** — destructive reap. Folds in:
   - `eval_kernel_wait_child` orphan dead Rust fn (src/fork.rs:258-290)
   - `BareLegacyForkProgram` + `BareLegacySpawnProgram` retirement
   - Process<I,O> legacy field cleanup; src/spawn.rs orphaned scaffolding
12. **G-fork-program-walker-notes** — sweep 6 doc hits adding walker-fires notes; deferred to AFTER Slice 4 for accuracy
13. **Slice 5** — arc 170 INSCRIPTION

### Phase 3 — Out-of-arc-170 doc cleanup (queued)

After arc 170 closes, separate Phase G-arc-191-purge cleans the arc 191 retirement residue (try / option::expect / result::expect doc rot in USER-GUIDE etc., ~70 hits). Tracked in commit message at `f2de549` SCORE delta.

### Rationale for the split

- **Phase 1 first** (4 Phase G slices) cleaned the prior retirement-theater lies before Phase 2 substrate work, giving clean ground for V3/V4 attempts.
- **Phase 2a (4 gap slices) blocks Phase 2b** per user direction "we do not attempt proper resumption of 170 until all gaps are closed."
- **Gap ordering within Phase 2a** — F-1 first because it mirrors known pattern (4th iteration of preregister-fn-defs); F-3 second (closure-extraction substrate adjacent); F-2 third (resolver quote-awareness, design-heavy); G fourth (macro shape + enforcement tests).
- **`eval_kernel_wait_child`** is NOT a lie right now — substrate comment honestly states "removed in slice 5 closure" with a specific deadline. FM 11 pre-INSCRIPTION grep enforces at slice 4/5 close.

---

## What this audit changes about the discipline

FM 14 (surface retirement leaving internal identifiers as leftovers) is fully validated by this audit. Every retirement so far has shipped INSCRIPTION with the substrate scaffolding correct but docs/comments behind. The Bucket A/B/C/D framework catches it.

**Going forward (new discipline candidate):** every "retire X" arc closure runs the inventory grep BEFORE INSCRIPTION ships:

```bash
grep -rln "X" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null \
  | grep -v "docs/arc/" \
  | grep -v "tests/wat_arc<retirement-arc>_*"
```

Surface the hits + classify per Bucket. INSCRIPTION ships ONLY when residue is Bucket C only.

This is the next discipline-gap arc candidate after the immediate purges drain.

---

## Cross-references

- `daa973d` — let* purge (the first instance of this purge pattern at scale)
- `SCORE-SLICE-3-LET-STAR-PURGE.md` — the precedent SCORE shape
- `BRIEF-SLICE-3-LET-STAR-PURGE.md` — the precedent BRIEF shape
- `docs/SUBSTRATE-AS-TEACHER.md` — the migration-discipline doc
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 11 (inscription-immutable) + § FM 14 (surface retirement leaving internal identifiers)
- `docs/arc/2026/05/154-kill-let-star/FOLLOWUP-SUBSTRATE-RETIREMENT.md` — the original "retirement theater" framing (corrected by this audit; arc 154 substrate actually IS clean post-arc-163; the rot was in the textual layer)
