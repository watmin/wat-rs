# DESIGN — the crash surface is enumerated

Builder, ruling on A/B/C: *"C has been reasoned — we are trying to enumerate all possible ways to crash
services ... and make them [un]crashable."*

**Drawn 2026-09-12. NOT STRUCK.** The enumeration is the deliverable; the injectors are the stones it ranks.

## Why this, and why now

`the-store-says-what-it-deleted` closed owed-ruling #2 and left exactly one thing blocking the banked
−17.1 % `rt-store` patch: **§2d, the unknowable write** — `sqs.wat:745/779/810` and `1208/1241/1272`,
where a store call returns `Lost` / `Closed` / `TimedOut` and the handler cannot know whether the write
landed. The ruling on offer was *"add an `:Unknown` state"* (A) or *"keep querying"* (B). Both were
refused in favour of **C**, and the reason C won is a measurement:

```
healthy run (2026-09-12, mine)   inbox-lost=0  inbox-closed=0  inbox-timedout=0
chaos   run (2026-09-12, mine)   inbox-lost=0  inbox-closed=0  inbox-timedout=0   ack-retries=0
```

**The queue's only fault knobs are `drop-recv-bp` / `drop-ack-bp`, and both suppress the queue's REPLY
TO ITS CALLER.** Neither touches a store call. There is **no store-fault injector anywhere in the
harness**, so §2d has never fired — the earlier SCORE could only report it *"from the code, not from a
measurement."*

★ Adding `:Unknown` under those conditions would mint a state nothing can construct — precisely
`docs/arc/2026/04/109-kill-std/NOTE-an-outcome-variant-no-primitive-can-construct.md`, whose ink is six
commits dry. **You cannot rule on how a service should behave when it cannot know, until you can make it
not know.**

## ⭑⭑ The generalization, and it is the whole point

Arc 109's sweep asked *"can a **primitive construct** this variant?"* — 59 variants, **0 unconstructable**.
That sweep is closed and it was the right question. **This is the next question, and it is strictly
harder:**

> **Can the HARNESS make this variant FIRE?**

Constructable ≠ reachable. A variant with a Rust constructor and no way to provoke it is a failure mode
we have written an arm for and never once executed. That is the gap the builder is naming: the ways a
service can be crashed are enumerable, and each one is either **exercised** or **unexamined**.

## The surface, counted from `src/types.rs` and `wat/service.wat`

**27 failure variants across 10 enums.** Success variants are excluded — they fire on every happy run.

```
RecvOutcome      5   Closed  Lost  Malformed  Stopped  TimedOut
SendOutcome      3   Closed  Lost  Stopped
TrySendOutcome   3   Closed  Lost  WouldBlock
ConnectOutcome   3   Failed  Refused  Rejected
AcceptOutcome    2   Closed  Failed
CloseOutcome     2   Failed  Signaled
SignalOutcome    1   Failed
GateOutcome      2   GaveUp  Gone
CallOutcome      4   Lost  Closed  DeadlineFired  Malformed        (wat-side, service.wat)
StopOutcome      2   Gone  GaveUp                                  (wat-side, service.wat)
```

⛔ **My own instrument needed FOUR corrections to produce that table, and each changed the answer.**
Recorded because the next person will reach for the same grep:

| # | defect | what it did |
|---|---|---|
| 1 | a fixed 40-line window | found **2** variants of `RecvOutcome`; the control (TimedOut **and** Malformed both present) returned 1 |
| 2 | widened to 120 lines | bled into the **next** enum — `Item` (from `NextOutcome`) attributed to `RecvOutcome` |
| 3 | ⭑⭑ matched the token `name: "X"` | **structurally blind to `EnumVariant::Unit("X")`, which has no `name:` field** — so `Closed`, `TimedOut` and every other nullary variant were invisible. `RecvOutcome` is **6**, not 2. |
| 4 | displayed the leaf name only | manufactured a phantom "duplicate `ReadFrameOutcome` registration" — they are two real, distinct enums (`:wat::kernel::` and `:wat::io::IOReader::`) |

★ **Defect 3 is "match the FORM, not the token" in its purest form**, and it is the reason this DESIGN
states 62 outcome/event variants where arc 109's sweep stated 59. Slice on
`env.register_builtin(TypeDef::Enum(EnumDef {` boundaries and match **both** `Unit("X")` and
`Tagged { name: "X"`.

## The one contract decision — what "reachable" means

> **Reachable = there is a command that can be run TODAY which makes the variant fire and be observed.**

Not *"a test asserts on it"* (weaker — an arm can exist unexecuted). Not *"possible in principle"*
(unfalsifiable, and the exact prose that let §2d sit unexamined for an arc). A cell is `FIRES` only with
the command beside it, or `UNREACHABLE` with the named reason nothing can provoke it.

## The four questions

**Obvious?** YES — a matrix of variant × can-we-fire-it, each cell carrying its command. **Simple?** YES
— one question asked 27 times; no code changes to the substrate. **Honest?** YES, and it is the stone's
purpose: it converts *"reported from the code"* into *measured*, in both directions. **Good UX?** YES —
it makes every later injector stone a ranked choice instead of a guess, and it makes the A-vs-B ruling on
§2d a measurement.

## Scope

**IN:** the 27 failure variants above · for each, a `FIRES` (with command) or `UNREACHABLE` (with reason)
· the ranked injector list the matrix implies · and **the store-fault gap named precisely**, since it is
the cell that blocks a banked win.

**OUT = REJECTED:**
- ⛔ **Building the injectors.** This stone ranks them; it does not build them. A stone that both
  enumerates a surface and rebuilds it braids the census with the cure, and the census is what the
  builder asked for. The first injector is the next stone, chosen **by the matrix**.
- ⛔ **Ruling §2d's `:Unknown`.** Explicitly the builder's, and it needs this matrix's store-fault cell
  first.
- **The IO-decode and math outcomes** — `Read*Outcome` (5 enums), `Vector/Cosine/Dot/Combine` (4),
  `FormOutcome`, `NextOutcome`. Real, 35 further variants, and a **different axis**: they fail on
  malformed *data*, not on a *service* dying. Named so the next reader knows the 27 is a deliberate
  subset of 62, not an undercount.

## Trap-doors

1. ⛔ **The census instrument.** Four corrections above. Re-derive the table with both variant forms and
   run the controls (`RecvOutcome` must be 6; `Closed` must appear; `Item` must attribute to
   `NextOutcome`) before trusting a single row.
2. **`Stopped` is not a failure on every enum.** `RecvOutcome::Stopped` means *"a stop was requested
   while this read was parked — NOTHING DIED"* (`src/types.rs:1877`), and `CloseOutcome::Closed` is the
   success. Read each variant's own comment before classing it.
3. **A cell that fires only under the `wat-tests` harness is still a FIRES** — but say which harness.
   A variant reachable only from a unit test and never from `circuit.wat` is a different fact from one
   the chaos run provokes, and the matrix must distinguish them.
4. **Do not count an arm as evidence.** 61 live `Malformed` placeholder arms exist; none of them proves
   `Malformed` ever fires. Arms are the question, not the answer.
5. ⭑ **Partial is permitted and must be reported as partial.** 27 cells is a lot. If the sweep runs long,
   finish `RecvOutcome` / `SendOutcome` / `CallOutcome` / `StopOutcome` (the four this campaign's own
   stones touched) and report the rest as **NOT SWEPT** — never as `UNREACHABLE` by default. A blank
   dressed as a finding is the one outcome that makes this stone worse than not running it.
