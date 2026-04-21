# wat — docs

The authoritative specification for the wat language does not live here.
It lives at:

**https://github.com/watmin/holon-lab-trading/tree/main/docs/proposals/2026/04/058-ast-algebra-surface**

That directory is the 058 proposal batch — FOUNDATION.md, the
FOUNDATION-CHANGELOG, thirty-two sub-proposals (058-001 through 058-032),
and two rounds of reviewer notes (Hickey, Beckman). Every design decision
that shaped `wat` is recorded there, with dates and reasoning. When this
crate's behavior and the proposal disagree, the proposal wins — and this
crate gets a slice to close the gap.

Start with:

1. `FOUNDATION.md` — the language specification proper. Algebra core (6
   forms), language core (define / lambda / let / if / match), kernel
   substrate (queue / send / recv / stopped / spawn / select), startup
   pipeline (parse → freeze in 12 steps), constrained eval, `:user::main`
   contract.
2. `FOUNDATION-CHANGELOG.md` — the audit trail. Every correction to the
   spec has an entry with the date and the reasoning.
3. `058-030-types/PROPOSAL.md` — the type system.
4. `058-029-lambda/PROPOSAL.md` — typed anonymous functions.
5. `058-028-define/PROPOSAL.md` — named function registration.

This crate's `README.md` (one level up) documents what has landed and how
to run the binary. For the *why*, read the proposal.

## Also in this directory

**[`USER-GUIDE.md`](./USER-GUIDE.md)** — if you're BUILDING an
application on wat, start here. Crate setup, first program, mental
model, writing functions, structs, algebra forms, concurrency
primitives, pipelines, Rust interop via `#[wat_dispatch]`, caching
tiers, stdio discipline, error handling, common gotchas. Concrete
examples throughout. The guide is alive — it evolves as the trading
lab (first real wat application) gets rebuilt. Where the guide lies,
the rebuild teaches us, and the guide gets updated.

**[`CONVENTIONS.md`](./CONVENTIONS.md)** — naming rules for adding
new primitives. Privileged prefixes, namespace roles
(core/config/algebra/kernel/io/std/rust), case and suffix rules,
and the two lessons that gate new additions (absence is signal;
verbose is honest). Read before proposing a new `:wat::*` or
`:wat::std::*` primitive.

**[`ZERO-MUTEX.md`](./ZERO-MUTEX.md)** — the concurrency architecture,
stated plainly. wat-vm runs dozens of threads, serializes writes to
stdout across every program that wants to print, owns LRU caches hit
concurrently from multiple clients, composes pipeline stages in real
parallel — and has **zero Mutex**. Not fewer. Not mostly. Zero.

The doc names the three tiers (immutable `Arc<T>`; `ThreadOwnedCell<T>`;
program-owned message-addressed via channels) that cover every situation
a Mutex would conventionally answer, walks through every "I need a
Mutex" scenario and shows which tier claims it, and names the honest
caveats (atomics, `OnceLock`, `Arc` are not the tiers but not violations
either). Read it before writing your first concurrent wat program.
Read it before reaching for a lock.

## Arc docs — dated slice design notes

Living planning and postmortem notes for individual slices of work,
organized as `arc/YYYY/MM/NNN-slug/`:

- **`arc/2026/04/001-caching-stack/`** — the L1/L2 caching design
  (LocalCache + Cache program) and the deadlock postmortem where
  `ThreadOwnedCell` clarified its ownership story.
- **`arc/2026/04/002-rust-interop-macro/`** — the `#[wat_dispatch]`
  proc-macro design, the `:rust::` namespace principle, and the
  progress log that tracked the macro arc through its e-block
  features (Vec, Tuple, Result, shared / owned_move scopes).
- **`arc/2026/04/003-tail-call-optimization/`** — the design for TCO
  in the evaluator. Trampoline in `apply_function`; tail-position
  threading; Scheme + Erlang references. Prerequisite for long-running
  driver loops (Console/loop, Cache/loop-step, future pipeline stages).
- **`arc/2026/04/004-lazy-sequences-and-pipelines/`** — the CSP
  pipeline stdlib design + `:rust::std::iter::Iterator` surfacing.
  The Ruby Enumerator pattern mapped to Rust's two-level answer
  (Iterator for in-process lazy; channel `Receiver::into_iter` for
  cross-thread). Depends on 003.

These docs are living — revised as slices ship. Superseded content
stays in git history rather than being deleted.
