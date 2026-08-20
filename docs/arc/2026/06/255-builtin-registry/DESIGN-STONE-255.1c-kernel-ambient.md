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

---

## ⊘ RULED 2026-08-19 — **OPTION B. THE REGISTRY IS THE AUTHORITY.**

**Builder:** *"B... the registry is the truth now... that's why we built it. the registry must answer
these kinds of questions... forms and funcs must be registered in a central authority who can resolve
such questions.. that's 255 purpose."*

That is not a patch to one function. It is a **direction of authority**, and once stated it decides
three sites at once — all of which currently ask a prefix what only the registry can know.

### The principle, in one line

> **Ask the registry. The prefix guesses only where the registry is silent.**

### Site 1 — `is_effectful_op` (`runtime.rs:29726`)

Splits in two. The public door consults the registry; the prefix rule survives as a **named private
fallback** for verbs not yet carved:

```rust
pub(crate) fn is_effectful_op(head: &str) -> bool {
    // Arc 255: the registry is the authority. A registered row DECLARED its
    // purity from its body; the prefix cannot see inside one.
    if let Some(e) = crate::intrinsic::registry().lookup_entry(head) {
        return matches!(e.purity, wat_doc::Purity::Effectful);
    }
    effectful_by_prefix(head)   // ← the guess, for the un-carved remainder
}
```

`registry()` is a `OnceLock` over an `inventory` sweep returning `&'static`, and `lookup_entry` is a
`HashMap::get` on `&str` — safe from any caller, no init-order hazard. On the step path the very next
statement is a `match head_kw.as_str()` over dozens of arms, so one hash lookup is not a new order of
cost. **That is an argument from shape, not a benchmark; no perf claim is made here.**

### Site 2 — `derive_pure_deterministic` (`runtime.rs:29724`) — ⚠ ITS DOC IS FALSE TWICE

```
;; claim 1: "the same derivation used by metadata-of's intrinsic branch ... and the
;;           verify-examples reflection seam. Extracted here so both callers share
;;           one source of truth."
;; claim 2: "this set is the residual for not-yet-registered intrinsics
;;           (dies when they migrate to #[wat_intrinsic])."
```

**Both are false, measured at HEAD.** It has **one** caller, not two — `metadata-of` now reads
`entry.purity`/`entry.determinism` directly (`runtime.rs:14181–14183`, *"declared enum values from
the doc, not derived bools"*). And that one caller is `reflect.rs:75`, inside
`for entry in registry().all_entries()` — so it is fed `entry.name` and computes a **prefix guess for
a row whose declared purity and determinism sit in the same struct it was handed.**

The verbs migrated. The hand-list did not die. Under the ruling, `reflect.rs` reads the entry it
already holds, and `derive_pure_deterministic` — with its `NONDETERMINISTIC: &[&str]` hand-list — is
left with the population it honestly describes, or no population at all. **The rider measures which,
and does not delete a capability on its own.**

### Site 3 — the gate must be RE-POINTED, or it becomes theater

This is the concern raised before the ruling, and the ruling does not dissolve it — it **relocates**
it. Once site 1 lands, `pure_declared_matches_is_effectful_op` compares `entry.purity` against a
function that *returns* `entry.purity`. It goes green automatically and **can never fail again for a
registered row.** Shipping it unchanged would be a gate that reads a copy of the truth and inherits
it — `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]` — one day after the seam recorded
*"hand-maintained lists, four times in one day."*

**So the biconditional stops being an ASSERTION and becomes a CENSUS.** It compares two things that
are still genuinely independent — each row's **declared** purity, and what **`effectful_by_prefix`
alone** would have said — and reports every disagreement as an inventory rather than a failure. The
split in site 1 is what keeps the second opinion computable; without it there would be nothing left
to compare against.

This preserves the finding instead of erasing it. The four readers carved by this stone are the
census's **first four entries**, and the instrument acquires a second honest job: it is a
**prefix-accuracy meter**, and the rows still answered by prefix are exactly the ones not yet carved.

What the gate keeps asserting, unchanged and still able to fail: **`Effectful ⇒ effectful_by_prefix`
for UNREGISTERED verbs** — the direction with teeth, where a doc could still lie about an effect the
runtime does not know to refuse.

### What this stone does NOT do

- **Widen the carve.** Still seven verbs. Sites 1–3 are the ruling applied where it lands; no
  additional kernel concern is carved.
- **Delete `derive_pure_deterministic` or its hand-list on the rider's judgement.** If it is left with
  zero callers that is a finding to report — `[[feedback_no_consumers_does_not_mean_dead]]`; zero
  consumers is not evidence of deadness, and the disposition is the builder's.
- **Change what `step_*` or `rete/purity.rs` are FOR.** Both keep refusing effectful ops. They will
  now refuse based on a declared fact instead of a namespace guess — for the four readers that is a
  behaviour change, and its exposure was measured before the ruling: **zero non-comment call sites in
  `wat/` outside `kernel/services/stdio.wat`, and none in the rete grid.** Small today; the guard is
  forward-looking, so the change is recorded as real, not dismissed as theoretical.
