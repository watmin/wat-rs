# `TypeExpr::Path` carries its colon; `TypeExpr::Parametric.head` does not — and 137 sites match both by hand

**Filed 2026-08-04, arc 278 (the client-validates-locally strike). Grounded, not fixed.**

> **⚠ AMENDED within the hour, twice, and both corrections matter more than the original filing.**
>
> **(1) The line citations in the first version were WRONG.** It cited `types.rs:4223`/`:4231`;
> those are function-type bracket errors and have nothing to do with this. The real site is
> **`:4292`** and the `<>` arm it mirrors. My fourth adjacent-site citation error of the day and the
> only one that reached disk.
>
> **(2) THE ASYMMETRY IS DELIBERATE, NOT AN ACCIDENT.** The first version implied a mistake ("two
> halves of one enum disagreeing"); the rider that found the bug independently inferred "an accident
> of two separate code paths, not a documented decision." **Both wrong** — there is a comment at the
> site giving the reason, quoted below. This changes the cure: making the storage consistent would
> BREAK unification. Corrections kept in place rather than rewritten away.

> ## ⚠ AMENDED 2026-08-05 — a THIRD instance, and it was LATENT. Plus: the surface GREW.
>
> **(a) A latent instance existed after all, and this note's own conclusion needs qualifying.**
> Below, this note concludes *"a REGRESSION introduced by the new mechanism, not a latent bug it
> uncovered."* That was true of the **two sites it examined** and it is not true of the file.
> `synthesize_surface_protocol`'s ruling-A SHAPE lock read
> `if let TypeExpr::Path(resp_path) = ret` — so the whole *"every serviceable Response must carry
> a well-shaped `RequestTooLarge`/`RequestMalformed`"* rule **never ran on a parametric response**,
> silently, since 16.1c shipped. Not a regression; not introduced by the new mechanism; older than
> both sites below. Proven with a non-vacuity control (parametric-without-RTL accepted silently
> while its monomorphic twin was refused, located), fixed the same day (`f7bd58f9`), recorded in
> `278/NOTE-ruling-a-lock-skips-parametric-responses.md`.
>
> **The pattern that hid it is the one this note already names**, applied to a *skip* rather than a
> *concat*: the `Parametric` arm is rarely exercised, so a consumer that silently ignores it looks
> identical to one that handles it. **Add that shape to the family — not only "a match that reads
> both arms and gets one wrong", but "a match that only has one arm at all."** The second is worse:
> the first produces a malformed name you can see, the second produces no output whatsoever.
>
> **(b) Instance 1 below is GONE.** `build_op_response_type_constants` was deleted outright by #74
> (the builder ruled `<Op>Request`/`<Op>Response` into law, so nothing needs to read the declared
> name any more). Instance 2 (Path B, `runtime.rs`) still reads `ret` — its code was already
> correct and needed no change; only its now-false justifying comment was rewritten.
>
> **(c) ★ THE SURFACE GREW ON THE DAY WE DELETED A SITE.** Re-measured 2026-08-05 with the
> pattern validated first (a bare `TypeExpr::Parametric` grep returns 359 — it counts every
> mention; the note's actual subject, destructures binding `head`, is what is quoted):
>
> | | 2026-08-04 | 2026-08-05 |
> |---|---|---|
> | destructures binding `head` | 137 | **141** |
> | files | 13 | **15** |
> | `impl TypeExpr` | none | **still none** |
>
> One site was deleted and **four were added** — #74's two law checks and #76's normalization, each
> a fresh hand-rolled `Path`/`Parametric` match, each written by someone who had just read this note.
> That is the strongest argument the accessor has: **the convention rung does not merely fail to
> stop new sites, it is where new sites come from.** The note's standing ⛔ *"do not ship the
> accessor as a side effect of a bug fix"* still holds — but the thing it is waiting for is now
> growing faster than it is being repaired.

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

Grounded at **`src/types.rs:4287-4300`**, and the comment states the reason outright:

```rust
// Extract the constructor head as a bare path string (no leading colon).
// Mirrors the <> arm in parse_type_inner which stores `raw_head = s[..lt_index]`
// (the FQDN before `<`, no colon). We must produce the SAME string for unification.
let raw_head: String = match &items[0] { … kw.strip_prefix(':').unwrap_or(&kw).to_string() … };
```

**So the bare head is a load-bearing storage convention, not a slip.** Two parametric-parsing paths
exist — the `Head<args>` keyword form and the `(Ctor arg…)` list form — and both must yield a
byte-identical head string or unification fails. The `Path` form separately re-prepends its colon.
Neither side is wrong; they are answering different questions.

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

**✅ CLOSED — the pre-existing guess was NOT exposed.** Asked at filing, answered within the hour and
grounded: `serve-op-arms`' old concatenation built from `proto-base` — the surface's own declared
keyword *as literally written*, sliced before the `<` — and **never touched `TypeExpr` at all**. It
is colon-correct by construction, and every sibling keyword built the same way (`cap-const-kw`,
`op-variant-kw`, `reply-variant-kw`) has always worked on parametric surfaces, confirmed on a stashed
pre-existing tree.

**This is therefore a REGRESSION introduced by the new mechanism, not a latent bug it uncovered** —
the new code is the first to read `ret: TypeExpr` directly for this purpose. Proven by a stash
differential, not argued: the two `service-parametric-messages.wat` deftests pass on the stashed tree
and fail on the working tree.

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

⛔ **DO NOT "make the two variants consistent."** That is the obvious cure and it is wrong — the bare
head exists so two parser paths unify, and re-adding the colon at storage would break that. The
storage is correct. **The defect is that reading it correctly requires knowing an invariant that is
documented at the PARSER and invisible at all 137 use sites.**

- **convention** — "remember the parametric head is bare." The current state, and the rung that just
  failed, in code written by someone who had read the surrounding file.
- **check** — an `impl TypeExpr` with one accessor returning the FQDN form on demand, leaving storage
  untouched. Makes the right thing available and the sweep mechanical. **This is the realistic rung
  and the only one that does not disturb unification.**
- **no form** — the head is not a bare `String` a caller may read raw but a type that renders itself
  in either form on request, so the un-normalized read has no expression. Costly; the answer if this
  recurs after the accessor lands.

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
