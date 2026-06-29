# 294 — The remaining path: from the half-collapsed repr to EDN-canonical aggregates

> **▶ The live close ORDER + STATUS for 293+294 is `CLOSE-SEQUENCE-293-294.md` (the single maintained tracker).
> This doc is the value-layer PATH detail/context, not the sequence — when they disagree, the tracker wins.**

> **Why this doc (2026-06-28, written after a drift).** Chasing 293's *"annihilate the variance,"* the apparatus did
> a **`Value`-repr collapse** and labeled it `293.R2.x`. But the `Value`-repr collapse is **294's deliverable** — the
> 293 DESIGN explicitly scopes it OUT (*"Unifying the `Value` reprs … Keep"*, DESIGN.md:182) and 294 owns it. The work
> is committed, green, pushed, and a **real (if off-design) down-payment on 294.** This doc reconciles what landed
> against 294's actual design and lays the ordered path to the destination. **No reverts — persist and change.**

## The destination (the one model — builder's vision, 2026-06-28, = 294's cure)
> *"holon-records are created from edn not vectors; structs, core-records, holon-records have a from-map interface to
> pass kwargs/map-literals who macro-expand to a positional function call — structs and records are the same
> underlying thing with different policies … identical in creation and usage … the only difference is who can be
> passed to what."*

ONE aggregate citizen. Struct / core-record / holon-record are the **same underlying thing**, identical in creation
and usage, differing ONLY in **policy** (the `Holder` trit: who-crosses-comms + who-satisfies-what):
- **EDN is canonical.** The EDN data (the fields) **is the identity** (Eq/Hash key on it). The hologram is a
  **derived index over EDN**, held in **eager parity** — never the identity, never stale, never absent (294 Q-C/Q-D).
- **Construction = ONE holder-dispatched `(aggregate-new :T field…)`** (varargs; mirrors `(:T a b c)`). Holon derives
  its hologram **internally from the EDN fields** (the codec), never a precomputed vector. `struct-new` / `Record::of`
  / `holon::Record::of` **die into it**. `:T/new` → bare `:T`.
- **`/from-map`** for all three: a kwargs/map-literal that **macro-expands to the positional `(:T …)` ctor**.
- **Wire = plain EDN.** `HolonRepresentable`, the `#wat-edn.holon/*` tags, the HolonAST↔tagged-EDN round-trip are
  **annihilated**. Portability = EDN-repr = `holder != Struct`.
- **`HolonAST` → `Hologram`** (it was always the VSA hologram wearing an AST coat), homed `src/holon/`.

## Where we are (grounded against the disk, 2026-06-28)
**294 ALREADY LANDED (before this session):** 294.0 census (weighed) · **294.a** direct-EDN measurement
(collections+scalars measure without manual `to-holon`; **base records deferred to 294.c**) · **294.b** the `#holon`
relaxed literal / clj↔wat seam (`664193f5`).

**This session's `R2.x` (commits `9d1e3ff3` → `e918c505`, mislabeled "293.R2"):**

