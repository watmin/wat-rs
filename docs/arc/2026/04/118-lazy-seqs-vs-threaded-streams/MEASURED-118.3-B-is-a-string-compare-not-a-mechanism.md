# MEASURED — 118.3 stone B: it is a **missing case**, and the case is a STRING COMPARE

**Builder, 2026-08-17: *"B has been reasoned... go measure it."*** The scoping question was:
*when a concrete `Builtin<Concrete>` is checked against a `Surface<T>` parameter, where does `T` fail
to bind, and is that a missing **case** or a missing **mechanism**?*

## The answer: a missing case. The arm exists; its COMPARISON is wrong.

`src/check.rs:14858-14869` — the `(Parametric actual, Parametric expected)` arm:

```rust
if let (TypeExpr::Parametric { head: ah, args: aargs },
        TypeExpr::Parametric { head: eh, args: eargs }) = (&a, &e)
{
    if ah != eh                                                    // Vector vs Seqable ✓ passes
        && transport_edge_keys(&a)
            .iter()
            .any(|k| crate::types::is_subtype(k, &format_type(&e), types))   // ⛔ EXACT STRING
    { … }
```

At the failing call site:

| side | value | source |
|---|---|---|
| `format_type(&e)` | `:sq::Seqable<?454>` | the surface param, **instantiated with a fresh unification variable** |
| the registered edge | `:sq::Seqable<T>` | `extend-type`'s target keyword, stored **VERBATIM** (`types.rs:2151`, per the arm-4 comment) |

`"<?454>" != "<T>"`. **The guard is right, the head check is right, and the comparison is a string
equality that can never succeed.** `T` never binds because nothing ever tries to bind it — the code
asks "are these two names the same text?" where it needs to ask "does this instantiate that?"

## ★ The two probes bracket the defect precisely

| probe | surface | which arm | comparison | result |
|---|---|---|---|---|
| `probe-seqable-is-spellable-today.wat` | **bare** `Seqable` | arm 3, `(Parametric, Path)` | `parametric_head_fqdn` — **head only, no args** | ✅ runs, `"3,4"` |
| the parametric probe + call sites | `Seqable<T>` | this arm, `(Parametric, Parametric)` | full string **with args** | ⛔ RED ×4 |

The *only* difference between running and red is whether the comparison includes the type arguments.
That is not a type-system limitation; it is a `format!` and an `==`.

## ★★ This is the house's own named recurring class

`holon/CLAUDE.md`, the arc-278 corollary, verbatim:

> **when a generic form misbehaves, suspect a string comparison with one side normalized and the
> other not before suspecting the type system. … The type system is usually fine; a
> `format!`/`split`/`==` on names is the culprit.**

Three instances were already recorded in arc 278 alone. **This is the fourth**, and it cost two
months of "Seqable is blocked" — because the failure was read as a missing type-system capability
rather than a name comparison.

**The corroborating tell is on disk one function away.** `types.rs:745`:

```rust
pub(crate) fn transport_satisfier_heads(head: &str) -> Vec<String> {
    let fq = parametric_head_fqdn(head);
    vec![fq.clone(), format!("{fq}<T>"), format!("{fq}<Xt>")]   // ← guessing the letters
}
```

Someone hit this same disease on the *sub* side and patched it by **hardcoding the literal parameter
letters `T` and `Xt`**. That works only when the declaration happens to use those names. It is the
same defect wearing a workaround.

## The 2×2, and who filled each cell

| actual ＼ expected | `Path` | `Parametric` |
|---|---|---|
| **`Path`** | subtype + nature floor | **arc 170 C2 Gap 1** — exact-string, both sides concrete |
| **`Parametric`** | **arc 267** — head-only edge | ⛔ **THIS CELL** — exact-string against a fresh var |

Arm 4's own comment shows the 2×2 was already noticed: *"This is the (Path actual, Parametric
expected) case the branch above (Parametric actual, Path expected) never covered — roles flipped."*
Three cells were filled by three named arcs. The fourth was filled with a comparison that cannot fire.

## Size, honestly

**Small-to-medium, and it is ONE arm.** The fix replaces the exact-string test with one that binds:
resolve the edge by the surface's **bare** key (`parametric_head_fqdn(eh)` — the machinery arm 3
already uses), then **unify** the surface's declared params against the actual's args, using `unify`
and `subst` which are already parameters of this very function.

⚠ **What I have NOT measured, and will not claim:**

1. **Whether binding here is sound in every direction.** Args elsewhere in this arm are explicitly
   **INVARIANT** (*"a channel's send/recv types are exact → unify, not covariant-assignable"*).
   Whether a surface's params may bind covariantly is a real design question this note does not
   settle.
2. **Blast radius.** `transport_edge_keys` / `transport_satisfier_heads` serve `Handle` /
   `TypedCapability` / `Dialable` today. Changing how this arm compares could move those. The floor
   is the instrument; it has 4698 rows and this is exactly the kind of change that finds them.
3. **Per-element dispatch COST — still unmeasured.** The other half of the scoping probe. `join` /
   `map` / `filter` walk every element; a surface dispatch per element could reverse the design, and
   nothing here speaks to it. **It must be measured before any migration**, and it can be measured
   with the *non-parametric* probe shape today, since that path already runs.

## The disposition

B is **not** a type-system arc. It is one match arm whose comparison is string equality where it
needs unification, in a 2×2 whose other three cells are already filled. That is briefable — after
the cost measurement, which is the one thing that could still change the answer.
