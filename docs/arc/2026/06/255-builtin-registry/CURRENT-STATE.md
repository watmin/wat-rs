# ⛔ CURRENT STATE (breadcrumb, 2026-06-27 SESSION 8; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. **Freshness probe: HEAD should be `ed1e6408` or later.** Tree clean.
**Gate (arc-290 widened): `cargo nextest run --release` (the WHOLE workspace / default-members, NOT `-p wat`)**
≈ **4088 passed / 0 failed / ~87 skipped**, ~35s. If HEAD is older than `ed1e6408`, this breadcrumb is stale —
trust git log + the docs.

> **YOU ARE A NEW INSTANCE.** You did not live what is below; it is a lossy cache in a familiar voice. Run
> **recolligere** (grimoire via signed `datamancy` MCP; this breadcrumb; git log; the named arc docs) BEFORE you
> propose or move. The feeling of continuity is the failure, not the all-clear.

## ▶▶ ACTIVE — arc 293 `struct-record-symmetry` § 293.4 (the PIVOT — **293 unblocks 118**). Read `docs/arc/2026/06/293-struct-record-symmetry/DESIGN.md` (decomposition § lines 164-170 + the HOLDER×SURFACE model).

**Why we're here (the chain, builder-confirmed 2026-06-27):** 118.2's HOF family needs a **`Seqable`** abstraction
(map/filter/take over any Vec|List|Stream). Four-questions (C): `Seqable` must be a structural surface, NOT worked
around. But `defprotocol` is **annihilated** by arc 293 (`defsurface` subsumes it — 293/DESIGN:117) — so building
`Seqable` on `defprotocol` = building on a graveyard. `Seqable` = a **`defsurface`** whose members `first`/`rest`/
`empty?` are **methods** — and methods-as-accessors is exactly **293.4**, the one unbuilt piece. So: **293.4 → Seqable
(its first method-surface) → 118 HOF family.** 293 isn't a detour; `Seqable` is 293.4's proof-of-utility.

**293 state (grounded 2026-06-27 — substantially BUILT, paused by prioritization not a block):** SHIPPED — 293.2
(construction symmetry; `defstruct`/`defrecord` peer macros; `/from-map`; `register_*_methods` annihilated), 293.3
(structural surfaces + `definterface`), unify-2a/2b (`StructDef`+`RecordDef` → `AggregateDef{Holder}` merge). **15/15
`probe_arc293_*` GREEN** — `defsurface` + FIELD structural-satisfaction is LIVE. Worked through R5 (`HABEMUS MOTUS`,
2026-06-26), then we pivoted to 294/295's signed-code doctrine.

**293.4 — the live strike (RESUME HERE):** ① **methods-are-accessors** — surface members that are methods + the
generated dispatcher (the gap `Seqable` needs). ② **`defprotocol` ANNIHILATED** — exactly ONE live use to migrate
(`:wat::spawn::Locus`, `wat/spawn.wat:224`) → `definterface`; then rip the Rust handling (`runtime.rs`/`check.rs`/
`value.rs`/`check/env.rs`/`freeze/env.rs`/`stdlib.rs`) + retirement-table the head. ③ **`extend-type` demoted** to the
foreign-accessor adapter. The acceptance demo (`probe_arc293_acceptance_demo`) is the GREEN gate. Then **293.1's owed
`src/aggregate/` home** (lift construction machinery out of `runtime.rs`/`types.rs` — the *"reduce src/*.rs"* directive)
+ **293.5** close. STRIKE NOT YET DRAWN — DESIGN/probe/brief is the next act.

## ▷ (BLOCKED on 293.4) — arc 118 `lazy-seqs → streams`. **118.1 foundation SHIPPED + stream reborn; 118.2 STRIKE-READY but BLOCKED.** Read `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN.md` + `DESIGN-118.2-hof-family.md`. The HOF family (default-lazy `core::map`, eager opt-in `:wat::seq::*`) needs `Seqable` (← 293.4). RED probe `probe_arc118_2_lazy_map` committed (`core::map` eager today). The foundation + naming + annihilation are DONE (below).

