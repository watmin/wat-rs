# NOTE (arc 255) — "an absence recorded as an answer": the class behind the `apply` defect, and six verified instances

**Filed 2026-08-28 at the builder's question, after Stone O.** He asked, of the fact that `&[WatAST]`
was the only signature `#[wat_intrinsic]` accepted: *"how many other poorly defined things do we
have?"* This note is the measured answer. **A POINTER, not a decision** — every instance below is
verified against the disk by the orchestrator, but none is ruled.

## The class

> **An ABSENCE recorded as an ANSWER** — a `None`, a default, a skipped gate arm, or a uniform
> shape that exists only because one shape was ever available, which some consumer then reads as a
> decision.

The exemplar, proven and fixed today: `IntrinsicEntry::value_handler: Option<ValueHandler>`.
`None` meant *nobody wrote one*; `:wat::core::apply` read it as *this cannot be applied* and told
331 of 380 registered, working verbs they did not exist. The root was not carelessness — until
Stone O-iii landed that morning, `&[WatAST]` was the only signature the macro accepted, so every
handler took ASTs whether it needed to or not. **A constraint of the tooling became a fact about the
verb, and a dispatch path answered from it.**

★ **This is `walk.rs:268` in a mirror, and that is why it belongs to arc 255.** The blanket-accept
says YES to names that do not exist; `apply` said NO to names that do. Both answer from a private
picture. An arc whose thesis is *"the registry is the sole authority for what exists"* has to make
the registry able to say **"I was not told"** distinctly from **"no."**

## The six, each verified by the orchestrator against the disk

