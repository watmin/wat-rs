# DESIGN — STONE: wire the wat side to `wat-doc`

> **Builder, 2026-08-30:** *"wire the wat side to wat-doc, then"*
>
> Ruled. But the pre-flight found that the literal reading of "wire it" is the wrong door, and the
> builder's own earlier words point at the right one.

## ⛔ THE FINDING — there is no wat-side TEXT to parse

`wat_doc::parse` takes **joined `///` text** and runs a line-based `@tag` grammar over it. For the
wat side to feed that parser, a wat form would need a docstring. **It has none:**

```
doc_string: Option<String>   — on every `Binding` variant, "the paved road for arc 141"
                               (src/runtime.rs:12897)

  6 of 7 construction sites:  doc_string: None          hard-coded
  the 7th (SpecialForm):      def.doc_string.clone()    — and special_forms.rs:92 sets it None

  src/macros/ :               no doc_string field at all
  arc 141 :                   DESIGN.md only. Never shipped.
```

The comment says *"arc 141 populates the `Some` cases as docstring sources arrive."* **They never
arrived.** So wiring the wat side to `wat_doc::parse` would mean **inventing docstrings for 409
forms purely to have text worth parsing** — building a prose channel in order to scrape it.

★ That is the "magic comments" problem the builder rejected, rebuilt on the wat side.

## THE ONE CONTRACT DECISION — pinned

**The wat side enters `wat-doc` through a METADATA-MAP entry point, not the text parser.** Same
crate, same enums, same required-directive enforcement, different input shape:

```
wat_doc::parse(raw: &str)            -> DocComment    the Rust `///` path, unchanged
wat_doc::from_metadata(map: &WatAST) -> DocComment    the wat path, NEW
```

This is the builder's own framing — *"declare all of these properties as actual wat data... not
'magic comments'"* — and it uses a slot that is already live rather than one that never shipped.

## Why the metadata map is the right carrier — measured

| property | status |
|---|---|
| it is a real form slot | ✅ `(defn :name {:restricted-to […]} [args] -> :Ret body)` |
| it parses today | ✅ 4 live corpus uses (`wat/spawn.wat:338`, `wat/kernel/services/stdio.wat`) |
| it reaches the runtime | ✅ lands on `sym.binding_metadata`, keyed by name, at def registration (`runtime.rs:861,978,1411`) |
| the checker already enforces one key | ✅ `:restricted-to` is a live capability wall |
| the axis enums are already shared | ✅ `wat_doc::{Purity,Totality,Determinism,ExpandTime,Category}` — 37 uses on the runtime side |

**Nothing new has to exist.** The slot, the parse, the storage, and the enums all ship today; only
the reading of property keys out of the map does not.

## What ships

1. `wat_doc::from_metadata` — reads `:purity` / `:determinism` / `:total` / `:expand-time` (and the
   universal required set) out of a metadata-map `WatAST`, producing the **same `DocComment`** the
   text path produces, enforcing the **same required directives** with the **same `DocError`s**.
2. The wat def-registration path calls it, so a wat `defn`'s declared properties reach
   `sym.binding_metadata`'s neighbour — a place the reflection layer can read.
3. **One wat verb declares its properties**, end to end, and `metadata-of` returns `Some` for it.

⚠ **Scope is ONE verb, not 409.** The smallest thing that proves the door, per the seam.

## Out of scope = REJECTED (not deferred)

- **Migrating the 409.** This stone builds the door and walks one verb through it.
- **`:defined-in Wat`** — `metadata-of` hard-codes `DefinedIn::Rust` (`runtime.rs:13619`). Making it
  discriminate is its own stone, and it is the seam's named "should not wait" item. This stone must
  not silently make that constant *more* wrong by registering a wat verb behind it.
- **Retiring the Rust `///` path.** `wat_doc::parse` keeps its two callers. Nothing about the Rust
  half changes — the builder was explicit that it is already configured.
- **Docstrings for wat forms / arc 141.** Affirmatively cut: this stone routes *data*, and prose is
  a separate question that this design argues against needing.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **metadata-map entry point into `wat-doc`** | YES | YES | YES | YES | ✅ **ADMITTED** |
| give wat forms docstrings, feed `wat_doc::parse` | YES | **NO** | **NO** | — | ⛔ **DISQUALIFIED** |
| a second parser for the wat side | YES | **NO** | **NO** | — | ⛔ **DISQUALIFIED** |
| derive the axes from the body instead | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **docstrings Simple? NO** — build a prose channel across 409 forms to have something to scrape.
  **Honest? NO** — it is the magic-comment shape the builder rejected, rebuilt on the wat side.
- **second parser Simple? NO / Honest? NO** — two implementations of one contract is exactly what
  `wat-doc`'s existence prevents; its header calls the shared crate *"parity by construction, not by
  discipline."*
- **derive Honest? NO** — for a *declaration*. Derivation says a verb can never be more trusted than
  what it calls; that is a fine default and a lie as a declaration. (It remains the right tool for
  *checking* a declaration, which is a later stone.)

## Acceptance

| what | command | expected |
|---|---|---|
| one wat verb declares | a `defn` with `{:purity :Pure …}` | loads, checks clean |
| the declaration is READ | `metadata-of` on that verb | `Some`, carrying the declared axes |
| the same errors as the Rust path | a metadata map missing `:purity` | the **same `DocError`** the `///` path raises |
| the Rust path is untouched | `wat_doc::parse` callers | still 2, unchanged |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
