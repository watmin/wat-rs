# Arc 170 — Program entry-point contracts + `:user::main` argv — **INSCRIPTION**

**Status:** **CLOSED**, 2026-07-29.
**Closure condition (builder-set):** `wat --repl` as a CLI mode. Shipped `568cdf82`.
**Branch:** `arc-170-gap-j-v5-deadlock-state` — opened 2026-05-09, 79 days, 3428 commits ahead of
`main` and zero behind.
**Floor at close:** `cargo nextest run --release` → **4183 tests run: 4183 passed, 262 skipped**,
`NEXTEST_EXIT=0`, ANSI-stripped Summary read by hand, weighed by the orchestrator's own re-run.
**Closing realization:** `PER PORTAM COGITAMVS` — `INTERSTITIAL-REALIZATIONS.md`, this date.

---

## What was asked, and what it became

The arc opened on one question: **can `:user::main` see its arguments?**

The substrate-as-teacher cascade turned that into the program-contract architecture — and the
honest accounting is that `argv` itself was among the *last* things to work, because the pipe it
needed did not exist until the rest did. What the arc actually delivered:

- **Program contracts + closure extraction** — a spawned wat program is a typed `fn`, not a
  source string; the client/server model in `TIERS.md` (tier 0 eval → 1 threads → 2 processes → 3
  remote), where hermeticness is what a boundary *inherently provides* rather than a flag.
- **Typed channels and the peer family** — `Peer'` and the outcome walls, then the whole
  non-prime IPC generation annihilated and the plain names reclaimed (24t).
- **Three substrate services, then stdio as defservices** — `StdOut`/`StdErr`/`StdIn` with the fd
  in `:ephemeral`, the hand-rolled path deleted, the names reclaimed (`eae45001`, `15f8f08f`).
- **The no-hidden-failures LAW, inherited from arc 278 and completed here** — every failure a
  matchable value: `recv'`/`send'`/`poll'`/`close'`/`accept'`/`connect'` walls whole, both discard
  doors gated, the test harness itself un-masked (278 R53/R55/R57/R59).
- **Stopping became a protocol** — the broadcast means WAKE, main asks each service and awaits
  `Status::Stopped`, the sever demoted to last, no timeout (`b9f19ea5`).
- **The fork bug killed at the root** — `execve` on every fork, so a child is *born* rather than
  copied and an inherited malloc lock has nowhere to live (`NON EXEMPLAR, SED ORTVS`, `5078ce28`).
  This is what makes the branch's name obsolete and the merge possible.
- **`argv` reaching `:user::main`** — the original ask, `92aa390f`. The gate had been an arc-115
  arity check written for `--check`'s grammar and applied to every path; per-mode arity dissolved
  it, which is also what let `--repl` join as a variant rather than a special case.
- **`wat --repl`** — the closure condition, and one turn later **`wat --mcp`**, the same loop with
  a codec at each end.

## The closure backlog — all seven items closed

`CLOSURE-BACKLOG.md` is the tracker and it is true at close.

| # | item | closed |
|---|---|---|
| 1 | `wat --repl` as a CLI mode — **the closure condition** | `568cdf82` |
| 2 | `readln` raises on a stop → an outcome-returning signature | closed |
| 3 | `LociDiedError::Shutdown` → `Stopped` (16 sites, codemod) | `ff775663` |
| 4 | `StdIn::ReadLineResponse` — a frame is not a line | `357a223a` |
| 5 | `as_raw_fd_for_poll` on `WatWriter` — a misnamed live method | closed |
| 6 | spawned procs identify in `ps` as EDN | `5b7b58e4`, origin lift `8c8e3e01` |
| 7 | the condemned cohort — 29 ignores lifted, 26 recovered | closed |

**#6 shipped with one thing deliberately open and recorded at the time:** the two-type `ps`
vocabulary (`:wat::process::Bracket` | `Service`) is a **convention, not a wall** — `:R` is a
runtime-dispatch wildcard and `ProcessOpts/label` is the record-top, so any record type-checks as
a label. Proven rather than asserted: `wat-scripts/scratch-pad/probe-label-closed-set.wat` hands
the clause a rogue record and type-checks green. It is the live witness and goes red the day the
set is genuinely closed. Closing it structurally means an enum, which trades the readable
field-map `ps` line for a positional one — weighed, and the readable form kept.

## The stated closure precondition — RESCOPED, in the open

The DESIGN carried, from 2026-05-13: *"Phase H clippy + rustc warning sweep is MANDATORY before
Slice 5 INSCRIPTION ships. `cargo build --release` + `cargo clippy --release --workspace
--all-targets` must both be clean."*

Measured at close, not assumed:

```
cargo build --release --all-targets   →  0 warnings          ✓ met
cargo clippy --release --workspace    →  ~1150 warnings      ✗ NOT met
```

