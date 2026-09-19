# DESIGN — the probe's ★ paragraph asserts a fence that does not exist

## Why

Rowed in `34ee46ce9`. Two places in this tree disagree about the same fence, and **I drew a strike
off the stale one** — the `produced_type` DESIGN predicted "outcome 2: the purity fence makes this
unconstructible", which the executor then disproved in minutes.

**The claim** — `tests/rete/probe_arc278_then_user_forms_userfn.wat:16-17`, inside a ★ paragraph:

> *"a composed fn whose body constructs a record is refused today with
> `` `:wat::core::kwargs-construct` is not pure ``"*

**The record** — `src/rete/kernel/stratify.rs:420-422`:

> *"constructs a record at all → `` `kwargs-construct` is not pure ``. **FALSE today.** A rete defn
> whose body is `(:bc::N :k (…::i64::+ k 1 :undefined 0))` is ADMITTED — it declares clean and
> reaches the cyclicity check below. **Driven 2026-08-28.**"*

`stratify.rs` also diagnoses *why* the probe got it wrong, and the diagnosis is the interesting
part: all three of the original attempts used a plain `:wat::core::defn`, so **Law A refused the
FN declaration, not the body** — *"the table measured ONE door three times and read it as three."*

**Driven again 2026-09-07, independently:** `wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.wat`
declares `:a2::mk-rate` as a `:wat::rete::core::defn` whose body **constructs** `(:a2::Rate :count k)`,
and it compiles — `COMPILE: Compiled`. That fixture is the one that exposed the oracle defect cured
in `21a5f8514`, so the construction path is not hypothetical; it is load-bearing evidence in the
tree today.

## The one contract decision

**Correct the paragraph; change no code.** `:tf::first-rate` may keep extracting — the probe's
subject is *"the `:then` item's HEAD is a user fn"*, which extraction demonstrates perfectly well.
What is false is the stated REASON for that choice, and the reason is what a future hand reads when
deciding whether construction is possible. Rewriting the probe to construct would be a different
strike with a different risk, and it would destroy a working fixture to prove a point the tree
already proves elsewhere.

## What the corrected paragraph must carry

1. The claim was **false**, and is struck rather than silently deleted — a reader who met it before
   needs to know it was withdrawn, not go looking for the sentence they remember.
2. **Why it was believed**: three probes that each used a plain `:wat::core::defn` and so met Law A
   three times, read as three independent fences.
3. **The two drivings**: `stratify.rs:418-424` (2026-08-28) and
   `arc278-produced-type-userfn-facts.wat` (2026-09-07, the fixture behind `21a5f8514`).
4. **What actually guards the minting body today**: `rete_fn_body_mints` and
   `probe_arc278_termination_fn_head.wat`, both named in `stratify.rs`'s block.

⚠ **The corrected text must not assert anything about `purity.rs`'s ratchet that this strike has
not read.** The original paragraph's failure was asserting a fence's behaviour from three probes
that never reached it. Repeating that shape while fixing it would be the fourth instance today of a
cure carrying the defect one level down.

## ⚠ And this file may be gated by nothing

`no_stale_path_in_doc`'s `ROOTS` are `src/rete` (`.rs`), `wat` (`.wat`), `wat-tests` (`.wat`).
`rete_citation_resolves` scans `src/rete` comments. **`tests/rete/*.wat` prose appears in neither.**
That is how a false ★ paragraph survived in a probe for weeks. Not fixed here — widening a gate's
population is its own strike with its own false-RED survey — but rowed, because it is the reason
this rot went unseen rather than an accident of this one file.

## Out of scope = REJECTED

- Changing `:tf::first-rate` to construct.
- `purity.rs`, its `KNOWN_UNREVIEWED` ratchet, and Law A.
- Widening `no_stale_path_in_doc` / `rete_citation_resolves` to `tests/rete/*.wat` — rowed above.
