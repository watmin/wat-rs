# `TypeExpr::Path` carries its colon; `TypeExpr::Parametric.head` does not — and 137 sites match both by hand

**Filed 2026-08-04, arc 278 (the client-validates-locally strike). Grounded, not fixed.**

Sibling of [`NOTE-macro-minted-names-are-unvalidated-string-concatenation.md`](NOTE-macro-minted-names-are-unvalidated-string-concatenation.md)
— that one is the **wat** layer (a name assembled by `string::concat` inside a macro, unvalidated).
This is the **Rust** layer, and it is worse in one specific way: the two halves of a single enum
disagree about whether a name carries its sigil, so a caller that handles both correctly *by
symmetry* is wrong.

## The asymmetry

```rust
TypeExpr::Path(p)                     // ":probe::Repl::EvalResponse"   — colon PRESENT
TypeExpr::Parametric { head, .. }     //  "probe::Repl::BoxResponse"    — colon ABSENT
```

Grounded: `src/types.rs:4223` and `:4231` — `kw.strip_prefix(':').unwrap_or(&kw).to_string()`. The
head is stripped before the parametric is built; the `Path` form keeps (or re-acquires) its colon.

So the natural, symmetrical-looking match is a bug:

```rust
let base = match ret {
    TypeExpr::Path(p)                 => p.as_str(),      // ":a::B"
    TypeExpr::Parametric { head, .. } => head.as_str(),   //  "a::B"   <- silently different
};
format!("{}::RequestTooLarge", base)                       // one of these is malformed
```

**It reads as correct. Both arms pull "the name." Only one of them carries the sigil.**

## The instances found so far — 2, and both in code written to remove a naming bug

`build_op_response_type_constants` (the Rust emitter) and the surface-namespaced dispatch path
("Path B") in `src/runtime.rs`, both authored 2026-08-04 to **replace a guessed name with the
declared one**. The replacement was correct about *which* name; it was wrong about the name's *form*.
Being fixed in that strike, two sites only.

**⚠ OPEN, and it is the interesting one:** the pre-existing `serve-op-arms` guess concatenated onto
`proto-base`. Was *that* colon-correct for parametric surfaces, or has the parametric path been
quietly malformed there too, for as long as it has existed? Unanswered at filing. Three parametric
fixtures exist to test it against: `wat-tests/service-parametric.wat`,
`service-parametric-messages.wat`, `service-parametric-two-params.wat`.

## The exposure surface — 137, and that is NOT a defect count

**Measured 2026-08-04:** `TypeExpr::Parametric { head, .. }` is destructured at **137 sites across 13
files** (`check.rs`, `runtime.rs`, `types.rs`, `types/surface.rs`, `edn_shim.rs`, `freeze.rs`,
`closure_extract.rs`, `macros/parse.rs`, `intrinsic/mod.rs`, `value/environment.rs`,
`collection/{infer,seq_container,map_container}.rs`).

**And there is no `impl TypeExpr` at all** — two patterns, both empty. No accessor exists for anyone
to normalize through, which is exactly why every consumer hand-rolls the match.

⛔ **137 is the surface, not the count of broken sites.** Many of those sites compare the head
against a *bare* name and are correct. Which ones concatenate or compare it against a
**colon-prefixed** name is unmeasured, and must be measured before any sweep is scoped
(`[[feedback_a_greps_count_is_not_an_enumeration]]`).

## Why nothing caught it

| instrument | why it is blind |
|---|---|
| the type system | a keyword built from a `String` is a keyword; nothing checks what it spells |
| the loader gate | it sees **literals in source**; a name computed in Rust never appears as a token |
| `--check` | an unknown callee defers to a **runtime** `UnknownFunction` — established again today |
| the corpus | monomorphic surfaces dominate, so the `Path` arm is exercised constantly and the `Parametric` arm rarely; the asymmetry hides in the arm nobody runs |

## The cure, on the ladder

- **convention** — "remember to re-add the colon for parametric." This is the current state, and it
  is the rung that already failed.
- **check** — an `impl TypeExpr` with one accessor returning the FQDN in a single consistent form.
  Makes the right thing available and the sweep mechanical. **This is the realistic rung.**
- **no form** — the head cannot be a bare `String` that a caller may read raw; it is a type that
  carries its own normalization, so the un-normalized read has no expression. Costly, and the right
  answer if this recurs after the accessor lands.

⛔ **Do not ship the accessor as a side effect of a bug fix.** With two callers it is
`[[feedback_no_consumers_does_not_mean_dead]]` in reverse — a door minted mid-strike that nobody is
made to walk through. It wants its own stone with the measured sweep behind it.

## Why this note exists in 109 rather than 278

It is not a rules-engine fact. It is a substrate-wide naming-discipline fact, and 109 is where those
live — beside the full-enum-match rule, the IO-boundary outcome-enum doctrine, and the
macro-minted-names sibling above.

## Kin

- `[[NOTE-macro-minted-names-are-unvalidated-string-concatenation]]` — the same family at the wat layer.
- **Arc 278's named recurring class**, from `CLAUDE.md`: *"when a generic form misbehaves, suspect a
  string comparison with one side normalized and the other not before suspecting the type system …
  The type system is usually fine; a `format!`/`split`/`==` on names is the culprit."* Three prior
  instances are recorded (the companion-name suffix appended past `<T>`; a flat `split(',')` tearing
  `State<K,V>`; a `:messages` membership check comparing a base against `Name<K>`). **This is the
  fourth, and the first to appear inside the repair for the class.**
- `278 R64 QVOD TVEBAMVR, NOS TVETVR` — the strike this surfaced during.
