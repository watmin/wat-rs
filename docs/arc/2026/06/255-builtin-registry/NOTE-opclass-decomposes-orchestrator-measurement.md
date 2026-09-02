# NOTE — `OpClass` decomposes into three booleans, two of which the registry already has

> The orchestrator's own measurement, written BEFORE the `solvere` cast returned, so the cast's
> verdict can be weighed against an independent read rather than credited.
> `[[examinare]]`: *weigh the kill against your own reading of the disk — never the returned report.*

## Method — read the CONSUMERS, not the definition

`OpClass`'s four variants say nothing about whether they are one concern or several. What decides it
is what the consuming code asks. There are **five** consumers in the tree:

```
src/runtime.rs:954        class == Form                        the tail re-mapping
src/runtime.rs:4330       Alias|Form|Redispatch  vs  Fallback  dispatch_rete_op's split
src/rete/expr_ir.rs:380   class == Fallback
src/check.rs:2588         Form|Redispatch        -> infer_rete_form (bespoke inference)
src/check.rs:17486        Alias|Fallback         -> env.register(TypeScheme)
```

★★ `check.rs:2588` and `:17486` are **exact complements** — one question asked twice, from opposite
sides. `runtime.rs:4330` and `expr_ir.rs:380` are likewise one question.

## The result — three independent booleans, and the classes are their coordinates

| class | rows | is-form | has-`:undefined`-fallback | gets-a-TypeScheme |
|---|---:|:---:|:---:|:---:|
| `Alias` | 35 | no | no | **yes** |
| `Fallback` | 20 | no | **yes** | **yes** |
| `Form` | 9 | **yes** | no | no |
| `Redispatch` | 10 | no | no | no |

Four of the eight combinations are used. **`OpClass` is not a concept; it is a 4-point enumeration of
a 3-dimensional space** — the shape a class system takes when three orthogonal facts get one name.

## ★★★ Two of the three already have a home in `IntrinsicEntry`

```
is-form            ->  kind == Kind::SpecialForm          ✅ EXISTS
gets-a-TypeScheme  ->  args / ret_type                    ✅ EXISTS, and PROBE(255) (bb1aa686d)
                       measured 384/386 of them reconstructing a real TypeScheme, 71/71
                       generic quantifiers recoverable. FROZEN_CHECKER_DEBT_LEDGER already
                       tracks the no-scheme population by name.
has-:undefined-    ->  ⛔ NO HOME — the one genuinely new property
  fallback
```

**So the answer to the RULING's third open question is: `OpClass` does NOT survive a fold. It
decomposes into two properties the registry already carries and exactly one it does not.**

## ★★ And that dissolves the SECOND open question

`Redispatch`'s 10 rows were flagged as the hard case — their own doc says their type *"cannot be
stated as a rank-1 `TypeScheme` at all."* In the decomposition, `Redispatch` is **all three booleans
false**: not a form, no fallback, no scheme.

**"No scheme" is a state the registry already represents in 71 other rows.** `Redispatch` is not a
special problem; it is the ordinary one, and `FROZEN_CHECKER_DEBT_LEDGER` is already the ledger it
belongs on.

⚠ What remains true and must not be lost: those rows still need *bespoke inference*
(`infer_rete_form`). The decomposition does not conjure a type for them — it says the registry can
record that they HAVE no rank-1 type, which is exactly what the debt ledger records for 71 others.

## What is still NOT answered

1. **`core_name` — the alias target.** Question 1's genuinely homeless field, separate from `OpClass`
   entirely. `IntrinsicEntry` has no alias concept; every `RETE_PREFIX` re-mapping in the tree
   hand-rolls one.
2. **`params` / `ret` / `type_params` as `ParamType`**, not doc strings. The probe measured doc
   strings round-tripping to `TypeScheme`; it did not measure `ParamType` → doc string.
3. **Whether the fallback boolean wants a field or a handler.** `dispatch_rete_op`'s `Fallback` arm
   derives its split from `op.params.len()` — so the property may be "carries a fallback handler",
   not "carries a flag".

⚠ Three is the one most likely to be got wrong by reading, and it is where the cast's independent
verdict matters most.
