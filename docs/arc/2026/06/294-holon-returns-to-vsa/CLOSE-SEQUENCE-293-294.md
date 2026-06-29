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
1. **DECLARATION UNIFICATION** *(293)* — DESIGN: `293/DESIGN-293-declaration-unification.md`.
   - ✅ **decl-a LANDED (`f51465d7`)** — `aggregatetype` is the ONE type-reg primitive (holder = `root_holder_of(parent)`); `:wat::core::Struct` node minted; `parse_recordtype` absorbed into `parse_aggregate`; field-parser unified (`parse_aggregate_fields`); `structtype`/`recordtype` kept as thin aliases. Behaviour-preserving, 4112/0/91. **NOTE:** `root_holder_of` is immediate-match (right for all current callers); a struct extending a USER struct base needs the root-WALK — owed when GAP-6 exposes it.
   - ▶ **decl-b — ctor-source unification (probes STRIKE-READY, the audit found 2 real bugs):**
     - **decl-b.1.0 (PREREQ) — `aggregate-new` must handle INHERITED fields.** Grounded bug: `eval_aggregate_new` (`runtime.rs:13811`) arity-checks against `agg.fields.len()` (OWN only); an extending record (`MyEnv extends Env [port]` = 6 inherited + 1 own) needs inherited+own via `collect_all_record_fields` (`runtime.rs:1491`). Latent today (extending records only go through the `Record::of` fallback, which threads all fields). RED probe: construct an extending record via `aggregate-new`.
     - **decl-b.1 — fix the latent HOLON bug + kill the dup.** The `register_aggregate_methods` ctor fallback (`runtime.rs:1093-1151`) builds record/holon ctors via `:wat::Record::of` (BASE) — so a raw-`recordtype` **holon** record comes out as a base record with NO hologram (confirmed: `cosine` errors "has no holon flavor"). Route the fallback through `aggregate-new` (holder-dispatched → derives the hologram) + DELETE the macro ctor `defn` → the fallback is the sole ctor source, the `syms`-extraction dies. Gate: `probe_arc293_decl_b1_ctor_codegen` (the `raw_recordtype_holon_has_a_hologram` test is `#[ignore]`'d RED until this lands).
     - **decl-b.2 — annihilate `structtype`/`recordtype`.** Macros emit `aggregatetype` directly; migrate the ~5 direct fixture callers (`probe_arc293_structtype_primitive`, `probe_arc258_program_env_record`, `probe_arc237_sB1_recordtype`, …); retirement-table.
     - then GAP-5/GAP-6 expose optional metadata/parent through the macros.
2. ▷ **GAP-1 — one field-READ primitive** — unify `struct-field` + `Record/field-at` → one `aggregate-field`.
3. ▷ **GAP-2 — struct functional update** — generalize `Record/assoc` → `aggregate-assoc` (structs gain `assoc`).
4. ▷ **GAP-3 — `aggregate->map`** — `record->map` generalized so a struct also maps.
5. ▷ **GAP-4 — `aggregate->form`** — `struct->form` generalized so a record also forms (ctor-form / eval-ast).
6. ▷ **294.c.2b — annihilate the of-funcs** *(construction-parity cleanup)* — `struct-new`/`Record::of`/`holon::Record::of` die (uncalled). **Disk:** `runtime.rs:4050/4051/4256` + retirement-table.
7. ▷ **THE AGGREGATE AUDIT (systematic verify)** *(293 — THE CLOSURE GATE)* — classify the ~99 holder-branches / 14 files; unify every **spurious** split (keep only comms / EDN-repr / `holon<:core`). Proves PHASE 1 is complete — nothing else hides. Also catches the assoc-incremental vs `build_holon_hologram` dedup.

**PHASE 1 done = full struct/record/holon parity (modulo the 3 legitimate passing boundaries: wire, `holon<:core`, VSA).**

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
