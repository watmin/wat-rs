# BRIEF — STONE 1a-β-0: the third regime gets its name

Add `SpecialFormRole::Declare` — the freeze/declare-time regime the role vocabulary cannot currently
name — teach the macro to accept it, make the registration gate demand it of declaration forms by
DERIVING that from the row, and prove the whole thing by registering the one form that can only be
registered this way: `:wat::core::defsurface`.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1a-beta-0-the-third-regime-gets-its-name.md`.
The blocker it answers: `NOTE-a-declaration-form-is-a-THIRD-regime-the-role-vocabulary-cannot-name.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. Run every command you do run in
the FOREGROUND and block on it. The orchestrator builds, floors and clippies centrally — you do not
run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`. You may run the pre-existing
`./target/release/wat` and `--check` for a fast read. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything. Tree clean, floor green at 5120.

## Read in order

1. **The DESIGN above** — the contract decision is pinned there and it is the whole stone.
2. **`src/intrinsic/mod.rs:328-350`** — `SpecialFormRole` and its `label()`. Read the doc comments on
   all three existing variants; they state each regime in one line and yours must match that voice.
3. **`crates/wat-macros/src/wat_special_form_impl.rs:53-66`** (`role_variant`) and **`:93-165`** —
   where `eval`/`tail` emit shims and `check` deliberately does not. **The file's own comment at
   `:100-106` is the argument for your contract**; read it before writing a line.
4. **`src/intrinsic/mod.rs:2390`** (`every_special_form_carries_check_and_eval_impls`) — the gate you
   are rewriting. Its current shape is two `any()` calls and one assert; keep that shape.
5. **`src/intrinsic/reflect.rs:324-340`** — `show-source`'s `role_order` and its header string. The
   comment there already states WHY the order is semantic; extend that reasoning by one.
6. **`src/types.rs:3058`** — `synthesize_surface_protocol`, `defsurface`'s declare-time
   implementation and your annotation target.
7. **`src/intrinsic/special/and_form.rs`** — the doc-only-struct template. Its five axis blocks are
   the standard: each is a RULING with grounds, not a label.
8. **`src/intrinsic/special/binding.rs:28`** — an `@syntax` line, for the FQDN-headed form.

## The work

### 1 — the variant

`SpecialFormRole::Declare`, with a doc line in the voice of its three siblings, plus its `label()`
arm returning `"declare"`.

### 2 — the macro accepts `role = declare`

One arm in `role_variant`. **Widen all three message strings** that enumerate the legal roles
(`:33`, `:40`, `:61`) — a stale list in an error message is a doc that lies at the exact moment
someone needs it.

⛔ **Emit NO shim and NO handler field.** `declare` behaves exactly like `check`: source text only.

### 3 — the gate derives its demand

```
entry.category == wat_doc::Category::Declaration   ⇒  must name a Declare impl
otherwise                                          ⇒  must name Check and Eval   (unchanged)
```

Not exclusive-or: a `Declaration` row may also carry Check/Eval and several will. The rule ADDS a
demand for those rows and removes none from any other. Keep the failure message naming the offender
and the missing role, as it does today.

### 4 — `show-source` renders the new role FIRST

`role_order` becomes `declare → check → eval → tail`, and the header string widens to match. The
order is semantic: declare runs at freeze, check runs once statically, eval/tail are the
mutually-exclusive per-invocation regimes.

### 5 — the witness: register `:wat::core::defsurface`

A new doc-only struct under `src/intrinsic/special/` (add its `mod` line to that directory's
`mod.rs`), plus `#[wat_special_form_impl(":wat::core::defsurface", role = declare)]` on
`synthesize_surface_protocol`.

The row needs prose, `@added`, `@Category Declaration`, and a **grounded ruling on each of the five
axes** — Purity, Determinism, Totality, ExpandTime — in `and_form.rs`'s shape. Derive each from what
`defsurface` actually does (it declares a surface protocol into the type registry at freeze time);
do not transcribe another form's grounds.

★ **The substrate already states this form's grammar** — its own malformed-declaration error reads
`(:wat::core::defsurface :Name :nature :<nature-root> :features [members])`. Use it (verify it
against `types.rs` first), FQDN head included.

## Blast radius

`src/intrinsic/mod.rs` · `crates/wat-macros/src/wat_special_form_impl.rs` ·
`src/intrinsic/reflect.rs` · `src/intrinsic/special/mod.rs` + one new file there · one attribute
line in `src/types.rs`. Nothing else. No `.wat` corpus change.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — `role = declare` emits SOURCE TEXT ONLY.** No shim, no `NativeHandler`, no
`TailHandler`, no handler field on the submission. A freeze-time processor has no calling convention
to point at, and fabricating one would make the registry assert a door that does not exist.

**⛔ STOP-2 — the gate's demand is DERIVED from `entry.category`. No name list, no exemption list, no
`matches!` on FQDNs.** And it must not weaken the existing demand: after your change the six
expression forms must still be required to name Check and Eval.

**⛔ STOP-3 — register `defsurface` and nothing else.** The other eight declaration forms are 1a-β.
A second row here makes the witness un-isolated.

**⛔ STOP-4 — do not touch the five hand-lists** (`runtime::is_mutation_head`,
`freeze::is_mutation_form`, `freeze::is_declaration_form`, `declare::DECLARATION_HEADS`,
`declare::RUNTIME_DECLARATION_HEADS`). With one of nine names registered, a registry query is NOT yet
equivalent to any of them; flipping one would ship a measured lie.

**⛔ STOP-5 — every `@syntax` you write is FQDN-headed.** wat is FQDN, always: anything that is not a
binder is illegal, and bound symbols are shadow-FQDN in `$bound`. A short head is not a rendering
style, it is not-wat.

**⛔ STOP-6 — if `synthesize_surface_protocol` is not the honest annotation target, STOP.** Do not
annotate a neighbouring function to satisfy the gate. An adjacent implementation is not the subject,
and a role that names the wrong fn is worse than a missing one — it is a false answer from the thing
we are making the sole authority.

**STOP-7 — verbatim otherwise.** No signature tidying, no reordering of untouched arms, no
opportunistic cleanup.

## Sabotage — report each as "predicted red, unverified"

You cannot run the suite, so predict precisely and say you did not run it:

1. delete `defsurface`'s `role = declare` annotation → what does the gate say?
2. delete `:wat::core::if`'s `role = check` annotation → what does the gate say? (proves the OLD
   branch survived)
3. `role = declaer` → what does the macro say? (proves the message you widened is the one that fires)

## Report

The variant + its doc line verbatim · the macro arm and all three widened strings verbatim · the
rewritten gate verbatim · the new `role_order` and header · **the whole `defsurface` doc struct
verbatim** (it is the stone's centre — the five axis rulings are the part the orchestrator must
read) · the `@syntax` you chose and where you verified it · the three sabotage predictions · and what
surprised you.

## Prior comparable

`BRIEF-STONE-1a-alpha-the-sketch-adopts-the-declared-grammar.md` — one stone ago, same files, same
report shape. `src/intrinsic/special/and_form.rs` is the row-authoring standard.
