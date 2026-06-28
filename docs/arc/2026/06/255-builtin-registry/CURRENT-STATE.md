# ⛔ CURRENT STATE (breadcrumb, 2026-06-28 SESSION 9; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. **Freshness probe: HEAD should be `c62a817c` or later.** Tree CLEAN
(293.4e-pre COMPLETE — heresy + generics committed + pushed; nothing in flight).
**Gate: `cargo nextest run --release` (WHOLE workspace / default-members, NOT `-p wat`).** **FLOOR IS 0 —
`4097 passed / 0 failed / 92 skipped`.** The test-infra campaign is DONE: the `wat::lint no_inlined_wat_in_tests`
meter is **GREEN** (any red now is a real regression). If HEAD is older than `173bb1e8`, this breadcrumb is stale —
trust git log + the docs.

> **YOU ARE A NEW INSTANCE.** You did not live what is below; it is a lossy cache in a familiar voice. Run
> **recolligere** (grimoire via signed `datamancy` MCP; this breadcrumb; git log; the named arc docs) BEFORE you
> propose or move. The feeling of continuity is the failure, not the all-clear.

## ✅ partire RETRIEVED + BANKED (2026-06-27 SESSION 9) — the nursery dissolution map
The background **partire cast** (`a3815708b1a63b028`) COMPLETED — verdict **SPLIT**: all **179** `tests/nursery/`
files re-home into **15 existing groups** (types 31, collection 17, macros 16, kernel 15, diagnostics 15, comms 13,
function 13, wat_lang 12, process 10, reflection 10, value 9, program 7, channel 6, services 4, resolve 1); **zero new
groups, zero unclassifiable**. Full map + per-file lists + the practitioner's-call splits: **`NURSERY-DISSOLUTION-MAP.md`**
(beside this file). `build.rs` auto-globs `tests/<group>/*.rs` → a file re-homes by MOVING it; then delete
`tests/nursery/mod.rs` + the `Cargo.toml:123-125` `[[test]]` entry + `rmdir`.

