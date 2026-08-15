# BRIEF — 296 H-1: a dot in a type name has NO FORM

> Read `DESIGN-STONE-H-variants-are-maps.md` first. This is H's **first** strike and the only one
> being released now.

## WHY THIS IS ITS OWN STRIKE

Stone H makes **a dot in the tag's name half mean "variant"**:
`#wat.telemetry/Numeric.I64 {:val 42}`. That discriminator is only real if a *record* cannot produce
a dotted name.

Measured 2026-08-15: no type name in the corpus contains a dot, and `Tag::try_ns` validates only the
name's **first** character — a dot inside is legal and would be accepted. **So the property holds by
luck.** A census is the weakest rung of the ladder; this strike moves it to the top one, where the
forgery has no form.

Builder: *"records must disallow dots in names... authoritatively."*

Landing it alone is deliberate. It is **observationally inert** (nothing in the corpus violates it),
so it lands green on its own — and once it is green, every red in H-2's tag/body flip is about the
tag or the body, never about the ban. Bundled, a red would be ambiguous between them.

## THE WORK

**One door, and it already exists.** `src/types.rs:~598` routes every registration through
`crate::resolve::gate(&name, privilege, existing)`, which returns a `Registration` and is matched at
`:604-614`. That match already carries a sibling rejection — `Registration::Unnamespaced` →
`TypeErrorKind::UnnamespacedName`. Add the dot rejection the same way:

- a new `Registration` variant (e.g. `DottedName`) decided inside `resolve::gate`
- a new `TypeErrorKind` variant carrying the offending name
- the arm in `types.rs`'s match, mirroring `Unnamespaced`

**The rule:** the type **name** — the segment *after the last `::`* — may not contain `.`. The
namespace half is untouched; `:wat::core::Fault` is namespace `wat::core`, name `Fault`, and only
`Fault` is checked. (Dots appear in the *dotted* form only later, in `tag_from_type_path`; at
registration the name is still `::`-separated.)

The diagnostic should teach, per R29 — say that a dotted name would forge a variant tag
(`#ns/Enum.Variant`), which is why the substrate reserves the dot.

## THE GATE — non-vacuous, with a positive control

A refusal test that never proves the *acceptance* side can pass while the checker rejects everything.
Assert **both**, structurally on the error kind (not on message text — `no_loose_string_assert` is
armed and has fired on this arc twice):

1. **negative** — declaring a record whose name contains a dot is REFUSED, and the error is the new
   kind
2. **positive control** — declaring an ordinary undotted record in the same test still SUCCEEDS

Without row 2 the test cannot tell "the wall works" from "registration is broken."

## STOP TRIGGERS — rejections. Report; do not improvise.

- **STOP-1 — a real type in the corpus has a dotted name.** The census says none does. If the wall
  goes red on live code, **the wall is right and that name is the finding** — report it with its
  `file:line`. Do not widen the rule to admit it, and do not rename the type on your own authority.
- **STOP-2 — the check cannot be expressed at `resolve::gate`** because the name arrives already
  transformed, or the gate lacks the span. Report where the honest door actually is. Do not scatter
  the check across call sites; the whole point is one door.
- **STOP-3 — the floor moves.** This strike is observationally inert. A changed test count means the
  ban caught something real, which is STOP-1, or the rule is over-broad. Either way, report before
  adjusting anything.

## BLAST RADIUS

`src/resolve.rs`, `src/types.rs`, the `TypeErrorKind` enum and whatever exhaustive matches the
compiler names downstream of a new variant, plus one new probe. **No `.wat` corpus changes** — nothing
violates the rule. Do not touch `tag_from_type_path`, `enum_variant_ns`, the writer, the reader, or
any enum rendering: that is H-2 and it is not in scope here.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D
warnings` (0), then `scripts/floor.sh` and read the **Summary line** — never a piped exit code.
Baseline **4417 passed / 0 failed / 263 skipped**; expect **4417 + your new probe**, nothing else moved.

**On any red: do NOT re-run.** A re-run that goes green destroys the only evidence. Copy the failing
test's whole stdout+stderr block verbatim — never a `| head` window — name the exact assertion that
fired, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Run
every build and test in the FOREGROUND and block on it. Anchor at `/home/watmin/work/holon/wat-rs`;
`pwd` first. Leave the work uncommitted; the orchestrator weighs and commits.

Report: where the check landed and why that door, the probe's two rows, the floor Summary line
verbatim, every STOP, and the honest deltas — especially anywhere this brief did not match the disk.
Each of the three riders before you on this arc found a defect in the orchestrator's own brief. That
is the bar.