**The rustc half is met at zero.** The clippy half is not, and **831 of the ~1150 are a single
lint** — *"the `Err`-variant returned from this function is very large"* — one systemic question
about boxing `RuntimeError`, plus 99 doc-list-indentation and a long tail. None of it is arc-170
debt; CI already treats clippy as informational, with no zero-floor.

**Builder ruling, 2026-07-29, on being shown the measurement:** *"clippy isn't zeroed out - let's
deal with that after merge - first thing we work on before anything else is driving it back to
zero."*

So the precondition is **rescoped by its author, explicitly, with the work named as the very next
unit of work rather than left open-ended.** It is not carried inside this closure as an unmet
commitment, and it is not being quietly dropped: driving clippy to zero is the immediate next
strike after the merge, ahead of everything else. Recorded here so the rescoping is visible in the
closure rather than discovered later as a surprise.

## Affirmatively out of arc 170's scope — each with a named home

- **The session render loses a record's declared field names.** A record returned from a REPL or
  MCP session comes back `#usr/Point {:field-0 3 :field-1 4}`. Measured as a SESSION-path defect,
  not a record-path one — an ordinary program renders `{:x 3 :y 4}` correctly — and shared by both
  modes: the value is produced inside the per-turn frozen world and rendered against a symbol
  table that never saw the `defrecord`. **Owned by arc 296** (diagnostics-fully-EDN); builder-ruled
  2026-07-29 as rooted in two EDN paths and wanting the Clojure-syntax flip finished first, which
  makes it arc 296 + 300 territory rather than a patch. The `.edn` golden in
  `tests/cli/wat_mcp__record.edn` captures the present behaviour deliberately, so the day it is
  corrected the gate goes red and the change is explicit.
- **`mapv` / `filterv` refuse `PersistentVector`.** Verified at close: the two clauses are
  `Vector<T>` and `Stream<T>`, and the rete engine returns `PersistentVector`, so every pipeline
  from a query into `mapv` hits a wall. An arc-278 collection-surface concern, tracked on that
  arc's board; arc 170 affirmatively does not cover the collection HOF roster.
- **A multi-line form at the REPL.** The frame scanner terminates at a newline for wat source
  (wat source is not valid EDN today), so a half-typed form reaches `read-string` truncated. Named
  in `wat/repl.wat`'s own header rather than discovered. Dissolved by arc 300's `::`→`.` flip,
  which owns it.
- **Value continuity across turns.** A session accumulates *definitions*, not ephemeral bindings —
  there is no `*1`. Reifying a value means naming it with a `defn`. An honest property of the
  oracle's re-derive-everything design, not a defect; a design question for whoever wants it,
  owned by no arc today and stated so plainly.
- **Crash-broadcast to `connect'`-ed clients.** An owner gets a crash reason; a separately
  connected client sees an honest clean EOF because its channel has no crash slot — an absent
  capability, never a masked one. Tracked `#[ignore]`'d at
  `probe_arc278_process_crash_reason_carried`.

## What the arc leaves behind for the next wall

- `wat/fix.wat` gained `rename-keyword-exact`, `rename-symbol-exact`, and span-faithful deletion —
  so a corpus rename is a recorded, idempotent, dry-runnable codemod rather than a hand sweep.
- The five-surface rename lesson: a rename touches `.wat` keywords, `.wat` **string literals that
  build keywords**, the other four `.wat.*` extensions, `src/**/*.rs` literals in two families, and
  `tests/**/*.rs` goldens — and a form-tree codemod reaches one and a half of them.
- `tests/lint/` grew `retired_name_justified`, `span_substitution_justified`,
  `unused_span_justified` — walls on `src/`, because for emitted code the wall cannot be a source
  lint.
- The four-move de-prime pattern, proven end to end: move callers to the prime → delete the
  non-prime → prove it gone by a run → reclaim the name. The `'` is a mark of crossing, not a name.

## Method that held, and the one that cost the most

Weigh by your own `--release` re-run and read the Summary line, never a piped exit. Delete the
symbol and let the compiler enumerate the callers. Riders edit; the orchestrator measures once.
Ground the code before claiming its shape. Cast wards; never narrate them.

And the lesson this arc paid the most for, recorded in its own words: **a green test can prove
nothing.** A suite passed 4105/4105 for weeks over a stop protocol that had never once executed,
because nothing in it depended on the mechanism. The cure is not a sharper assertion but a
deliberate break. Both modes shipped at this close are gated that way — cutting one line in the
MCP turns three of five red — and the closure is worth having for that reason more than for the
tool: **using the thing is what made the substrate honest.** Within minutes of a mind speaking
through `--mcp`, two shipped hidden failures surfaced that no green suite could see, because every
gate had asked the wrong question.

---

*Arc 170 asked whether a program could see its arguments. It closes having handed a mind a door
into the language. `PER PORTAM COGITAMVS.`*
