# FINDING — C-b: the transport slot has no declaration to read

**Measured 2026-09-24 against `main` @ `03e15f964`**, read-only. A C-a executor was live in the tree
at the time, and nothing was built. These are the measurements C-b's brief needs. Ruled order:
C-a → C-b → C-c (see `BRIEF-STONE-255.17a-the-child-says-wire.md`).

## What carries a transport slot today — always the LAST type argument, by convention

| carrier | declared at | how the slot is spelled |
|---|---|---|
| `:wat::kernel::Address :- [S R T]` | Rust-backed opaque (`src/types.rs` builtin roster, ~:3277 and ~:8267) | `T` |
| `:wat::spawn::Bound :- [S R T]` | `wat/spawn.wat:305` | `T`; comment: *"2-arg `(Bound :- [S R])` still means T unknown"* |
| `:wat::spawn::Launched :- [S R Sh Lu T]` | `wat/spawn.wat:319` | `T`; same 4-arg comment |
| every service's `<S>::Handle` and `<S>::Status` | emitted by `defservice` (`wat/service.wat:1002`, `:1107`) | `T`, **or `Xt` when the service already declares a `T`**: `transport-param (if binds-t? "Xt" "T")` (`:983`), chosen by spelling so the name does not collide |

The markers `:wat::kernel::Shared` and `:wat::kernel::Wire` are two empty `defstruct`s
(`wat/spawn.wat:297-298`). **Nothing declares that they form a family,** and nothing declares that any
parameter above is a transport slot.

## How the checker finds the slot — spelling and position

- `is_type_param_letter` (`src/check.rs` ~:10551): one uppercase letter or `Xt`. It has 8 call sites,
  and it doubles as "is a type parameter" (the C-c role).
- 5 transport sites key on "the last argument, spelled like a letter": `transport_edge_keys`,
  `satisfier_method_keys`, `is_transport_slot`, `transport_param_instantiates`, and the missing-slot arm
  (`aargs.len() ± 1 == eargs.len()`).
- `transport_satisfier_heads` (`src/types.rs:1625`) mints `(Head :- [:T])` and `(Head :- [:Xt])` keys
  by name.
- `is_shared_marker`/`is_wire_marker`: 6 call sites. The literal Shared/Wire strings appear 5 times in
  `src/` (the hardcoding step 2 of the locus plan names).

## The gap C-b must fill first — the language cannot say "this parameter is a transport"

**No type parameter in wat carries a bound or kind today.** A census for a `(X <- …)` form inside a
`:- [ … ]` list over every tracked `.wat` found 0. So "the type declares its transport slot" needs a
**declaration form that does not exist yet**. How it is spelled is a language decision, and it answers
to the locus ruling (a narrow waist: adding a locus or a transport is a language update with the least
friction). Candidate shapes to put through the four questions before the brief:

- A kinded parameter on the carrier (`:- [S R (T <- :wat::kernel::Transport)]`), with Shared/Wire
  declared members of a `Transport` family.
- One declaration beside the loci listing the transport-carrying types and their slot.
- Position stays last, but the family is declared (Shared/Wire as a closed `Transport` set), and the
  letter test becomes "a declared type parameter meeting a `Transport` member".

This is only enumeration. **None of these has been put through the four questions.**

## Addendum, same day — 255.17a measured the markers' declaration lies too

`Shared`/`Wire` are `defstruct`s, so to the purity wall they are **impure structs**, although
`spawn.wat:293-296` calls them *"phantom … type arguments only, not values."* The C-b declaration form
must therefore declare **the markers**, not only the slot: which parameter is a transport, and what a
transport marker **is** (a phantom type-level member of a closed family whose portability is
declared). See `SCORE-STONE-255.17a-the-child-says-wire.md`.
