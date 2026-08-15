# NOTE (arc 255) — a `:restricted-to` cannot be verified to name anything, so an unenforceable capability declaration is undetectable

**Filed 2026-08-15. A POINTER, not a decision.** Surfaced while briefing arc 198's **W1** wall (*an
unenforceable restriction fails at startup*), which turned out to be **unbuildable today**. Parked
here because the missing mechanism is 255's, not 198's, and because 255 already has it designed. This
note records the grounded flaw, why both available answers are wrong, what a fix must carry, and the
unmeasured blast radius — so whoever picks it up does not re-derive them.

## The flaw, in one line

**A capability whitelist can be registered under a name that does not exist, and there is no queryable
answer to "does this name exist?" — so the declaration is decorative, silent, and undetectable.**

## Why this matters more than a typo

This is the failure mode arc 198 just spent a day on, one level out. `{:restricted-to […]}` on a dead
key parses, registers, and is consulted by nobody. Nothing errors. **That is exactly how the
mention-position hole survived 44 days** — a capability that says it is guarded and is not. W1 exists
to make that impossible, and W1 cannot be built.

## Grounded (2026-08-15, read first-hand + `target/release/wat`)

`src/resolve/walk.rs:257` — the blanket-accept is **still live**:

```rust
if is_reserved_prefix(head) { … }
```

`BuiltinRegistry` — **does not exist anywhere in `src/`** (grep, this session).

All five Rust-side declarations target `:wat::`-prefixed names:

| site | `wat_name` |
|---|---|
| `src/io.rs:1275` | `:wat::io::IOWriter/from-fd` |
| `src/io.rs:1315` | `:wat::io::IOReader/from-fd` |
| `src/kernel/spawn.rs:452` | `:wat::kernel::spawn-thread` |
| `src/kernel/spawn.rs:524` | `:wat::kernel::spawn-process` |
| `src/runtime.rs:26993` | `:wat::kernel::close` |

Each `wat_name` is a hand-typed `&'static str` (`src/restriction_entry.rs`, via the
`#[restricted_to(…)]` proc macro at `crates/wat-macros/src/lib.rs:312`). **Nothing checks it names a
real binding. A typo is invisible.**

## ★ The two available answers are wrong in OPPOSITE directions

This is the whole reason the wall cannot be built, and it is not a checker limitation — it is the
asymmetry 255 is named after:

| ask | answer on the five risky sites | why |
|---|---|---|
| `resolve` | **"live"** — always | the reserved-prefix blanket-accept swallows every `:wat::` name, so the wall is **vacuous exactly where the risk is**. A wall that cannot fail is not a wall. |
| `sym.functions_iter()` | **"orphan"** — all five | builtins are registered **nowhere** (a 454-arm compile-time `match`), so every real builtin reads as a dead name. Five false positives. |

**There is no third source of truth today.**

## Why 255 owns it — the design already names the cure

`255-builtin-registry/DESIGN.md`, verbatim:

> *"Rust builtins … registered **nowhere** — a 454-arm compile-time `match`. resolve asks:
> **(can't)** → reserved-prefix blanket-accept."*

> *"**resolve**: membership → 'is this defined?' (the `+'2` bug, gone). The reserved-prefix
> blanket-accept hack is **deleted**; builtins resolve through the same path as user forms
> (registry/`sym` membership). **One resolution path for everything.**"*

W1 needs precisely that **membership** half. Not arity, not type signatures, not reflection — just a
uniform, honest answer to *is this name live?* that covers builtins.

## ★ What a fix must carry (each verified against the disk this session)

A naive "the key must be in `sym.functions`" predicate is **wrong on its face** — a `:restricted-to`
key legitimately names several kinds of thing:

1. **a registered function** — `sym.functions_iter()`
2. **an aggregate type name** — `src/runtime.rs:1453`
3. **a synthesized companion** — `T'` (positional prime ctor) and `is-T?` (membership predicate),
   both keyed since arc 198 strike 2 (`8f0e3939`)
4. **a field accessor** `T/field` — `src/runtime.rs:1460`
5. **a builtin** — the kind that has no registry, which is this note

**And it must NOT be a name-shape test.** Pattern-matching `ends_with("'")` / `is-…?` / `T/…` is the
**B3 forgery** arc 198 explicitly ruled out: a user-authored fn named `:my::Token'` would inherit
whatever the pattern grants. Ask the registry whether the key is live; never ask a string what it
looks like.

## Blast radius — UNMEASURED, and do not guess

Nobody has counted how many `:restricted-to` keys currently resolve to nothing, **including the author
of this note**. A census was dispatched 2026-08-15 and its result is not in this file; when it lands it
belongs here as an addendum. The cheap sizing instrument is the census itself — enumerate every key in
`binding_metadata` carrying a `:restricted-to` and report, per key, which mechanism (if any) can
confirm it is live. **Run it before committing to the stone.**

## Why it is parked here rather than acted on

Minting a membership door inside a 198 side-strike would **pre-empt 255's entry-shape**, which its own
DESIGN reserves: *"The entry-shape (DAY ONE) — this is what we shape together before code."* A
one-off resolution path built for W1 would also be a **second** resolution path in the arc whose
entire thesis is ONE — the same instinct 109's note already retracted once (*"a rete-only patch would
be a second resolution path in an arc whose whole thesis is ONE"*).

## ★ What this adds to 255's case — a THIRD consumer, and the first security one

255's cost has been justified by the undefined-func class and by reflection. It now has three
distinct consumers waiting on the same membership half:

- **the undefined-func class** (`+'2`, `make-*-queue`, `Bogus`) — 255's own framing
- **the annotation-position gap** — `109/NOTE-type-annotation-names-unchecked.md`, whose SIBLING FLAW
  block points at this exact line of code
- **W1** — *this note.* A **capability-enforcement** wall, blocked on the same door

Plus the **8 currently-`#[ignore]`d tests** whose reason strings name 255 directly (`metadata-of`
reflection ×6, checker rejection of undefined builtins ×2, and one banked leniency gate).

## Kin

- `docs/arc/2026/04/109-kill-std/NOTE-type-annotation-names-unchecked.md` — the **sibling**, same root
  (`is_reserved_prefix` blanket-accept), different consumer (type annotations, not capability keys).
  Its SIBLING FLAW block already routes to 255. **Do not merge the two**: that note is about a name in
  a *type annotation*; this one is about a name in a *capability declaration*, and the exemption sets
  are different.
- `docs/arc/2026/05/198-defn-restricted/DESIGN-STONE-a-restriction-governs-mention-not-head-position.md`
  — W1's home, and the ⚖ RULING that forbids the name-shape shortcut (B3).
- `docs/arc/2026/05/198-defn-restricted/BRIEF-198-W1-an-unenforceable-restriction-fails-at-startup.md`
  — the brief whose load-bearing half this note blocks. **It instructs the rider to "ask the registry
  whether the key is live"; there is no registry to ask.** That defect is recorded here rather than
  silently patched.
