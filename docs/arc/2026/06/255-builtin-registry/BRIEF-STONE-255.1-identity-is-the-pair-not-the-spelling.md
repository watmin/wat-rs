# BRIEF — STONE 255.1: a name's IDENTITY is the (namespace, name) PAIR, not its spelling

**Drawn 2026-09-20 against `main` @ `779b084e3`** (floor 5930/5930, clippy 0, census `no STOP-8`).
⛔ **251.8d is BLOCKED on this.** Evidence: `251/FINDING-8d-the-premise-is-FALSE-for-declaration-names.md`.

## THE BUILDER'S RULING

Weighed against the four questions; **Option A was the only 4-YES** (B and C scored 0, D scored 2):

> *"A has the only 4 YES — it has been reasoned"*
> *"So the sequencing … becomes: **255 next (identity + `wat.type` together), then 8d-ii, then
> 8d-iii**."*

**Option A, as ruled:** *make identity canonical; both spellings resolve to one key.* **A name's
identity is its `(namespace, name)` pair, not its characters.**

⚠ **And the SEAM said so first.** It reads ⛔ *"THE REGISTRY UNBLOCKS THE CLOJURIFICATION, NOT THE
REVERSE — if you remember it the other way round, that is the pivot talking."* The orchestrator
argued the chain had inverted and was **wrong**; #95 was closed without the registry, but 8d-iii is
gated by identity, which **is** registry work.

## ⛔ THE ROOT — `src/types.rs:4684`

```rust
let raw = match form {
    WatAST::Keyword(k, _) => k.clone(),
    other => return Err(… MalformedDecl {
        reason: format!("name must be a keyword; got {}", other.variant_name()) }),
};
…
Ok((raw, Vec::new()))          // ← the RAW KEYWORD is the TypeEnv KEY
```

**Two defects; the second is the arc.**
1. A `Symbol` declaration name is refused.
2. ⛔ Even if accepted, `wat.core/Option` would be a **DIFFERENT KEY** from `:wat::core::Option`.
   **Type identity is the bytes someone typed.**

## THE FOUR GAPS — all measured, all on real codemod output

Delta on a 179-file spread of 8d-i's converted corpus, each `--check`ed against its unconverted
original: **161 originals clean → 57 clean after ⇒ 104 REGRESSIONS (65%) ⇒ ~1,240 corpus-wide.**

| # | gap | mechanism | scale |
|---|---|---|---|
| **1** | **declaration names** | `parse_declared_name` — Keyword-only, raw key | `defenum` **301 files**, `typealias` **41**, `defclause` |
| **2** | **macro declarations** | `defrecord`/`defstruct` — *"program body eval failed"*, a **different door** from gap 1 | `defrecord` **623 files**, `defstruct` **119** · 52 of the 104 |
| **3a** | **a declared NAME walked as a REFERENCE** | `(wat.core/def u/x 1)` → `UnresolvedReference :path ":u::x"` — the **name**, not the head | part of 34 of the 104 |
| **3b** | **string-matched builtin heads** | dispatched by literal keyword match (e.g. `src/config.rs:504` `":wat::config::set-capacity-mode!" =>`), so **`is_resolvable_call_head` says no** and the symbol never normalizes | **43 of 260** string-matched heads, mostly `wat.config/*` + `wat.core/bigint` |

⭐ **3a IS THE WHITELIST FINDING.** `FINDING-8d-a-symbol-whitelist-entry-gets-RESOLVED.md` reports a
`:restricted-to` entry being walked as a reference. **Same defect: a name in NAME position treated as
a name in REFERENCE position.** One cure should close both — **confirm that, do not assume it.**

⭐ **3b IS THE #95 SHAPE, RESIDUAL.** 8c closed the *type-check* hole for slashed heads. This is the
*resolution* hole: the symbol path has a gate (`is_resolvable_call_head`) the keyword path does not.
⚠ **217 of the 260 already resolve**, so this is not "symbols don't work" — it is a **registration
hole**, which is exactly this arc's subject.

## AND `wat.type` — the builder's other ruling, same arc

`RULING-wat-type-must-exist-proper.md` (2026-09-20). `types.rs:632`:

```rust
/// Canonicalize `:wat::type::X` → `:wat::core::X`. Every store answers
/// `false` for a `:wat::type::` spelling; callers must not forget this.
```

