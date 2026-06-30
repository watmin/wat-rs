# CLOSE SEQUENCE — 293 + 294 close TOGETHER (the single maintained tracker)

> **THIS FILE IS THE CANONICAL ORDER + STATUS for closing arcs 293 and 294. Maintained: update it as each
> strike lands. Every other doc is detail or context — this is the sequence. DO NOT work out of order; DO NOT
> relabel a step across the two arcs.** (Builder, 2026-06-28: *"293 and 294 getting entangled is a problem we've
> not faced and i don't want to experience this again — no slip out of sequence again."*)

## Why they are entangled (read this so you don't mislabel — the SESSION-10 failure)
**294 (the value-layer gut) was discovered INSIDE 293 (the aggregate type system).** Chasing 293's construction
parity surfaced that the holon record was built backwards; pulling that thread became 294. So:
- 293's **construction tail folds into 294** (`aggregate-new`, `/from-map`) — `293/DESIGN.md:11`, `294/DESIGN.md:11`.
- The **homes are shared**: `src/aggregate/` (construction) is 293.1's owed home; `src/holon/` (VSA) is 294's.
- **293.5 close is GATED** on the 294 value-layer being done AND the aggregate audit reaching zero spurious splits.
They **cannot close independently.** The SESSION-10 drift — doing 294 work and labeling it "293.R2.x," inventing a
non-existent "R2.4" — is the exact failure this tracker prevents. **A `Value`-repr / construction / wire / hologram
change is 294. A surface / holder-policy / declaration-shape change is 293. When unsure, this file decides.**

## Status legend  ✅ done · ▶ next · ▷ queued · ⛔ gated

## THE SEQUENCE (ordered — do not slip)

### Done
- ✅ **293 type-system** — surfaces, methods-as-accessors, `defprotocol` annihilated (`cf89fb52`). *(293)*
- ✅ **294.c.1 — identity = EDN data** — Rust `Eq`/`Hash` key `(holder,class,fields)`, hologram out of identity (`ed7ecd50`). *(294, flaw #7)*
- ✅ **294.c.2a — `aggregate-new`** — one holder-dispatched ctor; hologram derived in Rust (`build_holon_hologram`); 3 macros + struct codegen emit it; `defholon` hologram-quasiquote deleted (`f301a6fc`). *(294, steps 2+3)*
- ✅ **kanerva_capacity dedup** — `floor(sqrt(d))` budget driven to ONE copy (`eaaa6930`). *(294, one-canonical-path)*

### ⛔⛔ PHASE 1 — AGGREGATE PARITY (THE BLOCKING PRIORITY, builder 2026-06-28)
> *"this is our priority — we block all 293 and 294 work until this is resolved. build solutions such that they
> satisfy the closing requirements for 293 and 294 if applicable."* **No PHASE 2 item starts until PHASE 1 = ZERO
> gaps.** Build each fix CLOSE-GRADE — canonical form, right home, one-canonical-path, no rework. The full ledger is
> `293/AGGREGATE-AUDIT.md` § PARITY LEDGER (6 grounded GAPs + the ~99-branch systematic verify).
> **THE GOVERNING MODEL is `293/AGGREGATE-MODEL.md`** (the canonical contract, 2026-06-29): the holder enum is the
> ONLY specialness; every operation is holder-blind + uniform; **NO inheritance** (flat + surface-splice); requirements
> are **surfaces** (bare holder is an illegal Any; `Value` for guts); the edn wall lives at the locus boundary. Every
> strike below is held to it. **The reshape this conversation forced: inheritance is ANNIHILATED, not supported — so
> the earlier decl-b.1.0 is DELETED.**

1. ✅ **decl-a LANDED (`f51465d7`)** — `aggregatetype` is the ONE type-reg primitive (holder from the parent's
   holder-root); `:wat::core::Struct` node minted; `parse_recordtype` absorbed; field-parser unified. 4112/0/91.
2. ✅ **INHERITANCE ANNIHILATION LANDED (`c7572929`)** *(293)* — `AggregateDef.parent` DELETED (it was the
   stringly-typed shadow of `holder` — the inheritance vestige; builder: *"parent as an attribute is wrong"*). Subtype
   edges derive from `holder` via new **`Holder::root_keyword()`** (the first brick of item 5). A non-holder-root parent
   is REJECTED at parse (closing `root_holder_of`'s `_ => Record` leak). `collect_all_record_fields` + inherited-field
   machinery + `ROOT_PARENTS` + abs-idx deleted; 2 inheritance fixtures (arc237 sA1 probe05, arc258 c02) deleted; the
   rest of arc-237's subtype suite (holder-membership: `:Circle <: :wat::Record`, `holon <: core`) UNTOUCHED. `4113/0/92`,
   weighed forced-clean. RED probe `probe_arc293_reject_user_parent` GREEN. `program::Env` is a **flat record** — NOT a
   surface (nothing requires "an Env"; the spawn-injection constructs it concretely — STOP cleared). decl-b.1.0 stays DELETED.
   ▷ The bare-holder migration SPLITS OUT (item 2a): `user.program` is NOT `Value` (it ships → must be ≥ Record) — a
   0-member `:holder :wat::core::Record` surface `:wat::spawn::user-env`; rete's `[fact <- :wat::Record]` → `Value` (in-locus).
2a. ▶ **SURFACE `:holder` VOCAB + `user-env` surface (NEXT)** *(293, item-5 surface piece)* — `surface.rs:332` hand-matches
   magic `:struct`/`:record`/`:holon-record`; route `:holder` through new **`Holder::from_root_keyword()`** so it takes the
   **holder-root symbol** (`:wat::core::Record`); migrate existing `:holder :<magic>` surfaces; mint `:wat::spawn::user-env`
   (`:holder :wat::core::Record` `[]`) + retype `program::Env.user.program` to it. RED probe: user.program rejects a struct,
   accepts a record. (The other 4 magic sites + `:wat::Record`→`:wat::core::Record` rename = the rest of item 5.)
3. ▷ **decl-b.1 — the latent HOLON bug + kill the dup** *(still real, not inheritance)* — the ctor fallback
   (`runtime.rs:1093-1151`) builds record/holon ctors via `:wat::Record::of` (BASE) → a raw-`recordtype` **holon**
   record has NO hologram (`cosine` errors "has no holon flavor"). Route the fallback → `aggregate-new` + DELETE the
   macro ctor `defn` (the `syms`-dup dies). Gate: `probe_arc293_decl_b1_ctor_codegen` (`#[ignore]`'d RED test).
4. ▷ **decl-b.2 — annihilate `structtype`/`recordtype`** — macros emit `aggregatetype`; migrate the ~5 direct fixture
   callers; retirement-table.
5. ▷ **HOLDER VOCAB UNIFICATION** *(the enum owns its name)* — `Holder::root_keyword()` + `from_root_keyword()`; the
   **5 hand-matches die** (`surface.rs:323`, `types.rs:2126`, `value.rs:1120`, `runtime.rs:6705`, `observe.rs:326`);
   surface `:holder` takes a **holder-root keyword** (decision A); the stale `:wat::Record` → **`:wat::core::Record`**
   rename falls out of the same change.
6. ▷ **GAP-1 — one `aggregate-field`** (kill `struct-field` + `Record/field-at`).
7. ▷ **GAP-2 — one `aggregate assoc`** (struct gains functional update).
8. ▷ **GAP-3/4 — uniform `aggregate->map` / `aggregate->form`** (struct→map, record→form).
9. ▷ **294.c.2b — annihilate the of-funcs** — `struct-new`/`Record::of`/`holon::Record::of` die.
9a. ▷ **CONSTRUCTION ERGONOMICS — kwargs is the DEFAULT surface; positional is a reserved escape hatch; `/from-map`
    DIES (QUEUED 2026-06-29, builder).** Invert today's "bare = positional, `/from-map` = kwargs" → **bare = KWARGS**:
    `(ns::Agg :field-1 22 :field-2 true :field-N #{1 2 3})` is the common, self-documenting, order-free form (a macro
    that reorders to the positional substrate call — kwargs is always a macro, `feedback_kwargs_is_always_a_macro`).
    The raw **positional** ctor moves to the **type-name PRIME**: `(ns::Agg' 22 true #{1 2 3})` — you can reach for it
    but never need to know positions. **`/from-map` is ANNIHILATED** (kwargs IS the map/kwargs path; one less form).
    **NAME DECISION: `:ns::Agg'`, NOT `/make`** (builder, correcting an apparatus error): `/make` would STEAL a name
    from the user's method namespace — `:ns::Agg/make` lives in the exact `:ns::Agg/*` space the user fills with their
    own accessors/methods; reserving it forbids a user method named `make`. **Don't take names from users.** The
    type-name prime `:ns::Agg'` lives ABOVE the `/` namespace, so the user's entire `:ns::Agg/*` stays free — zero
    collision. (The `project_prime_suffix_replaces_then_drops` migration convention is NARROW — substrate *verbs*
    pending their un-prime, e.g. `recv'`/`send'`; it does NOT govern a permanent type-name-prime ctor. The apparatus
    over-broadened it; corrected.) Pairs the of-funcs→aggregate-new work (9) + supersedes the 291 `/from-map` driver (293.5).
10. ▷ **THE AGGREGATE AUDIT (systematic verify)** *(293 CLOSURE GATE)* — classify the ~99 holder-branches; unify every
    **spurious** split (keep only comms / EDN-repr / assignability). Proves PHASE 1 complete — nothing else hides.

### ⊹ THE SURFACE KIT (2026-06-29 co-design — the landmark UX; `293/AGGREGATE-MODEL.md` § THE COMPLETE KIT)
> Additive to PHASE 1's parity work — the surface's projection + extension story. Settled by a long four-questions
> co-design; builder at close: *"we burned inheritance to the ground and lost nothing."* Four tools, no loss
> (inheritance · `defprotocol` · the extend-type confusion all collapse in). Build order, each RED-probe gated:

> ⚠⚠ **DESIGN PIVOT (2026-06-29, later co-design — supersedes the "all three tiers" decision) + A GROUNDED WIRE-WALL
> BREACH.** Pulling the projection-depth thread surfaced that **a `Struct` field nested in a `Record` CROSSES a process
> peer** (probe: `#w/S {:a 99}` reconstructed on the far side) — §7 / R3 *SUB SUPERFICIE QUOD ES* violated; the
> `is_portable_type` wall is shallow (top-holder only) and the deferral comment *"enum payload portability is not yet
> enforced"* (exigere violation) was sitting on it. The co-design that followed locked the kit's final shape AND the
> fix:
> - **NO `to-struct`** — projection is ONE-WAY UP. `to-record` only, **surface-targeted** (`to-record x :S` →
>   `:S$core-record` / `:S$holon-record`, the receiver's named backing record); the namespace picks hologram-or-not.
>   Surfaceless `to-record x` is rejected — no clean return type (no anonymous structural records in wat).
> - **A surface emits a PAIR** (`$core-record` + `$holon-record`), not a triple. `$struct` is dead.
> - **THE CONTAINMENT RULE** (`293/AGGREGATE-MODEL.md` § principle 8) — a portable aggregate (record/holon) may hold
>   ONLY portable fields; a `Struct` field is ILLEGAL at declaration (it could never be reconstructed across the wire).
>   This turns the wire wall into a TYPE guarantee (a record cannot *hold* a struct → can never *carry* one across).
> - **▶▶ 293.W — THE DEEP WIRE WALL (the priority, builder: this IS core 293 — the holder's categorical comms
>   boundary).** **SCOPE: PURELY COMPILE-TIME (2026-06-30) — *the compiler won't let you write code that reads or
>   writes a struct over non-thread memory.* Bad bytes (untrusted input) = the USER's validation problem, OUT OF
>   SCOPE.** Three compile-time rules, ZERO runtime code (`293/DESIGN-293.W` §contract):
>   - ✅ **W.1 — aggregate containment (`ff29f135`)** — a record can't HOLD a struct field (declaration gate).
>   - **2b — PURITY IS THE AXIS — ✅ LANDED (`76d1d890`, weighed forced-clean 4132/0/94).** (Ratified 2026-06-30; builder: *"a wonderful finding … our next
>     priority"*; canonical = `293/AGGREGATE-MODEL.md § THE PURITY AXIS`, strike-detail `293/DESIGN-293.W § 293.W.2b`).
>     The holder was always a PURITY classification wearing a movement name. Enums **declare** `:wat::enum::Pure` |
>     `:wat::enum::Impure` (`Purity{Pure,Impure}` on `EnumDef`); the holder is the purity axis refined (`Struct` permits
>     impurity; `Record`/`Holon` guarantee purity). **Rename the cause everywhere in ONE change** (long-term stability):
>     `Holder::is_portable`→`is_pure`, `is_portable_type`→`is_pure_type`, the wire wall → the **purity wall**, a
>     containment pass enforces "a pure aggregate/enum holds only pure fields". `:wat::kernel::Failure` is pure data
>     mis-declared `defstruct` → `defrecord` (the 2616-cascade root). One purity family with function-purity
>     (`:wat::runtime::Purity` = `:Pure`/`:Effectful`). **Supersedes** the `Mobility`/`Portable`/`Anchored` movement-frame
>     (the path) AND the earlier "enum arm recurses (derived)" framing. Surfaced VALID findings (`:svc::Request`
>     reply-`Sender`s = impure → its thread-tier `make-channel` exemption rides to 2d; `Failure` mis-declared). The
>     predicate the rules consume. **Falls squarely in 293's thesis (struct/record identical) — the NEXT PRIORITY.**
>   - **2d** (NOT optional) — **PEER-TYPE CONTAINMENT** (W.1 lifted to the peer): a wire peer (`Process'`/`ConnPeer'`)
>     may NOT be typed with a non-portable `I`/`O`; split overloaded `Peer'` → `ConnPeer'`/`ThreadSelfPeer'` to express
>     it. The ordinary type checker then forbids struct-on-wire (`send'(peer, struct)` = unify error; `recv'` can't
>     produce a struct; the "read a struct off a wire peer" call path has no form). **DELETE the interim runtime
>     guards** — 293.W.2a (`fe012223`, the inbound/outbound runtime checks) + 293.W.2c (`7a040b0e`, the send'-site gate)
>     — they held the line + caught/proved the breach while no compile wall existed; once containment is total a struct
>     can never reach the wire from any wat program. **The wall ends as zero runtime code.** Sits UNDER K3-revise + K5.
> - **▶ K3-REVISE** (after 293.W): annihilate `to-struct` + the `$struct` emission; `derive_surface_backing_records`
>   emits the PAIR; retire the `probe_arc293_k3_to_record` struct-tier assertions; retirement-table `to-struct`.
> - **▶ K5** (`extend-surface`) then rides the pair (STRIKE-READY at `7d2892b8`; the probe's surface is Struct-floored
>   — revisit if K3-revise changes tier availability). Then the showcase graduates `.wat.disabled` → `.wat`.
- **K0 — surface grammar: `:holder` MANDATORY + `self` EXPLICIT + the cycle-guard.**
  - ✅ **K0c — the self-reference cycle-guard LANDED (`311b20bf`)** — the enabler. Explicit `self <- :TheSurface`
    made `struct_satisfies_surface` (`surface.rs:83`) compare self's type (= the surface) via `is_assignable`,
    re-entering satisfaction → wrong-reject or stack-overflow (the showcase's exit-139). Fix: **SKIP position 0
    (self)** in the method arg-type compare — self is the receiver, tautological. RED probe `probe_arc293_self_explicit`
    RED→GREEN; weighed 4120/0/92 (multi-arg surface methods unbroken).
  - ✅ **K0a+K0b LANDED (`98639f0d`)** — `parse_defsurface` arity→5-only (no-holder form retires, MalformedDecl);
    `parse_method_member_sig` rejects bare untyped `[self]` (self must be `[self <- :TheSurface]`). 20 fixtures
    migrated (sonnet LEAF; orchestrator-weighed: parser diffs read end-to-end, **forced-clean re-run 4120/0/92**).
    Holder-testing probes kept intentional holders; structural-satisfaction fixtures defaulted to `:wat::core::Struct`
    (widest). `_bad` negatives still reject.
  - **⇒ K0 COMPLETE (K0a + K0b + K0c).** The surface grammar IS the model: `:holder` mandatory, `self` a normal binder.
- **K1 — THE HOLDER LADDER (contravariant satisfaction)** — the showcase pinned it to one line.
  - ✅ **K1a — the AGGREGATE ladder LANDED (`a952c908`)** — `Holder::rank()` (the trit, Struct −1 < Record 0 <
    HolonRecord +1) + `check.rs:14698` `agg_holder == req` → `agg_holder.rank() >= req.rank()`. Struct-floor accepts
    struct+record+holon, record-floor accepts record+holon, holon-floor accepts holon only. The aggregate ladder was
    never built (the 293.4 demo used holder-LESS surfaces); RED probe `probe_arc293_holder_ladder` RED→GREEN; weighed
    forced-clean (4118 run, the lone in-band fail `sigterm_…polling` passes isolated 2/2 = the arc-170 flake).
  - ✅ **K1b — the FOREIGN floor LANDED (`88818acd`)** — the plan mis-pinned this at `check.rs:14726` (dead for
    parametric foreigns). Foreign satisfaction flows through the **extend-type subtype edge** (`assignable` arms
    `14633`/`14641`), which returned `true` with NO holder check (option **(b)**, exempt — a String wrongly satisfied a
    `:holder :HolonRecord` surface). Fix: `derived_holder` (aggregate→declared, foreign→`is_holon_or_vector`/`is_portable_type`)
    + `holder_floor_ok` gating BOTH arms **only** for holder-bound surfaces (protocols untouched), same `rank() >=`.
    Upgrade (b)→**(b′)**: foreign satisfaction is holder-CHECKED, never exempt. RED probe
    `probe_arc293_holder_ladder_foreign` RED→GREEN; weighed forced-clean 4119/0/92.
  - **⇒ K1 COMPLETE — the holder ladder is aggregate + foreign, both honest.**
- ✅ **K2 — `$record` backing-type emission LANDED (`d3fe912b`)** — `derive_surface_backing_record` (Field members
  only, Method→None) + a 5-line inject in `register_types_impl` registers the twin `:S$record` `AggregateDef`
  (holder = surface's; fields = `:features` attributes) via the SAME `register` closure; it flows through the EXISTING
  `register_aggregate_methods` for ctor+accessors (no new codegen). `$` confirmed legal. Method-exclusion verified
  (`:t::Shape$record` = `color` only). RED probe `probe_arc293_k2_surface_record_emission` RED→GREEN; weighed
  forced-clean 4121 (the lone fail = the arc-170 `sigterm_…polling` flake, isolated 2/2). `to-record`'s return type is live.
- ✅ **K3 — `to-record` / `to-struct` (the THREE projection verbs) LANDED (`3c0c25ea`)** *(design EXPANDED 2026-06-29
  co-design — see `293/AGGREGATE-MODEL.md § to-record` superseding block).* Projection is a FREE EXPLICIT tier choice —
  the floor governs *satisfaction*, NOT *projection*. **Three verbs over ONE shared `project_surface_attrs(x, S)`**
  (reads S's Field attributes off x via the existing `:T/field` accessor route), differing only in the target holder:
  - `(:wat::core::to-struct  x :S)` → `:S$struct`        (in-locus; type forbids crossing comms)
  - `(:wat::core::to-record  x :S)` → `:S$core-record`   (portable EDN data)
  - `(:wat::holon::to-record x :S)` → `:S$holon-record`  (portable EDN data + a derived hologram, free from the holon ctor)
  **K3 SUBSUMED K2:** `derive_surface_backing_record` → `derive_surface_backing_records` emits the TRIPLE (`$struct` /
  `$core-record` / `$holon-record`, all three holders, same fields) instead of the single `:S$record`; K2 fixture
  amended to `$core-record`. runtime: `project_surface_attrs` + `parse_projection_args` + `eval_to_{struct,core_record,
  holon_record}`; check: `infer_projection_verb_check`. RED probe `probe_arc293_k3_to_record` RED→GREEN (3+4+3=10);
  weighed forced-clean **4122/0/92** (the arc-170 flake did not surface; the 2 E0283 were RA noise in untouched files).
  *FRANGE UT UNUM FIAT* on projection: one extraction tool, the holder is the only variance.
- ✅ **K4 — `extend-type` UN-DEMOTE — RESOLVED HONESTLY (no code; already general).** A disconfirming probe proved
  `extend-type` already binds method impls on your OWN aggregates, not just foreign types — 293.4c built the
  registration generically (`:T/method` for ANY T, never gated to foreign) + 293.4e-pre.i gave it the one canonical
  `ArgSpec` (same `:…/method` key as ambient `defn :T/method`; two front-doors, one mechanism). So K4 = (1) lock-in
  regression `tests/types/probe_arc293_k4_extend_type_own_aggregate.{rs,wat}` (GREEN at HEAD, → 25; guards K5's seam),
  (2) doc truing of the "foreign-only / demoted / monkeypatch" framing (`293/DESIGN.md § extend-type` superseding
  block; AGGREGATE-MODEL already stated it). No Rust change. (examinare: the thing you would build already existed.)
- ✅ **K5 — `extend-surface` LANDED (`06ede1dd`, weighed forced-clean 4138/0/91)** — a thin wat `defmacro` in
  `wat/core.wat` that emits one `extend-type` per PAIR backing tier (`$core-record` + `$holon-record`), forwarding the
  user's TYPELESS method body. **NO reflection seam needed** (the model's feared "one substrate dependency"): `extend-type`
  already fills the impl's types from the surface (the 293.4e-pre.iii capability, confirmed present on HEAD this session),
  so the macro is pure form-production, ZERO `src/` change. Per the K5 decision (option A) the default rides BOTH pair
  tiers → a `to-record`'d value at either inherits it for free. Probe `probe_arc293_k5_extend_surface` RED→GREEN (84 =
  42 core + 42 holon). **⇒ THE FOUR-TOOL SURFACE KIT IS COMPLETE: `defsurface` + `to-record` + `extend-type` + `extend-surface`.**
- ▶ **Landmark showcase (NEXT surface-kit step):** `wat-scripts/demos/aggregates/showcase.wat.disabled` — rebuild it
  against the settled kit (the pair, `is_pure` vocab, `to-record` + `extend-surface`), then rename `.wat.disabled`→`.wat`
  when green (done-detector `cargo wat <it>`; the wat-scripts load gate then owns it).

**PHASE 1 done = the AGGREGATE-MODEL holds: holder enum is the sole specialness, every op uniform, no inheritance,
surfaces-only params, one holder vocabulary. (The 3 legitimate holder differences — comms/EDN-repr/assignability — stay.)**

### PHASE 2 — the value-layer gut + close (UNBLOCKS only after PHASE 1 = 0)
8. ▷ **294.c.3 — base records lift / holon-from-EDN** *(294, step 4)* — `to_holon_inner` lifts base records. **Disk:** 5 "has no holon flavor" rejects.
9. ▷ **294.d — wire = plain EDN** *(294, step 6)* — kill `HolonRepresentable` (80) + `#wat-edn.holon/*` tags (47) + the round-trip.
10. ▷ **294.e — `HolonAST → Hologram` rename + mint `src/holon/`** *(294, step 7 — keystone)*. **Disk:** 1173 mentions; `src/holon/` absent.
11. ▷ **294.f — reflection-IR → WatAST** *(294, step 8)*. ~175.
12. ▷ **294.g — homes** *(293.1 + 294, step 9)* — `src/aggregate/` + `src/holon/`; both absent.
13. ⛔ **293.5 — CLOSE** *(293)* — `/from-map` (GAP — absent for ALL holders; the 291 driver) + SET-diff ∅ + ward homes + amend 291 + INSCRIPTION(s). **GATED on PHASE 1 = 0 AND 8–12 done AND the IGNORE LEDGER below is EMPTY.**

### Then
10. ▷ **arc 118** — `Seqable` → the HOF family (needs only the DONE 293.4; 294 was never a blocker, only a co-close).

## ⛔⛔ THE IGNORE LEDGER — must be EMPTY before 293/294 close (builder, 2026-06-30, NON-NEGOTIABLE)

> **THE RULE, verbatim:** *"all ignores we create must be removed before arc closure."* An `#[ignore]` is a **temporary
> debt with a named unlock**, NEVER a permanent state and NEVER a quiet way to keep the floor green. Every `#[ignore]`
> this 293/294 campaign introduces is logged HERE the moment it is created — with its exact unlock — and **293.5 cannot
> close until this ledger is EMPTY** (every entry un-ignored: the test passes, or it is deleted because its subject is
> gone, never merely left silent). A campaign `#[ignore]` not in this ledger is the bug this ledger exists to kill.

**Discipline when you create one:** add a row below AND mark the `#[ignore]` in code with `// ⛔ IGNORE-LEDGER(293):
<unlock> — see CLOSE-SEQUENCE` so the code site points back here. When the unlock lands, remove the attribute, remove
the row, and confirm the test is GREEN (not still skipped).

| # | test (`#[ignore]`'d) | why ignored | the UNLOCK that removes it | status |
|---|---|---|---|---|
| 1 | `deftest_svc_test_svc_assert_state` (`wat-tests/service-template.wat`) | `:svc::Request` is a declared `:wat::enum::Impure` enum (holds reply-`Sender`s), but its **thread-tier** `make-channel` still hits the 254.1 purity gate (not yet tier-aware) | **293.W.2d** — tier-aware `make-channel` (a thread channel accepts `:Impure`; a process/remote channel requires `:Pure`) | ✅ UN-IGNORED by arc 293.W.2d — `make-channel` purity gate deleted; thread channels accept any type |

**LEDGER IS NOW EMPTY.** The 293.W.2d unlock landed; the ignore was removed from `wat-tests/service-template.wat`.

*(Pre-existing ignores NOT created by this campaign — e.g. arc-170's `293.4e-pre.iii`, task #183's arc-170 gate — are
their own arcs' ledgers, not this one. This ledger tracks ONLY what 293/294 introduces.)*

## Maintenance rule
When a step lands: flip its ✅, add the commit hash, and (if it closed an `AGGREGATE-AUDIT.md` row) tick that row too.
Keep the ORDER. If a new sub-strike appears, it goes here FIRST (in sequence) before it is worked — an unlisted
sub-strike is the SESSION-10 "invented R2.4" tell. The breadcrumb (`255/CURRENT-STATE.md`) points here; this file,
not the breadcrumb, is the durable sequence.

## Pairs (subordinate detail — this file is the sequence; these are the depth)
`294/REMAINING-PATH.md` (the 9-step value-layer path — context) · `293/AGGREGATE-AUDIT.md` (the closure-gate detail
+ the ~99-branch checklist) · `293/DESIGN.md` (HOLDER × SURFACE; the closure gate) · `294/DESIGN.md` (the gut; the
six flaws) · `294/DESIGN-294.c.2-aggregate-new.md` (the ctor strike).