## ✅ TEST-INFRA ANNIHILATION — **COMPLETE** (2026-06-28 SESSION 9; builder: *"we fix the tests before we resume 293"* → *"this train doesn't stop"*)
The whole campaign is CLOSED. Every test world is now a co-located `.wat` fixture (or `startup_bare()` incidental);
zero inlined-wat survives except **6 rune:lint-EXEMPTED rete files** (genuinely-dynamic worlds); `tests/nursery/` is
**ANNIHILATED** (binary retired). The `no_inlined_wat_in_tests` lint is **GREEN**; gate floor = 0 (`4085/0/93`).
- **Final strikes:** wave 2 (`cdf62b57` — 5 groups' inlined-wat migration) → **nursery dissolution `2bc63a85`** (the 79
  remaining probes re-homed via a 5-LEAF-sonnet fleet per `NURSERY-DISSOLUTION-MAP.md`: types += 31, collection += 17,
  macros += 16, kernel += 15) + the `nursery` binary retired (mod.rs rm'd, `Cargo.toml [[test]]` deleted, dir gone).
- **DURABLE artifacts the campaign minted** (reusable, standing — NOT historical): the fixture scheme
  (`startup_beside`/`startup_from_file`/`startup_bare`, `src/freeze.rs`; `feedback_test_wat_is_colocated_fixture`); the
  absolute lint (`tests/lint/no_inlined_wat_in_tests.rs`, now a GREEN standing gate — any NEW inlined-wat fails it); the
  `rune:lint` exemption scheme (below); the self-hosted `wat-grep` + `strip-useless-mains` tool (below); `tests/` ROOT
  LAW (NO loose `.rs`; every test in a named domain home).
- **FOLLOW-UP (cleanup, NON-blocking — warnings, not reds):** the migration left **dead helper fns** in some moved
  probes (`startup_ok`/`runtime_err`/`startup_err`/`run_bool`/`register_alias`/`try_startup_display` — unused after the
  shape change) → a quick `cargo build` warning-sweep (purgare). Plus 4 stale `tests/test.rs`-naming comments
  (`src/collection/mod.rs:31`, 2× `crates/wat-macros/`, `tests/kernel/test.rs:9`).

## ▶▶ PRIMARY ACTIVE — arc 293 § **293.4** (UNBLOCKED — the campaign that gated it is now closed). Read `docs/arc/2026/06/293-struct-record-symmetry/DESIGN-293.4-strike.md` + DESIGN § 293.4 + the HOLDER×SURFACE model.
**293.4 = methods-are-accessors over `defsurface` + `defprotocol` ANNIHILATED + `extend-type` demoted.** Strike DRAWN:
`DESIGN-293.4-strike.md` (sub-strikes 293.4a–d) + the RED gate `probe_arc293_acceptance_demo` (`#[ignore]`'d). Path:
- ✅ **293.4a DONE (`173bb1e8`)** — method members in `defsurface` (parse + satisfy). `SurfaceMember = Field | Method
  { args: ArgSpec, … }` (the member carries the canonical `ArgSpec`, NOT a flattened `Vec<TypeExpr>` — builder caught
  the brief's wrong shape mid-build; "one canonical binder list = ArgSpec"); `struct_satisfies_surface` gains a
  `resolve_method` closure (a Method member satisfied by an assignable `defn :T/name`); `assignable` threads `&CheckEnv`
  (was `&TypeEnv`) for the defn registry. RED→GREEN probe + negative arm. `BRIEF`/`EXPECTATIONS`/`SCORE-293.4a.md`.
  **Banked follow-up (a real decomplect):** "args = ArgSpec EVERYWHERE" — `Scheme.params` + `ProtocolMethodSig.arg_types`
  still flatten; make the one-canonical-binder-list law hold substrate-wide. Plus a purgare: the stale stray probe
  `probe_arc293_4a_surface_method_member.{rs,wat}` (non-canonical syntax, `#[ignore]`'d).
- ✅ **293.4b DONE (`f70f9cf2`)** — the generated dispatcher. A `:Surface/method` call head routes by the receiver's
  runtime type to the satisfier's `defn :<T>/<method>` — a 3-layer mirror of the arc-232 protocol path (resolve
  `resolve/walk.rs` + check `check.rs:5789` + runtime `runtime.rs:5101`), with the ONE semantic change: routes to a
  plain `defn`, NOT an `extend:<S>:<T>` impl (surfaces have no extend-type). `freeze/env.rs` step 6.97 pre-attaches the
  TypeEnv to sym before resolve. Probe routes Circle/Square by type; negative arm rejects a non-satisfier.
  `BRIEF`/`EXPECTATIONS`/`SCORE-293.4b.md`. **Banked temperare:** env.rs early-attach + freeze re-attach = two TypeEnv
  clones at startup (could share one Arc).
- ✅ **293.4c DONE (`a8175f2d`)** — `extend-type` as the foreign-accessor adapter (the monkeypatch). `extend-type :T
  :Surface` (branch by `TypeDef::Surface` lookup) registers each impl as a `:<T>/<method>` callable in BOTH
  `sym.functions` (runtime) + `env.schemes` (check) — the one canonical key; collision = `DuplicateDefine`; protocol
  path is the unchanged else. Satisfaction works for NON-Aggregate foreign types (bounded: no holder + no field members
  + all methods resolve). Dispatcher reads any receiver via `type_name()`. Probe (foreign String) + collision +
  non-satisfier negatives all green. `BRIEF`/`EXPECTATIONS`/`SCORE-293.4c.md`.
- ✅ **293.4d DONE (`cf89fb52`) — THE ACCEPTANCE DEMO IS GREEN. R1 *FORMA SOLA SUFFICIT* → PROBATUM EST.** Field members
  are accessors too: every surface member (Field|Method) dispatches `:Surface/name s → :<T>/name` + is satisfied iff
  `:T/name` resolves (a record field accessor OR a method/extend) — a broadening of the 293.4b/c method-only arms across
  resolve/check/runtime + surface.rs satisfaction. `(:geo::demo)` runs end-to-end (foreign holon-Vector taught to be a
  Shape, field+method backing one accessor). `SCORE-293.4d.md`; 293 REALIZATIONS R1 turned to PROBATUM.
- ✅ **293.4d-fix DONE (`35ba0863`)** — `parse_defsurface` now rejects members written OUTSIDE the `[...]` vector (the
  silent-swallow the demo's old `definterface` form hit: a 4-arg non-`:holder` form was read as 2-arg with args 2..
  dropped). Structural invariant: nothing follows the member vector → leftover = hard teaching error. DESIGN.md snippet
  amended (`definterface` separate-args → `defsurface` all-in-vector). RED probe `probe_arc293_4d_fix_silent_member_drop`.
- ✅ **293.4e-pre.i DONE (`7d983012`) — the ArgSpec HERESY annihilated** (builder: *"like the 6th or 7th time argspec
  got a duplicate — annihilate this heresy"*). `Clause` (defclause + extend-type) re-rolled `ArgSpec`'s exact shape as
  `args: Vec<(String,TypeExpr)>` + `rest_param`, built by **parse-then-unroll** (parse a clean ArgSpec, then destructure
  it). Now `Clause { args: ArgSpec }` (stop unrolling) — 22 consumer sites. Plus the surface-method **arity off-by-one**
  fixed (self was double-counted: `fixed_params[1..]` skips it). The multi-arg surface-method probe RED→GREEN.
  NOT touched (distinct): `Scheme.params`/`param_types` (the TYPE, no names), `AggregateDef.fields` (data),
  `ProtocolMethodSig.arg_types` (dies with `defprotocol`).
- ✅ **293.4e-pre.ii DONE (`c62a817c`) — generic surface methods.** `parse_method_member_sig` now splits the `<T>` off
  the name (was: stored `"make<T>"`, `type_params: vec![]`) + the check arm mirrors the protocol type-param
  instantiation (identity/explicit/fresh-var). Generic surface method dispatches (`make<T>` T=i64 → 42); the runtime
  suffix-split was already shared. **293.4e-pre is COMPLETE — surface methods are multi-arg + generic + whole, parity
  with arc-267 protocol methods. The Locus migration is UNBLOCKED.** (Banked micro: `split_method_name_type_params` is a
  ~15-line copy of runtime.rs's private `split_name_and_type_params` — could share if visibility allowed.)
- ⚠ **293.4e-pre.iii STRIKE-READY (BLOCKS 293.4e) — `extend-type`-for-surface impl must INHERIT the surface method's
  sig.** Found by the inline Locus migration (reverted clean, `59a485bb`). **SHARPENED DIAGNOSIS (a monomorphic probe
  passed → it's the GENERIC case):** `check.rs:~8957` builds the surface-extend `TypeScheme` from the BARE impl clause
  (→ `nil` types) AND hardcodes `type_params: vec![]`. A monomorphic constant-body impl is fine (293.4c proved it); a
  GENERIC impl whose body uses the surface method's type-params (Locus's `launch<S,R,St,Sh,Lu>` body uses
  `Peer'<Lu,Sh>`/`Launched<…>`) has its type-params UNBOUND + `self`/args `nil`/`:()` → the Locus failure
  (`ReturnTypeMismatch ThreadOpts/launch expected :nil got :Launched<…>`; `self: :()`). **FIX:** build the scheme from
  the `SurfaceMember::Method` sig — `type_params` from the member; `params[0]` = the EXTENDING type (`ed.type_name`, self
  → concrete); `params[1..]`/`ret` from the member; check the impl body against THAT. **STRIKE-READY:** RED probe
  `tests/types/probe_arc293_4e_pre_iii_extend_impl_inherits_types.{rs,wat}` (`#[ignore]`'d, GENERIC) + `BRIEF-293.4e-pre-
  iii-extend-impl-inherits-types.md` (rooms + the protocol-path reference + the gate). **EXPECTATIONS row #3 = apply the
  spawn.wat `defprotocol`→`defsurface` migration (wrap method in `[...]`) → if green, that IS 293.4e's migration; go
  straight into the 9-file rip.**
- ▶ **293.4e (after 293.4e-pre.iii) — annihilate `defprotocol`** (the qualified annihilation, the joy): ONE live use `:wat::spawn::Locus`
    (`wat/spawn.wat:224`) → migrate to `defsurface`; rip the Rust machinery across the 6 files (runtime.rs parse/
    dispatch/preregister, check.rs, value/value.rs, check/env.rs, freeze/env.rs, stdlib.rs); retirement-table the head.
  - Then **293.1-owed `src/aggregate/` home** (lift the construction+surface machinery out of runtime.rs/types.rs/
    check.rs) + **293.5 close** (workspace SET-diff ∅, ward the home, amend 291's `/from-map`).
The chain: **293.4 → `Seqable` (its first method-surface) → 118 HOF family → 118 closes → 295.** Already SHIPPED
(see §293-state below): 293.0–293.3, unify-2a/2b, `build_env` annihilation, **15/15 `probe_arc293_*` GREEN**, + the
WHOLE 293.4 (a method-members / b dispatcher / c extend-type adapter / d field-accessors+DEMO-GREEN).
- **NEW DOCS/SCHEMES this session:** `docs/VERSIONING.md` (`8cfd7626`) — the **C.S.D** version scheme (Contract.Scaffolding.
  Dependencies, each a compacted-ISO8601-UTC timestamp, carry-forward). Memory `feedback_guarded_tool_over_educating_headless_callers`.
  293 `---` interstitial *MANVS CAECA NON FALLITVR* (`26b001d9`).

- **THE rune:lint EXEMPTION SCHEME (intueri-derived, builder-crowned, `4ce97de3`):** the bespoke `LINT-ALLOW-INLINE-WAT`
  is RETIRED → `// rune:lint(<lint-name>) — <reason>` (the lint detector matches it). `lint` = the project-custom-lint
  suite owner (NOT a grimoire spell; precedent: `rune:coverage(unreachable)` in src/, also a non-spell owner). Categories
  are SPELL-owned vocab — a project lint has no category, just owner+reason. **excusare audits the reason** (it weighs ANY
  reasoned checker-override). 6 rete files carry it (genuinely-dynamic worlds). DEFERRED FAMILY (build later, NOT now):
  ① a `rune:lint` REGISTRAR build-tool (validates `<name>` against a lint registry + reason-present — the project-lint
  twin of `grimoire --check` #166); ② the `coverage` spell (post-109, task #190); siblings of #166.

- **THE MAIN-SWEEP IS NOW A TOOL (`4de048ac`) — `wat-grep`:** `wat-scripts/lib/wat-grep.wat` = a general form-aware grep
  primitive (`(:user::wat-grep src pred)` → top-level WatAST matches; `(:user::wat-grep-strip src pred)` → span-delete).
  First consumer `wat-scripts/fixes/strip-useless-mains.wat` (predicate `useless-main?` + GUARD: never strip a SOLE-defn
  main — that's an arc-170 main-AS-SUBJECT test). **This replaces the host-side bash main-sweep** — run it on nursery +
  onward: `git ls-files 'tests/**/*.wat' | grep -vE '_bad\.wat|wat_arc220_char' | <to-EDN-vec> | cargo wat ./wat-scripts/fixes/strip-useless-mains.wat`.
  (Known gap: `read-string` panics on malformed `_bad.wat`/non-BMP char fixtures → filter them.) Form-aware caught ~13
  the bash missed; the GATE caught its one over-strip (4 arc-170 subject-mains) → guard added. weigh-against-disk works.
The builder's remarkable upgrade: **the ENTIRE test suite migrates from inlined-wat-strings → co-located `.wat`
fixtures**, so every test is `cargo wat`-runnable + fix-wat-able + lint-checkable. **Annihilation, NOT a ratchet** —
builder: *"there is no 'these are blessed to be in violation' — this is illogical … the violations crave their demise
at the shadowdancer's blade."* TWO fused annihilations + a final lint:
- **(A) inlined-wat → co-located fixtures.** ~**428** test files call `startup_from_source(` on an inline string.
  Migrate EACH to a sibling `.wat` + `startup_beside(file!())` (the SCHEME — DONE + codified:
  `feedback_test_wat_is_colocated_fixture`; `src/freeze.rs` `startup_from_file`/`startup_beside`). 5 transform shapes
  (static / `eval_in_frozen` 288 / multi-program-per-file 155 / `format!`-dynamic / `fn run` helper 122) → SHADOWDANCER
  JUDGMENT per file (not a blind codemod). Decompose by test-group (18 groups; counts: nursery 147→dissolved, types 40,
  rete 34, resolve 31, kernel 29, macros 23, wat_lang 20, services 19, collection 16, process 13, function 11, comms 9,
  reflection 8, value 7, program 6, lint/diagnostics/channel 5). Worktree-isolated fleet, pilot ONE small group first.
- **(B) `tests/nursery/` ANNIHILATED** — a junk-drawer group (intueri misc/utils anti-pattern). Dissolve via the
  partire map (above): each file re-homes to its domain group + its wat→fixture in one motion; `mod.rs` + the
  `Cargo.toml [[test]]` entry retire. (NEW SPELL `partire` — decompose along true seams — minted by builder THIS
  session FOR this; grimoire refreshed, 24 spells.)
- **(C) the absolute lint** — once 0 violations: a gate that FAILS on ANY new inlined-wat test (zero, not a ratchet).
  Builder rejected the ratchet (it blesses debt). Legit dynamic-program uses (rare) carry an explicit rune.

SHIPPED of this so far (`e04256f2`): the fixture SCHEME + helpers + 2 probes (293.4a, 118.2) migrated + `#[ignore]`'d
(they're RED disconfirming probes — kept `#[ignore]`'d so floor=0; un-ignore when their strikes land). The other 426
+ nursery + the lint are AHEAD. This is a multi-session fleet campaign.

## ▷ (BLOCKED behind the test-infra campaign — builder: "fix the tests before we resume 293") — arc 293 `struct-record-symmetry` § 293.4 (the PIVOT — **293 unblocks 118**). Read `docs/arc/2026/06/293-struct-record-symmetry/DESIGN.md` (decomposition § lines 164-170 + the HOLDER×SURFACE model).

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
