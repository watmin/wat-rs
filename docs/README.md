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
