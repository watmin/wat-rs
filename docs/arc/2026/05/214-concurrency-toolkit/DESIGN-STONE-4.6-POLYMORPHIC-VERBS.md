# Stone 214.4.6 — peer verbs (`send'` / `recv'` / `try-recv'` / `close'` / `select'`)

> The peer types (4.4) and the spawn dispatcher (4.5) exist; peers are
> `Value::RustOpaque(":wat::kernel::Thread'")` / `(":wat::kernel::Process'")`.
> 4.6 gives wat code the verbs to drive them. After 4.6, Slice 5 migrates the old
> `:wat::kernel::send`/`recv`/`select`/`close` callers onto these primes and
> Slice 6 retires `typed_channel`.

## The dispatch mechanism — DECIDED BY THE RUBRIC (`docs/DISPATCH.md`), not by gut

`docs/DISPATCH.md` is the substrate's objective partition: clause vs intrinsic is
decided by a *checkable property of the op's type*, not preference. Two earlier
drafts of this stone got it wrong by gut — a hand-rolled `type_path` match
(reinventing the clause matcher), then "all defclause." The rubric overrules
both. Running each verb through the decision procedure (projective? relational? else clause):

| Verb | Type | Verdict | Why (per DISPATCH.md) |
|---|---|---|---|
| `recv'` | `peer<O> -> O` | **intrinsic — projective** | return is a function of the peer's element type param `O` (same shape as `get : Vector<T> -> Option<T>`) |
| `send'` | `peer<I>, I -> nil` | **intrinsic — projective** | `I` flows from the peer's type param into another argument |
| `try-recv'` | `peer<O> -> Option<O>` | **intrinsic — projective** | return is a function of `O` |
| `close'` | `peer<I,O> -> nil` (thread) / `i64` exit code (process) | **intrinsic** (forward-corrected 2026-06-07) | first verdict said "clause — no type-var flow," but the peer types are PARAMETRIC: a clause param is a FIXED named type and cannot say `Thread'<∀I,∀O>` — covering all `(I,O)` instantiations is the same infinite-open-set argument that makes `get` projective-intrinsic. A ∀-parametric arg needs type-level matching even when nothing flows → intrinsic by the rubric's own principle. (Had the peers been bare heads, clause would have held.) |

