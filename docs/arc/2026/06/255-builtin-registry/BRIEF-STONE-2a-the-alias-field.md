# BRIEF — STONE 2a: the registry learns what an alias is

Add `@alias` — one field saying *"this name means that name"* — and make the dispatch door **read
it**, so an alias row needs no handler, no eval role and no delegate. Then register exactly one
witness and prove it dispatches through the registry rather than through `RETE_OPS`' arm.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-2a-the-alias-field-and-why-1b-was-blocked-twice.md`
— its ★★★ contract is the whole stone: **the alias field is not metadata, it is the dispatch.**

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. Run every command you do run in
the FOREGROUND and block on it. The orchestrator builds, floors and clippies centrally — you do not
run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`. You may run the pre-existing
`./target/release/wat` and `--check` for a fast read. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything. Tree clean, floor green at 5123.

## ★★ `@see` is your worked precedent — copy its shape end to end

`@see` is already an **FQDN-valued doc directive with a resolution gate**, threaded through exactly
the path `@alias` needs. Follow it at every step:

```
crates/wat-doc/src/lib.rs:215        pub see: Vec<String>            ← the parsed field
crates/wat-doc/src/lib.rs:463, 1204  "@see" in BOTH `recognized` tag lists  ⚠ TWO lists
crates/wat-doc/src/lib.rs:693        the parse arm
crates/wat-macros/…wat_intrinsic.rs:1158   `see: &[#(#see_lit),*]`   ← threaded to the submission
src/intrinsic/mod.rs:259, 303, 448   `pub see: &'static [&'static str]`  ← on all THREE structs
src/intrinsic/mod.rs:1824            all_see_fqdns_resolve_to_registered_intrinsics  ← the gate
```

⚠ **Note the TWO `recognized` lists** (`:463` and `:1204`). A directive added to one and not the
other is accepted in one parse path and rejected in the other. Find both.

## Read in order

1. **The DESIGN**, contract and gate.
2. **The `@see` path above**, in that order — it is the map.
3. **`src/runtime.rs:1990`** (`dispatch_keyword_head_value`) and **`:1898`**
   (`dispatch_keyword_head`) — the registry-first doors, and the `Unevaluated` guard added last
   stone. Your alias check goes in the same gap.
4. **`src/runtime.rs`'s `dispatch_rete_op`** — the `OpClass::Alias | Form | Redispatch` arm, one
   line, serving 54 rows. **You are not deleting it** — 73 rows still need it.
5. **`src/rete/vocabulary.rs`**, the `ReteOp` row for `:wat::rete::i64::+` (~line 269) — your
   witness's source of truth: `core_name`, `params`, `ret`, and `meta: OpMeta { pure, deterministic,
   total }` all true.

## The work

### 1 — the field, end to end

`alias_of: Option<&'static str>` on `IntrinsicEntry` and both submission structs; `@alias <fqdn>` in
the doc grammar and **both** `recognized` lists; threaded through both proc-macros.

### 2 — the dispatch reads it

In both doors, after the registry-first handler lookup fails and beside the `Unevaluated` guard:

```
lookup_entry(head).alias_of == Some(core)   ⇒   re-dispatch `core` with the same args and span
```

⛔ **Order matters and you must state your choice**: an alias row is not `Unevaluated`, so the two
guards should not collide — but say in your report which you placed first and why.

### 3 — the gate, both halves

- **every `alias_of` target is itself a registered row** — a dangling alias is a dispatch into
  nothing;
- **no alias points at another alias** — chains would make dispatch order-dependent. `RETE_OPS` has
  none today; freeze that while it is true.

Non-vacuity: the gate must inspect ≥ 1 row and name it.

### 4 — one witness row

`:wat::rete::i64::+` → `:wat::i64::+`. A doc-only struct like the others, five axes argued from the
`ReteOp` row's own `OpMeta` (all three true) and from what an alias *does*.

⬜ **`@Category` is yours to argue.** An alias performs no doing of its own — it names another verb.
`:Transform`? `:Reflection`? Something else? ⚠ The gate no longer forces a category (that coupling
was removed last week), so this is a free choice — argue it, and say what you refused.

## Blast radius

`crates/wat-doc/src/lib.rs` · `crates/wat-macros/src/{wat_intrinsic,wat_special_form}.rs` ·
`src/intrinsic/mod.rs` (field ×3, gate) · `src/runtime.rs` (two door checks) ·
`src/intrinsic/special/` (+1 row, +1 mod line). **`src/rete/vocabulary.rs` is NOT touched** — the
row stays; the registry gains a second, authoritative answer for one name, and `RETE_OPS` dies at
Phase 4a, not here.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — the witness must dispatch THROUGH THE REGISTRY, and you must prove it.**
`(:wat::rete::i64::+ 1 2)` returning `3` is not proof — it returns `3` today via `dispatch_rete_op`.
**Verify the registry path is the one taken**, e.g. by reasoning from where your check sits relative
to `dispatch_rete_op`, and say exactly how you established it. A structural gate that passes while
dispatch is unchanged is the failure this STOP exists to catch.

**⛔ STOP-2 — do not give the witness `role = eval`, a handler, or a delegate.** The whole contract
is that an alias needs none. If you find yourself adding one, the field is not carrying the dispatch
and that is a finding.

**⛔ STOP-3 — do not touch `src/rete/vocabulary.rs`, `resolve_core_name`, or `dispatch_rete_op`'s
arms.** 73 rows still route through them. Consumers ask, THEN the duplicate dies; one of 74 is not
that moment.

**⛔ STOP-4 — both `recognized` lists.** A directive in one and not the other is a parse path that
silently rejects it. If you find only one, look again; if there genuinely is only one, say so.

**⛔ STOP-5 — the chain half of the gate must be real, not asserted.** Write it so that pointing an
alias at an aliased row goes red. If you cannot make it fire, say so rather than shipping a branch
nobody has seen work.

**STOP-6 — verbatim otherwise.**

## Sabotage — report each as "predicted red, unverified"

1. point the witness's `@alias` at `:wat::core::zorble` → what does the target gate say?
2. add a second alias row pointing at `:wat::rete::i64::+` (itself an alias) → what does the chain
   half say?
3. delete the dispatch check but keep the field → does anything fail? ⚠ If the answer is "no, the
   `RETE_OPS` arm still returns 3", **say so** — that is exactly the vacuity STOP-1 names, and it is
   a finding about the acceptance rows, not a pass.

## Report

The field on all three structs · **both** `recognized` list edits · the parse arm · the two macro
threadings · the dispatch check verbatim **and where it sits relative to the `Unevaluated` guard,
with your reason** · the gate verbatim, both halves · the witness row verbatim with your `@Category`
argument · **how you established the registry path is the one taken** · the three sabotage
predictions · and what surprised you.

## Prior comparable

`BRIEF-STONE-1a-beta-0b-a-form-that-never-evaluates.md` — a field/pole added across the same
wat-doc → macros → registry path. `@see` is the closer structural precedent.
