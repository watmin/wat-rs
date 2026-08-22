# NOTE (arc 109) — the three surviving primes want a `$sigil`, and one fixture's premise died with the migration

**Filed 2026-08-22. A POINTER, not a decision.** Surfaced by the type-reference wall's first
corpus-wide census (1532 `.wat` files) — the first instrument that has ever asked whether a type
name names anything.

Builder: *"primed names are allowed with justification — we polluted our code base extensively while
doing several migrations… there's a very small amount of approved primed names… my lean is
`$something` so the name bears immediate explanation."*

## The census, comments stripped

⚠ **Strip `;;` comments before counting primed names.** My first pass did not, and it inflated every
figure — `insert's OWN cost` is an English possessive, and `wat/query.wat:5` names
`:wat::kernel::Peer'` in prose. I then USED the contaminated count to propose a fix (see the
retraction below). `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

In CODE, corpus-wide: **69 occurrences, and 56 are inside `wat-scripts/fixes/*.wat`** — the recorded
codemods that killed the primes, which must name the old spellings because those are their rewrite
SOURCES. That is the migration's own record, not debris.

```
reclaim-ipc-prime-names.wat  24    rete-oracle-sigil.wat            7
reclaim-stdio-prime-names.wat 6    deprime-telemetry-sqlite.wat     6
kill-make-deftest.wat         5    + 5 more reclaim-/drop-/retarget- codemods
```

**Live residue: two files.** `wat/core.wat` (4× `sort'`) and `wat/kernel/readln.wat` (3× `readln'`).
Rust declares exactly three primed names: `:wat::core::sort'`, `:wat::kernel::readln'`,
`:wat::kernel::Frame'`. **The prime migration completed.**

## What the prime currently MEANS, from the migration's own header

`wat-scripts/fixes/rete-oracle-sigil.wat`, verbatim:

> *"Prime `'` **stays the language native/IPC marker**; rete no longer uses it for the kernel.
> `$oracle` is the odd name a differential must type on purpose."*

So the sigils in play are not rivals for one job:

```
'          the native / IPC marker            sort' · readln' · Frame'
$oracle    the differential twin, typed on purpose
$native    a rete native that kept a wat wrapper
$impl      pre-existing
```

## The proposal — rename the three, because `'` explains nothing

All three survivors are the same shape: a public name delegating to a native implementation.

```
:wat::core::sort      defclause  →  :wat::core::sort'        wat/core.wat:1439-1451
:wat::kernel::readln  macro      →  :wat::kernel::readln'    wat/kernel/readln.wat:94-101
:wat::kernel::Frame   record     →  :wat::kernel::Frame'     positional-prime CONSTRUCTOR
```

`'` is a single character carrying four historical meanings, and a reader cannot tell from the name
which one applies. `$native` / `$impl` say it at the call site. **`Frame'` is the odd one** — it is a
constructor idiom, not a native delegate, and it already carries `// rune:lint(retired-name)` at
`runtime.rs:25855`, so it may want a different destination from the other two.

⚠ **Not ruled, and NOT a stone yet.** Renaming a Rust-declared intrinsic touches the registration,
the dispatch arm, and every wat call site; it is a `wat-fix` codemod plus a substrate edit, and it
should ride with whatever else moves those files.

## ⛔ `:wat::core::HashMap'` is ILLEGAL, and its fixture's premise is dead

One occurrence in 1532 files: `tests/types/probe_arc214_lexer_primed_generic_head_primed.wat`.

The type has never existed — `HashMap'` appears nowhere in the substrate. The fixture is a LEXER
probe: *"PRIMED two-param generic head — must pass the LEXER (CommaInKeywordBody must NOT fire).
Twin of the control; the apostrophe is the ONLY variable."* It grabbed an arbitrary two-param generic
and hung an apostrophe on it. It lexed, so it passed, and nothing downstream ever asked whether the
name meant anything.

★ **Its motivation is gone.** The driver states it plainly: *"The 4.5 peer types are PRIMED
(`:wat::kernel::Thread'` / `Process'`) and parametric (`<I,O>`)."* Those were all killed by
`reclaim-ipc-prime-names.wat`. **The only primed multi-param generic head left in the entire corpus
is this illegal one** — so the lexer capability it guards can no longer be exercised by any legal
expression.

That makes it a third kind of find, distinct from a typo (`Int`) or a registry gap (`char`):
**a test whose subject was retired out from under it, still passing because it only ever asked an
early-stage question.** The builder's read: it is really a `.wat.bad` example for a corrected lexer.

## ⛔ RETRACTION — I proposed rewriting it to `Peer'`, and `Peer'` is dead too

I wrote *"`Peer'<S,R>` is real, primed, two-param, and used three times."* **All three were comment
mentions.** In code, `Peer'` has two occurrences and both are inside codemods:
`reclaim-ipc-prime-names.wat:65` is literally the row `("…Peer'" → "…Peer")` that killed it. It is
not Rust-declared, and `Nature::Peer`'s root spells `:wat::kernel::Peer`, unprimed (`types.rs:205`).

**I would have replaced one dead name with another**, using a count I took with the wrong instrument
and then reused as evidence. The correct disposition for that fixture is open — retire it with the
migration that removed its subject, or re-point it at whatever primed head survives the rename above.

## Kin

- `wat-scripts/fixes/rete-oracle-sigil.wat` — the header that defines what `'` still means
- `wat-scripts/fixes/reclaim-ipc-prime-names.wat` — the codemod that killed the primed peer types
- `109/NOTE-type-annotation-names-unchecked.md` — why none of this was visible until now