⭐ **`wat.type` has ZERO members.** It is a `format!` at **3 sites** (the SEAM recorded 2 — it has
spread) plus a request that every caller remember. Measured: **6 of 18 Rust scalars**;
`:wat::type::Vector` **annotates but is an `UnresolvedReference` in call position**.

⛔ **The builder's cut — the container becomes real; the CONTENTS are deferred:**

| | |
|---|---|
| **this stone** | `wat.type` is a real namespace with **members**, holding the builtin types **under whatever names they have now**. Membership is queryable. **One position, not two.** |
| **NOT this stone** | short names (`set`/`map`/`vector`), the case convention (`Vector` vs `vec` — arc 109's), `HashMap` vs `PersistentMap`, the 12 missing scalars |

> *"i don't know if we're ready for the end state names.. but wat.type must exist proper."*

⭐ Making the container real is what makes the deferred question **answerable**: once it has members,
*"is `wat.type/set` a thing?"* has a yes/no instead of an argument.

## ⭐ THE METHOD — land the RULE, then RE-MEASURE. Do not fix four gaps four ways.

All four gaps share one root: **identity is the spelling**. A canonical-identity door may close
several at once.

1. **Land canonical identity** — one door, both spellings in, one key out. The codebase already has
   the shape (`canonicalize_type_kw`); this generalises it and **deletes the "every store answers
   false" sentence**.
2. ⛔ **RE-RUN THE DELTA** — the 179-file converted spread, `--check`ed against originals. **State
   which gaps closed for free and which remain.** Fix what remains, one at a time, re-measuring.
3. **Report the residue.** If a gap needs something outside identity, that is a finding, not a
   silent extra fix.

⚠ **Do not start by patching `parse_declared_name` to accept Symbols.** That closes gap 1 and leaves
identity ambiguous — two spellings, two keys — which is **Option B, scored 0-YES and explicitly
rejected**.

## The gate

- ⭐ **THE DELTA IS THE GATE**: re-run the 179-file converted-vs-original `--check` comparison.
  Baseline today: **104 regressions**. State the new number. ⛔ A number without the comparison is
  not the gate — `[[feedback_a_number_assembled_from_two_measurements]]`.
- `wat.type` answers membership: a bool for *"is this a valid namespace?"*, a set for *"what is in
  it?"*. ⛔ `contains()` must stop answering `false` for a `:wat::type::` spelling.
- **One position, not two**: `wat.type/Vector` means the same in annotation and call position, or is
  refused in both with the same error.
- **A non-vacuity control**: a NON-member is refused, and the refusal says *"not a member of
  `wat.type`"* — distinct from *"unknown type"*, or a typo and a real absence read alike.
- The 3 `strip_prefix(":wat::type::")` sites collapse to the registry. **A gate should catch a 4th.**
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` **0**; `census.sh --diff`
  → `no STOP-8`. Predict the test delta from the diff, confirm with `cargo nextest list`.
- ⛔ Run crate clippy **yourself** before scoring. 218.8's clippy fix tripped a *different* wall —
  **rune, don't revert**, when two walls disagree.

## Doctrine — `wat-rs/CLAUDE.md` does not reach a subagent

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole the first time; never re-run first.
- ⛔ **ASK THE TOOL THAT OWNS THE FACT.** Test counts → `cargo nextest list`.
- ⛔ **`wat/*.wat` is `include_str!`'d.** An on-disk edit is invisible until `cargo build --release`.
  **Three stones have lost time to this. If a cure appears not to work, REBUILD before theorising.**
- ⛔ **If this brief contradicts the code or the runner, THEY WIN.** The orchestrator's briefs have
  been corrected **seven times across five stones** — including the 8d premise this arc exists to
  repair, and a path glob that silently missed 33 of 64 files. **Assume an eighth.**
- Work only in `/home/john/work/holon/wat-rs`. No `git filter-branch`. **Do not push.**

## Out of scope — affirmatively cut

- **The `wat.type` NAME SET** — short names, case convention, the 12 missing scalars. Builder
  deferred it explicitly. **The container only.**
- **251.8d-ii and 8d-iii** — both wait on this stone. Do not convert a single `.wat` file.
- **The `:restricted-to` validation pass** (builder ruled: exempt from resolution **and** validate
  explicitly). ⚠ Its *resolution* half is gap 3a and may close here; the **validation** half is not
  this stone. Say which happened.
