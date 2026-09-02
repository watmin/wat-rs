# BRIEF — STONE 1a-β-i: the type-declaration family joins the registry

Register the five type-declaration forms — `defstruct`, `structtype`, `defenum`, `newtype`,
`typealias` — each naming its own declare-time parser, and build the meter that will license the
next stone to delete a hand-list: a bidirectional gate over `is_liftable_declaration_head`'s ACTUAL
domain, read from its source, not transcribed.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1a-beta-i-the-type-declaration-family.md`
— **read its ⛔ AMENDED block first**; the design was corrected after a homonym surfaced.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. Run every command you do run in
the FOREGROUND and block on it. The orchestrator builds, floors and clippies centrally — you do not
run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`. You may run the pre-existing
`./target/release/wat` and `--check` for a fast read. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything. Tree clean, floor green at 5123.

## Read in order

1. **The DESIGN above**, ⛔ AMENDED block included.
2. **`src/intrinsic/special/defsurface.rs`** — **the template.** It is the only existing
   `Declaration`-category, `Unevaluated`-purity row, and your five share every axis verdict with it.
   Read its five grounds closely; yours cite it where the argument is genuinely the same, and argue
   their own where it is not.
3. **`src/types.rs`'s `parse_type_decl`** — the router, and the proof each form has its own parser:
   ```
   parse_defstruct   src/types/defstruct.rs:520     parse_defenum     src/types.rs:4174
   parse_structtype  src/types.rs:4555              parse_newtype     src/types.rs:4370
                                                    parse_typealias   src/types.rs:4422
   ```
4. **`src/freeze.rs:1951`** — `is_liftable_declaration_head`, the meter's domain. Read its nine arms
   and its ⛔ RENAMED block.
5. **`src/intrinsic/mod.rs:2674-2700`** — `registry_first_door_owns_every_handler_row…`. ★ **This is
   the pattern for reading a predicate's domain without transcribing it**: `include_str!` the source
   file, find the fn, take its span, extract from the text. Copy that shape.
6. **`src/types/surface.rs:532`** — how `parse_defsurface` carries its `role = declare` attribute,
   including the import.

## The work

### 1 — five doc-only structs

One per form, under `src/intrinsic/special/`, each with its `mod` line. Every row:

```
@Category    Declaration
@Purity      Unevaluated     ← measured: each appears in runtime.rs exactly ONCE, inside
                               is_mutation_head — a hand-list, NOT a dispatch arm. No eval
                               arm, no tail arm, no handler. Verify this yourself per form.
@Determinism Deterministic
@Totality    Partial
@ExpandTime  RuntimeOnly
@added, prose, @syntax, @ret
```

⚠ **A shared verdict is not a shared ground.** Each row argues its own case; cite `defsurface`'s
ground where the argument is identical rather than retyping it, and say plainly where it differs.

★ **`@syntax` must be FQDN-headed and REAL.** Derive each from its parser's accepted shape — the
parsers state their grammar in comments (e.g. `structtype`'s at `src/types.rs:4555`) — and
**verify each by `--check`ing a concrete instantiation**, the way 1a-β-0's rider verified
`defsurface`'s. Report the grammar and where you verified it.

### 2 — five `role = declare` annotations

`#[wat_special_form_impl(":wat::core::<form>", role = declare)]` on each form's own parser. Stacking
several on one fn is proven precedent (`src/check.rs:15553`), but you should not need it — each form
has its own.

### 3 — the meter

A bidirectional gate in `src/intrinsic/mod.rs`:

```
name in is_liftable_declaration_head  ∧  no Declare impl   →  MISSING
Declare impl                          ∧  name not in it    →  FOREIGN
```

⛔ **The domain is read from `src/freeze.rs`'s source, never transcribed.** A frozen copy of the nine
would be a sixth hand-list — the one joke this campaign cannot make. Use the `include_str!` + fn-span
pattern from room 5.

`MISSING` should print as a worklist and is expected to be **3** after this stone
(`def`·`defmacro`·`defalias`). Assert `FOREIGN` is empty; assert `MISSING` is exactly those three, by
name, so the number can only fall by registering.

## Blast radius

`src/intrinsic/special/` (+5 files, +5 mod lines) · `src/types.rs` · `src/types/defstruct.rs`
(5 attribute lines, + the import where absent) · `src/intrinsic/mod.rs` (one gate). Nothing else.
No `.wat` corpus change. No consumer flipped, no hand-list edited.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — the meter's domain is `freeze.rs`'s source, not a list you typed.** If you find
yourself writing the nine names into `src/intrinsic/mod.rs` as a `const`, stop: that is the defect
this whole campaign exists to remove, committed inside its own meter.

**⛔ STOP-2 — verify `@Purity Unevaluated` per form, do not inherit it.** For each of the five,
confirm it has no `handler`, no eval arm and no tail arm before declaring it. If ANY of the five
turns out to have a real dispatch arm, that one is not `Unevaluated` — STOP and report it rather
than declaring the shared verdict.

**⛔ STOP-3 — do not touch any hand-list.** Not `is_liftable_declaration_head`, not
`is_mutation_head`/`is_mutation_form`, not `DECLARATION_HEADS`. MISSING is 3, not 0; flipping or
editing a consumer now would be a measured lie.

**⛔ STOP-4 — every `@syntax` is FQDN-headed.** wat is FQDN, always: anything that is not a binder is
illegal, and bound symbols are shadow-FQDN in `$bound`.

**⛔ STOP-5 — annotate each form's OWN parser.** If a form's honest declare-time target is ambiguous,
STOP and report rather than annotating the router (`parse_type_decl`) or a neighbour. ⚠ Stone 1a-β-0
shipped exactly this defect: it named `synthesize_surface_protocol` (a conditional secondary pass)
and missed `parse_defsurface` (the primary), because its census grepped for the FQDN string — and
these parsers **never spell their FQDN**, since the router dispatches them on the stripped leaf.
**Do not census by string here.**

**STOP-6 — verbatim otherwise.** No signature tidying, no opportunistic cleanup.

## Sabotage — report each as "predicted red, unverified"

1. drop one form's `role = declare` → what does MISSING say?
2. annotate `:wat::string::declare-acronyms` with `role = declare` → what does FOREIGN say?
3. add a tenth arm to `is_liftable_declaration_head` → does the meter's domain grow with it?
   (proves STOP-1 — the domain is the source, not a copy)

## Report

**All five doc structs verbatim** — they are the stone's centre · the five annotations and their
targets, with how you confirmed each is the form's own parser (**not by grepping the FQDN**) · the
meter verbatim, including how it reads the domain · the `@syntax` for each form and the concrete
instantiation you `--check`ed · MISSING before and after · the three sabotage predictions · and what
surprised you.

## Prior comparable

`BRIEF-STONE-1a-beta-0b-a-form-that-never-evaluates.md` — same report shape.
`src/intrinsic/special/defsurface.rs` is the row-authoring standard.
