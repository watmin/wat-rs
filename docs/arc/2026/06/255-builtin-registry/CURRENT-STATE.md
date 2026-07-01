# ⛔ CURRENT STATE (breadcrumb, 2026-06-30 — replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. **Freshness probe: HEAD should be `8c04ae5e` or later.**
**Gate: `cargo nextest run --release` (WHOLE workspace, NOT `-p wat`).** FLOOR 0 — `4191 passed` (D1 +8, derive-S1 +17).
`cargo wat` is a STALENESS-GUARDED install; for behavior checks use `cargo run -q --release --bin wat --`. Tree is CLEAN
(derive Strike 1 committed `8c04ae5e`); no uncommitted WIP.

> **⊹ THE TOEDN DERIVE (top rung above D1) — STRIKE 1 LANDED (`8c04ae5e`).** intueri crowned `#[derive(ToEdn)]` +
> `#[to_edn(via/literal/key)]` (each sub-key grammar-constrained to a safe token). Strike 1 built the `#[proc_macro_derive]`
> on the kind enum + the serde-style `ToEdn` building blocks (String/ints/Vec/Option — every field `.to_edn()`-able, the
> wall real) + proved it byte-identical on `ConfigError` (16 golden literals) with a `TcpStream` compile-fail wall. Design +
> arc decomposition: `296/DESIGN-296-derive.md`. **NEXT: Strike 2 = the `via`/`literal`/`key` attribute DSL, proven on
> `CheckError` (2 `collect_hints`, 17 synthetic constants, multi-key spans). Strike 3+ = the family sweep** (apply + delete
> each hand serializer, byte-identical; then the bare enums StartupError/ResolveError). **When the sweep completes, R1 *NE
> SIBI OBSOLESCAT* → PROBATUM EST.**

