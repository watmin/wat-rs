# BRIEF — the crash surface is enumerated

**Read `DESIGN.md` beside this first.** It carries the contract decision (what "reachable" means), the
27-variant surface, and the **four corrections my own census instrument needed** — you will reach for the
same grep, so read that table before writing one.

## The work, in one paragraph

Arc 109 proved every outcome variant can be *constructed* by some primitive. Nobody has asked whether the
**harness can make one fire**. Take the 27 failure variants on the service-call surface, and for each
produce either `FIRES` with the exact command that provokes it, or `UNREACHABLE` with the named reason
nothing can. The output is a matrix and a ranked injector list. **You are not building injectors.**

## Method, per variant — four steps, in order

1. **Arms** — where is it matched? (`grep -o … | wc -l`, never `grep -c`.) Arms are the *question*.
2. **Drivers** — does any test, probe, knob or chaos arg drive it? Search `tests/`, `wat-scripts/`,
   and the argv knobs in `wat-scripts/fanout/circuit.wat`.
3. **Fire it** — attempt it. A scratch probe under `wat-scripts/scratch-pad/` (it must type-check; the
   `every_wat_scripts_file_loads` gate parses and type-checks every file there), or an existing
   command with different args. **Record the command verbatim.**
4. **Classify** — `FIRES` (+ command + what you observed) · `UNREACHABLE` (+ what is missing) ·
   `NOT SWEPT` (+ say so).

## The rooms

| where | why you are going there |
|---|---|
| `src/types.rs:1869` | `RecvOutcome`'s registration. ⭑ Slice on `env.register_builtin(TypeDef::Enum(EnumDef {` boundaries and match **BOTH** `EnumVariant::Unit("X")` and `EnumVariant::Tagged { name: "X"` — a Unit variant has **no `name:` field**, which is why my first pass reported 2 variants instead of 6. |
| `src/types.rs:1877` | `RecvOutcome::Stopped`'s own comment — *"NOTHING DIED and NOTHING CLOSED"*. Not every non-success variant is a failure; read each comment. |
| `wat/service.wat` | `CallOutcome` (5) and `StopOutcome` (3) — the wat-side half, not in `types.rs`. |
| `wat-scripts/queue/sqs.wat:202–203` | the queue's only two fault knobs, `drop-recv-bp` / `drop-ack-bp`. ⭑ **Read what they actually do**: they suppress the queue's reply to its caller. They do **not** make a store call fail. This is the shape to copy when an injector is eventually built. |
| `wat-scripts/queue/sqs.wat:745`, `:779`, `:810`, `:1208`, `:1241`, `:1272` | the six §2d arms — a store call returning `Lost`/`Closed`/`TimedOut`. **Measured zero in both my healthy and my chaos run.** The store runs on `(:wat::spawn::process)` (`circuit.wat:2930`), a separate OS process, so these are reachable *in principle*. Establish whether anything today can provoke them. |
| `wat-scripts/fanout/circuit.wat` | the argv knobs: `inbox-cap` (12), `chaos-bp` (13), `drop-check-bp` (14), `drop-mark-bp` (15), `disrupt-bp` (16). ⭑ `disrupt` is the one existing mechanism that produces a genuine lost/closed — read its header at `:379–392` (*"disrupt-hits — the poisoned call came back lost/closed: it TORE"*). Establish which variants it reaches. |
| `docs/excursus/2026/08/001-sns-sqs/the-owner-wait-has-a-deadline/` | `RecvOutcome::TimedOut` was made primitive-reachable here, with a probe that fires it. ⭑ That probe is a worked `FIRES` cell — copy its shape for the others. |
| `docs/arc/2026/04/109-kill-std/NOTE-an-outcome-variant-no-primitive-can-construct.md` | the closed construction sweep, including the four defects **its** instrument needed. Different question, same trap. |

## Deliverables

1. **`FINDING-the-crash-surface.md`** — the matrix. One row per variant: enum · variant · arms · status ·
   the command (or the missing mechanism). This is the artifact.
2. **The ranked injector list** — which injector buys the most unexercised cells per unit of work,
   ranked, with the §2d store-fault cell called out because it blocks a banked −17.1 % patch and a
   builder ruling.
3. **Any scratch probes you wrote**, committed under `wat-scripts/scratch-pad/` (they must type-check).

## Blast radius

**No substrate change.** `git diff --stat -- src/ wat/` must be **empty**. New files only: the FINDING,
and scratch probes. If you find yourself editing `src/` or `wat/`, you are building the cure instead of
the census — STOP (see STOP-2).

## Verify

- `./scripts/floor.sh`, read the **Summary line** → **5237 passed, 0 FAIL** (a probe under
  `wat-scripts/scratch-pad/` is type-checked by the corpus gate, so it can redden the floor).
- ⛔ **Never a piped exit code.** A type-error run in this session reported `$?` = **0** through a
  `| head`; its true exit was **3**, on stderr.
- Run everything heavy through `./scripts/capped.sh --limit 8g`.
- `cargo nextest run --release --no-run` — `cargo build --release` does NOT compile tests.

## STOP triggers

1. **STOP-1 — if a variant's classification depends on reading code rather than running something**,
   mark it `UNREACHABLE (unproven)` and say so. An `UNREACHABLE` asserted from prose is the failure mode
   that let §2d sit for an arc.
2. **STOP-2 — do not build an injector.** If a cell is `UNREACHABLE` and the fix is obvious, write the
   fix down in the ranked list and move on. This stone ranks; the next one builds.
3. **STOP-3 — if the census instrument's controls fail, fix the instrument before reporting any row.**
   The controls: `RecvOutcome` must have **6** variants · `Closed` must appear · `Item` must attribute to
   `NextOutcome`, not `RecvOutcome` · the two `ReadFrameOutcome` entries are **distinct enums**, not a
   duplicate registration.
4. **STOP-4 — partial is permitted, and must be labelled.** If you run long, finish `RecvOutcome` /
   `SendOutcome` / `CallOutcome` / `StopOutcome` and mark the rest **NOT SWEPT**. Never let a blank cell
   render as `UNREACHABLE`.

## Shape to copy

`the-queue-knows-its-own-depth/SCORE.md` §2 — the last census in this campaign that named a live path
from the code and said plainly it had not fired. And
`NOTE-an-outcome-variant-no-primitive-can-construct.md` §"the sweep this note asked for" — how to record
an instrument that needed correcting.
