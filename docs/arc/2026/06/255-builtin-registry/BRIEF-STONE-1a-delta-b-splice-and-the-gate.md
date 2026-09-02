# BRIEF — STONE 1a-δ-b: mint `:Splice`, and stop the gate picking the taxonomy

Two changes, and the second is the one that matters. **①** Mint `:Splice` in the axis vocabulary and
give it to the three load forms, which register nothing and therefore never were `Declaration`.
**②** Re-derive `every_special_form_carries_check_and_eval_impls` from `@Purity Unevaluated` instead
of `@Category Declaration`, so a category never again decides what a row must implement.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1a-delta-b-splice-is-a-doing-and-the-gate-stops-picking-the-taxonomy.md`
— read its ★★★ measurement and its ★★★ contract; they are the whole stone.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. Run every command you do run in
the FOREGROUND and block on it. The orchestrator builds, floors and clippies centrally — you do not
run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`. You may run the pre-existing
`./target/release/wat` and `--check` for a fast read. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything. Tree clean, floor green at 5123.

## Read in order

1. **The DESIGN.**
2. **`wat/runtime-meta.wat:63-115`** — the `Category` header, which states the axis discipline
   (*"the DOING, not the moment it happens"*, *"not where its input comes from"*), and **`:116-197`**,
   the `defenum` itself. Read `:Combine` and `:Projection`'s prose: they are the grain your new
   variant must match.
3. **`crates/wat-doc/src/lib.rs:73-77`** (`CATEGORY_LEGAL_VALUES`) and **`:2071`**
   (`category_message_lists_every_variant`) — the message and the gate that already guards it.
4. **`crates/wat-macros/src/wat_intrinsic.rs:945`** and **`wat_special_form.rs:154`** — two
   **exhaustive** `match doc.category`. Adding a variant breaks the build at exactly these two, which
   is the sweep working.
5. **`src/intrinsic/mod.rs`** — `every_special_form_carries_check_and_eval_impls`, its
   `entry.category == Declaration` branch, and the ★★★ AMENDED comment above it recording that this
   coupling was never intended.
6. **`src/intrinsic/special/{load_file,digest_load,signed_load}.rs`** — the three rows. Read their
   Category grounds closely: **each argues `Declaration` twice**, once from prose and once from the
   gate. Both arguments go.

## The work

### 1 — mint `:Splice` in the source of truth

`wat/runtime-meta.wat`'s `:wat::runtime::Category`, with `;;` prose in the voice of its siblings. It
names a doing: **a form that replaces itself with another program's forms, spliced into this form
stream in place; it registers nothing itself.** Give it a differential against `:Declaration` — the
nearest neighbour, and the one it is being taken from — the way `:Entropic` carries one against
`:Io` and `:Ambient` against `:Probe`.

⚠ Also correct the header comment at `:82-84`, which lists `:Declaration`'s examples. It is prose
about a population you are changing.

### 2 — the widenings the compiler and the gates demand

`CATEGORY_LEGAL_VALUES`; the two exhaustive proc-macro matches. Nothing else should need to change —
**if you find a third hand-written enumeration of the categories, that is a finding**, and the
`Purity` sibling of this stone had to add a whole gate because five such messages had no guard.

### 3 — the gate re-derives from purity

```rust
entry.purity == wat_doc::Purity::Unevaluated  ⇒  must name Declare
otherwise                                     ⇒  must name Check and Eval
```

Rewrite the ★★★ AMENDED comment above it: it currently explains why `@Category` became load-bearing.
After this stone it must explain that the coupling was **removed**, and why `@Purity Unevaluated` is
the right axis — it is the one whose entire meaning is *"this form never evaluates"*, so it is the
one that can say a row may not name an eval impl.

### 4 — the three rows take `:Splice`

`@Category Declaration` → `@Category Splice`, and **rewrite each Category ground**. Both existing
arguments are retired: the prose one (which reached past the load form to the *spliced* forms'
visibility) and the structural one (which cited the gate). The new ground argues the doing directly.

## Blast radius

`wat/runtime-meta.wat` (one variant + header prose) · `crates/wat-doc/src/lib.rs` ·
`crates/wat-macros/src/{wat_intrinsic,wat_special_form}.rs` · `src/intrinsic/mod.rs` (one predicate
+ its comment) · the three `src/intrinsic/special/*_load*.rs` rows. **No other row's category
changes.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — do NOT add `:Splice` to the gate's branch.** The gate must stop asking `@Category`
entirely. A branch reading `category == Declaration || category == Splice` re-commits the exact
defect this stone exists to remove, one variant later, and the next never-evaluating form that is
neither will hit the identical wall.

**⛔ STOP-2 — the predicate swap must change NOTHING today.** Measured: 11 rows declare
`@Purity Unevaluated`, 11 carry a `Declare` impl, and they are the same 11. **Verify that yourself
before you swap** — if the two sets differ by even one row, the swap is not behaviour-preserving and
that is a finding, not a rounding error.

**⛔ STOP-3 — `:Splice` names a DOING, not a moment and not a source.** Not `:LoadTime`, not
`:FromFile`. `runtime-meta.wat`'s own header rules both out, and two variants were renamed
(`:Clock`→`:Entropic`, `:Encoding`→`:Transform`) to enforce it.

**⛔ STOP-4 — no other row changes category.** `intueri` found 13 of 14 variants keep their promise.
If a second row looks miscategorised to you, **report it — do not fix it here.**

**⛔ STOP-5 — do not touch the loaders' other axes.** `@Purity Unevaluated`, `@Determinism`,
`@Totality`, `@ExpandTime` were argued per form one stone ago and are not in scope.

**STOP-6 — verbatim otherwise.**

## Sabotage — the two that ARE the stone, plus two guards

⚠ **1 and 2 are the same experiment run twice and must come out OPPOSITE.** Report both, and if they
do not oppose, say so loudly — either alone proves nothing.

1. set `:wat::load-file!` to `@Category Io` → predicted **GREEN**: the gate no longer asks the
   category, and the row is `Unevaluated` so it is still permitted `declare`-only.
2. set `:wat::load-file!` to `@Purity Pure` → predicted **RED**: no longer `Unevaluated`, so the
   gate demands `check` and `eval`, which it has neither of.
3. drop a loader's `role = declare` → still RED, "missing role: declare".
4. omit `Splice` from `CATEGORY_LEGAL_VALUES` → RED from `category_message_lists_every_variant`.

## Report

The wat `defenum` diff verbatim, prose included · the header correction · every widening you found,
**with the count, and whether a third enumeration existed** · the new gate predicate verbatim and its
rewritten comment · **the three rewritten Category grounds verbatim** · your own verification of
STOP-2's 11/11 correspondence, with how you counted · the four sabotage predictions, 1 and 2
answered as a pair · and what surprised you.

## Prior comparable

`BRIEF-STONE-1a-beta-0b-a-form-that-never-evaluates.md` — the sibling stone that minted a pole in the
same file and had to widen five unguarded messages. Same report shape.
