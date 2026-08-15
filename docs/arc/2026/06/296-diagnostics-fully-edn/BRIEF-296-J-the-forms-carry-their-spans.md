# BRIEF — 296 J: the forms carry their spans

> Read `DESIGN-STONE-J-a-delivered-program-must-name-its-own-source.md` first. **One thing in it is
> now settled by measurement** — see the next section.

Baseline: HEAD `268da64d` + Wave A's uncommitted work. Floor **4529 run / 4528 passed / 1 failed /
154 skipped**; the single red is J's own test.

## ⛔ THE FORK IS CLOSED — measured, and the stone's recommendation was wrong

The stone offered three mechanisms (forms carry spans · deliver source text · carry the source
identity) and recommended the third. **That was drawn before measuring what the delivery surface
actually carries.** `wat/spawn.wat:306` settles it:

> *"Process clause: prog is a `(:wat::core::forms ...)` block — a forms-server program
> (`Vector<wat::WatAST>`) for the forked child universe."*

`prog` is **`Vector<WatAST>`**, not source text — and **every `WatAST` variant carries a `Span`**
(`WatAST::Keyword(path, span)`, `IntLit(n, span)`, and so on; `edn_to_watast_with` proves it by
stamping one on each).

So the parent **already holds the spans**. There is no source-tracking to add and nothing to
synthesise:

- **Deliver source text** — impossible in general. The surface is `Vector<WatAST>`; forms may be
  built programmatically and have no text.
- **Carry the source identity** — a lossy approximation of information already present per node.
- **The forms carry their spans** — *stop dropping a field the value already has.*

**This is stone G's ruling one layer out.** `AggregateValue` held its field names and the renderer
re-derived them; `WatAST` holds its spans and the round trip discards them. Same defect, same answer:
carry it.

## THE WORK

Two functions, one round trip:

- `watast_to_edn` / `watast_to_edn_with` (`src/wat_edn_bridge.rs:289`, `:315`) — **encode the span.**
  Its doc at `:287` currently says spans are unnecessary because *"`startup_from_forms` / `freeze`
  re-derives what it needs from the semantic structure."* True for resolution, false for execution.
  Rewrite that doc with the corrected boundary.
- `edn_to_watast_with` (`:420`) — **decode the span** instead of stamping `rust_caller_span!()`. Its
  doc at `:409` (*"Span is not preserved"*) becomes false and must change with it.

### The encoding is yours to choose, and it is an optimisation, not a fork

A span is `(file, line, col)`, and the file string repeats across every node of a program. A naive
per-node `#wat.core/Span {…}` triples the wire. An origin table (file interned once, nodes carrying
`line`/`col`) is compact and equivalent.

**Pick on measurement, not taste**: encode one real program both ways and compare byte size. Report
the number. Either is correct; only one is cheap.

### What an absent span must do

A form built programmatically in Rust genuinely has a `rust_caller_span!()`. **Carry that faithfully**
— it is the truth about where the form came from. The rule is *preserve what is there*, never
*invent something better-looking*.

## THE GATE — prove BOTH directions, and prove the payoff

1. **The payoff.** `probe_supervisor_select_lost::select_prime_yields_lost_when_process_child_crashes`
   goes green with `:location` naming the child's own crash site, not
   `src/wat_edn_bridge.rs:442:38`. That test is the reason this stone exists; its golden already holds
   the correct expectation and **must not be recaptured to match new output** — it is the oracle.
2. **Round-trip, both ways.** A probe that encodes a spanned `WatAST` to EDN, decodes it back, and
   asserts the span survives **identically**. Delivery and return are separate trips; proving one says
   nothing about the other (stone J, STOP-2).
3. **A negative control.** The same probe must FAIL if the encode is removed — otherwise it proves
   nothing (`[[feedback_a_green_test_can_prove_nothing]]`). State how you verified it can fail.

## STOP TRIGGERS — rejections. Report and leave the site.

- **STOP-1 — a span that is PLAUSIBLE rather than TRUE.** Defaulting to the entry file, synthesising
  line 1, reusing the parent's span for a node that has none. A believable wrong location is worse
  than an absent one — that is the `field-N` lesson, one boundary out, and it is this stone's whole
  subject.
- **STOP-2 — the wire format change breaks a consumer you did not expect.** The forms encoding is
  read by more than `spawn-program`. Enumerate the readers before declaring the class closed
  (stone J, STOP-3).
- **STOP-3 — goldens move that are not about spans.** This stone changes span carriage only. A golden
  whose *content* shifts is a second effect and a finding; report it rather than recapturing it.
- **STOP-4 — the round trip cannot preserve a span for some `WatAST` variant.** Report which variant
  and why. Do not special-case it into a fabricated span.

## BLAST RADIUS

`src/wat_edn_bridge.rs` (the two functions and their docs), plus whatever the compiler and the floor
name downstream of a changed wire encoding. The `select_prime_yields_lost` golden is the **oracle**,
not a target. No `.wat` corpus changes expected — report it if that turns out false.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D
warnings` (0), then `scripts/floor.sh` and read the **Summary line** — never a piped exit code.

Expect **`1 failed` → `0 failed`**, plus your new round-trip probe. Report the arithmetic.

**On any red you did not intend: do NOT re-run.** Copy the failing test's whole stdout+stderr block
verbatim — never a `| head` window — name the exact assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Run
every build and test in the FOREGROUND and block on it; a rider on this arc already lost a flight to
exactly that. Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first.

**The working tree holds Wave A's uncommitted work — 105 recaptured goldens and 109 lifted ignores.**
Do not revert it, do not touch it, and do not re-ignore anything.

Report: the encoding you chose with its measured byte comparison, both round-trip proofs including how
you verified the negative control can fail, the floor Summary line verbatim with the arithmetic, every
STOP, and the honest deltas — especially anywhere this brief did not match the disk. Every rider on
this arc has found a defect in the orchestrator's brief; this one already corrects its own stone.
