# BRIEF — there must be only one parser for a name

`Identifier` (`crates/wat-reader/src/identifier.rs`) already owns the name grammar and already carries
the discipline in its own doc comment — *"the namespace is DERIVED from the spelling … 251.8b is where
derived swaps for stored behind this same signature."* It was never extended past `namespace()`, so
**33 hand-rolled parsers grew up beside it.** You will give `Identifier` the missing accessors, route
all 33 through them, and put up a rune so a 34th cannot appear.

Read `DESIGN-STONE-one-name-grammar.md` first. Copy the report shape of
`SCORE-STONE-the-last-comma-lives-in-a-symbol.md`.

## The accessors

Each is the ONE spelling of one question. Free function on `&str` + a method on `Identifier` that
delegates to it — one implementation, two surfaces.

```
leaf()        the last `::` segment        :wat::cache::Lru  → Lru
path()        everything before the leaf   :wat::cache::Lru  → :wat::cache
receiver()    everything before the `/`    :S/mk             → :S
method()      everything after the `/`     :S/mk             → mk
prime()       is the name primed?          :sort'            → true
deprimed()    the name without its `'`     :sort'            → :sort
```

Pin the edge cases in unit tests beside them, because the 33 sites do not all agree today and the
door has to be right where they differ: **no separator at all** (`:foo` — what does `leaf()` return?
what does `receiver()`?), a **leading colon** (kept or stripped — pick one and say which), an
**empty segment** (`:a::`), and a name that is **both** primed and slashed (`:sort'/apply`).

## The 33 sites

```
── leaf / path  (rsplit("::") · rfind("::")) ───────────────────────────────────
src/rete/expr_ir.rs:680          src/rete/expr_ir.rs:1207     src/rete/kernel/session.rs:891
src/closure_extract.rs:1565      src/resolve/registration.rs:91
src/edn_shim.rs:1316             src/edn_shim.rs:2807         src/edn_shim.rs:3211
src/edn_shim.rs:4098             src/edn_shim.rs:4140
src/rete/validate.rs:861         src/rete/where_tree.rs:334   src/check.rs:12730
src/intrinsic/reflect.rs:355     src/runtime.rs:2246

── receiver / method  (rfind('/') · rsplit_once('/')) ──────────────────────────
src/rete/expr_ir.rs:692          src/rete/purity.rs:981       src/resolve/walk.rs:305
src/closure_extract.rs:1529      src/macros/expand.rs:562     src/edn_shim.rs:3215
src/types.rs:4866                src/types.rs:4982            src/check.rs:4961
src/check.rs:5581                src/resolve/normalize.rs:407 crates/wat-edn/src/vocab.rs:179
src/runtime.rs:3454              src/runtime.rs:3763          src/runtime.rs:7053

── prime  (strip_suffix('\'')) ─────────────────────────────────────────────────
src/check.rs:3476                src/check.rs:5627            src/check.rs:12711
```

⛔ **These two are the LEGITIMATE homes — do NOT convert them:**
`crates/wat-reader/src/identifier.rs:146` (the door itself) and `crates/wat-reader/src/lexer.rs:921`
(the reader building the name in the first place).

★ **And one pair to collapse while you are here:** `src/runtime_error_edn.rs::edn_path_segments` and
`src/runtime.rs::edn_coerce_path_segments` are two implementations of path segmentation in two files.
Same disease; make them one, on the door.

## The rune — `one_name_grammar`

The step that makes it STAY one. Refuse `rfind("::")`, `rsplit("::")`, `rfind('/')`,
`rsplit_once('/')`, `strip_suffix('\'')` applied to a name, anywhere outside `identifier.rs`. Copy the
shape of an existing rune — `tests/lint/no_loose_string_assert.rs` and `retired_name_justified` are
the two to read; both carry the allowlist-with-reason pattern (`rune:lint(...)` plus a justification).

⚠ **Draw it at the right tightness.** A `/` or `::` in a filesystem path, a URL, an EDN tag or a doc
string is NOT a name, and a rune that refuses those makes the honest path non-compliant — which is how
you get people writing worse code to satisfy a gate. Where you cannot discriminate structurally, the
allowlist-with-reason is the answer, not a narrower rule that quietly misses real sites.

## STOP triggers

- **STOP-1 — two sites disagree about an edge case** and no single accessor semantics satisfies both.
  That is a real finding about the name grammar, not a refactor detail: report both sites and what
  each expects. Do NOT add a second accessor to paper over it.
- **STOP-2 — a site is not parsing a NAME** (it is a path, a URL, an EDN tag). Leave it, allowlist it
  with a reason, and say so.
- **STOP-3 — the rune cannot be drawn without either missing real sites or refusing honest ones.**
  Report the shape you tried and what it could not separate. Ship the accessors and the conversions
  without the rune rather than shipping a rune that lies.

## Boundaries

- `crates/wat-reader/src/identifier.rs` (the accessors), the 33 listed sites, the
  `edn_path_segments` pair, and one new rune under `tests/lint/`.
- **Do NOT touch the ANGLE family** — `find('<')`, `split_type_params`,
  `split_name_and_type_params`, `split_method_name_type_params`, `canonical_callable_name`. Those are
  a sibling stone and a DELETION, not a unification. Converting them here would hide the deletion.
- **Do NOT do 251.8b** (derived → stored). The precedent's whole point is that it lands later without
  touching a caller.
- Do NOT commit, push, stash or amend. Keep the git index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- The orchestrator runs the full floor and clippy centrally. Use scoped checks —
  `cargo nextest run --release -E 'test(...)'` and `./target/release/wat --check <file>` (~0.2s).

Build with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000 cargo build --release`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## Your report

The accessor semantics you settled on, and the four edge cases with what each returns. The 33
conversions, with any site that turned out NOT to be a name. Whether the rune could be drawn, and at
what tightness. Any STOP that fired, with the arm captured verbatim before you diagnosed it. What
surprised you — in particular, any two sites that disagreed about the same question, because that is
the finding this stone exists to surface.
