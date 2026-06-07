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
| `close'` | `peer -> nil` (thread) / `ExitStatus` (process) | **clause** | concrete peer types; per-clause concrete return; NO type-var flow (`I`/`O` flow nowhere) |

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

## Proactive split

- **4.6a** — the four verbs above (3 intrinsic + `close'` clause) over the
  parametric peer types. Ships a complete, useful peer surface.
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
