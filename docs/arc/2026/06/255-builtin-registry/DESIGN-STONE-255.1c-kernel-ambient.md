# DESIGN STONE — 255.1c-kernel-ambient · HOME #4, and the first row the prefix rule gets WRONG

## The one-line claim

`:wat::time::now` and `:wat::kernel::sigusr1?` **do the same thing** — read ambient state, return a
value, cause no effect — and the substrate gives them **opposite purity answers**, entirely because of
what namespace they live in. This stone registers the seven `:Ambient` verbs with purity derived from
each **body**, and the disagreement that follows is the deliverable.

## ⛔ FIRST — THE CARVE BOUNDARY IS THE CATEGORY, NOT THE TABLE'S ROW

`255/DESIGN-STONE-255.1c-kernel-stdio.md`'s decomposition table puts six verbs in a **signals** row
and files `stopped?` under **misc**. `:Ambient`'s own prose in `wat/runtime-meta.wat:163–169` —
written by reading the bodies during `255.1c-taxonomy` — names **seven** members and says so out loud:

```
;; Reads or writes process-global state that no value the caller holds addresses —
;; `stopped?`, `sigusr1?`, `sigusr2?`, `sighup?`, `reset-sigusr1!`, `reset-sigusr2!`,
;; `reset-sighup!`. NOT `:Clock`: three of SEVEN members are writes ...
```

The table split a family; the taxonomy did not. **The taxonomy is the better authority** — it was
derived from bodies, the table was drawn from prefixes and adjacency. Home #4 carves `:Ambient`
whole: **seven verbs.** The table is stale by one and is corrected in this stone's own text, not
edited elsewhere.

That correction is itself the seam's standing alarm arriving on schedule: *hand-maintained lists,
four times in one day — the class 255 exists to kill.* This is the fifth.

## ★★ THE POINT — `is_effectful_op` is a NAMESPACE rule doing a PROPERTY rule's job

```rust
pub(crate) fn is_effectful_op(head: &str) -> bool {
    head.starts_with(":wat::kernel::")      // ← every verb in this stone
        || head.starts_with(":wat::io::")
        || head.starts_with(":wat::eval-")
        || head.starts_with(":wat::load")
        || head.starts_with(":wat::config::")
}
```

It cannot see inside a body. It has three live consumers, all **conservative denials** —
`step_*` refuses an effectful op (`runtime.rs:29626`, `EffectfulInStep`); `rete/purity.rs:1058`
default-denies the `Pure` axis; `derive_pure_deterministic` (`runtime.rs:29749`) answers for
**not-yet-registered** intrinsics only, and its own doc says it "dies when they migrate."

Home #3 gave the cross-check `pure_declared_matches_is_effectful_op` (`src/intrinsic/mod.rs:544`)
six rows that **agreed**. Its commit recorded the honest limit of that: *"before this stone it had
only ever agreed with itself."* Six agreeing rows do not falsify a biconditional either.

**This family can.** Four of the seven are pure reads in an effectful namespace.

### The derivation, per body — read them, do not take this on faith

| verb | body | Purity | Determinism |
|---|---|---|---|
| `stopped?` | `KERNEL_STOPPED.load(SeqCst)` → bool | **Pure** | **Nondeterministic** |
| `sigusr1?` `sigusr2?` `sighup?` | `flag.load(SeqCst)` → bool (`eval_user_signal_query`, `runtime.rs:25944`) | **Pure** | **Nondeterministic** |
| `reset-sigusr1!` `reset-sigusr2!` `reset-sighup!` | `flag.store(false, SeqCst)` → `Unit` (`eval_user_signal_reset`, `runtime.rs:25968`) | **Effectful** | **Deterministic** |

`:Pure`'s shipped prose is *"Same output for the same input, with no observable side effect."* A
`load` has no observable side effect. The varying-output half is what `:Determinism` carries — that
orthogonality is not a new argument here, it is **twenty registered rows**: home #2's `:wat::time::*`
family is `Pure × Nondeterministic`, and `:wat::time::now` reads the wall clock exactly the way
`sigusr1?` reads an `AtomicBool`. `time` is simply not in the prefix list.

**So the four readers are Pure by the substrate's own governing precedent, and
`is_effectful_op` says effectful. The gate asserts the biconditional. It goes RED.**

## THE ONE CONTRACT DECISION, PINNED

> **Purity is derived from the body. When the cross-check disagrees, the cross-check's red is the
> stone's RESULT — not a prompt to edit either side until it passes.**

This is verbatim home #3's pinned contract, which did not fire there. It fires here. Three moves are
**forbidden to this stone**, each with its reason:

