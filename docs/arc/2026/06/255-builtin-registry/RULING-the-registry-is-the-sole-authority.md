# RULING — the registry is the sole authority

> **Builder, 2026-09-01:**
>
> *"the registry must be the thing who knows all names.. who delegates to the code who performs for
> those names... it is what you query to know what exists... what they take.. what it returns....
> the properties these names have....*
>
> *the registry must know what is pure, what is deterministic, what is total.... this is the data
> store for reflection... holder of metadata-maps... and so on.....*
>
> ***we must eliminate every source of duplication or inconsistency.....***"

This is the standard. Every stone in this arc is measured against it, and it settles questions that
have been re-argued per-stone all session.

## The seven things the registry owns

1. **Every name.** Membership is a lookup — it is there or it is not. Never a prefix, never a
   namespace guess, never "reserved."
2. **The delegation.** The registry holds the pointer to the code that performs the name.
3. **What exists.** The answer to *"is this defined?"* comes from the registry, for every layer that
   asks — resolve, check, eval.
4. **What they take.** Parameters, with their types.
5. **What they return.**
6. **The properties.** `@Purity` · `@Determinism` · `@Totality` · `@ExpandTime` · `@Category`.
7. **Reflection.** `metadata-of`, `render-doc`, `show-source`, the metadata maps — all read from it.

## ⛔ THE COROLLARY, which is the operative half

**Anything else that answers one of those seven is a duplicate authority and must be eliminated.**

Not "kept in sync." Not "cross-checked by a gate." A gate that compares two tables is a *measurement
of the split*, not a cure for it — and this arc has built four of those. They were right as
instruments; they are wrong as destinations.

## The census — every competing authority found so far, measured

| authority | where | rows | answers |
|---|---|---:|---|
| **`RETE_OPS`** | `src/rete/vocabulary.rs` | **74** | names · aliases · signatures · a class system |
| **`register_builtins`** | `src/check.rs` | **350** `env.register` | signatures |
| **literal type-grammar arms** | `src/check.rs` | **118** | signatures, for forms with no scheme |
| **`RETIREMENT_TABLE`** | `src/remedy/retirement.rs` | **144** | names that no longer exist |
| `intrinsic_meta` residue | `src/rete/purity.rs` | 37 | purity · determinism · totality |
| `is_expand_time_legal` residue | `src/macros/eval.rs` | 54 | expand-time legality |
| `effectful_by_prefix` | `src/rete/purity.rs` | 8 prefixes | effectfulness, **by prefix** |
| `is_reserved_prefix` | `src/resolve/reserved.rs` | prefixes | membership, **by prefix** |
| `constructor_meta` / `accessor_meta` | `src/rete/purity.rs` | derived | properties (⚠ derived from `TypeEnv`, cannot go stale) |
| `step_list`'s table | `src/runtime.rs` | 19 | the stepper's competence (⚠ a capability, not a duplicate) |

### ⛔ AMENDED 2026-09-01 — the census MISSED a third registry, found by the `solvere` cast

| **`SPECIAL_FORMS`** | `src/special_forms.rs` | **19** | names · syntax sketches · a `doc_string` placeholder |

Its own header, verbatim: *"Arc 144 slice 2 — **special-form registry**… This registry lets
`:wat::runtime::lookup-form` return `Binding::SpecialForm` for each known form, exposing a
synthesized signature sketch… and a placeholder `None` doc_string (**arc 141 will populate it**)."*

★★★ **It calls itself a registry, it answers the RULING's items 1, 4 and 7, and its single consumer
is `src/reflect/lookup.rs:197` — reflection.** It predates the intrinsic registry (arc 144 vs arc
255) and still waits on a doc_string an arc-141 stone was to supply, while `IntrinsicEntry` now
carries prose, args, ret, examples and all five axes.

⚠ **My census had ten entries and this was not one of them** (`grep special_forms` on the original
returned 0). A census written from what I had been reading all session, not from a search. The cast
found it by following `OpClass::Form`'s rows to their targets and discovering two of them —
`:wat::core::and`, `:wat::core::or` — are registered *here* and nowhere else.

And the four **absence ledgers** — instruments that exist only because the split does:

```
FROZEN_CHECKER_DEBT_LEDGER   73     REGISTRY_MEMBERSHIP_GAP_A   89
REGISTRY_MEMBERSHIP_GAP_B   115     FROZEN_TYPES_UNCHECKED      10
```

★★★ **Every one of those four is a measurement of duplication. When the ruling is satisfied, all four
are empty and can be deleted.** That is the arc's finish line, stated as a falsifiable condition
rather than a feeling.

## ⚠ Two entries are NOT duplicates, and the distinction matters

- **`constructor_meta` / `accessor_meta`** derive from the frozen `TypeEnv` rather than holding a
  hand-list. The completeness gate's own note says so: *"they cannot go stale. Only the
  hand-managed `intrinsic_meta` needs a gate."* Derivation from one source is not duplication.
- **`step_list`'s 19 names** declare a *capability* — what the stepper can single-step — with
  `NoStepRule` as its honest refusal. Measured and refuted as a door. The registry could own the
  membership claim; it cannot supply a step rule that does not exist.

**A ruling that cannot tell a duplicate from a derivation would delete correct code.**

## ⛔ What the ruling does NOT license

- **It does not license deleting a table before its consumers can ask the registry.** The blanket
  accept is the worked example: flipping it today fails **578 of 599** corpus files, because the
  registry cannot vouch for `fn`. Order is forced — registry completes, then the consumer asks, then
  the duplicate dies.
- **It does not license folding a shape the registry cannot hold.** `RETE_OPS`'s `Redispatch` rows
  carry types *"that cannot be stated as a rank-1 `TypeScheme` at all."* Eliminating that duplication
  requires answering what happens to them, not asserting it away.
- **It does not license a gate as a destination.** A gate that freezes the split by name is how we
  see progress; it is not progress.

## The order the ruling implies

```
1  the registry can ANSWER            —  it holds the name, the pointer, the signature, the properties
2  the consumer ASKS the registry     —  resolve, check, eval, the property gates
3  the duplicate DIES                 —  and its absence ledger empties with it
```

Every stone states which of the three it is. A stone that does 1 without 2 is unfalsifiable — that
was measured this session, and it is why the ratchet exists.

## Standing verification

The four absence ledgers ARE the progress meter. A stone that claims to eliminate duplication and
moves none of them has not eliminated any.

```
GAP_A 89 · GAP_B 115 · DEBT 73 · TYPES_UNCHECKED 10        ← 2026-09-01, at this ruling
```