| what landed | 294-aligned? |
|---|---|
| 3 `Value` variants → one `Value::Aggregate{class, fields, holder, holon}` | ✅ the structural foundation 294 needs (prereq for `aggregate-new`) |
| one `register_aggregate_methods` (accessor codegen merged) | ✅ helps |
| bare `:T` ctor; `/new` dropped (struct+newtype) | ✅ **exactly 294's ctor-parity sub-task** (294 DESIGN:133) |
| purgare+intueri sweep (B1 `struct->form` regression fixed, dead-world names) | ✅ clean |
| **`holon: HolonForm{Empty \| Hologram(stored Arc<HolonAST>)}`** — hologram **STORED** | ❌ 294 wants it **DERIVED** in eager parity, not a stored field |
| **Eq/Hash for holon key on `(class, holon_form)`** — identity **ON the hologram** | ❌ 294 Q-D: identity is the **EDN data `(class, fields)`** (flaw #7) |

**Verdict (grounded):** R2.x did 294's **structural + ctor-parity** work but **carried 294's central disease
(hologram-as-identity, stored hologram) into the new repr.** Net effect on 294: **LESS difficult** — the 3→1 collapse
is done and the disease is now ONE localized branch (the `Value::Aggregate` Eq/Hash holon arm + the `holon` field)
instead of a whole `wat__holon__Record` variant. The disease is **relocated and concentrated, not cured.**

## The remaining path (ordered; reconciles R2.x — none of it reverts)

1. **Flip holon identity to the EDN data (Q-D + flaw #7).** Change the `Value::Aggregate` Eq/Hash holon branch from
   `(class, holon_form)` → `(class, fields)` — ONE equality contract on the data for every holder. (294 census:
   **no veto** — `hologram.rs:68` never keys records on holograms; similarity is cosine on `Vector`.) Also collapse
   the wat-surface `=` (`runtime.rs:8129`, keys on the fields) and Rust `PartialEq` into the one contract.
2. **Make the hologram derived + eager-parity (Q-C).** `holon: HolonForm` becomes a **parity-cache rebuilt on every
   construct/assoc** from the EDN fields via the codec — never canonical, never the identity, never stale.
   `CapacityExceeded` fires at the mutation (user-tunable), loud.
3. **`aggregate-new` — ONE holder-dispatched ctor.** `struct-new` + `Record::of` + `holon::Record::of` → `(aggregate-new
   :T field…)`. Holon derives its hologram internally (the codec — no precomputed-form arg). This is **"the of funcs
   die."** (R2.3 already did the `:T/new` → `:T` half.)
4. **holon built from EDN (294.c — the deferred 294.a piece).** `to_holon_inner` lifts **base records** (it can't yet —
   needs the field-names threaded, `runtime.rs:14565` STOP-1). Base records measure directly; holon records construct
   from the **same EDN fields** as a core record + the derived hologram.
5. **`/from-map` for all three.** kwargs / map-literal → positional `(:T …)`. The original **291 driver** — unblocks
   291 (293.5 / `291/CURRENT-STATE.md`).
6. **Annihilate the wire scar tissue.** Kill `HolonRepresentable` (trait + 7 impls) + `#wat-edn.holon/*` tags + the
   HolonAST↔tagged-EDN round-trip → the wire ships **plain EDN**.
7. **`HolonAST` → `Hologram` rename + mint `src/holon/`** (the keystone). Home the survivors: the codec, the hologram
   store (`hologram.rs`), the type + its aliases (`BundleResult`/`Holons`), `to-holon`/`from-holon`.
8. **Reflection-IR → WatAST (Q-A).** signatures-as-`HolonAST::Bundle` → WatAST (`function_to_signature_ast` →
   WatAST; the 3 positional walkers; ~15 sites). Consumers: `metadata-of`, `signature-of-defn`/`-fn`, the docs system.
9. **Homes: `src/aggregate/` (construction) + `src/holon/` (VSA)** — evacuate the megafiles (the 293.1-owed home +
   294's holon home; siblings).

## Sequencing notes
- 1–4 are the **value-layer core** (identity → derived hologram → one ctor → EDN records). They cluster — likely one
  or two strikes, RED-probe gated (the equality flip + a holon-constructs-from-EDN probe).
- 5 (`/from-map`) and 6–9 (annihilations, rename, reflection, homes) follow and are largely independent sweeps.
- Each strike: RED probe → BRIEF → sonnet → orchestrator weighs forced-clean → commit on green. **Read this doc +
  294 DESIGN + 293 DESIGN first — never re-derive the surface from grep** (the apparatus's failure this session).

## What this resolves
- **Re-labeling:** the "293.R2.x" commits are **294 work**. Label this work `294` going forward (294.c onward).
- **293's status:** the **type-system (surfaces, methods-as-accessors, `defprotocol` annihilated) is DONE** (demo
  green, `cf89fb52`). 293 **closes via 294** (its construction tail folded in) + **293.5** (amend 291's `/from-map`).
- **The forward path you can take WITHOUT 294:** `Seqable` → 118 needs only 293.4 (methods-as-accessors, done) — 294
  is **not** a blocker for building forward; it is required only to **close** 293's construction story.

## Pairs
`294/DESIGN.md` (the cure, the census, Q-A…Q-E) · `293/DESIGN.md:182` (Value-repr collapse out of 293 scope) ·
`293/NOTE-base-struct-horizon.md` (Layer 1 / Layer 2) · `291/CURRENT-STATE.md` (the blocked `/from-map`) ·
`feedback_grimoire_after_wide_annihilation` · `feedback_ground_codebase_claims_in_codesign` (the disk, not grep).
