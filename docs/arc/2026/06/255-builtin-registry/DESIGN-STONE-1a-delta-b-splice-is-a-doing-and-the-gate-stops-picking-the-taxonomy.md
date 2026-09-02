# DESIGN — STONE 1a-δ-b: `:Splice` is a doing, and the gate stops picking the taxonomy

> **Builder, 2026-09-02:** *"is load a declaration?... so we need a new term for it?... we've been
> growing category as we find it incomplete"* — then, on the proposed name: *"is splice too close to
> what macros do?.. or is load's slice /exactly/ what macros do when splicing?.."*
>
> Answered by measurement, below. **It is exactly what macros do**, and the two differences are the
> two the axis explicitly forbids as axes.

## ★★★ The measurement — one node becomes N, in both

```rust
// src/macros/expand.rs · flatten_template_children        THE MACRO SPLICE
if let Some(splice_arg) = match_unquote(child_items, ":wat::core::unquote-splicing") {
    let spliced = splice_argument(splice_arg, …)?;
    out.extend(spliced);          // one node → N nodes in the parent's child list
    continue;                     // the splice node itself does not survive
}

// src/load/loader.rs · process_forms                      THE LOAD SPLICE
if let Some(load_spec) = match_load_form(&form, …)? {
    process_single_load(load_spec, …, out)?;   // one node → N nodes in the stream
} else { out.push(form); }
```

Both walk a `Vec<WatAST>`, both replace one node with N in the output, both drop the node. **The
same operation.** The two things that differ:

```
where the N forms come from    a bound template value   vs   a file on disk
when it happens                expand time              vs   load-resolution
```

`runtime-meta.wat`'s own header: *"Not what it returns, **not where its input comes from**… The axis
is the DOING, **not the moment it happens**."* **Both differences are ruled out by the discipline the
file states about itself.**

★ And the family discriminates cleanly, which is the test of a real category rather than a bucket:

```
unquote            ,x     one node → ONE node     substitution — NOT splice
unquote-splicing   ,@x    one node → N nodes      splice
load-file! · digest-load! · signed-load!          splice
forms · struct->form      return a VALUE          neither
```

## Why `:Splice` and not `:Inclusion`

`intueri`, cast on all five enums, found `:Declaration` **Level 2 (mumbles)** for these three and
named the word. Verified against the disk: **"splice" appears in all three rows' own prose, in their
`@ret` text and their `@example` text, and in `loader.rs`** — written with no naming pressure on
anyone. `:Splice` is unused anywhere in the axis vocabulary.

★★ The ward also caught the tell I had missed: **the loaders' docstrings argue `Declaration` by
pointing at the *spliced declarations'* visibility, not at the load form's own effect** — a shift of
referent no sibling `Declaration` row needs. The prose was conceding the stretch while making it.

`:Inclusion` is honest and defensible. `:Splice` is better on one measurable ground: it is the word
the substrate already speaks for this doing, three files over, and it names the operation at the same
grain the rest of `Category` uses (`:Combine` builds a larger value; `:Projection` takes a part out).

⚠ **`unquote-splicing` is UNREGISTERED** (one of the remaining 18). "The category will cover it too"
is a **prediction**, checkable when 1a-γ registers the homoiconic family. If the reasoning does not
survive contact there, that is a finding about `:Splice` — not a licence to stretch it.

## ★★★ THE ONE CONTRACT DECISION — the gate stops asking `@Category`

Minting `:Splice` and adding it to the gate's branch would repeat the defect one variant later.
`every_special_form_carries_check_and_eval_impls` currently reads:

```rust
if entry.category == wat_doc::Category::Declaration { … require Declare … }
else                                                { … require Check AND Eval … }
```

**That is the wrong axis.** The gate's real question is *"does this form ever evaluate?"* — and there
is already an axis whose entire meaning is that:

```
@Purity Unevaluated   ⇒   must name Declare        (it cannot name eval)
otherwise             ⇒   must name Check and Eval
```

★ **Measured, both directions, after 1a-δ:** 11 rows declare `@Purity Unevaluated`; 11 rows carry a
`Declare` impl; **the two sets are identical, with no row on either side alone.** The swap is exact
and behaviour-preserving today.