1. **Declaring the four readers `Effectful`** to satisfy the gate. Fails Honest: it makes the
   declaration copy the prefix rule instead of deriving from the body, which is the one thing the
   registry exists to stop.
2. **Carving a hole in `is_effectful_op`.** It guards the step machine and the rete `Pure` axis.
   Narrowing it changes what those two admit. That is a substrate-semantics ruling with its own blast
   radius — not a side effect of a registration stone.
3. **Splitting the stone to carve only the three writers.** They alone would pass green. Choosing the
   scope that avoids the finding is `feedback_a_worklist_filter_is_a_claim_about_what_you_expect`
   wearing a stepping-stone's clothes.

The honest fourth move — **the gate's assertion may be wrong**, because a conservative
over-approximation is legitimate: the runtime may refuse *more* than the registry calls effectful, and
must only never call an effectful verb pure. That reads as `Effectful ⇒ is_effectful_op`, an
implication, not a biconditional. **That is a builder ruling, and this stone does not take it.** It
surfaces the collision with both sides measured and stops.

## The four questions

- **Obvious?** YES. Seven verbs, one category, purity read off seven small bodies.
- **Simple?** YES. One axis: register what the body does.
- **Honest?** YES — and only because the predicted red is written down *here*, before the strike, as
  the expected outcome rather than a surprise the rider has to adjudicate alone.
- **Good UX?** YES for the caller: `metadata-of` starts telling the truth about four verbs it
  currently mis-answers through `derive_pure_deterministic`'s prefix fallback.

## Rooms

```
src/runtime.rs:6737           stopped?                     (literal arm)
src/runtime.rs:6882-6906      the six signal arms          (literal arms, contiguous)
src/runtime.rs:25944          eval_user_signal_query       (shared body — 3 readers)
src/runtime.rs:25968          eval_user_signal_reset       (shared body — 3 writers)
src/runtime.rs:/eval_kernel_stopped/                       (body — 1 reader)
src/runtime.rs:68,122-124     KERNEL_STOPPED / SIGUSR1 / SIGUSR2 / SIGHUP — already `pub`
src/check.rs:17930            stopped?  → bool_ty()
src/check.rs:18044-18070      six schemes: 3 → bool_ty(), 3 → TypeExpr::Tuple([])
src/intrinsic/kernel_stdio.rs                              THE SHAPE TO COPY (211 lines)
src/intrinsic/mod.rs:544      pure_declared_matches_is_effectful_op
```

**Blast radius is smaller than home #3's.** That stone had to add `Io` to the taxonomy, which dragged
in `wat-doc`, both `wat-macros` files, `RuntimeCategory` and `eval_metadata_of`'s hand-match. Two of
those hand-mirrors were **collapsed by `69e16fc3`** (`RuntimeCategory` is gone; `ToEnumValue` is
macro-generated), and `:Ambient` **already exists** — that was `255.1c-taxonomy`'s entire purpose.
So: one new file, one `mod` line, seven arms deleted, three helper fns made `pub(crate)`.

## The nullary shape already exists

`:wat::kernel::read-frame` (`kernel_stdio.rs:203`) is a registered zero-arg intrinsic — no arg
params, just `env, sym, list_span`. All seven verbs here are nullary. No new macro capability.

`@ret` spellings, checked against the registered schemes so
`doc_arg_ret_types_match_checker_scheme` agrees: readers `:wat::core::bool`; writers
`:wat::core::nil` — `TypeExpr::Tuple([])` renders to that spelling via the arm home #3 added at
`mod.rs:541` (*"nil IS the unit type"*, the builder's ruling).

## Out of scope — affirmative cuts, homes named

- **Ruling the `is_effectful_op` collision.** Surfaced by this stone; ruled by the builder; whatever
  ships from that ruling is its own stone. This stone stops at the measurement.
- **The other five kernel concerns** — concurrency, networking, errors, handles/capability, and
  what remains of misc. Named in `255/DESIGN-STONE-255.1c-kernel-stdio.md`; each its own home.
- **`eval_poll_prime` has no doc comment** (`runtime.rs:33253`) — real, Level 2 mumble, belongs to
  the `:Message` carve that files `poll'`, not here.
- **`require-wire-address` → `as-wire-address`.** A corpus rename with its own blast radius, and the
  builder has since named `must-*` as the family prefix. Untouched.

## Progress meter

Registered production names **53 → 60**. Seven arms leave `runtime.rs`. `:Ambient` goes from a
variant with zero tenants to the home of a whole family — and the honest claim is not the count. It
is that **the registry's purity column and the runtime's prefix rule can finally be shown to
disagree, on rows where the body proves which one is right.**
