# BRIEF — ③b-i: `register_variant` is its own door

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`; a path containing `.claude/worktrees/`
is harness state — use `git -C /home/john/work/holon/wat-rs` for git.

Read `DESIGN-the-variant-name-is-COMPOSED-never-TYPED.md` (this brief's sibling) first — it carries
the ruling and why the option before it was retracted.

## The work, in one paragraph

`src/resolve/registration.rs` has one gate with one rule table. Its `DottedName` arm (arc 296 stone
H-1) refuses any declared name whose leaf carries a `.`, and that ban is what makes the EDN wire
discriminator sound — a dot in the name half means *variant*, and no record can forge one. An enum's
variant constructor path is composed by the substrate through `identifier::compose_variant`, and
when the variant separator becomes `.` that composed name would hit this wall and every stdlib
variant would fail to register. This stone gives the composed path **its own entry point** so the
wall never has to weaken: `register` keeps refusing a dot in a name a caller typed;
`register_variant` takes the two halves separately and accepts the dot because it wrote it.

**Nothing observable changes.** `compose_variant` writes `::` today, so a composed name has no dot
and the arm being origin-gated cannot currently fire.

## The rooms — read in order

`src/resolve/registration.rs` — `has_dotted_name` (~:92), the `gate` rule table and its doc
(~:100–145), `register` (~:178–192), and the in-file `mod tests` (~:196+).

### ① `NameOrigin`

Add beside `Privilege`, with the same shape of doc: `Privilege` says WHO made the name;
`NameOrigin` says HOW IT WAS MADE.

```rust
pub(crate) enum NameOrigin {
    /// A name a caller TYPED — a def form's own name. A name-half dot is refused.
    Declared,
    /// A name the grammar's own `compose_variant` built from (parent, leaf). Its dot,
    /// when the variant separator becomes one, is the composer's, not a caller's.
    ComposedVariant,
}
```

### ② The rule table gains ONE condition

`gate` takes `origin: NameOrigin`. Exactly one arm changes:

```rust
} else if origin == NameOrigin::Declared && has_dotted_name(name) {
    Registration::DottedName
```

Every other arm is untouched — `Duplicate`, `Unnamespaced`, and `Reserved` all still apply to a
composed variant. Update `gate`'s doc-comment rule table to show the new condition on that row, and
extend the paragraph that explains why `DottedName` is a wall with what the origin split preserves.

### ③ `register_variant`

A sibling of `register`, same generic shape (`insert: impl FnOnce() -> Result<T, E>`, `E: From<Rejection>`):

```rust
pub fn register_variant<T, E>(
    parent: &str,
    variant_leaf: &str,
    privilege: Privilege,
    existing: Existing,
    span: &Span,
    insert: impl FnOnce() -> Result<T, E>,
) -> Result<Option<T>, E>
where E: From<Rejection>
```

It composes `wat_reader::identifier::compose_variant(parent, variant_leaf)` internally and calls
`gate(&name, NameOrigin::ComposedVariant, privilege, existing)`. `register` keeps its current
signature and passes `NameOrigin::Declared`.

⚠ It takes the halves **separately on purpose** — a caller must not be able to hand this door a
string it built itself. That is the whole ruling.

### ④ The three call sites — each already holds both halves

```
src/declare/preregister.rs:314   compose_variant(type_name, variant_name)          ctor path
src/declare/register.rs:1341     compose_variant(&enum_def.name, variant_name)     unit variant
src/declare/register.rs:1371     compose_variant(&enum_def.name, variant_name)     tagged variant
```

Only `preregister.rs:314` currently feeds `resolve::register`; route that one through
`register_variant`. The two in `register.rs` compose a key for `sym.has_*` lookups rather than for
the gate — **leave their `compose_variant` calls exactly as they are**, and say so in your report.

## The rows this stone owes

In `registration.rs`'s `mod tests`:

- the two existing H-1 rows (`dotted_name_from_user_is_rejected`, `dotted_name_from_stdlib_is_still_rejected`)
  keep asserting `DottedName` — they are `Declared`. **If either moves, that is STOP-1.**
- `gate(":my::Shape.Circle", ComposedVariant, User, Absent) == Insert` — the new arm, non-vacuous.
- a composed leaf with TWO dots is still refused — D is not "any dot is fine".
- `Reserved` still fires for `ComposedVariant` + `Privilege::User` on a `:wat::*` name.

As a probe under `tests/resolve/` (mirror a sibling there for shape) — **the builder's question,
which nothing in the tree pins today**: a `defn` and an enum variant that want the same name
collide, in BOTH definition orders, and the collision is caught at `--check`:

```
defn :my::app::Foo::Bar   then  defenum :my::app::Foo :Bar
defenum :my::app::Foo :Bar  then  defn :my::app::Foo::Bar
      -> both refuse; `duplicate define: :my::app::Foo::Bar already registered`
```

Plus one row that a `defn` with a dotted name is still refused end to end, so the wall is pinned
from the surface and not only at the gate.

⚠ House lints apply to new probes: no loose `contains`/`starts_with` assertions where an exact
comparison belongs (`tests/lint/no_loose_string_assert.rs`), and no inlined EDN — a structured
value goes in a co-located `.edn` golden.

## Blast radius

`src/resolve/registration.rs`, `src/declare/preregister.rs`, and one new probe file. **No behaviour
change anywhere.** Do not touch `compose_variant`/`decompose_variant`, the `display` runes, or any
`.wat` file — the flip is a later stone.

## STOP triggers — each REJECTS the stone; ship nothing further and report

**STOP-1.** Any existing test expectation moves. This stone is behaviour-preserving by
construction; a moved expectation means it is not, and the input that moved it is the deliverable.
Report the test, the arm that fired, and its whole verbatim stdout+stderr block.

**STOP-2.** The collision probe does NOT produce `duplicate define` in one or both orders. That
contradicts a measurement taken on this tree at `7ccce48ba` and is worth more than the rest of the
stone — report both orders' verbatim output.

**STOP-3.** A fourth call site composes a variant name for the gate that this brief does not list.
Report it; do not route it. The census is the orchestrator's to close.

**STOP-4.** `register_variant` cannot take the halves separately — a caller has only the composed
string in hand. Report it: that would mean the ruling's premise is wrong, not that the signature
should change.

## What to run

`cargo build --release`, and the scoped rows you added:
`cargo nextest run --release -E 'binary_id(wat::resolve)'` plus the in-file gate tests via
`cargo nextest run --release -E 'test(registration)'`. **Do not run `scripts/floor.sh` and do not
run clippy** — the orchestrator measures those centrally, once, on a quiescent tree. Run every
command in the FOREGROUND and block on it. Do not commit. Do not contact any peer.

## Report

The diff per room; the verbatim output of both collision orders; confirmation that the two
existing H-1 rows still read `DottedName`; and anything that surprised you.
