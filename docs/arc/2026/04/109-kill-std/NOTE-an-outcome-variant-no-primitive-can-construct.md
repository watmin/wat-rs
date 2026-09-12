# NOTE (arc 109 vocabulary) — an outcome variant NO PRIMITIVE CAN CONSTRUCT; the wall with a painted brick

**Filed 2026-09-11, builder-directed** (*"i don't want this forgotten"*), as the tracking home for a
gap found while striking `the-owner-faces-an-outcome` in `docs/excursus/2026/08/001-sns-sqs/`.
**NOT STARTED.** The builder's sequencing: after `grant` (the fifth owner-method sibling).

## Kin, and why this belongs in 109

`NOTE-io-boundary-outcome-enum.md` (this arc, 2026-07-23, builder directive) is the doctrine:

> every **FAILING** IO boundary … returns an Outcome enum

**`recv` complies with that law and is still broken, which is the point of this note.**
`:wat::kernel::recv` returns `(:wat::kernel::RecvOutcome :- [O])` — a matchable enum, five variants,
exactly as the doctrine requires. But **one of those variants is never constructed by the primitive
that returns it.** The doctrine says *return an outcome enum*; it does not say *every variant of that
enum must be reachable from the boundary that returns it*. This note is the gap between those two
sentences, and the law is incomplete without it.

★ **The proposed refinement to the doctrine, in one line:** an outcome enum is only a wall if every
variant is reachable from at least one primitive that returns the enum. A variant no primitive can
construct is a **painted brick** — it type-checks, it forces callers to write an arm, and that arm is
dead code that reads as coverage.

## The measurement

`:wat::kernel::RecvOutcome::TimedOut` is **never constructed as a `Value` in Rust.** Every occurrence
of it in `src/` is one of three non-constructing kinds:

| where | kind |
|---|---|
| `src/types.rs:1878` | the variant *declaration* (`EnumVariant::Unit("TimedOut")`) |
| `src/kernel/spawn.rs:1159`, `:1299` | **wat source inside Rust string literals** (test fixtures) |
| `src/runtime.rs:6656`, `:6690`, `:6859`, `:6860` | **generated AST templates** — `WatAST::Keyword` nodes, i.e. wat source being *built*, not an outcome being *returned* |

There is no `recv_outcome_timedout()` constructor beside `recv_outcome_message` / `_closed` / `_lost` /
`_shutdown` (`src/runtime.rs:~21965–22040`).

**The only thing that mints a `TimedOut` is wat code**: `:wat::service::call-by-deadline`
(`wat/service.wat:~3825`) maps `CallOutcome::DeadlineFired → RecvOutcome::TimedOut`, and it gets a
deadline by racing the peer against `:wat::kernel::after` inside `select`.

**So a bare `(:wat::kernel::recv peer)` blocks forever and can never return `TimedOut`.**

## Where it bites today

`:wat::service::owner-recv-loop` (`wat/service.wat`, added by
`001-sns-sqs/the-owner-faces-an-outcome`) is a wall-clock-bounded re-recv for `<S>/stop` and
`<S>/hibernate`. It dispatches on five `RecvOutcome` arms. Its `TimedOut` arm **cannot fire**, so:

- `StopOutcome::GaveUp` is reachable only while the peer keeps *emitting* something
  (`Malformed`/`Stopped`);
- **a genuinely silent service still hangs the owner forever.**

⚠ **Not a regression.** A blocked `recv` hung before that stone too; the stone converted four fatal
raises into faced values and killed the crash it was drawn for. What is false is the *headline
property*: "bounded by wall clock, never by an attempt count" holds **between returning outcomes** and
**not against silence**. That is why owed-ruling #1's gate closed its `.wat` half and left this open.

## Why it cannot be fixed at the wat level

The obvious move — race the owner handle against `after` in a `select`, the way `call-by-deadline`
does — **does not type-check.** The owner lineage handle is a `Thread` or `Process`; `after` yields a
`Peer`; `select` refuses the mixed vector.

⚠ **Attribution and confidence:** the refusal and its error text
(`peers[1] has wrong tier (expected Process)`) are **grok's report** from that strike's SCORE
(`the-owner-faces-an-outcome/SCORE.md`, §SELECT+TIMER DID NOT FIT THE OWNER HANDLE). **I did not
independently reproduce it.** The `TimedOut`-never-constructed measurement above IS mine and was
verified against `src/` this session. Whoever takes this stone should re-derive the select refusal
before designing around it — it is the load-bearing assumption, and this campaign's record on
unverified load-bearing assumptions is poor.

## The fix shape (not a design — a direction)

A **deadline-bearing recv for owner handles**: either

1. a `recv-by-deadline` primitive for `Thread`/`Process` handles that constructs `TimedOut` itself
   (which would make the variant reachable from a primitive and satisfy the refined doctrine above); or
2. `select` learning to mix an owner handle with a timer `Peer`, which makes the wat-level race legal
   and needs no new outcome constructor.

(1) is the smaller surface and the one that repairs the doctrine. (2) is more general and touches
`select`'s tier checking. **Either reaches `src/`.**

## The generalization worth sweeping for

⭑ **`TimedOut` may not be the only painted brick.** The census that found this one is mechanical and
cheap: for each variant of each outcome enum registered in `src/types.rs`, is there a Rust
constructor, or does only wat code mint it? Any variant with no Rust constructor and no wat minter is
**unreachable**, and every exhaustive match on it carries a dead arm. Running that sweep across
`RecvOutcome` / `SendOutcome` / `CallOutcome` / `ServiceEvent` / `ConnectOutcome` is the cheap half of
this stone and may find siblings — this campaign found five sibling-pairs in a single day, and never
once found the first instance to be the only one.

## Provenance

- Found: `docs/excursus/2026/08/001-sns-sqs/the-owner-faces-an-outcome/SCORE.md`
  (orchestrator's grading, §"the `TimedOut` arm is DEAD CODE").
- Scoped out deliberately: `.../a-give-up-has-no-form-for-attempts/DESIGN.md` §Scope, and its BRIEF's
  STOP-4.
- Doctrine it refines: `NOTE-io-boundary-outcome-enum.md` (this arc).
