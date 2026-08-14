# ⛔ NOTE — the type converter the drive calls emits the SUPERSEDED form. Fix it before 300.1.

Measured 2026-08-15 at HEAD `0d43266e`. **Nothing here is a guess; every claim has a `file:line`.**

## The defect in one sentence

**All three faithful-conversion drives route type conversion through one Rust function that emits the
pre-2026-07-24 *bare* parametric form — the spelling the builder superseded when he ruled the
bracketed grammar.**

## The chain, end to end

| step | site | emits / does |
|---|---|---|
| the drives | `wat-scripts/fixes/to-faithful-clojure.wat:9` — *"type-shaped keyword (parametric `Head<…>` or tuple) → **list type-form**"* | call ↓ |
| | `to-faithful-clojure-rete.wat:194,209` · `to-faithful-clojure-net.wat:228` | `(keyword/to-type-form …)` ↓ |
| the wat door | `:wat::core::keyword/to-type-form` = `eval_keyword_to_type_form`, `src/edn_shim.rs:1267` | parses the keyword, calls ↓ |
| **the converter** | **`type_expr_to_clojure_form`, `src/edn_shim.rs:1183`** | ⛔ **splices type args FLAT** |

The offending arm, `src/edn_shim.rs:1232–1236` (`TypeExpr::Parametric`):

```rust
let mut items = vec![WatAST::Symbol(Identifier::bare(sym), unk.clone())];
for a in args {
    items.push(type_expr_to_clojure_form(a)?);   // ← spliced FLAT into the list
}
WatAST::List(items, unk)
```

Its own doc comment states the output verbatim (`:1248`):
> *"Convert an old rust-scheme TYPE keyword (`:wat::core::Vector<wat::core::i64>`) into the
> faithful-Clojure type FORM (**`(wat.type/Vector wat.type/i64)`**)."*

## Why that is the wrong form

It is the **2026-06-06** grammar, superseded **2026-07-24**.

- **2026-06-06** — `109/NOTE-generic-bracket-syntax-edn.md` + `109/NOTE-typed-form-and-type-namespace.md`
  addenda: parametrics become forms, **bare args** — `(wat.type/HashMap wat.type/String wat.type/i64)`.
- **2026-07-24** — `109/NOTE-typed-literal-constructors.md` addendum, *"BUILDER-DIRECTED CONCRETE
  GRAMMAR: type-params in a `[…]` vector"*, which names the change against its own sibling:
  > *"That note's addendum wrote parametric types as **bare** form args …; the refinement **brackets
  > the type-params into a `[…]` vector** … so neither the reader nor a human has to guess where the
  > type-params end and the value payload begins. The annotation is `(Head [params])`; the
  > constructor is `(Head [params] …values…)` — one shape, the `[…]` the seam."*
- **2026-08-14** — formalized as `(<head> [<type>…] & <members>)` in
  `251/DESIGN-STONE-251.8-symbol-proper.md:275`. Builder, this session: *"we needed an unambiguous
  generics form.... `(type [parametrics] & literals)`"*.

The reason is **unambiguity**, and the flat form cannot supply it: in
`(HashMap wat.type/String wat.type/i64 :first "foo")` the type/member partition lives in a per-head
arity table the reader must consult (`251/DESIGN-STONE-251.8:286`). The `[…]` puts the seam **in the
form**.

Corroborating: `278/NOTE-parametric-type-forms-already-parse.md:38` records that the bracketed form
**is refused today** — so the substrate does not yet accept what the ruling requires.

## ✅ THE GOOD NEWS — the corpus is CLEAN. The blast radius is 13 fixtures.

| form | occurrences in `wat/` + `wat-scripts/` + `tests/` |
|---|---|
| bracketed `(wat.type/X [ … ])` — **the ruling** | **0** |
| flat `(wat.type/X wat.type/Y …)` — **superseded** | **13** |

All 13 live in `tests/resolve/probe_arc251_*` — the **contract fixtures for the converter itself**
(`keyword_to_type_form__contract-02-parametric`, `…-03-nested-parametric`, `…-05-multi-arg`,
`…-06-tuple`, `parametric_target`, `stone3_parametric_form`, `decl_migrator__c03-generic-decl-parametric`,
`type_namespace_fix__c02-core-parametric`, …). Spellings: `(wat.type/Vector wat.type/i64` ×10,
`(wat.type/Tuple wat.type/i64` ×2, `(wat.type/HashMap wat.type/String` ×1.

**This confirms 300's own status line — *"the faithful conversion drive was drawn and never run — the
abandoned boss."*** The bad form never reached the corpus because the drive never ran. **We are
catching this at exactly the right moment: before the first drive, not after.**

## ⛔ THE CONSEQUENCE FOR 300

**Running ANY drive today converts every parametric type in the corpus to a spelling that was
superseded three weeks before.** 300.1 is *"the pilot — run it on ONE stdlib file, `diff`, verify
EXACTLY the faithful conversion."* A pilot against the current converter would produce a **flawless
diff of the wrong grammar**, and its own STOP triggers would not fire — they check for *unintended*
edits, not for an intended edit in a stale shape.

**A new stone is required BEFORE 300.1.** Proposed **`300.0` — fix the converter**:

- one arm in `type_expr_to_clojure_form` (`edn_shim.rs:1232`): wrap the args in `WatAST::Vector`
  instead of splicing them into the `List`;
- update the **13** `probe_arc251_*` contract fixtures to the bracketed spelling — they are the
  mechanism that goes RED and teaches the change (R65 `SCVTVM IDEM INDEX`: the fixtures enumerate
  the worklist);
- ⚠ **open, and not decided here:** whether `TypeExpr::Fn` (`:1237`, emits
  `[args… :-> ret]`) and `TypeExpr::Tuple` (`:1246`) also need re-shaping. The 2026-07-24 ruling
  speaks to **parametric heads**; the fn-type vector form is core.typed's own and may be correct as
  written. **Do not change them on this note's authority — it is a builder question.**

## The annotation-vs-literal question is CLOSED (do not re-open it)

`109/NOTE-typed-literal-constructors.md` left a "genuine open" — whether `(wat.type/HashMap [K V])`
is a type annotation or an empty typed literal. **Closed 2026-08-15** (addendum on that note): the
grammar has exactly one production yielding a type form, `:- <type-form>`, so a type is unreachable
by any other path. `:-`-preceded ⇒ type; everywhere else ⇒ data literal. **Every site the drive
touches is therefore mechanically decidable — there is nothing for it to guess.**