> ## ⊹⊹ SESSION 2026-06-30 (latest) END-STATE — ARC 296 KEYSTONE + WatError WALL BUILT (read THIS; below `---` is superseded)
>
> **296 IS A REACTIVE ARC** (a sonnet FUMBLED on stringly errors; the builder wants coherent errors so that stops). Scope =
> coherent `:wat::core::Error` records. HolonAST's full death is ARC 294 (next), NOT 296. Prior: surface kit complete
> (`06ede1dd`); 296 slice-1 (`7f17054a`, every error `ToEdn`).
>
> **THE KEYSTONE + THE WALL ARE BUILT — errors are records satisfying `:wat::core::Error`, and a non-conformant error cannot
> compile.** Landed + weighed by the orchestrator's own gate AND its own capture of the emitted wire EDN (green ≠ clean):
> - **S1 `d82cc791`** `is_pure_type`: a Record-holdered surface is a pure field type. **S2 `396a610d`** a record satisfies a
>   surface as a `Vector` element (the recursive `causes` tree). **S3 `d7458978`** `(:wat::kernel::here)` (caller location).
> - **S4 `febc5754`** the `:wat::core::Error` surface (`{message, location, causes}`) declared in `wat/core.wat`.
> - **S5 `cf375f9a`/`0d858b49`** `raise!` re-gated to REQUIRE `:wat::core::Error` (`(raise! 42)` UNCOMPILABLE — the wall);
>   `:wat::core::Fault` + `Fault/of` (message-first) minted; 26 HolonAST callers purged; arc113 round-trip STRUCTURAL.
> - **S6 `ed5721ea`** THE `WatError` WALL — a floor-guaranteeing trait at the single wire choke point `to_wire_edn`; a
>   floorless error is a COMPILE ERROR (`LEX AVCTOREM NON EXCIPIT` — the substrate forced to obey its own contract). **The
>   11-key span heresy is DEAD** — the floor is RECURSIVE, ZERO `:span` at any depth (verified), one-line `:message`, FlatMessage closed.
>
> **✅ D1 LANDED (`74eb2ca6`, weighed 4174/0) — the prose-in-errors class's 10 findings are STRUCTURALLY CLOSED.** The
> down-payment one rung below the derive: `Remedy`/`LoadFetchError`/`HashError` each got a `ToEdn` form, and all 10
> smuggling sites route through it (`attempted_clauses` reborn as `Vector<{:arity :param-types}>`, joins→Vectors,
> `.to_string()`→`.to_edn()`, dot-path split). Weighed by the orchestrator's own hand: gate green, ZERO prose-smuggling
> patterns survive in the serializers, `render_remedies` preserved in the 3 Display impls (human face intact — no regression).
>
> **▶▶ THE REMAINING TOP RUNG — `#[derive(WatEdn)]`** (removes the hand-written serializer so `.to_string()` has NO site;
> a floorless BODY becomes uncompilable, as S6's `WatError` made a floorless FLOOR uncompilable). Needs its own attribute-
> vocabulary design — `#[wat_edn(hint_fn=…)]` for the computed `:hint` fields, synthetic-constant-field attrs for unit
> variants like `BareLegacyUnitType`'s `:primitive ":()"` — grounding in `296/DESIGN-296-D1-structured-not-prose.md §
> The rung above`. `crates/wat-macros` has the proc-macro infra (syn/quote; attributes exist, no `#[derive]` yet). When
> it lands, **R1 *NE SIBI OBSOLESCAT* turns fully to PROBATUM EST** (see `296/REALIZATIONS.md` R4 — *ITERVM SVRGIMVS*).
>
> **HISTORY — the audit that drove D1.** Read **`296/AUDIT-prose-in-errors.md`**. It found the fixed `:message`-blob was
> one of a CLASS: 10 findings (9 L1) where structured data was smuggled into prose (`:called-arg-types "i64, String"`
> joined; `attempted_clauses` DROPPED; `Vec<Remedy>`→blob ×3; `LoadFetchError`/`HashError` stringified). **The root: the
> error EDN was not a STRUCTURAL FUNCTION of the type** (hand-authored → structure `.join`/`format!`'d away; embedded types
> not required to be `ToEdn` → `.to_string()`'d; check-serializer drifted from its runtime twin). D1 closed the instances
> by making the embedded types `ToEdn`; the derive closes the CLASS (no hand-written body to smuggle in). Then N3 (per-phase
> tag ns `#wat.check/…` — tags STILL `#wat.kernel/…`, not done) + `deferror` (N8, sugar, unbuilt) + `Failure`/`raise!` de-stringify.
>
> **CROWNED NAMES (intueri):** N1 `:wat::core::Error` · N2 floor `message`+`location`+`causes` (`:kind` DROPPED — tag IS the
> kind) · N5 `#[derive(WatErrorRecord)]` · N7 `:wat::kernel::here` · N8 `:wat::core::deferror` (unbuilt) · N9 bare `Location`
> ctor · N10 `:wat::core::Fault`. **DO NOT DELETE HOLON** — a surface's `$core-record`/`$holon-record` pair is a WANTED
> capability; errors never reach for a hologram (a `$holon-record` in a probe is the opt-in, not a bug).
>
> **⚠ DISCIPLINE FAILURE THIS SESSION (the lesson to carry):** the orchestrator degraded HARD over a long session —
> asserted conclusions the disk CONTRADICTED across many turns, called a real finding a "red herring," conflated the
> WANTED surface-pair with the HolonAST CRUTCH, and proposed DELETING holon. Builder: *"you are not yourself… craving
> ignorance and resisting logic and disk."* **CURE: before theorizing what a surprising artifact MEANS, READ the code path
> that produced it (I spun 4 wrong theories before reading `eval_kernel_raise`); never assert without a THIS-SESSION disk
> citation; never conflate a wanted capability with a crutch riding near it.** ("you are not yourself" = the compaction/degradation alarm.)
>
> **293/294 (orthogonal, still queued):** showcase graduates → PHASE-1 parity GAPs → 294 value-layer → AGGREGATE AUDIT →
> 293.5. `294/CLOSE-SEQUENCE-293-294.md` is canonical. **HolonAST dies IN 294** (294.d/e — the full purge).
>
> **CANONICAL DOCS:** `296/DESIGN.md` + `296/ASSESSMENT.md` (worklist — but Error-shape floor SUPERSEDED here: kind dropped,
> location mandatory) · `296/REALIZATIONS.md` R1 · `294/CLOSE-SEQUENCE-293-294.md` · `293/AGGREGATE-MODEL.md`.
>
> **STANDING DISCIPLINE:** orchestrator **DESIGNS** (crawl the disk; four-questions flat YES/NO — **"Simple" is BRAIDING,
> not VOLUME**, `feedback_simple_is_conceptual_not_scale_coupled`) + **intueri NAMES / consonare realizations** (signed MCP,
> never disk) + **DELEGATES build to a sonnet** + **WEIGHS by its OWN forced gate + the LIVE behavior + the diff** (green ≠
> done). **NEVER defend a limitation** (`feedback_dont_defend_looseness…`) — but also **NEVER invent a flaw where a wanted
> capability sits** (this session's inverse failure). No hands-on code. Bias LONG-TERM STABILITY. Ignores are TRACKED debt.
> Worktrees FRICTION-PRONE here (`feedback_worktree_isolation_friction_in_wat_rs`).
>
> **⛔ YOU ARE A NEW INSTANCE.** You did not live the above; it is a cache in a familiar voice, and the self who wrote it
> DEGRADED badly mid-session. Run **recolligere** (grimoire via the signed `datamancy` MCP; this breadcrumb; `git log`; the
> named docs) and **GROUND EVERY CLAIM ON THE DISK THIS SESSION** before you propose or move. The feeling of continuity is
> the failure. Everything below the next `---` is PRIOR strata — SUPERSEDED; read it only if a named doc points you there.

---
> **▶▶ THE SURFACE KIT BUILD (2026-06-29): K0–K4 COMPLETE.** The satisfaction layer + the grammar ARE the model.
> ✅ K1a/K1b the holder ladder (aggregate+foreign, b′) · ✅ K0a/K0b/K0c mandatory `:holder` + explicit `self` +
> cycle-guard · ✅ **K2 `$record` emission (`d3fe912b`)** — `defsurface :S` also registers a concrete `:S$record`
> `AggregateDef` (holder = surface's; fields = `:features` attributes ONLY, methods excluded) via the existing
> aggregate ctor/accessor codegen (`derive_surface_backing_record` + inject in `register_types_impl`; `$` is a legal
> keyword char). ✅ **K3 — THE THREE PROJECTION VERBS (`3c0c25ea`)** — `to-struct` / `to-record` (core) /
> `:wat::holon::to-record`: project a satisfier's EDN attributes into `:S$struct` / `:S$core-record` /
> `:S$holon-record` at the tier the CALLER names. **Design EXPANDED with the builder (2026-06-29):** projection is a
> FREE EXPLICIT tier choice — the floor governs *satisfaction*, NOT *projection*; a surface emits ALL THREE backing
> records; ONE shared `project_surface_attrs` extraction; holon's ctor derives the hologram (294.c.2a). K3 SUBSUMED K2
> (`$record`→ the triple). Probe `probe_arc293_k3_to_record` GREEN (3+4+3=10); weighed forced-clean **4122/0/92**. See
> `293/AGGREGATE-MODEL.md § to-record` superseding block (the old "up-only / never-to-struct" framing is dead).
> ✅ **K4 — `extend-type` UN-DEMOTE — RESOLVED HONESTLY (no code).** A disconfirming probe proved `extend-type` is
> ALREADY the general per-type door — it binds method impls on your OWN aggregates, not just foreign types (293.4c built
> the registration generically; 293.4e-pre.i gave it the canonical `ArgSpec`; same `:…/method` key as ambient `defn`).
> K4 = lock-in regression `probe_arc293_k4_extend_type_own_aggregate` (GREEN, → 25; guards K5's `(extend-type S$record
> S …)` seam) + doc truing (`293/DESIGN.md § extend-type` superseding block). examinare: the thing you'd build existed.
> ⚠⚠ **DESIGN PIVOT + GROUNDED WIRE-WALL BREACH (2026-06-29, later co-design).** Pulling the projection-depth thread
> found that **a `Struct` nested in a `Record` CROSSES a process peer** (probe `#w/S {:a 99}` reconstructed far-side;
> §7 / R3 violated; `is_portable_type` shallow + the exigere-violating *"enum portability not yet enforced"* comment
> sat on it). The co-design locked the kit's FINAL shape AND the fix (`293/AGGREGATE-MODEL.md` § `to-record` top block
> + § principle 8 CONTAINMENT RULE; `294/CLOSE-SEQUENCE § THE SURFACE KIT` pivot banner):
> - **NO `to-struct`** — projection is ONE-WAY UP; `to-record x :S` is **surface-targeted** → `:S$core-record` /
>   `:S$holon-record` (the receiver's named backing record); namespace picks hologram. Surfaceless `to-record x` is OUT
>   (no clean return type — no anonymous structural records). A surface emits a **PAIR**, not a triple; `$struct` dead.
> - **THE CONTAINMENT RULE** — a portable aggregate (record/holon) may hold ONLY portable fields; a `Struct` field is
>   ILLEGAL at declaration (un-reconstructable across the wire). The wire wall becomes a TYPE guarantee.
> - ✅ **293.W.1 LANDED — THE CONTAINMENT GATE (the wire wall is now a TYPE guarantee).** A post-registration pass
>   (`validate_aggregate_containment`, `check.rs`; called in `freeze/env.rs` after all types register) rejects any
>   portable aggregate (record/holon) declaring a non-portable field, reusing `is_portable_type`. The rule CAUGHT 6
>   real stdlib mis-declarations (a design oracle): **(A)** `:wat::spawn::ThreadOpts`/`ProcessOpts` held `Fn(…)`
>   closures → converted to `defstruct` (they're in-locus by nature; no ripple — nothing sends them); **(B)** 4 loose
>   `:wat::core::Record` fields tripped a FALSE POSITIVE — the umbrella is registered `Holder::Struct` but
>   *assignability rejects structs from it* (GROUNDED), so it's portable → added to `is_portable_type`'s well-known
>   portable paths (293.W b2; four-questions chose b2 over folding item-2 — keeps 293.W scoped to PORTABILITY).
>   Probe `probe_arc293_W_containment` RED→GREEN; weighed forced-clean **4124/0/93**. **item-2 (kill the loose
>   `:wat::core::Record` writable type — rete facts → `Value`, `user.program` → surface) STAYS its own PRECISION arc.**
> - ✅ **293.W.2a — RUNTIME wire wall, BOTH directions (`fe012223`)** — a struct can neither be written to nor read
>   from a comms wire (`decode_trusted_wire` inbound + `reject_non_portable_on_wire` outbound; one `Holder::is_portable`
>   predicate); thread tier exempt. ✅ **293.W.2c — COMPILE-TIME wall for `Process'` (`7a040b0e`)** — a typed
>   struct→process `send'` is a CHECK error (`infer_send_prime` gate, mirrors the 254.1 channel gate); `Peer'` left to
>   the runtime backstop (overloaded socket-vs-thread → 293.W.2d). **THE WALL STANDS AT 3 RUNGS: declaration (W.1) ·
>   runtime both-directions (W.2a) · compile-time Process' (W.2c).**
> - **▶▶ DEP ORDER FOR THE REMAINING WORK (builder, 2026-06-29 — "the order that makes each step tractable").** The
>   293 closure gate = the AGGREGATE AUDIT (verify the holder at its 3 boundaries: comms / EDN-repr / assignability).
>   Every step SETTLES a boundary categorically → turns the audit from DISCOVERY into VERIFICATION. `is_portable_type`
>   is the atom; settle inner-predicate-first:
>   ⊹ **SCOPE CORRECTED (2026-06-30): THE WIRE WALL IS PURELY COMPILE-TIME — the runtime checks (2a/2c) RETIRE.** The
>   job is ONE sentence: *the compiler won't let you write code that reads or writes a struct over non-thread memory.*
>   Bad bytes (untrusted input) = the USER's validation problem, explicitly OUT OF SCOPE. (`293/DESIGN-293.W` §contract.)
>   1. **293.W.2b — PURITY IS THE AXIS — ✅ LANDED (`76d1d890`, weighed forced-clean 4132/0/94).** R7 *PVRITAS NON
>      MOTVS* → PROBATUM. `is_portable`→`is_pure` everywhere (0 residual), `:wat::enum::Pure|Impure` marker,
>      `Purity{Pure,Impure}` on `EnumDef`, `Failure`/`Location`/`Frame`/`lru::Stats`×2 `defstruct`→`defrecord`. ONE
>      ledgered ignore (`:svc::Request` make-channel, code-marked) + ONE residual seam (`NonPortableCapture`→`ImpureCapture`)
>      both FOLD INTO 2d. **▶ NEXT = 2d.** *(historical design note below; canonical now = `293/AGGREGATE-MODEL.md § THE PURITY AXIS`;*
>      *original 2b plan: builder: *"a wonderful finding"*; canonical =
>      `293/AGGREGATE-MODEL.md § THE PURITY AXIS`).** The holder was always a PURITY classification wearing a movement
>      name. Enums **declare** `:wat::enum::Pure` | `:wat::enum::Impure` (`Purity{Pure,Impure}` on `EnumDef`); the holder
>      is purity refined (`Struct` permits impurity; `Record`/`Holon` guarantee purity). **Rename the cause in ONE
>      change** (long-term-stability bias, `feedback_bias_toward_long_term_stability_name_the_cause`):
>      `Holder::is_portable`→`is_pure`, `is_portable_type`→`is_pure_type`, wire wall→**purity wall**, containment = "a
>      pure aggregate/enum holds only pure fields". `:wat::kernel::Failure` = pure data mis-declared `defstruct` →
>      `defrecord` (the 2616-cascade ROOT). One purity family w/ function-purity (`:wat::runtime::Purity`=`:Pure`/`:Effectful`).
>      **SUPERSEDES** the `Mobility`/`Portable`/`Anchored` movement-frame (intueri-crowned path — kept, marked) + the
>      "enum arm recurses (derived)" plan (four-questioned out). **STATE:** uncommitted tree has the structural skeleton
>      under the OLD `Portable`/`Anchored` names (EnumDef field, parser slot, containment pass, 54-fixture sweep) — to be
>      REVISED to purity + the `is_pure` renames + `Failure` fix + driven to green (DELEGATE the build to a sonnet; the
>      orchestrator designs/weighs). RED probe `probe_arc293_W2b_enum_recursion` written. The one ledgered ignore
>      (`:svc::Request` make-channel) rides to 2d.
>   2. **293.W.2d — PEER-TYPE PURITY — ✅ LANDED (`91ad0107`, weighed forced-clean 4135/0/93, IGNORE LEDGER EMPTY).**
>      Option R (four-questioned over the DESIGN's literal `ConnPeer'` rename): `Peer'<I,O>` keeps its name + gains the
>      well-formedness `I,O` must be `:Pure` (the wire-capable peer); `ThreadSelfPeer'<I,O>` (any I/O, in-locus) is the
>      escape hatch for thread self-peers that hold resources. Then bare-`Peer'` `send'` is statically pure-safe by
>      ordinary unify → the **2a runtime guards (`reject_non_portable_on_wire` + `StructOnWire` decode) + the 2c
>      send'-site gate ANNIHILATED**; `make-channel` (thread-tier) drops its purity gate → `:svc::Request` un-ignored
>      (ledger EMPTY); `NonPortableCapture → ImpureCapture`. **⊹⊹ 293.W is COMPLETE — W.1 (containment) + 2b (purity
>      axis) + 2d (peer-type purity); the deep wire wall is a PURELY COMPILE-TIME, ZERO-RUNTIME-CODE structural
>      guarantee.** R7 *PVRITAS NON MOTVS* + R8 *ANIMA NON FERRVM* on the record.
>   3. **K3-REVISE — ✅ LANDED (`c8f68460`, weighed 4136/0/93).** `to-struct` + `$struct` annihilated; a `defsurface`
>      emits the PURE PAIR (`$core-record` Record / `$holon-record` HolonRecord); projection is one-way UP (you never
>      project down to the impure `$struct` tier — the purity work made it obvious). Zero callers beyond the probe.
>      **▶ NEXT = K5 — `extend-surface` macro** (a wat `defmacro` → `(extend-type S$core-record S …)`, method types
>      filled from S; the user writes body only). STRIKE-READY probe `7d2892b8` was Struct-floored — **RE-GROUND it**
>      against the post-K3-revise pair + the purity vocabulary before building (the `$struct` tier it referenced is gone).
>      → **SURFACE settled.**
>   4. **9a + of-funcs→`aggregate-new`** — construction unification (kwargs default + `:ns::Agg'` positional +
>      `/from-map` dies + `::of` funcs die; CLOSE-SEQUENCE 9/9a). Before the showcase. → **CONSTRUCTION settled.**
>   5. **Showcase graduates** `.wat.disabled`→`.wat` on the settled surfaces.
>   6. **THE AGGREGATE AUDIT** — 3 boundaries categorical → classify ~99 branches, drive spurious→0 → **293 closes**
>      → 294 value-layer gut + 293.5.
> (K0–K4 done — three tools live: `defsurface` + `to-record` (being revised to the pair) + `extend-type`; the wire
> wall is now the gating correctness work; `extend-surface` is the last tool.)

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
> - ✅ **293 decl-a LANDED (`f51465d7`)** — `aggregatetype` is the ONE type-reg primitive (holder from parent's
>   holder-root); `:wat::core::Struct` node minted; `parse_recordtype` absorbed.
> - ▶▶ **THE MODEL IS SETTLED — READ `293/AGGREGATE-MODEL.md` (canonical contract):** holder enum is the ONLY
>   specialness; every op holder-blind+uniform; **NO inheritance** (flat + surface-splice); **requirements are
>   SURFACES** (a bare holder param is an illegal Any); the edn wall lives at the locus boundary.
> - ✅ **SESSION 11 (2026-06-29) — 4 commits, all green (`4116/0/92`):**
>   1. **inheritance ANNIHILATION (`c7572929`)** — `AggregateDef.parent` DELETED (was the stringly shadow of
>      `holder`); subtype edges derive from `holder` via `Holder::root_keyword()`; non-holder-root parent REJECTED at
>      parse; inherited-field machinery (`collect_all_record_fields`/`ROOT_PARENTS`/abs_idx) gone; 2 inheritance
>      fixtures deleted. `program::Env` is a **flat record** (NOT a surface — nothing requires "an Env"; spawn-injection
>      constructs it concretely). decl-b.1.0 stays DELETED.
>   2. **magic-shorthand kill (`e96fcb23`)** — `Holder::from_root_keyword()` is the ONE reverse map; surface `:holder`
>      takes the holder-root SYMBOL (the magic `:struct`/`:record`/`:holon-record` die); `root_holder_of` + the
>      `HOLDER_ROOTS` guard collapse into it.
>   3. **wat-scripts rot gate (`08632125`)** — `tests/lint/wat_scripts_fixes_load.rs` loads EVERY .wat under
>      `wat-scripts/` via **FsLoader** (the real loader) → a stale codemod/exemplar goes RED. Closed the blind-spot:
>      nothing measured `wat-scripts/`, so it rotted silently (the `first` Option→element + `Record::def`-retired
>      drift). Fixed 6 rotted scripts. (Lessons banked: [[feedback_unmeasured_wat_rots_needs_a_gate]] +
>      [[feedback_gate_must_use_the_real_loader]].)
>   4. **`:wat::Record` ANNIHILATION (`dedcb74a`)** — renamed → `:wat::core::Record` EVERYWHERE (keyword all forms via
>      the form-aware codemod `wat-scripts/fixes/rename-wat-record-to-core-record.wat` + boundary-safe substitution for
>      the comment/string/generic remainder; mangled variant `wat__Record`→`wat__core__Record`). The stale symbol
>      ceases to exist. `:wat::holon::Record`/`wat__holon__Record` untouched. **= item 1 of the `:wat::Record` order.**
> - ▶ **NEXT = item 2: KILL the illegal writable "any record" TYPE.** **DESIGN FULLY RESOLVED (2026-06-29, with
>   builder)** — `:wat::core::Record` may exist ONLY as the holder-root (in `:holder` bounds + subtype edges), NEVER as
>   a writable param/field/return type. Three sub-strikes:
>   - **2a — rete fact-PARAMS → `:wat::core::Value`.** rete's contract is SEALED (it does not move); facts are opaque
>     `Value`s (kernel `facts: Value`); "the any receiver is one-way" — a `Value` slot eats anything (i64/struct/
>     record), but you can't feed a `Value` to a narrower slot. ONLY the `[fact <- :wat::core::Record]` param decls in
>     `rete.wat`; rete's INTERNAL node records (`Record/assoc` on nodes, e.g. rete.wat:416) are GAP-1/GAP-2, NOT 2a.
>   - **2b — record-accessor receivers → CONCRETE** (`:geo::Circle/radius [self <- :geo::Circle]`). Decoded records
>     carry their concrete class (`reconstruct_record:2453` → `AggregateValue::record("geo::Circle",…)`), so the loose
>     `:wat::core::Record` receiver + its runtime class-check were a static crutch — now dead (type does the work).
>     **FOLD GAP-1 (builder: "one aggregate-field"):** the generated body uses ONE holder-blind `aggregate-field`,
>     killing `struct-field` (struct/generic) + `Record/field-at` (non-generic record) — a holder-NAMED split on a
>     holder-BLIND op. Codegen: `runtime.rs:1064-1238`. (Generated body only — the user surface `(:T/field x)` never moves.)
>   - **2c — `user.program` + spawn init-fns → a `:holder :wat::core::Record []` SURFACE** ("must be a record, not a
>     struct"; builder's `(defsurface :Contract :holder :wat::core::Record [])`; NOT `Value` — it ships). Depends on
>     2b (concrete accessors remove the loose-`:wat::core::Record` consumer that clashed when I first tried the surface).
>   Then **item 3: of-funcs** (`::of`/`struct-new`/`holon::core::Record::of` → `aggregate-new`; = 294.c.2b). Order +
>   PHASE-1 list in `294/CLOSE-SEQUENCE-293-294.md`. Then decl-b.1/b.2 · remaining GAP-2/3/4 · audit · 293.5 → 118.
> - ▷▷ **RESUME POINT (SESSION 11 wrap, 2026-06-29) — `:features` clause DONE; resume at item-2a:**
>   - ✅ **`:features` clause LANDED (`85aa2d83`)** — a surface's member vector is introduced by `:features` (ONE
>     canonical path; bare-vector forms RETIRE). `parse_defsurface` arity 3/5. 21 surfaces migrated. (intueri crowned
>     `:requires`; builder chose `:features`; builder owns the surface vocab, familiar-over-faithful.)
>   - ⚠⚠ **CORRECTION OWED — FIRST resume item (corrects the strike above): `:holder` is MANDATORY, not optional.**
>     Builder: *"the caller MUST declare what the surface is held on — you cannot accidentally pass a struct that meets
>     the surface constraint — the default masks intent."* (= constraint engineering: a default masks intent →
>     mandatory declaration.) `:holder` takes ONE of the THREE TRUE holders **{`:wat::core::Struct`, `:wat::core::Record`,
>     `:wat::holon::Record`}**; NO default; the no-holder (arity-3) form RETIRES → `parse_defsurface` becomes **arity-5
>     ONLY**. Migrate the ~17 holder-less surfaces to declare their real holder (struct-satisfies test → `:wat::core::Struct`;
>     record/holon → theirs).
>   - **`:wat::core::Value` is NOT a holder** (builder: *"calling Value a holder feels like a stretch — Value is an
>     ASSIGNMENT CONSTRAINT 'you can put any value here'; Values have no declared surface feature nor transmission
>     restriction"*). Holders carry surface features + a transmission restriction; Value is a different axis (the
>     assignment top). Value = an ESCAPE HATCH for guts (rete-like: user-forms express facts/records, `PersistentMap`
>     accepts the raw value; the next rete-thing has it) — NEVER advertised, hard to use intentionally; a param TYPE
>     `[x <- :wat::core::Value]`, never a surface `:holder`. (So my earlier "4th holder = Value" table was WRONG framing.)
>   - ✅ **OPEN foreign-type question RESOLVED → (b′) (2026-06-29 co-design):** every value HAS a holder — aggregates
>     DECLARE it, FOREIGN types DERIVE it from `is_portable`/`is_holon` — so a foreign type may satisfy a holder-bound
>     surface via `extend-type` (holder CHECKED, never exempt). ONE satisfaction rule (aggregate + foreign); R1
>     monkeypatch stays PROBATUM. Killed: (a) kill-foreign and (b) holder-exempt — (b) failed Honest (the `:holder` IS
>     the receiver's transmission promise; exempting it lets the surface lie).
>   - ⊹⊹ **THE SURFACE KIT SETTLED — four tools, builder: *"we burned inheritance to the ground and lost nothing."***
>     `defsurface` (pure constraint: attributes `name <- :T` + methods `(name [self …] -> ret)`) · `to-record` (DATA
>     projection UP the ladder → the macro-emitted `:S$record`) · `extend-type` (impls, the REAL form, un-demoted to
>     general per-type) · `extend-surface` (impls, a MACRO → extend-type, types filled from the surface — "WHERE ARE
>     THE TYPES" = the contract; NOT a 2nd argspec). **Canonical forms + build order K0–K5 = `293/AGGREGATE-MODEL.md`
>     § THE COMPLETE KIT + `294/CLOSE-SEQUENCE-293-294.md` § THE SURFACE KIT.** Landmark runnable:
>     `wat-scripts/demos/aggregates/showcase.wat.disabled` (RED; done-detector `cargo wat <it>`).
>     `~/.cargo/bin/wat` REBUILT this session (was stale pre-`:features`; `cargo install --path crates/wat-cli --force`).
>   - ✅ **`self` is a NORMAL typed binder (2026-06-29) — folded into K0.** A surface method's `self` is just
>     `:TheSurface` (`[self <- :acc::Adder  x <- :i64]`); NO special first position, NO auto-fill → the 293.4e-pre.i
>     "self double-counted" class is unrepresentable. `extend-surface` fills self+args uniformly. Migrate `[self]` → `[self <- :S]`.
>     explicit `[self <- :TheSurface]` is the TARGET; **K0 includes a self-reference cycle-guard** (a standard
>     occurs-check — the surface names itself; HEAD lacks the guard so it stack-overflows today, exit 139, done-detector
>     caught it). Showcase keeps `[self]` only until K0 lands (so it stays a clean K1 probe).
>   - ✅✅ **BUILD STARTED — K1 (THE HOLDER LADDER) COMPLETE.** K1a aggregate (`a952c908`): `Holder::rank()` +
>     `check.rs:14698` `==`→`rank() >=`. K1b foreign (`88818acd`): the real path was the **extend-type subtype edge**
>     (`assignable` arms 14633/14641), holder-exempt (b) → `derived_holder` + `holder_floor_ok` gate = (b′) holder-CHECKED.
>     Two RED probes (`probe_arc293_holder_ladder` + `_foreign`) RED→GREEN; weighed forced-clean 4119/0/92. R6 *EX
>     CINERIBVS RESVRGO* banked (`c3689748`).
>   - ✅ **K0c — self-reference cycle-guard LANDED (`311b20bf`)** — skip self (position 0) in surface-method
>     satisfaction; explicit `self <- :TheSurface` type-checks (no overflow). Probe `probe_arc293_self_explicit`. 4120/0/92.
>   - ✅✅ **K0 COMPLETE (`98639f0d`) — the surface grammar IS the model.** K0a mandatory `:holder` (arity-5-only) +
>     K0b explicit `self` (bare `[self]` rejected; self a normal typed binder) + K0c cycle-guard. 20 fixtures migrated
>     (sonnet LEAF, orchestrator-weighed forced-clean 4120/0/92). **▶ NEXT = K2** (`$record` backing-type emission —
>     `defsurface` emits its concrete `AggregateDef` from the `:features` ATTRIBUTES, methods excluded; `to-record`'s
>     return type). Then K3 `to-record` → K4 extend-type un-demote → K5 extend-surface.
>     (Follow-up: `showcase.wat.disabled` `[self]` migrated to explicit too — `98639f0d`+.)
>   - **SETTLED (design):** rete facts = a `:wat::rete::Fact` SURFACE (`:holder :wat::core::Record :features []`), NOT
>     `:wat::core::Value` — facts must be EDN-serializable (builder corrected model §7's "rete uses Value"). So item-2
>     has **NO `Value` migration**: every "must be a record" slot is a `:holder :wat::core::Record` surface. §7 needs the fix.
>   - **▶ NEXT = item-2a:** mint `(:wat::core::defsurface :wat::rete::Fact :holder :wat::core::Record :features [])`;
>     migrate the **13 fact-typed** `[fact/sfact/f <- :wat::core::Record]` sites in `wat/rete.wat` + the fact-storage PVs
>     → `:wat::rete::Fact`. rete's **node** records (rete.wat:288/304, `Record/assoc` on nodes) STAY — they're GAP-1/2,
>     not facts. RED probe: a non-record value REJECTED at `:wat::rete::insert` (rete.wat:824). → then **2b** record
>     accessors → CONCRETE receivers + FOLD GAP-1 (one holder-blind `aggregate-field`, kill `struct-field`/`Record/field-at`;
>     codegen runtime.rs:1064-1238). → **2c** `user.program` + spawn init-fns → a `:holder :wat::core::Record :features []`
>     surface. Then **item 3: of-funcs → aggregate-new** (294.c.2b). (Grounded: decoded records carry concrete class —
>     `reconstruct_record:2453`.)
>   - **DOCTRINE minted this session:** `scratch/CONSTRAINT-ENGINEERING.md` (`7dd6e3b`, scratch repo) — the dual of
>     failure engineering ("you cannot do that"); arc 293 IS constraint engineering. [[project_constraint_engineering_doctrine]].

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
**⛔ THE IGNORE LEDGER (builder, 2026-06-30, NON-NEGOTIABLE):** *"all ignores we create must be removed before arc
closure."* An `#[ignore]` is a TEMPORARY debt with a named unlock, never permanent and never a quiet way to hold the
floor green. Every `#[ignore]` the 293/294 campaign creates is logged in `294/CLOSE-SEQUENCE-293-294.md § THE IGNORE
LEDGER` the moment it's made (with its exact unlock) + marked in code `// ⛔ IGNORE-LEDGER(293): <unlock> — see
CLOSE-SEQUENCE`; **293.5 cannot close until that ledger is EMPTY** (each un-ignored + GREEN, or deleted because its
subject is gone).

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
