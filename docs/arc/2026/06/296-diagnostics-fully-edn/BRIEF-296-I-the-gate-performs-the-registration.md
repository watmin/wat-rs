# BRIEF — 296 I: the gate PERFORMS the registration

> Read `DESIGN-STONE-I-the-gate-performs-the-registration.md` first — it carries the why, the four
> questions, and the rung that looks like the fix and is not.

Baseline: HEAD `3a77a845`, floor **4421 / 4421 / 263 skipped**, clippy 0.

## THE WORK IN ONE PARAGRAPH

`crate::resolve::gate` returns a `Registration` verdict and **thirteen callers each decide what to do
with it** — twelve write the same three-arm match, one propagates it as `Err`, and further out at
least two more consume the verdict second-hand. Twice today the door said no and nothing stopped, in
two shapes that resemble each other not at all. Make the door **perform the registration**, so
inserting *is* passing through, then make `gate` private so asking-and-ignoring has no form.

## ⛔ STEP 1 IS THE CENSUS — make `gate` private FIRST and let rustc enumerate

**Do not grep for the call sites.** Change `pub fn gate` to `pub(crate) fn`/module-private and build.
Every outside caller becomes a compile error, and *that list is the worklist* — complete, by
construction, including the second-hand consumers a search would miss.

This matters because my own census of this exact question has already been wrong twice today: once
counting arm names instead of correct handling (it flagged `register_overlay`, which is in fact the
*strictest* caller), and once missing a whole class of consumer. The orientation below is orientation.
**rustc is the authority.**

What I know is there, offered so you recognise the shapes, not so you trust the list:

| file | fn |
|---|---|
| `types.rs:603` | `register_validated` |
| `check/env.rs:322` | `register_overlay` — **propagates** `Err(verdict)`; the strictest caller |
| `check.rs:8102` | `collect_splice_defs_ctx` |
| `runtime.rs:945`, `:984` | `register_defines` |
| `runtime.rs:1720` | `register_aggregate_methods` — gated only hours ago, in `77af7d71` |
| `runtime.rs:2619` | `register_defalias` |
| `runtime.rs:2814`, `:2882` | `preregister_struct_accessors_from_form` |
| `runtime.rs:3002` | `preregister_enum_constructors_from_form` |
| `runtime.rs:3539` | `preregister_fn_defs_in_do` |
| `runtime.rs:3619` | `preregister_fn_defs_in_let` |
| `runtime.rs:6947` | `parse_defclause_form` |

**Plus second-hand consumers** — code that matches a `Registration` it received rather than calling
`gate` itself. `src/macros/registry.rs:84-95` is one. `src/check/env.rs:158` is the other, and it is
the live bug: it takes `register_overlay`'s correctly-propagated `Err` and prints it —

```rust
// Loud on purpose while we learn what the corpus holds.
eprintln!("GATE-REJECT\t{path}\t{verdict:?}")
```

There may be more. The compiler will say.

## THE SHAPE

```rust
resolve::register(&name, privilege, existing, &span, || sym.register_function(name, f))?;
```

The door checks, then runs the insert closure. Callers `?`. No arms anywhere.

### Bridging the seam — measured, and the design must carry it

| axis | what is actually there |
|---|---|
| **registries** | `sym.register_function` (×4) · `self.types.insert` · `env.schemes.insert` · `env.register_defined_value_ast` · the macro registry · the defclause table |
| **error taxonomies** | `RuntimeError` · `TypeError` · `MacroError` · `CheckError` — `DottedName` already exists in all four |
| **span sources** | `form.span()` · `span.clone()` · `rust_caller_span!()` · and some sites have none |

So the insert arrives as a **closure**, the span as a **parameter**, and the rejection becomes a
`Rejection { verdict, name, span }` with `From<Rejection>` for each of the four error types. Then `?`
performs the taxonomy conversion at every site and no caller writes a match at all.

`Existing` stays the caller's to compute — it needs the registry the caller owns. Only the *decision*
and the *insert* move behind the door.

## ⛔ THE RUNG THAT IS NOT THIS STONE

**`#[must_use]` on `Registration` is not the fix.** It forces a caller to LOOK at the value, never to
ACT on it — and the disk already proves that insufficient: a `_ => {}` wildcard looks, and an
`eprintln!` looks. Both satisfy must-use. Both are the bug. If you find yourself reaching for it,
that is the signal the closure seam got hard — report the site instead
(`[[feedback_a_match_with_identical_arms_is_a_discard]]`).

## STOP TRIGGERS — rejections. Report and leave the site.

- **STOP-1 — a site cannot express its insert as a closure**, typically a borrow conflict: the
  registry is borrowed to compute `existing` and again inside the closure. Report it with its
  `file:line`. **Do not leave that one site on the old `gate` as an exception** — an exception is the
  hole reopening with a rationale attached, and it would make `gate` public again for one caller,
  which defeats the whole stone.
- **STOP-2 — a site has no span.** `rust_caller_span!()` is the honest answer and is already used in
  these files. Report anything that fits neither that nor a real form span; do not fabricate one.
- **STOP-3 — privatising `gate` breaks a caller you cannot classify** as either a registrar or a
  second-hand consumer. That is a shape the design did not anticipate and it is the finding.
- **STOP-4 — the floor moves.** This is a refactor, not a behaviour change: every rejection that fired
  before must still fire, and none that did not must start. A moved count means a site's semantics
  changed in translation. Capture it whole before adjusting anything.

## WHAT CHANGES BEHAVIOUR — exactly one thing, and name it in your report

`check/env.rs:158`'s `eprintln!` becomes a real error. **That is the point of the stone**, and it is
the one place where the floor legitimately might move: if any corpus form currently reaches that
swallow, it has been silently accepted and will now be refused. If that reds a test, the red is the
finding — capture it verbatim, name the form, and report. Do not restore the swallow to get green.

## BLAST RADIUS

`src/resolve/` (the new `register`, the `Rejection` type, the four `From` impls, `gate` privatised),
plus every site rustc names. No `.wat` corpus changes. **Do not change what the gate rejects** —
`has_dotted_name`, `Reserved`, `Unnamespaced`, and the idempotent-before-reserved ordering are all
correct and stay byte-for-byte. This stone changes only *who may ignore them*.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D
warnings` (0), then `scripts/floor.sh` and read the **Summary line** — never a piped exit code.
Baseline **4421 / 0 / 263**; expect it unchanged except for anything the `check/env.rs` swallow was
hiding, which you name explicitly.

**On any red: do NOT re-run.** A re-run that goes green destroys the only evidence. Copy the failing
test's whole stdout+stderr block verbatim — never a `| head` window — name the exact assertion that
fired, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Run
every build and test in the FOREGROUND and block on it. Anchor at `/home/watmin/work/holon/wat-rs`;
`pwd` first. Leave the work uncommitted; the orchestrator weighs and commits.

Report: the rustc-produced worklist (not my table), the seam decisions you had to make, whether the
`check/env.rs` swallow was hiding anything, the floor Summary line verbatim, every STOP, and the
honest deltas. Every rider on this arc has found a defect in the orchestrator's brief; this one
describes a class the orchestrator missed twice in one day, so read it with that in mind.
