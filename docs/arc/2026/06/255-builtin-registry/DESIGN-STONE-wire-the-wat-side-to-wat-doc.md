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

1. `wat_doc::from_metadata` — reads `:purity` / `:determinism` / `:totality` / `:expand-time` (and the
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

## ★ THE VALUES ARE ENUM SYMBOLS, not bare keywords — builder, 2026-08-30

> *"can we hvae the category value... be an actual enum symbol?"*

Yes, and it generalises past `:category`. wat spells a variant `:namespace::Enum::Variant` (live
uses: `:wat::bracket::PoolMsg::Setup`, `:wat::cache::Cache::GetRequest`), and every axis is already a
`defenum` in `wat/runtime-meta.wat`: `Purity` · `Determinism` · `Totality` · `ExpandTime` ·
`Category` · `Kind` · `DefinedIn` · `Layer`.

So each closed-domain value is written as the enum value it actually is:

```clojure
   :category    :wat::runtime::Category::Transform
   :purity      :wat::runtime::Purity::Pure
   :determinism :wat::runtime::Determinism::Deterministic
   :totality    :wat::runtime::Totality::Total
   :expand-time :wat::runtime::ExpandTime::Legal
```

★ **A bare `:Transform` is a keyword the checker cannot validate — it could be `:Transfrom` and
nothing would notice until a reader tripped over it.** An enum symbol is a closed domain: the
variant either exists or the form does not check. Same reasoning as examples-as-forms and
types-as-keywords — **the declaration stops being text that happens to be right.**

## ✅ `@Total` → `@Totality` — DONE, 2026-08-30

> *"i think totality reads better than total... we should probably do a mass sed on the rust side"*

Done: **658 occurrences → 0**, `@Totality` 1 → 659, across 102 files. Boundary-safe (`@Total\b`
cannot match inside `@Totality`, so the one pre-existing use was untouched), verified by before/after
counts and a `Totalityity` corruption check.

★ **It made four things agree that were three-versus-one.** The error variant was already
`MissingTotality`, the enum `Totality`, the struct field `totality` — **only the directive said
`@Total`.** It was the odd one out, not the standard.

⚠ No `INSCRIPTION.md` was touched — verified. Only living docs (DESIGN/BRIEF/NOTE/SEAM/RULING) and
`.rs`. The three `.wat` hits were `;;` prose, not forms, so no codemod was owed.

## ✅ SHIPPED 2026-08-30 — and two things the design did not predict

**The round trip works.** A wat-defined verb now answers `metadata-of`:

```clojure
(:wat::runtime::metadata-of :wat::string::capitalize)
;; => Some {:purity :wat.runtime.Purity/Pure  :totality :wat.runtime.Totality/Total
;;          :category :wat.runtime.Category/Transform  :determinism …/Deterministic
;;          :expand-time …/Legal  :args [[w :wat.core/String "…"]]  :ret […]  :examples […]  :doc "…"}
```

Proven by perturbation, not construction: deleting `:purity` from the map raises
**`MissingPurity`** — *the same `DocError` variant the `///` path raises.* The shared contract holds
under a broken door.

### ⚠ 1. The rider's gate was a SILENT SKIP, and it is amended

`from_metadata` was called only when the map contained `:doc`. Measured:
`(defn :probe::half {:purity …} [x] -> :i64 x)` ran **clean, exit 0** — a map declaring `:purity`
and nothing else was never validated. **A declaration that does not declare.**

That is the silent-skip class Stone P4 killed at `intrinsic/mod.rs:512` and `:742`, and it is worse
than a missing feature: the author wrote a property expecting it to mean something.

**Gate is now: ANY doc-axis key ⇒ validate the FULL required set.** A partial declaration is an
error naming what is missing; a capability-only `{:restricted-to […]}` map carries no doc-axis key
and is untouched — which is what keeps the three pre-existing stdlib verbs the rider found
(`write-fd-raw`, `flood-stdout-raw`, `str-double`) out of a migration nobody asked for.

★ **That near-miss was the rider's best catch.** Unconditional validation would have failed stdlib
startup entirely — migrating three unrelated verbs by accident while walking one through the door.

### ⚠ 2. ONE registration path of SIX is wired — say so plainly

`binding_metadata.insert` has **six** call sites (`runtime.rs:861,978,1491,2832,3991,4066`). The
validation is wired to the one `capitalize` travels. **A user-program `defn` with a metadata map is
not validated today** — measured. That is honest for a stone whose scope is one verb through the
door, but it must not read as "declarations are checked".

### Known limits, from the rider, none needed by `capitalize`

Type tokens in `:args`/`:ret` are bare `Keyword`s only — the bracket forms the text grammar accepts
(`(Head :- [args])`, `[args :-> ret]`) would need an AST→source printer this leaf crate does not
have. `:examples` are always `run: true` (no `@example-norun` equivalent). `:args` has no
`is_rest`/variadic. **A future migration hits all three.**

## Acceptance

| what | command | expected |
|---|---|---|
| one wat verb declares | a `defn` with `{:purity :wat::runtime::Purity::Pure …}` | loads, checks clean |
| the declaration is READ | `metadata-of` on that verb | `Some`, carrying the declared axes |
| the same errors as the Rust path | a metadata map missing `:purity` | the **same `DocError`** the `///` path raises |
| the Rust path is untouched | `wat_doc::parse` callers | still 2, unchanged |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
