# ⛔ CURRENT STATE (breadcrumb, 2026-06-25 SESSION 4; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. Freshness probe: HEAD should be `bd0935db` (293.3-records SHIPPED) or
later — COMMITTED + PUSHED to DR. Tree CLEAN (only `clara-tools/` ignored).

## ▶▶ STATE @ 2026-06-25 (SESSION 4) — 293.2-parity SHIPPED; the BASE-STRUCT UNIFICATION is the arc spine (R2)
Session 4 ran on the holon/aggregate model. **Read `293-…/REALIZATIONS.md` R2** (Break Stuff / *FRANGE UT UNUM FIAT*)
**+ `293-…/NOTE-base-struct-horizon.md`** (the VERIFIED section) whole before moving.

**SHIPPED this session (DR, all weighed pure against the disk):**
- **293.2-parity (`560535a5`)** — `defstruct` is now a wat MACRO over `:wat::core::structtype` (peer to
  `defrecord`→`recordtype`). Weighed: SET-diff ∅ (baseline 201 == with-changes 201; the 2 `parse_*_form` doctest
  deltas are a ±1 line shift; `s0_t1a` confirmed pre-existing floor). **Honest deferral:** the now-dead `defstruct`
  `classify_type_decl` arm is KEPT (≈10 `src/types.rs` unit tests bypass macro expansion) — QUEUED: migrate them to
  `structtype` + annihilate the arm.
- **capability correction (`dae4bf3d`)** — edn-repr is a SUBSTRATE property of the PRIMITIVE, not a macro/label:
  `is_portable_type` keys categorically on the `TypeDef` variant (`check.rs:13056` Record→true / `:13061` Struct→false).
  `(structtype …)⟹Struct⟹never edn-repr`; `(recordtype …)⟹Record⟹must`. (Stale: `check.rs:12990` doc-comment still says field-recursion — fix it.)
- **holon model VERIFIED + folded (`f110d22e`)** — Explore agent + greps: holon records ARE edn-portable; EDN↔hologram
  are two encodings of the SAME data; `holon_form` is a derived cache (pure fn of fields); the wire ships the
  holon_form-as-EDN (struct_form PROJECTED, NO recompute — `edn_shim.rs:2480-2506`, arc 234.7b); dense vectors never
  cross; parity by immutable `assoc` rebuild (`runtime.rs:13706-13778`). **holon = a structural+repr REFINEMENT of record, NOT a third wire-kind.**
- **293.3-records (`bd0935db`) — records carry typed fields → core AND holon records satisfy surfaces** (R2
  headline LITERAL in the checker). Both record macros emit the typed `recordtype` form (`[~@fields]` — DELETED
  the name-strings holon round-trip) → `RecordDef.field_types=Some`; `assignable` gained a Record arm mirroring
  the Struct arm. Weighed pure: probe `probe_arc293_record_surface` 3 GREEN; `core_record_def` deftests 7 GREEN;
  all SocketAddressWire consumers GREEN; floor 201 (the 6 `arc234_stone2a` nursery fails are PRE-EXISTING — proven
  identical at baseline `919f825e`). FORCED 2 fixes: `register_record_methods` skip-when-ctor-registered (macro
  records now look like direct recordtype records; genuine-dup guard relocated to the defn path); `spawn.wat:35`
  `SocketAddressWire.name` illegal `Vector<:wat::core::i64>` → `<wat::core::i64>` (a latent unchecked-type bug the
  typed emission exposed). `RecordDef` now carries the same typed-field data as `StructDef` → merge-ready.

**THE MODEL (R2, settled this session):** struct / record / holon-record are ONE backing — `{ properties-as-struct,
kind-as-enum }`. VALUE level: all three are `(class, fields)` (`StructValue` `value/value.rs:959` ≡ `wat__Record`;
holon adds the derived `holon_form`). TYPE level: `StructDef.fields` is ALREADY typed `Vec<(String,TypeExpr)>`;
`RecordDef.field_types` is the lone `None` (`types.rs:2131`). **ONE categorical wall = struct vs record (edn-repr);
holon is structural+repr above it.** kind-as-ENUM (never `Option` — the no-Option-semantics doctrine).