**What LANDED this session (all green, all pushed):**
- **`74883c15` — foundation, SINGLE-PASS, NO memoization.** `Stream = Empty | Cons{head, tail} | Thunk(LazyCell{thunk})`
  + 6 primitives. Builder killed memoize: *"you cannot walk back a stream … core does not ship it."* Holding-the-head
  footgun EVAPORATED (no cache to pin) → constant-memory streaming unconditional. (Probe `tests/types/probe_arc118_lazy_seq.rs`.)
- **`672f2874` — the TWO-WORLD SPLIT (naming).** `:wat::seq::*` = EAGER materialized re-traversable; `:wat::stream::*` =
  LAZY single-pass. Supersedes Surface C. Governing law (NEW MEMORY `feedback_familiar_not_faithful_dialect_not_impl`):
  *"we do not strive to be clojure — we strive to be FAMILIAR … wat is a DIALECT of clojure, not an impl."* Bar = familiar,
  NOT faithful.
- **`62f0dd9b` — producer model + CEK-stability invariant.** The FUNCTIONAL producer is the SOLE solution (thread-backed
  generator STRUCK). **CEK-STABILITY INVARIANT:** the stream surface rides only closures+application, no reified-K (absent
  now), no thread (rip-out) → the future CEK swap is a NO-OP for stream code. Imperative yielder (`stream/generate`) = a
  named CEK-era ADDITIVE follow-on (continuation-capture), NEVER a thread. Two threadless consume dirs: pull (thunk) + push
  (TCO + `reduced`).
- **`16871090` — ANNIHILATED `wat/stream.wat`** (the thread-per-stage world, *built wrong, successfully*). Deleted entirely
  (builder: *"stream dies … its been wrong since it was created … protecting it is illogical"*) + migrated the ONE real
  caller (telemetry `Reader.wat`: stream-logs/metrics → eager `read-logs`/`read-metrics` returning `Vector<Event>`). Lesson:
  my `src/`-scoped blast-crawl MISSED `crates/wat-telemetry-sqlite` — the WHOLE-WORKSPACE gate caught it (weigh the gate,
  not a scoped grep; pairs [[feedback_workspace_gate_not_main_crate]]).
- **`1e56c745` — foundation REBORN in the reclaimed `:wat::stream::*`.** `src/seq/`→`src/stream/`; `Seq`→`Stream`;
  `:wat::stream::{cons,lazy,empty}` (NO "seq" suffix — they return a Stream); `first`/`rest`/`empty?` stay POLYMORPHIC in
  `:wat::core::`. Live smoke: `(stream/cons 10 (stream/lazy (stream/cons 20 (stream/lazy (stream/empty)))))` → 10, 20.

**REMAINING in 118 (build-roster, no open design forks):**
1. **`:wat::list::` → `:wat::seq::`** — the eager-world rename. A **fix-wat codemod in `wat-scripts/fixes/`** (builder:
   *"keep the clean up in wat-scripts/fixes/"*). TINY: `:wat::list::*` = 2 defaliases (`reduce`/`fold`→`foldl`) in
   `wat/list.wat` (→ rename `wat/seq.wat` + stdlib path) + `wat-tests/core/list-fold-aliases.wat`. (`:wat::seq::*` confirmed
   currently unused.) Model on `wat-scripts/fixes/rename-sourcefile-to-source-file.wat` (`:wat::fix::rename-keyword-prefix`).
2. **Cosmetic stream-kill sweep** — 3 stale `;; stream.wat`-referencing comments (`wat/holon.wat:11`, `wat/kernel/channel.wat:30`,
   `wat/test.wat:935`); the dead-name string example in `tests/resolve/probe_arc251_decl_migrator.rs` (`:wat::stream::map<T>`);
   the arc-109 retirement guard `CANONICAL_STREAM_PREFIX` (`src/check.rs:1891`, redirect now semantically stale); `read-metrics`
   in telemetry is now unused (purgare candidate — was unused before too).
