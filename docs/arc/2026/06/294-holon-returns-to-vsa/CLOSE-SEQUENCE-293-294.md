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

### Remaining (in order)
1. ▶ **DECLARATION UNIFICATION** *(293)* — `structtype`+`recordtype` → one `aggregatetype` (holder + parent-root + metadata, keyed by holder); `parse_defstruct`+`parse_recordtype` → one `parse_aggregate`; the 3 def macros → thin holder-keyed delegations over one emission. **Disk:** split at `types.rs:1730/1732/1740`; the 2 record macros are byte-identical except the `recordtype` holder keyword (confirmed by c.2a). Open micro-decision: type-reg-first vs macros-first.
2. ▷ **294.c.2b — annihilate the of-funcs** *(294)* — `struct-new` / `Record::of` / `holon::Record::of` die (now uncalled by generated code). **Disk:** 3 still registered (`runtime.rs:4050/4051/4256`) + retirement-table the heads.
3. ▷ **294.c.3 — base records lift / holon-from-EDN** *(294, step 4)* — `to_holon_inner` lifts base records. **Disk:** 5 "has no holon flavor" rejects remain.
4. ▷ **THE AGGREGATE AUDIT** *(293 — THE CLOSURE GATE)* — classify the ~99 holder-branches / 14 files (`AGGREGATE-AUDIT.md`); unify every **spurious** split (keep only comms / EDN-repr / `holon<:core` assignability). Run against the stable post-declaration tree. **Also catches:** the assoc-incremental vs `build_holon_hologram`-from-scratch hologram-derivation dedup.
5. ▷ **294.d — wire = plain EDN** *(294, step 6)* — kill `HolonRepresentable` + `#wat-edn.holon/*` tags + the HolonAST↔tagged-EDN round-trip. **Disk:** 80 `HolonRepresentable` uses, 47 tag sites.
6. ▷ **294.e — `HolonAST → Hologram` rename + mint `src/holon/`** *(294, step 7 — the keystone)*. **Disk:** 1173 `HolonAST` mentions; `src/holon/` absent.
7. ▷ **294.f — reflection-IR → WatAST** *(294, step 8)* — signatures-as-`HolonAST::Bundle` → WatAST. ~175 (census).
8. ▷ **294.g — homes** *(293.1 + 294, step 9)* — `src/aggregate/` (construction) + `src/holon/` (VSA); both absent.
9. ⛔ **293.5 — CLOSE** *(293)* — `/from-map` (the original 291 driver, step 5) + workspace SET-diff ∅ + ward the homes + amend 291 + INSCRIPTION(s). **GATED on:** the aggregate audit = zero spurious AND the 294 value-layer (1–8) done.

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