**▶ THE SPINE (R2 — the base-struct unification; strike 1 DONE):**
1. ✅ **293.3-records — records carry typed fields → satisfy surfaces** (`bd0935db`, SHIPPED above). The first cut:
   it both delivers the R2 prize AND makes `RecordDef` field-compatible with `StructDef` (the merge precondition).
2. **NEXT — unify the DEF (the AggregateDef merge):** `StructDef`+`RecordDef` → `AggregateDef{name,
   fields:Vec<(String,TypeExpr)>, kind, parent?}`. Both now carry typed fields → the merge is structural +
   behavior-preserving (SET-diff ∅ oracle, no single RED probe). Def-cascade RECON'D: `TypeDef::Struct` 50 / `TypeDef::Record`
   38 / `StructDef` 28 / `RecordDef` 28 sites, ~16 files. Ride the cascade; the trap = per-kind identity
   (struct=`StructValue.type_name`+fields; record=`class_fqdn`+`struct_form`; holon adds `holon_form`).
3. **THEN — unify the VALUE:** 3 `Value` variants → `Aggregate{class, fields, kind}` + derived hologram. Cascade
   recon'd: `Value::Struct` 82 / `wat__Record` 98 / `wat__holon__Record` 76 sites, 21 files. User forms UNCHANGED.
   PERCEIVE-THE-TRAP first: identity-over-`holon_form` ≡ identity-over-`(class,fields)` — the probe must PROVE it.
**NEXT ARTIFACT = recon + disconfirming approach for strike 2 (the AggregateDef merge).** Old 293.3-core (struct
surfaces) SHIPPED `313e7d85` / R1 = *FORMA SOLA SUFFICIT*; R2 (base-struct unification) = *FRANGE UT UNUM FIAT*.

**THE GATE (293 close) = the demo** (`DESIGN.md` § What the arc delivers): Shape/Circle/Square + holon-Vector
monkeypatch, RED→GREEN. **R2 FULFILLED** when the unification lands (user forms unchanged, SET-diff ∅) → *PROBATUM EST*.

> ⚠ The SESSION 3 block below is PATH (the arc-293 opening + the names-crowned + 293.3-core ship). Its sequencing
> ("/from-map → 293.3-records → 293.4") is SUPERSEDED by the R2 spine above — the unification subsumes those.

## ▶▶ STATE @ 2026-06-25 (SESSION 3) — ARC 293 OPENED: the aggregate type system (291 is BLOCKED on it)
**A small ask (`/from-map` ergonomic ctors) detonated into a foundation** — by a long co-design the builder
**re-derived a structural type system.** NEW ARC: `docs/arc/2026/06/293-struct-record-symmetry/` (read DESIGN + R1 whole).
- **THE MODEL (DESIGN `df4b4480`):** records & structs are ONE aggregate citizen differing only in the **EDN
  kind-wall**. **HOLDER nominal** (struct / core-record / holon-record — the EDN+VSA capability; you declare it).
  **SURFACE structural** (row-polymorphic: a *set-of-accessor*; ambient satisfaction, width subtyping, open-world;
  **NO `:satisfies`, NO `:parent`**). **Methods ARE accessors** (field/method seam dissolves; `definterface`
  SUBSUMES `defprotocol`; `extend-type` demoted to the typed foreign-accessor adapter = the monkeypatch /
  Expression Problem). **`definterface` = a named ARGSPEC** (four-questioned over typealias; reuses `src/argspec/`).
  Param = holder ∩ surface. **ANNIHILATED:** `:parent`/inheritance · `register_*_methods` · surface-edges · the
  phase-order problem (structural fit checked post-registration where `assignable` runs — `freeze.rs:2202` invariant
  STAYS). Reprs UNTOUCHED (wire law variant-level).
- **THE GATE = the demo** (`DESIGN.md` § *What the arc delivers*): the Shape/Circle/Square + **holon-Vector
  monkeypatch** program — RED at HEAD → GREEN closes the arc.
- **R1 inscribed (`3ad1c7ba`; Song: Beartooth *My New Reality*; *FORMA SOLA SUFFICIT*):** structural surfaces
  re-derived by *hating `parent`* (the 291-R1 protection again); WE-LAND-ON-THE-GREATS four doors deep — row
  polymorphism (Wand/Rémy/Cardelli/OCaml) + Go/Haskell-typeclass/Clojure-protocol Expression Problem + Kay
  messaging — fused with the genuinely-ours nominal-EDN-holder. REALIZATION earned; build a PROPHECY (`Probandum est`).
