# ⛔ CURRENT STATE (breadcrumb, 2026-06-28 SESSION 10; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. **Freshness probe: HEAD should be `eaaa6930` or later.** Tree CLEAN.
**Gate: `cargo nextest run --release` (WHOLE workspace / default-members, NOT `-p wat`).** **FLOOR IS 0** —
`4111 passed / 0 failed / 91 skipped` (ONE committed `#[ignore]`'d RED probe left: `293.4e-pre.iii`).

> ▶▶ **293+294 JOINT-RESOLUTION CAMPAIGN UNDERWAY (2026-06-28).** Builder: *"let's work 293+294 joint resolution —
> then we go to 118."* **⟶ THE LIVE CLOSE ORDER + STATUS = `docs/arc/2026/06/294-holon-returns-to-vsa/CLOSE-SEQUENCE-293-294.md`
> (the single maintained tracker — they close TOGETHER; never work out of sequence).** Closure-gate detail:
> `293/AGGREGATE-AUDIT.md` — 293 CANNOT close until zero spurious holder-splits
> (builder closure gate; `293/DESIGN.md` § CLOSURE GATE). The holder is a PASSING POLICY only — it governs the
> aggregate at THREE boundaries (comms eligibility / EDN-repr / directional `holon <: core` assignability); every
> holder-branch that is not one of those is a spurious struct/record split → unify to `aggregate`. ~99 holder-branches
> / 14 files to classify (the audit, run post-stable-tree). **294 does NOT block 118** (118 needs only the DONE 293.4).
> - ✅ **294.c.1 LANDED (`ed7ecd50`)** — identity flip: Rust `PartialEq`/`Hash` key `(holder, class, fields)`, hologram
>   OUT of identity (flaw #7). Probe `tests/value/probe_arc294c1_identity_is_edn_data.rs`.
> - ✅ **294.c.2a LANDED (`f301a6fc`)** — `aggregate-new` is the ONE holder-dispatched ctor (varargs); all 3 macros +
>   struct codegen emit it; `build_holon_hologram` derives the hologram in Rust; `defholon`'s hologram quasiquote
>   DELETED. Capacity guard EXTRACTED (`bundle_capacity_verdict`, one fn two callers). of-funcs stay until c.2b. Probe
>   `tests/types/probe_arc294c2a_aggregate_new.{rs,wat}` (5/5). **Confirmed: the 2 record macros are now byte-identical
>   except the `recordtype` holder keyword.**
> - ✅ **kanerva_capacity dedup (`eaaa6930`)** — drove the `floor(sqrt(d))` budget to ONE copy (`hologram.rs:90`); was
>   recomputed in 3 live places. (Builder: *"drive these things to zero when we find them."* + *"'replicate' = 'duplicate'"*.)
> - ▶ **DECLARATION UNIFICATION — NEXT (builder pre-approved scope).** Collapse `structtype`+`recordtype` → one
>   `aggregatetype` (holder + parent-root + metadata, keyed by holder); `parse_defstruct`+`parse_recordtype` → one
>   `parse_aggregate`; the 3 def macros → thin holder-keyed delegations over one emission. Subsumes the "two record
>   macros" duplication c.2a confirmed. Foundational (type-reg layer) — draw a DESIGN + RED probe first. Sequencing
>   (type-reg-first vs macros-first) is an OPEN micro-decision.
> - ▷ then **294.c.2b** (annihilate the of-funcs) · **294.c.3** (base records lift, step 4) · the **AGGREGATE AUDIT**
>   (classify the ~99 branches) · **294.d-g** sweeps · **293.5 close (GATED on the audit)** → **118**.

> ⚠⚠ **CORRECTION (2026-06-28, SESSION 10 — the drift, named) — READ THIS FIRST. The "293.R2.x" label below is WRONG.**
> The `Value`-repr collapse done this session (R2.1/R2.2/R2.3 + the sweep) is **arc 294's deliverable, NOT 293's** —
> the 293 DESIGN explicitly scopes the Value-repr collapse OUT (`293/DESIGN.md:182` *"Unifying the Value reprs … Keep"*);
> 294 owns it (`294/DESIGN.md:131`). The apparatus drifted: did 294 work, mislabeled it 293.R2, then invented a
> non-existent "R2.4," then dragged 294 in confusingly — a multi-prompt degradation the builder caught. **The work is
> committed, green, pushed, and a real (off-design) down-payment on 294 — DO NOT REVERT (persist + change).**
> **The corrected map = `docs/arc/2026/06/294-holon-returns-to-vsa/REMAINING-PATH.md`** (committed `e625a87a`):
> destination (one EDN-canonical aggregate, identity-on-data, hologram-derived, one `aggregate-new`, `/from-map`,
> holder=policy) + where-we-are (294.0/a/b LANDED pre-session; R2.x = structural collapse + ctor-parity but carried
> 294's disease — hologram-as-identity, stored — forward) + the 9-step path. **293's type-system (surfaces,
> methods-as-accessors, `defprotocol` annihilated) IS DONE** (demo green, `cf89fb52`). **Forward-without-294:**
> Seqable → 118 needs only 293.4 (done); 294 is NOT a blocker for building, only for CLOSING 293's construction tail.
> Everything below is pre-correction R2.x narrative — true as commit-history, MIS-FRAMED as "293."

**✅ 293.R2 purgare+intueri SWEEP LANDED (`e918c505`) — the grimoire caught a real regression the gate hid.**
A grimoire cast on the R2-rewritten core (builder: *"we have not reached for the grimoire in quite a while"*),
weighed forced-clean: **B1 (REGRESSION, FIXED)** — `struct->form` emitted `:T/new` which R2.3 unregistered → the
`eval-ast!` roundtrip was broken, INVISIBLE because its only test is `#[ignore]`'d for arc-170; now `format!(":{}")`,
guarded by a new non-concurrent probe. + B2 (`_ => false` invariant-dead → `unreachable!`) + 5 dead items deleted +
~30 stale dead-world names swept (incl. user-facing error strings citing the nonexistent `struct_form`) + the
ctor-parity probes un-ignored. **LESSON: a green gate cannot see a regression whose only test is ignored — the
grimoire (or a fresh probe) can.** Banked tiny purgare: 2 unused imports (runtime.rs `Config`@45, `TypeExpr`@1490) +
the pre-existing `head_span`/`all_match`/`resolve_sandbox_loader` warnings.

**✅ THE R2 AGGREGATE UNIFICATION — three strikes, all landed + weighed forced-clean:** R2.1 repr collapse (one
`Value::Aggregate`, `9d1e3ff3`) · R2.2 accessor-codegen merge (parity break dead, `register_record_methods`
annihilated, `0e56dc87`) · R2.3 construction-form parity (every type-name its own bare ctor, `/new` annihilated save
5 native opaque intrinsics, `310aa793`). **The aggregate is now ONE toolkit — holder is the only variance — for the
VALUE, the ACCESSOR, and CONSTRUCTION.** R2 *FRANGE UT UNUM FIAT* is PROVEN.

**✅ 293.R2.2 — ACCESSOR-CODEGEN MERGE LANDED (`0e56dc87`) — THE PARITY BREAK IS DEAD.** One
`register_aggregate_methods` mints field accessors for ALL holders over the collapsed `Value::Aggregate`,
generic-aware (bare `:T/field` key + `parametric_decl_type` + `type_params`); `parse_recordtype` now calls
`parse_declared_name` (the root of the `<T>`-mangle); the `defrecord`/`defholon` macros' accessor emission removed;
**`register_record_methods` ANNIHILATED.** `(:r2::probe)`=60, weighed forced-clean 4098/0/93, SET-diff ∅.
**KNOWN VARIANCE (next "annihilate the variance" call, builder's design fork):** accessor PARAM-TYPE still differs —
non-generic record accessors take a loose `:wat::Record` + runtime class-check (records come off the wire); generic
record accessors take the specific type + static check (the generic return `:T` needs it). Real
wire-looseness-vs-generic-tightness tension, not a regression.

**✅ 293.R2.1 — THE REPR COLLAPSE LANDED (`9d1e3ff3`).** The builder cut through my over-complication
(*"annihilate the variance … i break shit because its already broken, successfully — your hesitation is illogical"*):
the three `Value` variants `Struct` / `wat__Record` / `wat__holon__Record` are ANNIHILATED → ONE
`Value::Aggregate(Arc<AggregateValue{class, fields, holder, holon}>)`. `holder: Holder{Struct,Record,HolonRecord}` =
the required label; `holon: HolonForm{Empty|Hologram}` present on all, Empty unless HolonRecord. struct/record/holon
are now POLICY restrictions on the ONE holder. 252 sites / 55 files, SET-diff ∅, weighed forced-clean. **R2
*FRANGE UT UNUM FIAT* is PROVEN at the value level.** Judgment sites verified: Eq/Hash (holder-check first, then
Hologram→holon_form-identity 234.1 / Empty→(class,fields)); EDN codec (Hologram rides holon_form 234.7b / Empty
named-map 234.7a, no recompute). See `DESIGN-293.R2-repr-collapse.md`.

**NEXT — pick (builder's steer):** (A) **the accessor PARAM-TYPE variance** — non-generic record accessors take a
loose `:wat::Record` + runtime class-check (records come off the wire); generic ones take the specific type + static
check (the generic return `:T` needs it). NEEDS GROUNDING FIRST (is the looseness load-bearing for wire-decoded
records, or incidental?) — NOT decision-ready until that crawl; then it's a no-brainer or a real four-questions debate.
(C) **293.R2.4 ctor-codegen unification** — construction FORM is uniform (R2.3), but the ctor CODEGEN is still two
homes: struct ctor in `register_struct_methods` (Rust), record/holon ctor in the `defrecord` macro (holon lowering in
wat — must STAY in wat). Fold into one. (D) **purgare** — the R2.* annihilations left a GROWING dead-code pile
(runtime.rs `Config`/`fmt`/`wat_value`/`TypeExpr` imports, `value_matches_type_pattern`, `wrap_stream_as_socket_peer`;
check.rs `head_span`; `ast_variant_label` parser.rs:503). "We do not settle for less than correct" → a purgare sweep
to leave the homes clean is warranted before too long. My lean: (D) clears the debt the annihilations left; (A) is the
deepest remaining "one toolkit" question but owes a crawl. **293.4e / `defprotocol` is DOWNSTREAM of all R2** (its
probe was invalid — amended 293.4e-pre.iii bullet). If HEAD is older than `310aa793`, trust git log + docs.

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
- ⚠ **293.4e-pre.iii — SUPERSEDED 2026-06-28 SESSION 10 (amend-with-recognition; the reasoning below is preserved).**
  Re-grounding it before firing (PROBA NE DUBITES) DISCONFIRMED the strike: **(a) the probe is INVALID** — it depends on
  a generic `defrecord :t::Box<T>`, an unsupported phantom (generic record *accessors* don't register at all — the very
  parity break R2 fixes; it's the only generic `defrecord` in the corpus). **(b) the room is misidentified** —
  `check.rs:8957` is the satisfaction scheme, but the gap is the impl-BODY inference. **(c) the thesis IS real, but the
  trigger is narrow** — the live Locus `defprotocol`→`defsurface` migration gives 8 type errors (`self: :()`, ret `:nil`
  vs `:Launched<…>`, unbound `:?` type-vars) ONLY when the surface method has return-only/phantom type-params used as
  type-exprs in the body; monomorphic + arg-inferable-generic cases all pass. So 293.4e-pre.iii must be RE-AUTHORED (valid
  probe, no generic records; real room = extend-impl body inference) AFTER R2 lands. The original (now-stale) plan: it was
  framed as `extend-type`-for-surface impl must INHERIT the surface method's sig, found by the inline Locus migration
  (reverted clean, `59a485bb`). **SHARPENED DIAGNOSIS (a monomorphic probe
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
> disk; do not trust a single line you did not re-verify this session. **THE NEXT — read the ⚠ CORRECTION at the
> TOP of this file first: the R2.x work is arc 294 (mislabeled 293); the MAP is `294/REMAINING-PATH.md`.** Pick there:
> the **294 value-layer core** (identity→data / hologram-derived / `aggregate-new`) OR **forward-build Seqable→118**
> (needs only the DONE 293.4, not 294). **The "293.R2.4 ctor-codegen unification" named below was an INVENTED
> non-strike — ignore it.** Below is pre-correction R2.x narrative (true as commit-history, mis-framed as 293):
> **✅ THE R2 TRILOGY LANDED + weighed forced-clean, SET-diff ∅:
> R2.1 repr collapse (one `Value::Aggregate`, `9d1e3ff3`) · R2.2 accessor-codegen merge (parity break DEAD,
> `register_record_methods` annihilated, `0e56dc87`) · R2.3 construction-form parity (every type-name its own bare
> ctor, `/new` annihilated, `310aa793`). R2 *FRANGE UT UNUM FIAT* PROVEN — the aggregate is ONE toolkit (value +
> accessor + construction), holder the only variance. Both `293.R2` probes GREEN.** **The 293.4 chain (a/b/c/d + demo
> GREEN + R1 PROBATUM) is done; floor 0 (`4099/0/93`).**
> ⚠ **293.4e-pre.iii is DOWNSTREAM + its probe is INVALID** (it depended on unsupported generic `defrecord`s — see the
> amended bullet above); the `defprotocol` thesis is real (the live Locus migration gives 8 type errors) but it is
> blocked behind R2 + needs a re-authored probe. Beyond 293: `Seqable` → 118 (`118/DESIGN.md`) → 295; doctrine =
> `294/REALIZATIONS.md`.