**This is the SAME treatment the verbs they replace already have:** the old
`:wat::kernel::send`/`recv`/`try-recv` are custom-inference intrinsics in
`check.rs` (`validate_comm_positions` + contextual typing; the projective element
type flows from the typed channel). The peer verbs inherit that, dispatching to
comms peers instead of `typed_channel` — consistent with Slice 5 ("typed_send/recv
become shims").

**The payload-any question dissolves.** Because the peers are **parametric**
(`Thread'<I,O>` / `Process'<I,O>`), the element type *flows* — that is precisely
what "projective ⇒ intrinsic" means. No universal "any value" type needs minting;
the intrinsic's `infer_<op>` projects `I`/`O` out of the peer type and flows them
into the payload arg / return. (My earlier "mint an Any type" open was an artifact
of the wrong all-defclause framing.)

## What 4.6a builds

- **Parametric peer types** `:wat::kernel::Thread'<I,O>` / `:wat::kernel::Process'<I,O>`
  registered as wat types (4.4 deferred this to here, peer.rs:33-37). The peer's
  element types are carried at the wat level so the intrinsics can project them.
  `declared_type_name` for a peer `RustOpaque` must report its specific parametric
  type (not the generic `:rust::opaque`, runtime.rs:5220) so inference + the
  `close'` clause matcher see the real peer type.
- **`recv'` / `send'` / `try-recv'` — intrinsics.** Add `infer_<op>` (check-side,
  projective over `peer<I,O>`) + `eval_<op>` (runtime). The eval downcasts on the
  `RustOpaque.type_path` to the concrete peer and bridges: Thread tier passes
  `Value` through; Process tier encodes/decodes `Value↔EDN` via the `spawn.rs`
  `HolonRepresentable for Value` impl. (The Rust-side `match type_path` in `eval`
  is fine — the rubric governs the *type-check* mechanism, not the eval impl;
  intrinsics are custom Rust by definition.) Mark the partition in-code at the
  `infer_list` / `dispatch_keyword_head` arms per DISPATCH.md § "Where it's declared."
- **`close'` — defclause.** A `Thread'` clause and a `Process'` clause routing to
  per-peer close primitives (thread: `close()`→join; process: `close()`→wait).
  Concrete peer types, no type-var flow → clause, per the rubric.

## Grounded template + the foundation split (2026-06-07)

Studied the lair: the typed-handle + projective machinery **already exists** —
`fn infer_make_channel` (check.rs:10423) produces
`TypeExpr::Tuple([Parametric{head:"rust::crossbeam_channel::Sender", args:[T]}, Parametric{…Receiver, args:[T]}])`,
and `recv` projects `T` back out. The peer types mirror this exactly: a peer is a
`TypeExpr::Parametric{head:"wat::kernel::Thread'"|"…Process'", args:[I,O]}`. So
4.6 is **mirror existing machinery, instantiated for the peers** — not net-new
parametric typing.

BUT grounding found a gap: **`spawn-program'` has NO check-side inference today**
(only the legacy `spawn-program`/`-ast` appear in check.rs; the 4.5 prime is
runtime-only, exercised only by Rust-level tests). So the peer *type* must be
minted at check time before any verb can project from it. That is the riskiest
new piece → split it out and de-risk it alone:

- **4.6a-i — typed-peer FOUNDATION** (this strike). (1) Register
  `:wat::kernel::Thread'<I,O>` / `:wat::kernel::Process'<I,O>` as valid parametric
  type heads (mirror the `Sender<T>`/`Receiver<T>` registration). (2)
  `infer_spawn_program_prime` (NEW, mirror `infer_make_channel`) → reads the
  program-fn arg's `[I] -> O` signature, returns `Parametric{Thread'|Process', [I,O]}`
  keyed on the `:tier`. (3) `declared_type_name` for a peer `RustOpaque` reports
  its specific parametric type (not `:rust::opaque`, runtime.rs:5220) so the
  runtime `close'` clause matcher can dispatch. **Proof:** a check-side probe —
  `(spawn-program' :thread env prog)` infers to `Thread'<I,O>`; a deliberate
  type-misuse fails. Smaller, isolatable, de-risks the new inference before 4 verbs
  ride on it.
- **4.6a-ii — the four verbs** (3 intrinsic + `close'` clause) over the settled
  foundation. Ships the complete, useful peer surface; proven by a wat-level
  `send'`/`recv'`/`close'` round-trip.
- **4.6b — `select'`.** Heterogeneous multiplex over N peers → first ready + its
  value. Thread peers use crossbeam select; process peers use io_uring
  `comms::process::Select` (POLL_ADD+POLLHUP). Its own stone on the 4.6a
  foundation (stepping-stone: 4.6a's downcast + EDN-bridge helpers are what 4.6b
  reuses). `select'`'s own clause-vs-intrinsic classification runs through
  DISPATCH.md at 4.6b design time (it is projective over a heterogeneous peer set
  → almost certainly intrinsic; confirm then).

## `close'` consume semantics (resolve at strike)

`close()`/`wait()` take `self` (consume), but 4.5 wraps peers in ref-only
`Arc<ThreadOwnedCell<…>>`. `custodia.rs` has `OwnedMoveCell::take(op, span)`
(atomic move-once). Either represent peers in an `OwnedMoveCell` so `close'` takes
once, or add a `&self` shutdown (drop endpoints → EOF) + separate wait. Grounded
decision at strike against `custodia.rs`; small, downstream of the dispatch shape.

## Cadence

4.6a: this DESIGN → FM-2-bis probe (wat-level thread-peer round-trip via
`send'`/`recv'`/`close'`, RED at HEAD; + process-peer round-trip integration probe,
`--test-threads=1`) → BRIEF + EXPECTATIONS → spawn sonnet → SCORE vs own re-run
→ commit + push. Then 4.6b (`select'`).