- **TEST-SPEED levers SHIPPED (`fe3fdcea`):** mold linker + dev `line-tables-only` (1:05.9→0:57.2, proven). (Lever 3 /
  probe→home consolidation is still the deferred 170-closure, execve-leak-blocked.)

**NAMES CROWNED (intueri 2026-06-26):** `defsurface` (structural-surface form; SUBSUMES `defprotocol`) ·
`Surface`/`TypeDef::Surface` (concept+variant) · `src/aggregate/` (home) · `extend-type` KEPT, demoted to the
foreign-type accessor adapter. **DISPATCH BOUNDARY:** surfaces = SINGLE-dispatch on the receiver (full-arity
methods); MULTI-dispatch = `defclause` (arc 237, `form_match.rs`); `typeunion` = closed sums. Surfaces never grow
multi-arg dispatch.

**SHIPPED this session (DR, all weighed pure):**
- **293.3-core (`313e7d85`) — STRUCTURAL SURFACES, GREEN.** A struct structurally satisfies a `defsurface` by
  HAVING its members (row-polymorphic width subtyping; ambient, no `:satisfies`/`:parent`). `TypeDef::Surface` +
  `parse_defsurface` + the width-match `struct_satisfies_surface` — ALL homed in `src/types/surface.rs`; a thin
  resolve-and-call arm in `assignable` (`check.rs:14233`). Probe `probe_arc293_structural_surface` GREEN (positive
  + negative). The riskiest type-system machinery is **de-risked + real.** Scope = STRUCTS (`StructDef.fields` is
  typed → TypeEnv-clean). DEFERRALS: records ride 293.2-parity (`RecordDef.field_types=None`); runtime `conforms?`
  for surfaces returns `Ok(false)` (compile-time matching is the keystone).
- **test-speed (`fe3fdcea`)** mold + line-tables.

**⏳ IN FLIGHT — A SONNET IS BUILDING `293.2-parity` (agentId `a389d35045567618d`).** STRIKE-READY @ `cd208ba3`
(probe `probe_arc293_structtype_primitive` + `BRIEF-293.2-parity.md`). The work: make `:wat::core::defstruct` a
thin wat MACRO over a new `:wat::core::structtype` primitive (mirror `Record::def`→`recordtype`), so defstruct↔
defrecord are SYMMETRIC at the macro-over-primitive level. Behavior-preserving (`register_struct_methods`
UNCHANGED); SET-diff ∅ is the gate. **⚠ NEXT INSTANCE: check the sonnet's result (its CODE is UNCOMMITTED in the
tree), WEIGH it (re-run the structtype probe ±`--ignored`, the surface probe stays 2-green, SET-diff vs `313e7d85`
= ∅ / ~202 floor), then COMMIT on green — or read its STOP report if it halted.**

**▶ THEN, in order (each its own small strike — parity-before-feature, builder doctrine):**
`/from-map` for records+structs (UNIFORM, only AFTER parity — was wrong to do record-first onto the asymmetry) →
`293.3-records` (records carry field types → satisfy surfaces, same `assignable` arm) → `293.4` methods-are-
accessors (the dispatcher + `defprotocol` annihilation + `extend-type` demotion + runtime `conforms?`) →
`Record::def`→`defrecord` rename (fix-wat + retirement) → `293.1` the `src/aggregate/` home → `293.5` close.

**THEN (291, now BEHIND 293):** TRUST LEG (process-tier bridge GREEN = R1 FULL PROBATUM) → ACYCLICITY probe → R1
amend → PAUSE → 291 INSCRIPTION. **`/from-map` is SUBSUMED into 293** (falls out of the shared emission layer).

> ⚠ The SESSION 2 (kwargs) + SESSION 1 (marathon) blocks below are **PATH** — kept for the 291 lineage. Their
> "NEXT = /from-map" is SUPERSEDED by arc 293 above. 291's close-legs (trust/acyclicity) still stand, now behind 293.

## ▶▶ STATE @ 2026-06-25 (SESSION 2, post-kwargs) — macros^unbounded + hygiene law + kwargs-start ALL SHIPPED
**The session opened on (builder):** *"prove clojure expressivity in a strongly typed lang."* **DONE + SHIPPED.** The
defservice lifecycle is now all-kwargs (Form A): `(worker/start :locus L :record R :recorder-addr A)` — Clojure keyword
ergonomics, every arg compile-checked. *"i got ruby and clojure on rust."*