**This is the half that matters.** `@Category` becomes a taxonomy again instead of a registration
constraint, and the next author choosing a category is choosing a *name*, not negotiating with a
gate. `[[NOTE-the-sloppy-registries-a-measured-census]]`'s thesis, applied to the campaign's own
instrument.

## THE FOUR QUESTIONS — the name

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **`:Splice`** | YES | YES | YES | YES | ✅ **PICKED** |
| `:Inclusion` | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |
| keep `:Declaration` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| `:Load` / `:Source` | **NO** | YES | — | — | ⛔ DISQUALIFIED |

- **`:Inclusion` Good UX? NO** — one grain more abstract than the rest of the enum, and it asks the
  reader to translate to the word the codebase already uses in the very rows it labels.
- **`:Declaration` Honest? NO** — it claims *"registers a program-level entity"*; measured, a load
  registers zero.
- **`:Load` names the verb** (circular), **`:Source` names the object** — neither names a doing.

## THE FOUR QUESTIONS — the gate

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **derive from `@Purity Unevaluated`** | YES | YES | YES | YES | ✅ **PICKED** |
| add `:Splice` to the category branch | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| leave the gate, take `:Declaration` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |

- **add-to-the-branch Honest? NO** — it re-commits the defect with one more variant, and the next
  never-evaluating form that is neither a declaration nor a splice hits the identical wall.
- **leave-it Honest? NO** — the measurement stands: a load registers nothing.

## Blast radius

```
wat/runtime-meta.wat                     + :Splice with its ;; prose   ← the source of truth
crates/wat-doc/src/lib.rs                CATEGORY_LEGAL_VALUES widens (its OWN gate forces this)
crates/wat-macros/wat_intrinsic.rs       + one match arm (compiler-forced, exhaustive)
crates/wat-macros/wat_special_form.rs    + one match arm (compiler-forced, exhaustive)
src/intrinsic/mod.rs                     the gate's predicate: category → purity
src/intrinsic/special/{load_file,digest_load,signed_load}.rs
                                         @Category Declaration → Splice; the ground rewritten
```

★ **`Category` already has the message gate `Purity` lacked** (`crates/wat-doc/src/lib.rs:2071`,
`category_message_lists_every_variant`) — so a forgotten `CATEGORY_LEGAL_VALUES` goes red on its own.
That gate was there before this campaign; Stone 1a-β-0b had to *add* `Purity`'s. This one collects.

## Acceptance — rows chosen to be unfakeable

| what | expected |
|---|---|
| the pole exists in the SOURCE OF TRUTH | `wat/runtime-meta.wat`; the Rust enum follows |
| ⛔ the gate swap changed NOTHING today | the 11 declare-rows and the 11 `Unevaluated` rows are the same 11, before and after |
| ⛔ the gate still bites — declare side | drop a loader's `role = declare` → RED, "missing role: declare" |
| ⛔ the gate still bites — eval side | drop `:wat::core::if`'s `role = check` → RED |
| ⛔ `@Category` no longer decides | set a loader to `@Category Io` → **GREEN** (it is `Unevaluated`) |
| ⛔ and the purity axis now does | set a loader to `@Purity Pure` → RED, missing check + eval |
| the message gate collects | omit `Splice` from `CATEGORY_LEGAL_VALUES` → RED naming it |
| the loaders' grounds are rewritten | no ground argues Category from the *spliced* forms' effect |
| floor · clippy | green · 0 |

★ **Rows five and six are the stone.** They are the same experiment run twice, and they must come out
opposite: the category must stop mattering to the gate, and the purity must start mattering. Either
one alone proves nothing.

## Out of scope = REJECTED

- **`unquote-splicing`'s category.** Predicted `:Splice`; decided when 1a-γ registers it, on its own
  measurement.
- **Re-categorising anything else.** `intueri` found 13 of 14 `Category` variants keep their promise,
  and all four other enums clean. This stone changes one variant's population.
- **The `MacroRegistry` fork.** Untouched, still open.
