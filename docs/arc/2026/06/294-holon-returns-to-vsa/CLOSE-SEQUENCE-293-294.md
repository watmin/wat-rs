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
2. ▶ **INHERITANCE ANNIHILATION (NEXT — replaces decl-b.1.0)** *(293)* — `program::Env` → a **surface** ("must be a
   record with these minimal properties"); `aggregatetype`/`recordtype` parent restricted to a **holder-root** (reject
   user parents); **delete** `collect_all_record_fields`, inherited-field storage + abs-idx, the record subtype edges.
   Flat records only → `aggregate-new`'s own-only arity becomes correct, **decl-b.1.0 + its probe DELETED**. The
   `rete.wat`/`program.wat` `[x <- :wat::Record]` params → `:wat::core::Value` (guts) or a surface — the **surfaces-only
   migration** (`[fact <- :wat::Record]` → `[fact <- :wat::core::Value]`, confirmed opaque/reflective). **STOP:** verify
   `program::Env`'s spawn-injection (#211/#238) works via the surface, not a nominal base.
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
10. ▷ **THE AGGREGATE AUDIT (systematic verify)** *(293 CLOSURE GATE)* — classify the ~99 holder-branches; unify every
    **spurious** split (keep only comms / EDN-repr / assignability). Proves PHASE 1 complete — nothing else hides.

**PHASE 1 done = the AGGREGATE-MODEL holds: holder enum is the sole specialness, every op uniform, no inheritance,
surfaces-only params, one holder vocabulary. (The 3 legitimate holder differences — comms/EDN-repr/assignability — stay.)**

### PHASE 2 — the value-layer gut + close (UNBLOCKS only after PHASE 1 = 0)
8. ▷ **294.c.3 — base records lift / holon-from-EDN** *(294, step 4)* — `to_holon_inner` lifts base records. **Disk:** 5 "has no holon flavor" rejects.
9. ▷ **294.d — wire = plain EDN** *(294, step 6)* — kill `HolonRepresentable` (80) + `#wat-edn.holon/*` tags (47) + the round-trip.
10. ▷ **294.e — `HolonAST → Hologram` rename + mint `src/holon/`** *(294, step 7 — keystone)*. **Disk:** 1173 mentions; `src/holon/` absent.
11. ▷ **294.f — reflection-IR → WatAST** *(294, step 8)*. ~175.
12. ▷ **294.g — homes** *(293.1 + 294, step 9)* — `src/aggregate/` + `src/holon/`; both absent.
13. ⛔ **293.5 — CLOSE** *(293)* — `/from-map` (GAP — absent for ALL holders; the 291 driver) + SET-diff ∅ + ward homes + amend 291 + INSCRIPTION(s). **GATED on PHASE 1 = 0 AND 8–12 done.**

### Then
10. ▷ **arc 118** — `Seqable` → the HOF family (needs only the DONE 293.4; 294 was never a blocker, only a co-close).

## Maintenance rule
When a step lands: flip its ✅, add the commit hash, and (if it closed an `AGGREGATE-AUDIT.md` row) tick that row too.
Keep the ORDER. If a new sub-strike appears, it goes here FIRST (in sequence) before it is worked — an unlisted
sub-strike is the SESSION-10 "invented R2.4" tell. The breadcrumb (`255/CURRENT-STATE.md`) points here; this file,
not the breadcrumb, is the durable sequence.

## Pairs (subordinate detail — this file is the sequence; these are the depth)
`294/REMAINING-PATH.md` (the 9-step value-layer path — context) · `293/AGGREGATE-AUDIT.md` (the closure-gate detail
+ the ~99-branch checklist) · `293/DESIGN.md` (HOLDER × SURFACE; the closure gate) · `294/DESIGN.md` (the gut; the
six flaws) · `294/DESIGN-294.c.2-aggregate-new.md` (the ctor strike).
