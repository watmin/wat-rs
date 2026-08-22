# BRIEF — a `derive`-only marker is a TYPE NAME

**RULED A** (builder, 2026-08-22), tiebroken on the long-term narrow-waist assessment, not on
performance — see the ruling note at the end.

⚠ **You are working on a DIRTY tree.** The un-park stone's work is in the working tree, uncommitted
and unfloored: the type-reference wall plus the `defsurface` generics fix. **Do not revert, stash or
commit any of it.** Your change lands on top and the whole thing commits together once green.

## The work in one paragraph

`(:wat::core::derive :t::A :t::Marker)` calls `env.register_subtype(child, parent)` and **nothing
else** — so `:t::Marker` gets a lattice edge and no `TypeDef`, and the registry has never known it is
a type name. The new wall correctly reports it as unresolved, and is correctly wrong to. Make
`register_subtype` record both endpoints as **names with membership but no structure**, in the store
stone 255 built for exactly that category.

## The two failures this closes — reproduce them BEFORE you change anything

Both are pre-existing tests, red on the current tree:

```
wat::types    probe_arc237_derive_verb::derive_registers_marker_edge_usable_as_a_bound
              :path ":t::Marker" :context "type in the signature of :user::take-marker, parameter #1"

wat::services probe_arc209_spawned_marker::thread_handle_derives_the_spawned_marker
              :path ":wat::spawn::Spawned" :context "type in the signature of :user::take-spawned, parameter #1"
```

`tests/types/probe_arc237_derive_verb.wat` declares `:t::Marker` **nowhere** — only
`(:wat::core::derive :t::A :t::Marker)` — and then uses it as a parameter type. That is arc 237's
documented feature, and the stdlib does the same at `wat/spawn.wat:235-236` with
`:wat::spawn::Spawned`. **Run both and see them fail before you fix anything**, so you know your
change is what turns them.

## Read in order

1. `src/types.rs`, the `":wat::core::derive"` arm (~:3804) — `env.register_subtype(&child, &parent, …)`
   is its ONLY registration. That is the defect in one line.
2. `src/types.rs:716` — `register_subtype`. The single funnel: **six call sites**, `derive` and
   `extend-type` among them. This is where the fix goes, not in the `derive` arm.
3. `src/types.rs` — the `builtin_names` field and `register_builtin_leaf`, from stone 255. The store
   already models "membership without structure"; you are widening WHO writes to it, not adding a
   mechanism.
4. `wat/spawn.wat:235-246` — real stdlib derive-only markers (`Spawned`, and note `Peer` which IS
   declared). Your change must not disturb the declared ones.

## The shape

In `register_subtype`, record **both** `child` and `parent` as structureless names. Both, not just
the parent: `extend-type` and `derive` each take either side, and a name that appears in the lattice
is a name the language knows.

⚠ **A structured `TypeDef` must always win.** If a name is already in `types`, recording it as
structureless must not shadow, replace, or duplicate it — `contains` is an `||`, so membership is
already correct; make sure `get` is unaffected and no double-registration assert fires. Most edge
endpoints ARE declared types (`:wat::kernel::Thread`, `:wat::core::Record`); the markers are the
minority.

**Widen the field's name and doc** if `builtin_names` no longer says what it holds. It is no longer
"builtin leaf types" — it is *every name with membership and no structure*, from two producers now
and more later. Rename it to say that. This is the whole point of the ruling: **one store, many
producers, one door.**

## STOP triggers — ship nothing further and report

- **STOP-1 — if fixing `register_subtype` does not turn both tests green**, the marker's membership is
  not what the wall is missing. STOP and report what it is actually asking for.
- **STOP-2 — if recording an already-declared name breaks anything** (a double-registration assert, a
  `get` result, a `Nature` lookup), STOP. A structured type must be unaffected; if it is not, the
  store's contract is wrong and that is a finding.
- **STOP-3 — if the sweep now reports a THIRD class** — neither the nine phantoms, nor a builtin-name
  gap, nor a derive marker — STOP and report the full list before touching it. That list is worth
  more than this stone.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★ | `probe_arc237_derive_verb` | **red before, green after** — report both |
| 2★ | `probe_arc209_spawned_marker` | **red before, green after** — report both |
| 3★★ | the wall is still NOT blind | `tests/resolve/arc109_type_reference_must_resolve_row1_uncalled.wat` → still EXIT 1, still names `:user::NoSuchType` |
| 4★ | the innocent one-liner | still EXIT 0, zero output |
| 5 | a structured type is unaffected | `get(":wat::core::Struct")` still returns its `TypeDef` |
| 6 | scoped | `binary_id(wat::types)` · `binary_id(wat::resolve)` · `binary_id(wat::services)` green |
| 7 | clippy | 0 under `-D warnings` |

**Row 3 is the row that decides this stone, exactly as it decided the last one.** Teaching the
registry that lattice names are types is one small step from teaching it that *every* name is a type.
Rows 1 and 2 go green the moment the wall stops firing on anything; only row 3 proves it still fires
on a genuine phantom. Report rows 1-2 and row 3 **together**.

## Boundaries

- `src/types.rs` and tests. Nothing else.
- Do NOT touch the un-park stone's work already in the tree (`src/resolve/`, `src/freeze.rs`,
  `tests/resolve/`) except to run it.
- Do NOT touch `src/value/symbol_table.rs`. Its diff has stayed empty across two stones.
- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — I measure centrally, and the tree carries
  another stone's work, so the floor is mine to read.
- Do NOT commit, push, stash, revert or amend.

⚠ `no_loose_string_assert` has a known FALSE-POSITIVE class on `assert!(registry.contains("literal"))`.
It has cost two stones today. Do NOT add a `rune:lint(loose-assert)` — the site is not loose. Ask
through the door: `sym.registrations(name).contains(RegistryKind::Type)` takes an enum.

Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 1800`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## The ruling note — why A, and why not B

B was "have `contains` also consult `subtype_edges`" — nothing to populate, drift-proof by
construction. Both options scored four-for-four. **The tiebreak was the narrow waist, and it is a
question about shape, not speed:**

- **A — every producer writes to ONE membership store.** A future name-creating mechanism plugs in
  with one registration call. Many producers → one store → many consumers.
- **B — `contains` reads each producer's PRIVATE storage.** Membership's truth stays spread across N
  structures and every future mechanism costs `contains` another `||`. The consumer is made
  responsible for knowing every producer — **the waist inverted**, widening forever.

I had originally marked B's Good UX `?` on an unmeasured performance concern and let the question mark
decide. That is not an evaluation. B loses on shape whether it is fast or slow.

## Your report

Rows 1, 2 and 3 with verbatim output, stated together. The before-state of rows 1 and 2 (red) as well
as the after. What you renamed the field to and why. Whether any already-declared name misbehaved
when recorded. What surprised you. Anything you inspected and left alone.