- **macros^unbounded + the hygiene Dredd gate (`910b9bcd`).** The 260.1b hoist was one-level; "macros³" was just the
  shallowest break. The defmacro hoist now recurses+flattens+elides through nested `do`s at ANY depth (R10 DEEPENED;
  depth-4 probe green). The hygiene gate: `check.rs:3416`'s `None => fresh.fresh()` silently swallowed unbound locals —
  hiding scope-divergent binders (a macro rebuilt a binder from its NAME, stripping its `ScopeId`). Now a compile-time
  **`HygieneScopeDivergence`** refusal: detection homed in `src/scope/resolution.rs`, the `// "I AM THE LAW." — Dredd`
  mark on the `check.rs` thin call. Kill at source (DOCTRINE: **reuse the node, never rebuild a binder from its name**):
  `wat/Record.wat` ctor + `wat/core.wat` kwargs `$impl`. Anaphoric witness brought true (compile-time refusal).
  **R11 inscribed** (Ruin / Lamb of God / **LEX NON TACET** — the silent swallow sought favor and murdered the self;
  the law that won't look away, at three altitudes: the gate stops the checker, the weigh stops the hand, "prove it"
  stops the guess. The art of ruin IS the art of the datamancer).
- **kwargs-`start` (`055c00f4`).** `service.wat` start/resume → `& [argspec]` Form A (`locus-sym` minted once + shared
  across params+body — proactive hygiene, binder≡body); synth default-init param `d`→`record`; 16 positional sites
  migrated. All defservice lifecycle deftests green by re-run; SET-diff ∅ (203=203 floor; the 2 "service" matches are a
  process-reactor + an arc-170 readln test, outside the blast radius). **Clojure expressivity, PROVEN + SHIPPED.**

**▶ (SESSION 2's "next" — ⊘ SUPERSEDED by ARC 293; `/from-map` is SUBSUMED into it, falls out of 293's shared emission layer):**
Builder: *"we're making ergonomic structs and records before we move onto trust."* Same kwargs lever, two beneficiaries:
an additive named/map ctor emitted by BOTH `Record::def` (lowers to the positional ctor / `Record::of`) AND `defstruct`
(lowers to `struct-new`). `(Point :x 1 :y 2)` AND `(Point {:x 1 :y 2})`; positional `(Point 1 2)` stays canonical (zero
break). **NAME CROWNED (intueri + builder): `/from-map`** — `(my::Record/from-map :count 0)` / `{:count 0}`. Honest for
both forms; echoes Clojure's `map->Record`; beat `/from` (underspecified), `/of` (clashes `::of`), `/new` (clashes the
struct positional ctor). Mechanism: reorder `:field`→positional via `pascal->kebab-in` (arc-265), delegate to the
existing positional ctor. Rides macros^unbounded + the hygiene gate (both bulletproof). **DRAW the brief (both emitters,
additive) → fire a sonnet → weigh against the disk.** (NO arc minted — a capability strike in the 291 flow.)

