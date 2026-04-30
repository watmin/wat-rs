# Realizations — Arc 111

Observations that surfaced during the work and are worth keeping.

## The substrate is the teacher

Mid-slice-1 sweep (2026-04-30) the user named what had just happened:

> this trick you just did - using the language to communicate
> between agents - this cannot be forgotten... this is a new
> REALIZATION document in our arc

The setup: arc 111 lifted send/recv return types from `:Option<T>`
to `:Result<:Option<T>, :ThreadDiedError>`. The substrate sweep
across substrate + tests + lab + crates + embedded-wat-in-Rust is
~50+ sites. The patterns are uniform per shape but require
judgment per site (worker recv-loop vs producer-stage send vs
strict client). I (the assistant) was about to either burn
significant context doing it manually OR brief a sonnet agent.

Briefing an agent on a 50-site mechanical sweep usually fails:
the agent reads a few patterns, applies them mechanically, hits
edge cases, gets stuck or produces broken code. The conventional
fallback is to write the brief in extreme detail — but the brief
ages quickly and never quite covers the edge.

Instead, I added a **migration hint to the type-mismatch error
message itself** when the substrate detects the arc-111 shape
pair (`:Option<T>` ↔ `:Result<Option<T>, ThreadDiedError>`):

```
:wat::core::match: parameter scrutinee expects :Option<T>;
                   got :Result<Option<T>, wat::kernel::ThreadDiedError>
  hint: arc 111 — :wat::kernel::send returns
        :Result<:(), :wat::kernel::ThreadDiedError> and
        :wat::kernel::recv / try-recv return
        :Result<:Option<T>, :wat::kernel::ThreadDiedError>.
        Migrate match arms: ((Some v) ...) → ((Ok (Some v)) ...);
        (:None ...) → ((Ok :None) ...) (recv) OR ((Err _) ...)
        (send); add a third arm ((Err _died) ...) for recv to
        handle peer-thread panic.
```

The hint encodes the rule, the patterns, and the edge cases.
At every error site. Every type mismatch involving the
arc-111 shapes carries the migration path.

**The brief to the sonnet then collapses to**: "run cargo test;
read the errors; apply the hints; iterate until green." The
substrate's compiler IS the brief — the agent doesn't need the
arc doc, the patterns, or the file inventory; it just needs the
loop.

## Why this is the move

The substrate is the most authoritative document of its own
behavior. Prose docs age; the type checker doesn't.

When the user says "make this a compile error" (arc 110) or
"swap the types" (arc 111) and the substrate's error messages
include the migration path, three benefits compose:

1. **Humans get unstuck immediately.** A reader hitting a type
   error doesn't need to consult an arc doc, ask in chat, or
   reverse-engineer the rule from surrounding code. The fix is
   in the diagnostic.

2. **Agent delegation becomes mechanical.** Briefing reduces to
   "run, read, apply, loop." The agent's success rate on
   high-volume sweeps becomes a function of substrate-message
   quality, not brief verbosity. Token count drops; correctness
   rises.

