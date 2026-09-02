# DESIGN — STONE 3a-i: the registry at the front of `lookup_form`

> **Builder, 2026-09-01:** on my phrase *"gains a registry fallback"* — *"is this yet another thing
> that the registry should be doing?...."* then *"the registry at the front... sequence it next....
> draw it...."*
>
> The word was the tell. Under `[[RULING-the-registry-is-the-sole-authority]]` there is no fallback;
> **a fallback makes the registry the backup to `special_forms.rs`.** This is Phase 3 work arriving
> early, because Phase 1a cannot finish without it.

## The measurement

```
crate::intrinsic::registry()  in ALL of src/reflect/  (6 files) .......  0
                              inside lookup_form ....................... 0
                              inside eval_metadata_of .................. 1
```

★★★ **Two reflection verbs, two different authorities.** `metadata-of` asks the registry.
`:wat::runtime::lookup-form` asks five other things:

```
0. types (auto-ctor special case)      3. CheckEnv::with_builtins_and_types  ← asks the TYPE CHECKER
1. sym.get — user defines                 what substrate primitives exist
2. macro registry                      4. types
                                       5. special_forms::lookup_special_form
```

Every branch returns `doc_string: None`, while `IntrinsicEntry` holds `prose`, `args`, `ret_type`,
`examples`, `see`, `added` and five axes for the same name.

⚠ **Step 3 builds a whole `CheckEnv` per lookup** to use the checker's scheme table as a membership
oracle — because that table was the only membership list `lookup_form` knew about.

## ⛔ The ordering question, and why it is safe — measured, not assumed

`lookup_form`'s own doc claims *"Lookup precedence mirrors the runtime's call dispatch"* and puts
**user defines first**, *"shadow builtins per call-dispatch precedent."* The runtime disagrees:

```
eval_tail:969    the registry door
eval_tail:1019   other if sym.has_function(other)      ← user functions come AFTER
```

★★ **But the disagreement is unreachable.** `:wat::` is in `RESERVED_PREFIXES`
(`src/resolve/reserved.rs:25`) and *"user definitions under these paths are refused at registration
time."* A user define can never collide with a registry name, so the shadowing rule protects against
a state that cannot exist — **for exactly the names the registry owns.**

⚠ **And the guarantee is underwritten by `is_reserved_prefix`, the check this arc is deleting.** When
the registry becomes the membership authority, "you cannot define `:wat::x`" must become "the
registry already owns `:wat::x`." **This stone must not silently inherit a guarantee whose source is
scheduled for removal** — it states the dependency so the resolve stone knows it is load-bearing here
too.

## THE ONE CONTRACT DECISION — pinned

**The registry is consulted FIRST among the builtin steps — before `CheckEnv`, before
`special_forms` — and user defines/macros keep their existing precedence.**

Not "at the very front": user defines and macros stay ahead, because that ordering mirrors runtime
dispatch for names the registry does *not* own. The registry goes where the *builtin* question is
asked, and it answers that question completely.

## What falls out, and what does NOT

- **Step 3's `CheckEnv` construction** stops being a membership oracle. ⚠ It may still be needed for
  names the registry lacks (the 89 in `GAP_A`) — so it does not die in this stone; it becomes the
  residue the ledger already measures.
- **Step 5 `special_forms`** becomes reachable only for names the registry lacks. ⛔ **Its rows must
  NOT be deleted here.** Stone 1a-i measured that `and`/`or`'s rows are the *only* path by which
  `lookup_form` resolves them today; once the registry answers first, they become dead — but the
  proof is the floor, not the argument.

## ⛔ The `Binding` question — the stone's real design decision

`Binding` has five variants and none of them fits a registry entry cleanly:

```rust
Primitive   { name, scheme: TypeScheme, doc_string }   ← 89 registry rows have NO scheme
SpecialForm { name, signature: HolonAST, doc_string }  ← a synthesized sketch, not registry data
```

★ A registry row carries strictly more than either variant can hold: prose, examples, `see`, `added`
and five axes, plus a `Kind` that already distinguishes intrinsic from special form.

**Three shapes, and the stone must pick one:**

| shape | what it does |
|---|---|
| **A** | a new `Binding::Registered { name, entry: &IntrinsicEntry }` variant |
| **B** | map a registry row onto the existing `Primitive`/`SpecialForm` variants by `Kind`, synthesizing what they need |
| **C** | populate `doc_string` from the registry, keep the variants as they are |

⚠ **A is the honest shape and the most disruptive** — every `Binding` consumer must learn a variant.
B loses data at the boundary (a schemeless row has no `TypeScheme` to put in `Primitive`).
C is the smallest and answers only item 7, not item 3.

★★ **This DESIGN does NOT pick.** The pick needs the consumer census — how many places match on
`Binding`, and what each does with it — and that census is the stone's first act, not its premise.
`[[feedback_a_design_is_unfalsifiable_until_something_consumes_it]]`

## THE FOUR QUESTIONS — on the sequencing, not the shape

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **registry first among the builtin steps** | YES | YES | YES | YES | ✅ **ADMITTED** |
| a registry *fallback* after `special_forms` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| registry before user defines/macros too | **NO** | YES | YES | — | ⛔ DISQUALIFIED |
| finish Phase 1a's 22 first, then this | YES | YES | **NO** | — | ⛔ DISQUALIFIED |

- **fallback Honest? NO** — the builder's own catch. A fallback makes the registry the backup to a
  table the RULING says must be eliminated. It would also let both answer, silently, forever.
- **before-user-defines Obvious? NO** — it would change precedence for names the registry does not
  own, on no evidence, in a stone about something else.
- **1a-first Honest? NO** — measured in 1a-i: `and`/`or`'s `special_forms.rs` rows cannot be deleted
  until `lookup_form` asks the registry. Phase 1a is *blocked* on this, so doing 1a first means 22
  more registrations that each leave a row nobody can remove.

## Out of scope = REJECTED

- **Deleting any `special_forms.rs` row.** Proven-dead is a floor result, and the rows go in 1a's
  own stones once this lands.
- **`is_reserved_prefix`.** Named as a dependency, untouched.
- **The `CheckEnv`-per-lookup construction.** It survives while `GAP_A` is non-empty.

## Acceptance — rows chosen to be unfakeable

| what | check | expected |
|---|---|---|
| the registry is asked | `grep -c "crate::intrinsic::registry()" src/reflect/lookup.rs` | 0 → **≥1** |
| it is asked BEFORE `CheckEnv` and `special_forms` | read the chain | registry consult precedes both |
| user defines and macros keep precedence | the chain | steps 1–2 unmoved |
| ⛔ reflection does not regress | `lookup-form` on `and`/`or`/`if`/`let` | still resolves |
| ⛔ the two verbs now agree | `metadata-of` and `lookup-form` on the same registered name | same authority answers both |
| a schemeless row resolves | `lookup-form` on a `GAP_A` name with no `TypeScheme` | resolves, does not fall to step 3 |
| ⛔ `special_forms.rs` untouched | `git diff --stat src/special_forms.rs` | empty |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5119/5119, 0 failed |
| clippy | `-D warnings --all-targets` | 0 |

★ **The fifth row is the one that matters**: two reflection verbs answering from one authority is the
RULING's item 7 satisfied for the first time.
