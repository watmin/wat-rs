# DESIGN — there must be only one parser for a name

> *"why do we have two things, not one?... how do we kill this?.. unify ...however many of these
> grammars we have.... we attack these when we encounter them...."*
> *"there must only be one"*
> *"we very often find that we have the tool we need, we just didn't reach for it.... so... we use it"*
> — the builder, 2026-08-23

## Why there are two

**A name is an atom. Structure encoded inside an atom must be re-parsed by every consumer of that
atom.** The reader's grammar turns source text into keywords; but a keyword like
`:wat::spawn::Locus/launch<Op,Reply>` carries a namespace path, a surface method, and a type-argument
list *inside its own text*. Anything that needs those has to parse the string again — and with no
shared door, each site hand-rolled its own `rfind` / `rsplit` / `strip_suffix`.

**`:-` already proved the cure for one of the four.** `(Vector :- [i64])` is a **List** — the type
arguments arrive as separate AST nodes and nothing re-parses anything. Source became a FORM. What
this stone attacks is everything that stayed a NAME.

## The census — measured, `grep -c` distrusted

`grep -c` counts lines, and it has lied repeatedly in this arc. The list below is per-site, comments
excluded, and the two LEGITIMATE homes are called out rather than counted as violations.

```
rsplit("::").next() / rfind("::")     15 sites   → the LEAF segment of a path
rfind('/') / rsplit_once('/')         15 sites   → RECEIVER + METHOD of a surface call
strip_suffix('\'')                     3 sites   → the PRIME
                                    ── 33 hand-rolls across 16 files
legitimate homes (NOT violations):    crates/wat-reader/src/identifier.rs:146
                                      crates/wat-reader/src/lexer.rs:921
```

Separately and by the same disease: **`runtime_error_edn.rs::edn_path_segments` and
`runtime.rs::edn_coerce_path_segments`** are two implementations of path segmentation in two files.
In scope for the wall, and a worked example of what the wall is for.

⚠ **The ANGLE family (48 sites) is deliberately NOT in this stone.** It is not unified — it is
*eliminated*, by making `defservice` mint a FORM instead of concatenating a name, at which point
those parsers have nothing to parse. Tracked as `STONE-the-minted-identity-is-a-form`. Mixing the two
would hide a deletion inside a refactor.

## The tool we already have, and the precedent already written on it

`Identifier` (`crates/wat-reader/src/identifier.rs`, 238 lines) already owns exactly one accessor of
this kind, and its doc comment states the discipline verbatim:

> *"STONE 251.8a: the namespace is DERIVED from the spelling (split on the last `/`), not stored —
> `Identifier` still holds one `name` string. **251.8b is where derived swaps for stored behind this
> same signature.**"*

That is the whole design, already ruled on and already shipped once. One signature; derived today;
stored tomorrow; **callers never change either way.** It was never extended past `namespace()`, so 33
hand-rolls grew up beside it. This stone extends it.

## What ships

**1. `Identifier` grows the missing accessors**, each the single spelling of one question:

```
leaf()        the last `::` segment            (`:wat::cache::Lru`      → `Lru`)
path()        everything before the leaf       (`:wat::cache::Lru`      → `:wat::cache`)
receiver()    everything before the `/`        (`:S/mk`                 → `:S`)
method()      everything after the `/`         (`:S/mk`                 → `mk`)
prime()       is the name primed?              (`:sort'`                → true)
deprimed()    the name without its `'`         (`:sort'`                → `:sort`)
```

**2. Free-function twins taking `&str`**, because most call sites hold a keyword's text and not an
`Identifier`. The methods delegate to the free functions, so there is one implementation and two
surfaces — never two implementations.

**3. The 33 hand-rolls become 33 calls.**

**4. A rune — `one_name_grammar`** — refusing `rfind("::")`, `rsplit("::")`, `rfind('/')`,
`rsplit_once('/')` and `strip_suffix('\'')` on a name **anywhere outside `identifier.rs`**. This is
the step that makes it *stay* one, and it is the same move as the lexer wall one layer up: impose it
and let the survivors scream. The repo's rune library (`no_loose_string_assert`, `no_inlined_edn`,
`retired_name_justified`) is the home and the shape to copy.

⚠ The rune must not fire on a `/` or `::` in something that is **not a name** — a filesystem path, a
URL, an EDN tag, a doc string. That discrimination is what an allowlist-with-reason is for
(`rune:lint(...)` with a justification), exactly as the existing runes do it. **A rune drawn too tight
makes the honest path non-compliant** — see `[[feedback_a_guard_drawn_too_tight_makes_the_honest_path_noncompliant]]`.

## The four questions

- **Obvious?** YES. "Where is a name taken apart?" gets one answer: `identifier.rs`. Today it gets 34.
- **Simple?** YES. Six accessors, each one question. No new type, no new concept — an existing struct
  finishing a job it started in arc 251.
- **Honest?** YES, and this is where it bites: 33 hand-rolls are 33 chances for two of them to
  disagree, and this arc has now watched that exact shape cause nine separate defects. A second
  implementation of a grammar is a lie waiting for a call site.
- **Good UX?** YES. `id.method()` at the call site instead of `k.rfind('/').map(|p| &k[p+1..])`, and
  the reader of that line no longer has to reconstruct what `p+1` meant.

## What this stone does NOT do

Out of scope, affirmatively cut, not deferred:

- **The angle family** — a sibling stone (`STONE-the-minted-identity-is-a-form`), because it is a
  deletion and not a unification.
- **251.8b** — swapping DERIVED for STORED behind these signatures. The whole point of the precedent
  is that it can happen later without touching a caller; doing it here would double the stone for no
  gain at the call sites.
