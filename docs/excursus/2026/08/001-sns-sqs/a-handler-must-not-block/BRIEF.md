# BRIEF — a handler must not block

Make `:fanout::worker` stop blocking inside its handler, using what the substrate already affords — or
report precisely what the substrate is missing. **Both outcomes are the deliverable.**

**Read in order:**

1. `docs/excursus/2026/08/001-sns-sqs/a-handler-must-not-block/DESIGN.md` — the confirmed measurement,
   and the half of the mechanism that already exists.
2. `wat-scripts/fanout/circuit.wat:471` — the blocking call, inside `:fanout::worker`'s tick handler:
   `Queue/receive … :wait (Wait::UpTo (Milliseconds 250))`. **This is the site.**
3. `wat/service.wat:1542` and `:1564` — *"client op is RE-TAGGED into its `<service>::Op` counterpart at
   the Message arm"*, and `selectable-peer-ty: (Peer :- [proto::Reply service::Op])`. **The inbound half
   already exists: a peer's reply can arrive as one of the service's own ops.** Your first job is to
   establish how a peer gets into that set and whether an author can reach it.
4. `wat/service.wat:72-92` — `:wat::service::Directed` is `[conn-id, reply]`, and `Outcome::Continue`
   carries `state` / `reply` / `sends` / `arms`. **Read this to confirm the outbound half is missing** —
   nothing here issues a *request* to a peer.
5. `wat-scripts/scratch-pad/probe-self-scheduling-loop.wat` — the in-tree disconfirming probe for
   *"select over a mutating peer-set delivering a shared Op enum, insert a timer mid-loop"*. It proves
   the loop shape composes; copy its method if you need a probe of your own.
6. `wat/service.wat`'s recognized clause list — `:durable :ephemeral :ops :init :hibernate :stop
   :durable-parent :satisfies :impls :peers :max-frame-bytes :deadline-ms`. **There is no
   `:selectables` clause**, so whatever populates that set is internal. Find out from what.

**Your first act is a probe, not an edit.** Establish whether a handler can issue a request to a
declared peer and return, with the reply arriving as an op. Write it small, put it in
`wat-scripts/scratch-pad/`, and let the type checker answer. That question decides everything after it.

**Blast radius, as a property:** if the affordance exists, `wat-scripts/fanout/circuit.wat` should be the
only file that changes. **If you find yourself editing `wat/service.wat` or `src/`, STOP** — that is
STOP-2 and it means the answer is "the substrate needs an addition", which is a finding for the builder,
not work for you.

**The measured guard — the fix must not become a busy poll.** `Wait::Immediate` was measured on this
workload and is NOT the answer: correctness held and `store-calls` rose only 1.5 %, but it makes one
worker responsive while leaving the defect everywhere else, and its cost depends on the workload. So:

- `store-calls` must not rise materially against baseline
- `drain` must not regress
- `distinct = n×m` and `dup = 0` at every size

**STOP-1** — if the substrate has **no** affordance for a non-blocking peer request, **STOP and report
exactly what is missing from the outcome surface.** Name the form you would need and why the existing
four fields (`state`, `reply`, `sends`, `arms`) cannot carry it. **That is a complete and valuable
result** — it is the arc-shaped finding, and opening an arc is the builder's ruling, not yours or mine.

**STOP-2** — if the change requires touching `wat/service.wat` or `src/`, **STOP before the first edit
there** and report what forced it. Same reason as STOP-1.

**STOP-3** — if making the worker non-blocking loses messages or changes `distinct`/`dup`, **STOP and
report with the run.** Correctness outranks every millisecond in this stone.

**STOP-4** — on any red in the floor or corpus gate: do **not** re-run. Capture whole, name the exact
arm, surface it. ⛔ From earlier today: a chaos run piped through `grep` lost its failure text, and the
re-run went green, destroying the only evidence.

**Measure before and after**, `2000 4 3 8192 true 1000` ×3 each and `20 4 3 8192 true 1000` ×3 each (the
second is where the per-worker cost is cleanest), box quiet, reporting every phase plus `store-calls`.

**Run everything heavy through `./scripts/capped.sh`** (`--limit 8g`). `.wat` is edited with an editor.
Scratch `.wat` goes in `wat-scripts/scratch-pad/`. **Read the floor's Summary line, never a piped exit
code.** Leave everything uncommitted.

**Write your SCORE** to `docs/excursus/2026/08/001-sns-sqs/a-handler-must-not-block/SCORE.md`, graded row
by row. Copy the shape of
`docs/excursus/2026/08/001-sns-sqs/the-drain-gives-up-on-a-stall-not-a-budget/SCORE.md`.
