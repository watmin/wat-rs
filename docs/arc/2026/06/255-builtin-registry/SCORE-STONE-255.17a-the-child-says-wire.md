# SCORE — STONE 255.17a: the child says `Wire` — STOPPED at STOP-1; the brief was wrong

**Nothing landed.** The executor kept the attempted change as `scratchpad/255.17a-C-a.patch` and
reverted it. Weighed by the orchestrator against disk on 2026-09-24.

## What happened

- **The site is exactly one.** The generated `:user::main` was expanded at the form level
  (`wat-scripts/scratch-pad/255-17a-child-main-transport.wat`, committed as the measurement). Only
  `status-ty-runtime` (`wat/service.wat:1108`, used at `:2432`) carries the transport letter.
  `admin-ty-runtime` carries only the service's own parameters. With the change applied, the diff of
  the expansion is exactly two tokens.
- **C-a alone turns the same 57 tests red, at an earlier check.** Floor `.floor/2026-09-24T03-48-37Z`:
  `6027 tests run: 5970 passed, 57 failed`. Every block fails on one arm: the self-peer **§7 purity
  wall**, *"type `(:my::counter::Status :- [:wat::kernel::Wire])` is not pure"*, raised in the forked
  child at startup.

## ⛔ Where the orchestrator's brief was wrong

The brief said: *"The child is a process, so its transport **is** `Wire`. Saying so is the truth, not a
workaround."* The code refuses it, and the reason is the same root one layer over:

- `:wat::kernel::Wire` is declared `(:wat::core::defstruct :wat::kernel::Wire [])` (`wat/spawn.wat:298`).
  To the checker a struct is **impure**, so `(Status :- [… Wire])` fails purity through the
  parametric fallthrough (`args.iter().all(is_pure_type)`).
- Its own comment (`:293-296`) says the opposite of that declaration: *"phantom transport markers …
  Type arguments only. Not values."* **The declaration lies about what Wire is.**
- The free letter passed only because an **unregistered** name hits `is_pure_type`'s
  `None => true` arm: *"unknown path ⇒ a formal type parameter ⇒ portable by convention"*. That is
  **the spelling root a third time**, on the purity side. The checker assumes an unknown name is a
  type parameter.
- `Address` passes because it is special-cased **by head name** (`Address :- [_ _ Wire]` is pure).
  `Status` has no such arm.

The brief took "the truth" from the runtime meaning and never asked what the checker's declaration says.
**C-a is not independent of C-b.** Saying `Wire` needs the transport markers **declared as what they
are**: phantom type-level markers of a closed transport family, pure or impure by declaration rather
than by a head-name arm.

## Consequence for the ruled order

C-a folds into C-b. The child main saying `Wire` becomes one row of C-b, which now has three sites
fixed by one declaration:

1. the transport slot is declared (the 5 transport sites);
2. the markers are declared as markers, so purity reads the declaration (the `Address` name-arm and
   the struct-impure misreading both go);
3. the child main says `Wire`, and D2 closes (the parked 255.17 patch plus tests).

C-c (a type parameter is known from its declaration) now also covers `is_pure_type`'s
`None => true` "unknown ⇒ type parameter" arm.