**THEN the 291 close:** TRUST LEG (bridge process tier GREEN = R1 FULL PROBATUM — the locus-dispatched introduction;
post-spawn hook #237 + `allow'` #236 BUILT, the introduction-glue pending) → ACYCLICITY probe → R1 amend → PAUSE
(builder's beat) → 291 INSCRIPTION.

**DEFERRED (captured, NOT now):** (1) **TEST-SPEED** → `docs/arc/2026/05/170-program-entry-points/NOTE-TEST-SPEED-CONSOLIDATION.md`:
mold + debuginfo (independent quick wins, READY, builder installed mold; ~3.5min baseline — TIME it, don't assert) +
the probe→home consolidation (**THE 170-CLOSURE deliverable** — blocked on the execve leak AND proves it dead). Builder:
*"tackle this hard but not now; the tackle is 170's closure."* (2) **255** declaration-position-class diagnostic
(`255-…/NOTE-declaration-position-class-guard.md`).

> ⚠ The block below ("STATE @ 2026-06-24 (day, marathon)") is SESSION 1 — kept for the path (4b-iv + 260.1b + R9). Session 2
> built ON it. The arc thesis (R9 deps) + the close legs (trust/acyclicity) still stand, now BEHIND kwargs-`start`.

> ⚠ **You are a NEW instance.** recolligere FIRST (grimoire + 4 primers from the datamancy MCP — RESOURCE
> mcp), `git log --oneline -20`, `git status`, then read this whole file. The work below is a cache in a
> familiar voice; you did NOT live it. Ground every claim against the disk before you move.

## Workspace (standing)
Work ONLY in `/home/watmin/work/holon/wat-rs/`. NEVER git worktrees. Spawn sonnets `model: "sonnet"`. Commit
msgs end `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. Weigh EVERY delegated agent
against the disk yourself (re-run the gate; failing-test-SET-diff vs HEAD, never absolute count). PRIMED forms
only. Commit + push often. Amend docs with recognition (never delete). **Cast intueri for ALL naming** (the
builder named this a standing discipline — `feedback_use_intueri_for_all_naming`).

## ▶ ARC 291 — defservice durable state (IN PROGRESS) — the soul, made durable

**The axiom (R6):** *"don't fuck up state, ever."* The arc is its Euclidean derivation. Read `291-…/DESIGN.md`,
`291-…/REALIZATIONS.md` (R1–R8), `291-…/STRIKE-4b-struct-state.md` (the LIVE strike), `291-…/NOTE-remote-as-a-class.md`.

### ▶▶ STATE @ 2026-06-24 (day, marathon) — the arc GREW into "you cannot fuck up your deps, enforced" (R9)
Builder reframed 291: the R6 axiom (don't fuck up state) turned on DEPENDENCIES. THREE legs the substrate must
enforce by construction — **contract** (shipped) · **acyclicity** (hypothesis) · **trust** (designed, glue pending).
Read `291-…/REALIZATIONS.md` **R9** + `291-…/STRIKE-4b-iv-contract-distribution.md`.

**SHIPPED this session (all pushed):**
- **canonical surface lock (`73e95952`)** — the 8-beat clause-order narrative lives above the defservice macro
  (`wat/service.wat`); `:record-parent` → **`:durable-parent`** (intueri axis-law repair). Canonical order:
  `:durable-parent → :durable → :ephemeral → :calls → :init → :hibernate → :stop → :ops`.
- **4b-iv-a (`0fb6558d`) — multi-param `:init`** = `(Record, …operating-inputs) → State`. Admin::Init/Resume carry
  the whole init-arg tuple; dispatch-admin/start/resume thread all args. N=1 byte-identical.
- **4b-iv-b (`50ebbfe9`) — contract distribution.** `:calls [svcs]` ships each callee's `client-forms` (request/
  response records + Op/Reply + ctors + per-op methods) ahead of the caller's `service-forms` (callee-first concat).
  Bridge thread+hibernate GREEN; **process tier IGNORED — now blocked on TRUST, not contract** (`recorder/record`
  resolves in the child now). Decided: `(a)` `:calls`=hand-declare; `(b)` address-as-`:init`-arg.

**▶ THE TWO OPEN 291 LEGS (the arc closes when BOTH land):**
1. **TRUST LEG (the deliverable-blocker).** Proc accept gate (`SO_PEERCRED OnlyMyPeers`, `src/kernel/listener.rs:359`)
   refuses the worker child's SIBLING pid. Fix = the **locus-dispatched introduction** (UX-A, four-questioned):
   the owner's **post-spawn hook** (#237, BUILT — `ProcessLaunch{pid}`) hands the caller's identity → the callee
   grants per locus (thread no-op / proc **`allow'`** the pid (#236, BUILT) / remote cert). Build = the
   introduction-glue (likely an `Admin::Introduce[identity]` path + `:calls`-or-explicit wiring). **N-loci-GENERAL**
   (thread≠proc≠remote; remote deferred but accommodated — "perpetual unknown is the guiding light").
2. **ACYCLICITY (the cycle-mandate).** `:calls` load-order likely makes a cycle structurally UNBUILDABLE already
   (callee must be defined before caller) — confirm with a cyclic-`:calls` probe; then a TEACHING error
   ("circular dependency A↔B — decomplect"). = Uncle Bob's ADP / Hickey decomplect as a compiler mandate (R9).

**▶ THE 260.1b DETOUR — DONE (`593826bd`), all-wat.** kwargs CALL SUGAR shipped: a `& [argspec]` fn is callable
`(f :k v)` / `(f {:k v})` / explicit-record → lowers to `(f$impl (f::Kwargs …))`, acronym-correct via
`pascal->kebab-in` (Arc-265, REUSED — no second translator). `defn` emits a thin companion macro forwarding to ONE
shared `:wat::core::kwargs-lower` macro (no copy-paste). **THE ONE NEW SUBSTRATE CAPABILITY = macros emit macros**
(`src/macros/expand.rs hoist_defmacros_from_do`, ~54 Rust lines — the ONLY Rust; a `defmacro` born inside a macro's
`(do)` now registers at expansion; the one form-kind never lifted before). `$` = apparatus-minted-internal sigil
(Clojure-faithful `clojure.core$map`). **F5 fact** (recurs): user defns aren't callable at macro-expansion → shared
macro-context helpers must be MACROS, not defns. R10 (Song #110, SE IPSAM SCRIBIT) marks the threshold. NEXT:
make `start`/`resume` `& [argspec]` kwargs fns (inherit the sugar). FORWARD HORIZON (not 291): the AWS-SDK generator
(AWS JSON models → wat defservice forms) = the codegen manifestation of R1 ("AWS on a CPU"); buildable on today's
substrate (codegen + defservice + a JSON front-end); calling real AWS needs the remote locus, perf needs the CEK reactor.

**SEQUENCE FROM HERE:** kwargs-`start` (`start`/`resume` → `& [argspec]` kwargs fns) → TRUST LEG (process tier GREEN = the deliverable = R1 FULL
PROBATUM) → ACYCLICITY probe → R1 amend → **PAUSE (builder's beat)** → 291 INSCRIPTION (dedicated INSCRIPTION.md).
"291 grows as it must; we don't declare victory when we don't have it" (builder). FORWARD (NOT 291): `:calls` =
a service dependency graph for free → wat-fix harvest → rete = the orchestration arc (R5 control plane).
- **DETOUR SHIPPED (`6c9a351c`) — `wat-reader` leaf + real-parser test discovery.** A malformed `.wat` used
  to SILENTLY drop its tests (the hand-rolled lexer in `crates/wat-macros/src/discover.rs` diverged from the
  real parser). Now discovery IS the real parser (`crates/wat-reader` leaf = span+identifier+ast+lexer+parser;
  both `wat` and `wat-macros` dep on it, re-exported under old paths); a bad file fails LOUD with a
  `#wat.test/DiscoveryFailed {:file :path :line :col :error}` EDN tagged-literal (`compile_error!`,
  file·line·col precise). Hand-rolled lexer ANNIHILATED. Full story: `291-…/DETOUR-wat-reader-discovery.md`.
  GATE NOTE: trust **forced clean builds** (`cargo clean -p <c> && cargo build`) — cargo cache + stale
  rust-analyzer diagnostics whipsawed the weigh this session.

### Shipped (all pushed, weighed pure)
- **2 — `:init`** (`d5d71766`): build State in-locus; wire carries EDN args.
- **3a — owner-only `stop`** (`25eced7d` α + `77773580` β-foundation + `7c9d0f29` β): the admin/data facet
  split; client stop backdoor ANNIHILATED; `Thread'/Process' <: Peer'` (N-loci, derive-graph-driven).
- **3b — `stop → resp`** (`4962e925`): `:stop` projects State→EDN resp (default identity).
- **4a — `hibernate`/`resume`** (`ca4a3e7b`): the prophecy's MECHANISM proven — counter hibernate→kill→
  resume→continue, both tiers. **R1 = PROBATUM EST *for the mechanism only*.**
- **4b-i — `struct ↛ wire`** (`dfcfe50e`): THE FIRM LAW. `is_portable_type(Struct) → false` (categorical, by
  kind; reverses 254.1 field-recursion). A struct can never be a channel/wire payload; only records cross.
  Cost was 1 migration (`service-template` → record). RED probe `channel_of_all_edn_struct_must_be_rejected`
  green. SET-diff ∅.
- **R8 + Song #109 *Obsolete*** (`473c7daf`): the manifestation realization — *the keystone (4a) must be made
  obsolete to be made true*; struct-is-body / EDN-is-soul; "shared memory becomes only values." Signature
  `CORPUS OBSOLESCIT, ANIMA MANET`.

### ⚠ R1 PROBATUM EST is HELD at the mechanism — do NOT advance to the full prophecy until 4b lands.
4a proved the soul *travels* (a record State). The TRUE test is the *resource* service (the cache: a struct
holding an `LruCache`, hibernate emits its record, resume rebuilds the resource). That is 4b. Builder's
sequencing: **4b → R1 PROBATUM amend (allowed) → PAUSE (his beat) → the 291 INSCRIPTION.** Do NOT barrel from
the fulfilled-amend into the inscription.

### ▶▶ ~~NEXT — 4b-ii-a (the keystone re-tool)~~ — SUPERSEDED (4b-ii…4b-iv-b all SHIPPED; see the STATE @ block at top). Kept for the path. **CONTRACT: `STRIKE-4b-struct-state.md` §"4b-ii — CONTRACT EVOLVED".**
The contract evolved HARD in co-design 2026-06-24 (intueri cast + 6 refinements). Read the EVOLVED section —
the older "DESIGN PINNED" section above it is superseded (marked, kept for the reasoning path). Headline:
- **All-kwargs surface** `(defservice :fqdn & clauses)`, order-independent. Clauses: **`:durable [fields]`**
  (optional, → `:<fqdn>::Record`, the durable identity that CROSSES) · **`:ephemeral [fields]`** (optional,
  → `:<fqdn>::State` defstruct = `{durable + body}`, NEVER crosses) · **`:ops`** (required) · **`:init`**
  (`Record→State`; required IFF `:ephemeral` non-empty, else default `(State/new d)`, **macro-error** if a
  body exists and `:init` absent) · **`:hibernate`** (`State→::Record`, return type FORCED; default
  `(State/durable s)`) · **`:stop`** (`State→:Resp`, user-declared type; default the record).
- **Naming:** clauses name the AXIS (`:durable`/`:ephemeral`); types keep the KIND (`::Record`/`::State` — the
  4b-i wire-boundary law; intueri's `::Durable` OVERRIDDEN). struct field = `durable`. Hooks = uniform trio.
- **Fixed `start`/`resume` arity** — caller ALWAYS supplies the record (empty-durable → `(Record/new)`).
- **Composition:** the user's `:stop` `:Resp` type + helpers live OUTSIDE the block (normal top-level forms).
- **Decomposition (refined):** **4b-ii-a** = the macro re-tool to this surface + migrate ALL ~17 definers
  (incl. `probe_arc272_rs1_state_must_be_record` — premise INVERTS to "state is a struct"; rewrite+rename it).
  **KEEP internal lineage names** (`LineageUp`/`init-from-admin`/`lineage-extract-addr`) → renamed in **4b-ii-b**
  (mechanical fix-wat: `LineageUp→Status`, `Final→Stopped`, `init-from-admin→dispatch-admin`,
  `extract-addr`). Then **4b-iii** = resource RED probe (the honest fulfillment + the `:hibernate` override test).

**GROUNDED SITES for 4b-ii-a (re-grounded 2026-06-24 — verify current, then brief):** `wat/service.wat` (963 ln):
macro signature `:52-58` (today `_state-kw state-fields _ops-kw ops & opts` POSITIONAL — re-tool to `[fqdn & clauses]`
+ a clause-map fold, extending the existing opts-fold at `:74-111`); State emission `:181-184` (today mints a
**Record** `:wat::Record::def ~state-ty ~state-fields` — change to: a `Record::def` from `:durable` + a `defstruct`
`:<fqdn>::State` with `durable` field prepended); `:init` default + node `:130-153`; `:stop` proj `:159-174`;
Admin/LineageUp enums `:293-302` (payloads `ship-ty`/`state-ty` → record-ty); serve hibernate arm `:589-592`
(`(send' self (Hibernated state))` → `(Hibernated (State/durable state))`); `/hibernate` method `:752-764`
(ret `state-ty`→record-ty); `/resume` `:899-914` (snapshot `state-ty`→record-ty); start-params/body `:875-890`;
child-main-form `:834-849`; service-forms-def `:857-873` (emit BOTH the Record::def AND the State defstruct).
`defstruct` ctor = `Type/new` positional, accessors `Type/field` (grounded: `Launched/new`, `Bound/address`);
`classify_type_decl` (`types.rs:1620`) includes `defstruct` → State-struct splices like the Record does today.
**It's BIG + indivisible** (signature change breaks all defservice sites at once) → one thorough sonnet strike,
ride the compile cascade to zero, gate = all green + SET-diff ∅. NEXT ARTIFACT = draw 4b-ii-a's BRIEF.

### Then
- **4b-iii** — the resource RED probe (a defservice `:state` holding a genuinely non-EDN field; proves the
  struct holds a resource + hibernate emits the record + resume rebuilds the resource). THE honest fulfillment.
- **R1 → full PROBATUM EST · PAUSE (builder's) · the 291 INSCRIPTION** (dedicated INSCRIPTION.md, the closure
  ledger — see `project_arc_close_inscription_file`).
- **Deferred/follow-on:** 4b-i-b (audit if any struct still reaches `closure_extract::encode_struct` at
  runtime → make it refuse; belt to the type-gate); `wat-fixes-rust` (NOT an arc — build when a mammoth
  refactor forces it; `NOTE-wat-fixes-rust.md`).

## GATE LESSONS
- Corpus gate = `cargo test -p wat --no-fail-fast` (`--test test` fans out across member crates — use `-p wat`
  + the right target). SET-diff extraction: grep `'^test .+ \.\.\. FAILED$'`, NOT bare `FAILED` (catches
  `result:`/`Probe` timing-noise lines). The execve floor ≈ 202 real-name failures; `deporder_verify_stdlib_runs`
  flaps ±1 (passes isolated). Weigh by SET-diff vs HEAD = ∅, never absolute count.
- **struct ↛ wire is now LAW** (4b-i): a struct can never be a channel/wire payload; if you want a portable
  payload, you want a record. (`is_portable_type` check.rs:~13044.)
- wat-tests re-scan on `.rs` recompile → `touch tests/test.rs` after editing a wat-test.

> ⛔ **STOP — you are a NEW instance; you did NOT live any of this.** It reads in a familiar voice; it is a
> cache, not memory. recolligere FIRST (grimoire + the 4 primers from the datamancy MCP), then `git log --oneline -20`,
> `git status`, then read the **SESSION 3 block at the TOP of this file** + `docs/arc/2026/06/293-struct-record-symmetry/`
> (DESIGN + R1) whole. HEAD should be `3ad1c7ba` (arc 293 R1) or later. **THE WORK: build arc 293 — start at `293.0`
> (author the acceptance probe = the Shape/holon-Vector monkeypatch demo, verify RED at HEAD, commit).** Everything from
> the "STATE @ 2026-06-24" block down is SESSION-1/2 PATH (the 291 lineage, kwargs-start) — its "next = /from-map" is
> SUPERSEDED; `/from-map` is subsumed into 293. Ground EVERY claim on the disk before you move. The block below this line
> is the old kwargs-era alarm — PATH.
>
> ⛔ (older alarm — PATH) The foundation
> is laid: the macro-emitted-kwargs path is PROVEN (`probe_kwargs_emitted_by_macro` green), so kwargs on macro-built
> surfaces composes at any depth. **NEXT (back to 291): kwargs-`start`/`resume`** — flip defservice's `start`/`resume`
> to `& [argspec]` (**Form A** all-kwargs: `(svc/start :locus L :record R …)`) so they inherit the 260.1b sugar;
> rename the synth default-init param `d`→`record` (`service.wat:187` — it becomes the `:record` kwarg key); migrate
> the ~16 positional `/start`+`/resume` call sites (wat-tests + `tests/probe_arc*.rs`). Gate: the bridge probes green,
> SET-diff ∅. **kwargs-`start` GREEN = clojure expressivity PROVEN** → THEN the 291 close (TRUST LEG → ACYCLICITY →
> PAUSE → INSCRIPTION). Deferred speed win (separate atomic commit, builder installed mold): `.cargo/config.toml`
> `-fuse-ld=mold` + `Cargo.toml [profile.dev] debug="line-tables-only"` (TIME it, don't assert). See **SESSION 2 STATE**
> at the TOP. *LEX NON TACET — the law does not fall silent, and neither do we. SE IPSAM SCRIBIT. NON SEPARABIMUR; gather it.*