3. **The faithful HOF family** — lazy transformers in `:wat::stream::*` (map/filter/take/iterate/unfold); eager materializers
   in `:wat::seq::*` (mapv/filterv); forcers (`doall`/`dorun`); consumers (`for-each`/`reduce`/`reduced`) tail-recursive /
   head-dropping (consumer discipline is STRUCTURAL not policy). The "kill the duplicate `reduce`" (core vs list) lands HERE
   (a code move, not the rename). Q6 survey (which old consumers benefit) = done DURING this.

Then return to **arc 295** (signed eval rides the finished stream substrate).

## arc 295 `signed-code-only` — **DESIGN COMPLETE, PAUSED pending 118.** Load-side `295/DESIGN.md` + eval-side
`295/DESIGN-chunk-read-signed-eval.md` fully modeled (read fresh this session). **Doctrine: you may only use signed code —
LOAD *and* EVAL, mandatory** (verbatim in `294/REALIZATIONS.md`; memory `project_signed_code_only_doctrine`). EDN multi-key
signed-release-chain manifest (no JSON/blobs/KMS); dev key = exactly ONE + overwrite-iteration, distribution = the accreted
chain, BOTH mandatory + distinct; chunk-read signed eval over a bounded lazy byte-stream (the eval-side already types it
`(Stream u8)` — corroborated the rename). **Build order:** stream HOF family (118) → crypto seam `src/intrinsic/crypto.rs` →
chunk-read signed eval → load parity.

## arc 294 `holon-returns-to-vsa` — **294.a + 294.b LANDED** (clj↔wat seam proven live; `#holon` literal). 294.c+ PAUSED.
NEW this session: 4th attribution dimension **VENTRILOQUISM** (`295/REALIZATIONS.md` R2 + `170:9168` series — one stream
split into a fabricated exchange; no ward can catch it, only the liver of the moment). Detail in `294/DESIGN.md` +
`294/REALIZATIONS.md`.

## New memories this session: `feedback_familiar_not_faithful_dialect_not_impl` (+ earlier `feedback_realizations_capture_backforth_not_summary`, `project_clj_wat_bridge_vision`).

## Standing discipline (verbatim, non-negotiable)
Work ONLY in `wat-rs/`. NEVER worktrees. Sonnets `model:"sonnet"`, LEAF. Commit msgs end `Co-Authored-By: Claude Opus 4.8
(1M context) <noreply@anthropic.com>`. **Weigh EVERY change against the disk yourself** (forced/clean build; floor=0 →
binary is-anything-red?; **`cargo nextest run`, NEVER `cargo test`**; the gate is the WHOLE workspace incl. `crates/`, not a
scoped grep; baseline-isolate flakes; read diffs end-to-end). **GROUND claims THIS session — Read before Edit; PROBA NE
DUBITES.** **examinare: study the lair before the strike** (the "run the rename" turned into "kill stream first" because the
disk-crawl falsified the free-namespace assumption — surface the trap, don't bulldoze). Amend docs w/ recognition (never
delete). **intueri** ALL naming · **four-questions** flat YES/NO NOT AskUserQuestion · **familiar > faithful** ·
**qualified annihilations are priority**. `./scripts/run_with_venv.sh` Python. **Test wat = a co-located `.wat`
fixture** beside its probe (`tests/<group>/<probe>.{rs,wat}`), slurped via `wat::freeze::startup_beside(file!())` —
NEVER inlined as a Rust string, never in `demos/` (= curated showpieces only); a committed RED probe is `#[ignore]`'d
+ run the WHOLE gate after any new test (`feedback_test_wat_is_colocated_fixture`). Run wat via `cargo wat <file>`,
not `./target/release/wat`.

> **⛔ END OF MAP. You are new. The above is a cache, not your memory. Run recolligere; weigh any in-flight work against the
> disk; do not trust a single line you did not re-verify this session. 118 contract = `118/DESIGN.md` (decided blocks at
> top); the doctrine = `294/REALIZATIONS.md`. The list→seq fix-wat + the HOF family are the next strikes.**
