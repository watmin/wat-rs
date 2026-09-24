# DESIGN — one name per message type

**Drawn 2026-09-23 by the orchestrator. Not yet a stone:** the probe
(`PROBE-SPEC-one-name-per-message-type.md`) runs first; this design is drawn only if it survives.

## Why

The builder ruled *exactly one way to do things* (SEAM, 2026-09-23). Today every operation's message type
has **two registered names**:

| name | minted by | source |
|---|---|---|
| `<S>::<OpPascal>Request` — e.g. `StdOut::WriteRequest` | the user's `:messages` declaration; **required** by the checker | `src/types.rs` ~3647, Arc 278 #74b (*"convention is law"*) |
| `<S>::<op>/Request` — e.g. `StdOut::write/Request` | Rust, one `TypeDef::Alias` per op | `src/types.rs` ~4132-4170, Arc 278 surface-minted op alias |

Same for `Response`. **Target: one name — the required one.** It is the name the user wrote.

## What the alias does — measured (`FINDING-the-message-alias-carries-the-arg-mapping.md`)

1. **Naming** — lets `defservice` name a message without the registry. ⛔ **Duplicated by the law**, whose
   stated purpose is the same (*"so the wat macros can go back to splicing a literal ctor keyword"*).
2. **Argument mapping** — ⭐ **load-bearing.** A message's type arguments are *not* a uniform function of
   its surface's parameters: `Cache :- [K V]` has `PutRequest :- [K V]` but `GetResponse :- [V]`;
   `BareBox :- [T]` and `Pair :- [K V]` have bare messages. The alias records the mapping per op
   (`type_params: surf.type_params`, target *"EXACTLY as `:features` declared it"*).

## What the macro needs it FOR — measured, all four sites in `wat/service.wat`

| site | role |
|---|---|
| `:1381` | the field type of each `Op` enum variant (`[req <- (<S>::<op>/Request :- proto-args)]`) |
| `:1784` | the expected type when decoding a request off the wire (`mpath` / `mexpected`) |
| `:2036` | the client method's request parameter type |
| `:2045` | the client method's return type |

⭐ **Every use is a TYPE ANNOTATION. None constructs a value** — constructors use the required base name.
So the mapping is needed only to *annotate*.

## Why the macro cannot compute the mapping itself — an ordering, not missing information

`src/freeze/pass_order.rs` (pinned by 255.12): `3 resolve-loads` → `4 expand-all` → `5 register-types`.
Every `defsurface` is parsed by step 3, but is findable **by name** only at step 5; `defservice` expands at
step 4. Types register after expansion because expansion *generates* types — **including surfaces**
(`wat/query.wat:362` emits a `defsurface`).

**Rejected, with reasons:**
- ⛔ *Register surfaces before expansion* — hand-written surfaces early, generated ones late: **two
  registration paths for surfaces**, which the ruling forbids.
- ⛔ *Require every message to take exactly its surface's parameters* — forces phantom parameters
  (`GetResponse :- [K V]` with `K` unused; bare messages gaining `:- [T]`). A constraint on users to paper
  over a macro's limitation.
- ⛔ *Keep the alias as internal plumbing* — still a registered, user-visible second name.

## Direction — defer the mapping to where the information is indexed

The macro emits the **required name** and leaves the arguments to be resolved **at or after step 5**,
where every surface — hand-written or generated — is registered. **One registration path, one name.**

**The one contract decision this design must pin — and the probe must answer:** *how does the macro write
"the request type of operation `op` of surface `S`" using only the required name, such that resolution
supplies the right arguments?* Candidates, in order of preference:

1. **Omit the annotation and let inference supply it.** The client method's request and return types are
   already fixed by the surface method the service satisfies. If wat can type a generated method from the
   surface signature it satisfies, the macro need not write the type at all. Cleanest; depends on whether
   wat allows an unannotated parameter/return in that position.
2. **A type-level projection, resolved after registration** — the macro writes a form meaning *"the
   request type of `S`'s op `op`"*, reduced at step 5 against the surface's declared signature. ⚠ It is a
   *computation*, not a registered name — but it is still a second way to *write* the type, and must be
   weighed against the ruling, not assumed past it.

⛔ **Rejected outright:** writing the required name with *all* the surface's parameters and projecting
away the extras. A type silently accepting more arguments than it declares is unsound-leaning.

## Files (expected)

`wat/service.wat` (the four sites), `src/types.rs` (retire the alias mint), and whichever of `src/check.rs`
inference or a type-level reducer the probe selects.

## Out of scope = rejected

Pipeline reordering. Phantom parameters. Any change to the law itself. The `<svc>/<op>` *method*
separator (a separate question under the narrow-waist ruling).