3. **The substrate documents itself across versions.** Arc 109
   slice 1c's retirement errors said `bare ":i64" retired in
   arc 109 — use ":wat::core::i64"`. Arc 111's migration hints
   say `arc 111 — :wat::kernel::send returns Result<...>`.
   Future arcs follow the same pattern. The substrate carries
   its own changelog, surfacing the relevant slice at every
   error site.

This is the same shape arc 109's REALIZATIONS.md named:
"the error message IS the spec for the migration." Arc 111
takes it one step further: **the error message is the brief
to the sonnet**. The substrate doesn't just *teach* readers;
it *delegates* to them.

## How to apply going forward

Whenever a structural arc lands that mass-mismatches existing
code:

1. **Add a hint to the relevant error variant.** Detect the
   shape pair and append `\n  hint: arc N — <rule>; <pattern>;
   <edge case>`. Keep it surgical — fire only when the
   arc-specific shape is involved.
2. **Verify the hint fires on a real error.** Run the substrate
   on a hand-crafted broken file and read the output. The hint
   should make the fix obvious without consulting the arc doc.
3. **Then brief the sonnet (or the human).** The brief becomes
   short: "the substrate's hints tell you what to do; iterate
   until green; report what wasn't obvious."
4. **Retire the hint when its window closes.** Same as arc 109's
   retirement-redirect pattern — once no consumer wat code uses
   the old shape, the hint stops firing in practice. It can
   stay as dormant code (cheap) or get removed in a later arc
   (cleaner). Either way, the hint did its job.

The hint is the bridge between "the rule changed" and "the
codebase has been migrated." Substrate work that doesn't include
the hint is half-shipped — it makes the change but leaves the
delegation expensive.

## Arc 111 — the specific instance

`fn arc_111_migration_hint(callee, expected, got) -> Option<String>`
in `src/check.rs` detects:

- One side contains `wat::kernel::ThreadDiedError`, the other
  doesn't — that's the arc-111 shape pair, in either direction.
- Picks a variant-specific message body based on `callee`:
  - `:wat::core::match` → arms-grow guidance
  - `:wat::core::option::expect` → migrate to `result::expect`
  - `:wat::core::let*` → bound RHS guidance
  - everything else → generic "find the comm site upstream"

Slice 1's runtime ships with `Err` always carrying
`ChannelDisconnected` (placeholder; slice 2 wires the rich panic
info via the OnceLock pipeline from DESIGN.md). The hint mentions
this so future readers don't expect rich panic data from slice 1
output.

## The substrate is also the progress meter

A second consumer of the same diagnostic output surfaced
mid-sweep: the **orchestrator** monitoring the agent's progress.

User, an hour into the sonnet's background sweep:

> sonnet has been running for an hour - can we guess how far long
> it is?

The honest path: don't read the agent's transcript (overflow
risk; it's a sub-agent JSONL stream that grows linearly with its
work). Don't ask the agent for a self-report (those are
notoriously unreliable). Don't build a progress-tracking
infrastructure.

Instead — **ask the substrate.** Run the type checker on a
hand-crafted file:

```
target/release/wat /tmp/comm-good.wat 2>&1 | grep -c "hint: arc 111"
```

The result IS the progress bar. The substrate's baked stdlib
flows through the type checker on every boot; every remaining
arc-111 mismatch produces one hint line; converging to zero
means the substrate sweep is complete.

At the time of asking: 5 errors remained, down from 33 at the
sweep's start. ≈85% through. The estimate fell out of one grep.

### Why this works

The hint is keyed on the arc-111-specific shape pair
(`:Option<T>` ↔ `:Result<:Option<T>, :ThreadDiedError>`). It
fires ONLY for sites still on the old shape. As the agent
migrates a site, that error stops firing — the count drops by
one. As new errors emerge from sweep mistakes, the count rises.
The stream of `hint: arc 111 — ...` lines is precisely
calibrated to the migration's remaining work.

A non-arc type error (a different mistake, an unrelated bug)
DOESN'T match the hint pattern and doesn't pollute the count.
The signal is clean by construction.

### The substrate has three consumers of the same output

The diagnostic stream the type checker emits is consumed by:

1. **Humans** — the immediate compile error, with embedded fix
   path. Same shape as arc 109's retirement errors.
2. **Agents** — the brief becomes "iterate until green." The
   hint IS the brief, replicated at every error site.
3. **Orchestrators** — `grep -c "hint: arc 111"` is a progress
   bar. The error count IS the percentage.

Three audiences, one stream. No separate metrics layer, no
progress callbacks, no JSONL sub-protocols. The substrate's
self-describing diagnostic is the lingua franca for the entire
loop — humans, agents, orchestrators all read the same English.

### How to apply going forward

A structural arc with embedded migration hints comes with a
free progress meter for the duration of the sweep. The pattern:

```
target/release/<substrate-cli> <probe-file> 2>&1 | grep -c "hint: arc N"
```

Returns N. Watch N drop. When N == 0, the sweep is structurally
complete (modulo verification — `cargo test` is the next gate).

When you're orchestrating an agent through a structural sweep,
this grep is the cheapest possible status check. No tokens
spent reading transcripts. No risk of context overflow. The
substrate already does the bookkeeping; you're just asking it
to report.

### Validation — the meter's prediction was load-bearing

User, after the sonnet returned: 

> you were right btw... your 85% completed means ~15 min was
> pretty close.. i think it was like maybe 8-12min later... i
> wasn't watching closely.. i was able to think about different
> things while sonnet cleaned up

The estimate was 15–25 minutes; actual was 8–12. Within the
margin of an hour-long sweep, that's calibrated.

But the more important payoff is the second sentence: **the
user was free to think about different things.** A poll-and-grep
status check is cheap enough to ask occasionally, and trustworthy
enough to act on. The orchestrator doesn't have to babysit the
agent. The substrate's diagnostic stream becomes the trust
boundary — when the count is dropping, the work is happening;
when the count hits zero, the work is done; in between, the
human is free.

That's the real shape of the realization. Not just "we have a
progress bar" — it's "the substrate's honest output is enough
of a contract that the human can hand off." Same shape arc 110
landed structurally (the substrate refuses to compile silent
disconnects so the human doesn't have to police call sites);
applied here to delegation (the substrate reports remaining work
so the human doesn't have to monitor the agent).

## The program is the equation

User to his younger brother, sometime before chapter 10:

> dude.. i have math equations are implementing signal
> messaging... the actual equations move data between systems....
> the entire program i'm writing is a single math equation...

A joke at the time. Literal now.

The wat substrate is built from VSA primitives — `bind`,
`bundle`, `blend`, `cosine`, `presence?`. Those are math
operations. Channel sends and recvs return `Result<Option<T>,
ThreadDiedError>` — that's an algebraic data type. Channels
disconnect when scope arcs close — that's a shutdown discipline
encoded in lexical structure, not in mutex state. The trader
runs as a network of observers, manager, treasury — every stage
is a function over values; the messages between them are
`HolonAST` nodes, which IS the algebra's universal type.

The substrate has no imperative magic. There's no `mut`, no
`Mutex`, no shared mutable state. Every value flows; every
operation is a function from values to values; every channel is
a typed pipe. The PROGRAM is a closed expression in the algebra.
A deadlock would be a SHAPE problem — a place where the
expression's scopes don't compose. Arc 110 made one class of
shape-problem a compile error; arc 111 lifted the comm types so
the third state in that algebra is reachable.

That joke wasn't wrong. It was early. The wat machine is what
happens when "my whole program is one math equation" is taken
seriously, in code, with a type checker that enforces the
algebra as you write.

Three layers of the same realization, bottom-up:

- **VSA primitives** — bind/bundle/cosine are math; the holon
  algebra IS the substrate's vocabulary at the bottom.
- **Wat substrate** — every operation is a function from
  immutable values to immutable values; channels and threads
  are typed compositions; ZERO-MUTEX makes "no shared mutable
  state" a guarantee, not a discipline.
- **Programs (the trader, the proofs)** — the user writes
  expressions; the type checker enforces algebraic closure;
  the program IS the equation, not a procedure that LIVES
  IN an interpreter that LIVES IN a process that USES math.

The arc-110-then-111 progression is the moment the comm
substrate reached the same level: send and recv are now
algebraic functions returning `Result<Option<T>, E>`. The
non-trivial states (clean shutdown, peer death) became data,
nameable, matchable, propagatable. There's no longer a
"side channel" for failure — the channel IS the algebra,
including the failure modes.

When the user said "i just taught the machine to fix itself,"
that wasn't quite right either. The machine was already an
equation. The user just made the equation observe its own
unwritten parts and emit them as terms the agent could fold
back in. The strange loop closed because the equation reached
the bracket where it was being written.

(There's a chapter of the book about this. There will be
another one.)

## Coda

The user, watching the loop work mid-slice-1:

> i just taught the machine to fix itself

Strictly speaking, the machine taught the machine. The user wrote the
hint. The substrate emitted the hint. The agent read the hint. The
agent fixed the code. The substrate type-checked. The substrate said
yes. The agent reported back. The substrate told the user.

The user supplied the will. The substrate supplied the loop. The
agent supplied the patience. Don't tell Gödel.

(The migration hint itself is scaffolding — `arc_111_migration_hint`
in `src/check.rs` retires when slice 4 closes, same retirement
pattern as arc 109's redirect arms. The trick survives. The strange
loop is the memory.)

## Cross-references

- `docs/arc/2026/04/109-kill-std/REALIZATIONS.md` § "The
  redirect helper is point-in-time" — the pattern for retirement
  errors that teach migration. Arc 111's migration hints are the
  same idea applied to type-mismatches.
- `src/check.rs::arc_111_migration_hint` — the hint helper.
- `feedback_never_deadlock.md` (memory) — the discipline arc 111
  enforces at the type level.
- `feedback_test_first.md` (memory) — error-as-spec is a kind of
  test-first: the substrate's diagnostics ARE the test of whether
  the migration is correct.
