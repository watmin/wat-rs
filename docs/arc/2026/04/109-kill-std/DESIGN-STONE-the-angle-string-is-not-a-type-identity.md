# DESIGN — the subtype lattice keys on the BASE NAME. The args were never doing anything.

> **RULED A-i, builder 2026-08-21.** The edge key is the base name; type args never enter the lattice.
>
> ⚠ **THIS DESIGN PREVIOUSLY ARGUED THE OPPOSITE** — it recommended structured identity
> `(base, args)`, on the theory that collapsing to the base would merge distinctions the corpus
> relies on. Measurement refuted that, and the correction is kept visible because the route matters:
> I recommended preserving, with care, the thing that turned out to be the defect.

## What the measurement found

```
subtype_edges: HashMap<String, Vec<String>>      is_subtype(sub: &str, sup: &str, env)
30 read sites (23 check.rs · 3 types.rs · 2 runtime.rs · 2 collection/map_container.rs)
 8 write sites
```

★ **The args in an edge key are a BOUND VARIABLE'S NAME.** The corpus states it, in the header of
`tests/types/probe_stone118_3b_seqable_parametric_satisfaction.wat`:

> *the declared param name, e.g. `:sq::Seqable<T>` … against the CALL SITE's rendered expected type
> (a fresh unification var, e.g. `:sq::Seqable<?454>`) — `"<?454>" != "<T>"`, **always***

An edge asserts *"this type satisfies that surface."* The surface's params are the SURFACE's bound
variables — they are not arguments of the edge. Putting a bound variable's *name* into a hash key
cannot match anything, and the two helpers that surround the lattice are the scar tissue:

```rust
transport_satisfier_heads(head) -> vec![fq, format!("{fq}<T>"), format!("{fq}<Xt>")]
    // guesses THREE keys because the caller cannot know which declared param name was registered

satisfies_bare_surface(sub, surface, env)
    let prefix = format!("{surface}<");     // …and then IGNORES whatever is inside the brackets
```

**Both exist to work around args that should not be in the key.** Remove the args and
`satisfies_bare_surface` collapses into `is_subtype`; `transport_satisfier_heads` collapses to one key.

**Nothing merges.** Censused over `wat/` + `tests/` + `wat-scripts/`: the only base appearing both bare
and parametric is `:sq::Seqable`, and that is two *separate scratch-pad probe files* each declaring
its own — never loaded together. No base is used both ways within one program.

⚠ My first census missed this by dropping `wat-scripts/` from the path set while the earlier one
included it, and the two disagreed. **Two instruments over different inputs are not a second
opinion.** `[[feedback_two_instruments_agreeing_is_not_corroboration]]`

## The strike

Both doors already exist — no new helper:

```
TypeExpr::base_fqdn()          types.rs:131      node → head FQDN, args stripped
                                                 (its own doc: "One implementation, two doors")
split_type_params_pub(s)       runtime.rs:14266  &str → (base, suffix)
```

```
register_subtype(child, parent)      key both by BASE
is_subtype(sub, sup, env)            compare by BASE
satisfies_bare_surface(…)            DELETE — it becomes is_subtype
transport_satisfier_heads(head)      DELETE — it becomes one key
```

The 30 `is_subtype` call sites keep their `&str` signature untouched: the stripping happens at the
lattice boundary, not at every caller.

## ⚠ It must accept BOTH spellings — this is what closes ②-iii's blocker 3

After ②-iii, `extend-type`'s parent slot is a FORM — `(:wat::core::Seqable :- [T])` — not a keyword
with a `<…>` suffix. So base extraction must take the head from either spelling. That is precisely
②-iii blocker 3's lattice half, and this stone closes it.

`defservice`'s `{b}::Op{p}` concatenation is **NOT closed by this stone** — those build type NAMES
for emission, not lattice keys. That is a separate question and it re-shapes **decision B**.

## ⚠ Named, NOT in scope: base extraction is hand-rolled 16+ times

`grep` finds **sixteen inline `find('<')` sites** across `runtime.rs`, `types.rs`, `check.rs` and
`types/surface.rs`, plus four named helpers (`base_fqdn`, `split_name_and_type_params`,
`split_type_params`, `split_type_params_pub`). They do different jobs — rendering, accessor lookup,
scheme naming — so consolidating all of them is its own stone, not this one.

**But A-i's correctness depends on the LATTICE's extraction being singular**, so the four functions
above must route through one door and be seen to. A second hand-roll inside the lattice would
reintroduce exactly the inconsistency this stone removes.

## The four questions

*Shared premise: the lattice needs one comparable key. Holds — `is_subtype` walks a graph.*

| | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| **A-i** edge key is the BASE NAME; args never enter the lattice | YES | YES | YES | YES |
| **A-ii** structured `(base, args)`, compared structurally | YES | **NO** | YES | — |
| **A-iii** canonical string in the `:-` form `(Head :- [args])` | **NO** | **NO** | **NO** | — |
| **A-iv** canonical string in the angle form | **NO** | **NO** | **NO** | — |

**A-i** — *Obvious*: an edge is a satisfaction relation; the surface's binders are not part of which
edge it is. *Simple*: it DELETES both workarounds rather than re-spelling them. *Honest*: measured to
merge nothing, and today's key claims a precision it demonstrably lacks. *Good UX*: `is_subtype` keeps
its signature; all 30 call sites are untouched.

**A-ii fails Simple** — `Hash` on `TypeExpr`, 30 signature changes, and then it must implement
`<T>` ≡ `<?454>` unification *inside the lattice* — the checker's job, which already lives there. It
would carefully preserve the thing that should be deleted.

**A-iii / A-iv fail all three** — the key keeps args that every consumer then works to ignore, both
workarounds survive re-spelled, and the `<T>`/`<?454>` mismatch persists in new clothes. A-iv also
pins the internal truth to the spelling ③ makes illegal.

## The risk the floor must answer

`transport_satisfier_heads` guesses on the **sub** side as well as the sup side. Both get stripped
under A-i, and only a floor run says whether something was leaning on that — arc 293's transport
machinery (`Handle<Wire>` satisfying a bare `Dialable`) is the likeliest place. If it lights up, the
finding is that some edge genuinely needed an instantiation, and the design gets a NAMED exception
rather than a general one.

## What this stone does NOT do

- **No corpus migration** — ②-iii re-runs after, unchanged.
- **No ③** — legality is untouched.
- **No `defservice` emission change** — that is B's question.
- **No 16-site consolidation** — named above, its own stone.
