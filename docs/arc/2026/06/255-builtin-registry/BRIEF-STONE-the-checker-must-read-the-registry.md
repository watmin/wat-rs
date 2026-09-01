# BRIEF — STONE: the checker reads the registry, and the type residue gets a ratchet

Two deliverables, one stone. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-checker-must-read-the-registry.md` — read
its § "THE ROOT", its § "AMENDED", and its § "The expected cascade".

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at
5115, HEAD `96a9e98d9`.

⚠ **This stone CHANGES BEHAVIOUR.** Every stone before it moved code verbatim. This one turns on a
check that has never run, so the corpus will meet it for the first time. **The cascade is the
orchestrator's to meet, not yours** — ship the gate, do not go hunting call sites to fix.

## Read in order

1. The DESIGN, whole. Its measurement is what you are implementing.
2. **`src/runtime.rs:5631`** — the arity enforcement you are mirroring at check time. Same
   predicate, same `ArityMismatch` shape. Read it before writing yours; do not invent a second one.
3. **`src/intrinsic/mod.rs`'s `checker_skip_debt_is_named_and_frozen`** — the bidirectional
   name-freeze your second deliverable copies. It is the working example of the exact gate shape.
4. **`src/check.rs`'s test-module `fn check(src: &str) -> Result<(), CheckErrors>`** (near
   `stdlib_loaded`) — the `OnceLock`-cached harness your probes drive.

## The work

### 1 — the checker consults the registry's declared arity

For every registered row the checker has **no `TypeScheme` for**, enforce `entry.arity` at check
time. `Arity::Exact(n)` and the call's argument count disagree → the same `ArityMismatch` the runtime
raises at `runtime.rs:5631`. `Arity::Variadic` imposes nothing.

⚠ **A row that HAS a `TypeScheme` keeps being checked by its scheme, and only by it.** The scheme is
strictly stronger and a second arity check in front of it is a second authority for one question —
the shape this arc exists to delete.

`src/check.rs` references `crate::intrinsic::registry()` **zero** times today. That is the root; this
is the line that fixes it.

### 2 — `FROZEN_TYPES_UNCHECKED` and its bidirectional gate

A second frozen list beside `FROZEN_CHECKER_DEBT_LEDGER`, holding the registered rows whose **types**
nothing checks. Measured — these eleven, from a behavioural probe:

```
:wat::core::fresh-symbol   :wat::core::struct-field   :wat::core::type-equal?
:wat::core::type-params-used-in   :wat::core::variant   :wat::kernel::peer-pid
:wat::runtime::metadata-of
:wat::linkedlist::get   :wat::linkedlist::length   :wat::linkedlist::empty?   :wat::linkedlist::contains?
```

**Each row carries its own wrong-typed call, DERIVED FROM ITS OWN `@arg` DECLARATION** — read the
row's declared arg type in its registration doc and pass an argument of a different type. The doc is
the source; do not guess a signature from the verb's name.

The gate drives `check(src)` per row and asserts **both directions**, exactly as
`checker_skip_debt_is_named_and_frozen` does:

- a row **not** on the list whose wrong-typed call is ACCEPTED → **NEW**, named, fail.
- a row **on** the list whose wrong-typed call is now REJECTED → **STALE**, named, fail until deleted.

Both messages name the offending row and say what to do. The list can only shrink.

⚠ **Measure by driving the checker, never by grepping for an `infer_*` arm.** A text predicate said
twelve rows were unchecked; the behavioural probe corrected it to eleven. The grep is the wrong
instrument and its error is already on the record.

## Blast radius

`src/check.rs` (the registry consult) · `src/intrinsic/mod.rs` (the list + its gate) · whatever the
compiler names. No `.wat` corpus change. No registrations. No verb changes behaviour at RUNTIME.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — A ROW WHOSE `@arg` TYPE IS FULLY GENERIC HAS NO WRONG ARGUMENT.** If a row's declared
parameter is `:T` or equivalent, no value is ill-typed for it and you cannot write its probe. **Do
not invent one, and do not drop the row.** STOP and report which rows, with their declared types —
that is a finding about what the residue actually contains, and it changes the list's meaning.

**⛔ STOP-2 — DO NOT CHASE THE CASCADE.** Turning on arity checking for ~71 rows against a corpus
never held to it will surface failures. **They are not yours to fix.** You cannot build, so you
cannot even see them. Ship the gate and stop. A rider that starts editing call sites it cannot test
is editing blind.

**⛔ STOP-3 — DO NOT WEAKEN THE GATE TO MAKE SOMETHING PASS.** If a row's sniffed arity appears to
disagree with how it is really called, that is the finding — the arity came off the handler's own
Rust signature and cannot drift from it. Report the row and the disagreement. Do not special-case it,
do not add it to a skip list, do not relax `Exact` to `Variadic`.

**⛔ STOP-4 — THE TWO LISTS ARE NOT THE SAME LIST.** `FROZEN_CHECKER_DEBT_LEDGER` means "no
`TypeScheme`" (71 rows, its own W4 correction: *"no TypeScheme, not unchecked"*).
`FROZEN_TYPES_UNCHECKED` means "nothing checks the types at all" (11 rows) and is a strict subset.
Do not merge them, do not derive one from the other, do not re-word either's criterion.

**⛔ STOP-5 — YOUR PROBE MUST BE ABLE TO FAIL.** Before you finish, confirm each direction of the new
gate fires: drop a name and see NEW; add a name that does reject and see STALE. ⚠ And confirm the
probe is not vacuous — a scaffold with NO call must not produce the same error your probe reads as a
rejection. The first draft of this measurement used a bare `()`, retired by arc 179, and every
"rejection" was `BareLegacyUnitValue` about the scaffold. Report both sabotage results.

**STOP-6 — one authority per question.** Do not add an arity check for rows that have a scheme.

## Report

Per-file diff summary; the arity-consult code verbatim and the `runtime.rs:5631` lines it mirrors;
the eleven wrong-typed calls with **the `@arg` declaration each was derived from**; the two sabotage
results from STOP-5 (with the messages the gate printed); any row that tripped STOP-1 or STOP-3. Then
the part the orchestrator cannot reconstruct: **what surprised you** — a row whose declared arity
looks wrong, a row whose doc type could not produce a wrong argument, or a place where the checker
already had a registry-shaped answer and did not use it.