Found by casting `circumspicere` (the datamancy ward for *"what is true about this artifact that no
lens examined?"*) at `src/intrinsic/` + `crates/wat-macros/src/wat_intrinsic.rs` + their gates, then
re-measuring every claim independently. Ranked.

### 1. ⛔ THE DUPLICATE-NAME GUARD IS COMPILED OUT ON THE ONLY FLOOR THIS REPO TRUSTS

`src/intrinsic/mod.rs:348` is the **sole** defense against two homes claiming one FQDN:

```rust
debug_assert!(!self.entries.contains_key(entry.name), "duplicate intrinsic registration: {}", entry.name);
self.entries.insert(entry.name, entry);
```

`Cargo.toml` has **no `[profile.release]` section**, so debug assertions are off in release, and
`scripts/floor.sh:96` runs `cargo nextest run --release`. Behind the dead assert, `HashMap::insert`
**silently overwrites** — last `inventory::iter` writer wins, and that order is not guaranteed.
No `#[test]` anywhere walks `all_entries()` for duplicate names.

**Measured: no duplicates today** — 380 `#[wat_intrinsic]` + 2 `#[wat_special_form]` sites, 382
distinct names, anchored grep. The hole is not currently occupied; it is simply open, on the artifact
this whole arc exists to make authoritative.

> Closure: a real `#[test]` over `registry().all_entries()`, alive in release. A `debug_assert` is
> the CHECK rung of the ladder in debug and the CONVENTION rung in release.

### 2. ⛔ `show-source` SHIPS A CLAIM ITS CODE DOES NOT HONOUR — and the claim is user-visible prose

**The claim** (`src/intrinsic/reflect.rs:218`) — and this text is `entry.prose` for
`:wat::core::show-source` itself, so a wat caller reads it through `render-doc`/`metadata-of`:

> *"Returns the FQDN keyword form string for primitives and special forms (no source available)."*

**The code**: `reflect.rs:238-240` consults `lookup_entry` first and returns `entry.source`
unconditionally on a hit; `mod.rs:419` hardcodes `source: ""` for every `Kind::SpecialForm`. The
honest fallback that would say "no source available" is unreachable for any migrated special form.

**Proven live:** `(:wat::core::show-source :wat::core::if)` → `""`. No error, no hint, no FQDN.
Highest severity by the ward's own scale: a shipped claim the code contradicts.

### 3. ⛔ THREE `#[ignore]`s CARRY ONE REASON STRING AND THREE DIFFERENT TRUTHS — and they are arc 255's own unlock list

All three read *"RED-at-HEAD: arc-255 metadata-of reflection (builtin-registry) not yet built;
unlock when we circle back to arc 255."* Run with `--run-ignored all`:

| test | reality |
|---|---|
| `probe_arc255_reflection_parity.rs:71` `metadata_of_answers_for_a_rust_builtin` | **PASSES.** It IS built. The ignore masks a GREEN test. |
| `probe_arc255_ivc_metadata_plain_values.rs:68` `metadata_of_emits_plain_values_and_enums_not_holon_ast` | FAILS — but for a DIFFERENT cause: `metadata map missing key :pure`. The feature shipped to another contract (`:purity`/`:determinism` as enums); the test was never told. |
| `probe_arc255_reflection_parity.rs:83` `user_form_carries_guaranteed_baseline` | FAILS. Reason accurate. |

`probe_arc255_reflection_parity.rs:94-121` already records that two SIBLING tests were deleted in
2026-08-16 for exactly this staleness — and the survivors kept the identical string, unrechecked.
`[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`

> ⚠ These are the gates the 255 seam calls *"the worklist, written by a prior self as the unlock
> condition."* One of them can be un-ignored today.

### 4. ⛔ THE `@yields` GATE'S ONLY SUBJECT IS THE FIXTURE WRITTEN TO EXERCISE IT

`yields_type_matches_fn_arg_param` (`src/intrinsic/mod.rs:727+`) opens:

```rust
let yields_type = match entry.yields_type { Some(yt) => yt, None => continue }; // no @yields — skip
```

It verifies a **declared** yields-type is correct. It cannot detect a **missing** one. Measured:
`@yields` appears in exactly **one** file, `src/intrinsic/witness.rs` — *"Arc 255 spec-complete
witnesses"*, the synthetic HOF written to exercise this very gate.

Meanwhile `:wat::kernel::spawn-thread` (`src/intrinsic/kernel/resource.rs:350`) declares **three**
fn-typed `@arg`s — `prog`, `init_fn`, `post_spawn_fn` (lines 326-328) — and `@yields` count in that
file is **0**. `spawn-process` (`resource.rs:400`) is the same shape.

**A gate whose entire measured population is its own test fixture is not a gate.**
`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

### 5. THE SAME GATE FAMILY HAS A SECOND SILENT SKIP, AT TWO SITES

`None => continue, // not yet in checker — skip` appears at `src/intrinsic/mod.rs:512` and `:742` —
`doc_arg_ret_types_match_checker_scheme` and the `@yields` gate. An entry absent from
`register_builtins` is skipped by both, so its `@arg`/`@ret` strings are unverified against anything.

⛔ **MEASURED AND CLOSED 2026-08-28 by Stone P4 — the ward's number was WRONG.**

```
49 of 382 registered entries are absent from CheckEnv        (the ward said 96 of 384)
```

The ward cross-referenced `:wat::` string literals inside `register_builtins`'s body against
`#[wat_intrinsic]` attributes — two text instruments over two different populations, which is why
its 384 disagreed with the anchored 382 and why this NOTE refused to publish the figure. **P4 asked
the gate's own instrument the gate's own question instead:** it builds
`CheckEnv::with_builtins_and_types(&TypeEnv::new())` and calls `check_env.get(entry.name)` over
`registry().all_entries()` — the same construction and the same method both gates use, so the
measurement *cannot* disagree with what they skip.

**The shape this NOTE recorded was right; only its size was open, and the open size was the
honest state.** By namespace:

| | skipped / total | |
|---|---|---|
| `:wat::kernel::` | 23 / 46 | the largest pocket — half the namespace |
| `:wat::holon::` | 7 / 91 | |
| `:wat::core::` | 6 / 18 | includes `if` and `let` |
| `:wat::linkedlist::` | 5 / 5 | **wholly absent** |
| `:wat::seq::` | 3 / 3 | **wholly absent** |
| `:wat::string::` · `:wat::time::` | 2 each | |
| `:wat::edn::` | 1 / 13 | |

The population is now a **frozen DEBT LEDGER by name** in `src/intrinsic/mod.rs`
(`FROZEN_CHECKER_DEBT_LEDGER` + `checker_skip_debt_is_named_and_frozen`). It goes red in both
directions — a new unregistered name, or a name that gets registered and must leave the list — and
its failure text names the FQDN rather than a delta. Verified independently by the orchestrator by
removing a different name than the rider used.
`[[feedback_a_gate_freezes_names_never_a_count]]`

★ **The concrete cost the number stands for:** `:wat::core::fresh-symbol` declares
`@arg base :wat::core::String` and `@ret :wat::WatAST`, has no `register_builtins` entry and no
bespoke `check.rs` arm at all — **those two doc strings are asserted nowhere.** Several other ledger
names (`:wat::linkedlist::conj`, `:wat::seq::zip`, `:wat::string::interpolate`) *do* have bespoke
type-check arms outside `register_builtins`, the same pattern as `spawn-thread`/`spawn-process`;
they remain on the ledger because **this gate family never cross-checks their docs against those
arms.** Driving the ledger to zero is a separate, larger stone.

### 6. `metadata-of` REPORTS `:arity -1` FOR A FIXED-ARITY SPECIAL FORM

`src/intrinsic/mod.rs:410` hardcodes `arity: Arity::Variadic` for every `Kind::SpecialForm`
(*"special forms handle their own arity"*), ignoring the entry's own `args`. `:wat::core::if`
declares three fixed `@arg`s (`src/intrinsic/special/control_flow.rs:14-16`); live,
`(:wat::runtime::metadata-of :wat::core::if)` → `:arity -1`. `render-doc` gets it right (it derives
from `entry.args`, `reflect.rs:349-356`); only `metadata-of` ships the wrong number, and it does not
expose `:args` for a caller to cross-check. Blast radius today is one entry (`let` is genuinely
variadic) — the hardcode covers every future fixed-arity special form.

## The shared shape, and why it is worth a name

Five of the six are the same move: **a place where the system does not know something, filling in a
definite answer.** `debug_assert` absent in release reads as "checked". `source: ""` reads as "no
source" and as "empty source". `None => continue` reads as "nothing to check". `Arity::Variadic`
reads as "any arity". `value_handler: None` read as "cannot".

**The discriminator to reach for, every time:** *does this representation distinguish NO from
NOT-ASKED?* If it cannot, some consumer will eventually read the second as the first — and the
consumer will be right to, because nothing told it otherwise.

## Dispositions — none of these is ruled here

- **1, 2, 6** are small, unambiguous, and each closes with a check or a branch. They are the
  builder's to order, not this note's.
- **3** is arc 255's own unlock list and interacts with the seam's ignore ledger.
- **4** wants a macro-expand-time requirement (`@arg` contains an Fn shape ⇒ `@yields` mandatory) —
  the top rung, and a bigger change than the others.
- **5** needs its size measured with a validated instrument before anything is decided.

## Refs

- `DESIGN-STONE-O-one-declaration-feeds-both-doors.md` — the exemplar, and its correction sections.
- `src/value/signal.rs:600-628` — `NotValueDispatchable`'s comment, which records the exemplar and
  points at `walk.rs:268` as the same defect.
- `wat-scripts/hunt/stone-o-shell-census.awk`, `stone-o-delegate-census.awk` — the instruments, both
  carrying the story of how their own numbers were wrong first.
