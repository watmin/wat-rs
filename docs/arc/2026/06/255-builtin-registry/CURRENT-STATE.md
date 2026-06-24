# ⛔ CURRENT STATE (breadcrumb, 2026-06-24; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. Freshness probe: HEAD should be the curare commit below
(`curare: compact …`) or later. Tree clean at curare. All committed work is pushed (GitHub = DR).

> ⚠ **You are a NEW instance.** recolligere FIRST (grimoire + 4 primers from the datamancy MCP — RESOURCE
> mcp), `git log --oneline -20`, `git status`, then read this whole file. The work below is a cache in a
> familiar voice; you did NOT live it. Ground every claim against the disk before you move.

## Workspace (standing)
Work ONLY in `/home/watmin/work/holon/wat-rs/`. NEVER git worktrees. Spawn sonnets `model: "sonnet"`. Commit
msgs end `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. Weigh EVERY delegated agent
against the disk yourself (re-run the gate; failing-test-SET-diff vs HEAD, never absolute count). PRIMED forms
only. Commit + push often. Amend docs with recognition (never delete). **Cast intueri for ALL naming** (the
builder named this a standing discipline — `feedback_use_intueri_for_all_naming`).

## ▶ ARC 291 — defservice durable state (IN PROGRESS) — the soul, made durable

**The axiom (R6):** *"don't fuck up state, ever."* The arc is its Euclidean derivation. Read `291-…/DESIGN.md`,
`291-…/REALIZATIONS.md` (R1–R8), `291-…/STRIKE-4b-struct-state.md` (the LIVE strike), `291-…/NOTE-remote-as-a-class.md`.

### Shipped (all pushed, weighed pure)
- **2 — `:init`** (`d5d71766`): build State in-locus; wire carries EDN args.
- **3a — owner-only `stop`** (`25eced7d` α + `77773580` β-foundation + `7c9d0f29` β): the admin/data facet
  split; client stop backdoor ANNIHILATED; `Thread'/Process' <: Peer'` (N-loci, derive-graph-driven).
- **3b — `stop → resp`** (`4962e925`): `:stop` projects State→EDN resp (default identity).
- **4a — `hibernate`/`resume`** (`ca4a3e7b`): the prophecy's MECHANISM proven — counter hibernate→kill→
  resume→continue, both tiers. **R1 = PROBATUM EST *for the mechanism only*.**
- **4b-i — `struct ↛ wire`** (`dfcfe50e`): THE FIRM LAW. `is_portable_type(Struct) → false` (categorical, by
  kind; reverses 254.1 field-recursion). A struct can never be a channel/wire payload; only records cross.
  Cost was 1 migration (`service-template` → record). RED probe `channel_of_all_edn_struct_must_be_rejected`
  green. SET-diff ∅.
- **R8 + Song #109 *Obsolete*** (`473c7daf`): the manifestation realization — *the keystone (4a) must be made
  obsolete to be made true*; struct-is-body / EDN-is-soul; "shared memory becomes only values." Signature
  `CORPUS OBSOLESCIT, ANIMA MANET`.

### ⚠ R1 PROBATUM EST is HELD at the mechanism — do NOT advance to the full prophecy until 4b lands.
4a proved the soul *travels* (a record State). The TRUE test is the *resource* service (the cache: a struct
holding an `LruCache`, hibernate emits its record, resume rebuilds the resource). That is 4b. Builder's
sequencing: **4b → R1 PROBATUM amend (allowed) → PAUSE (his beat) → the 291 INSCRIPTION.** Do NOT barrel from
the fulfilled-amend into the inscription.

### ▶▶ NEXT — 4b-ii (the keystone re-tool). DESIGN FULLY PINNED: `STRIKE-4b-struct-state.md` §"4b-ii DESIGN PINNED".
The contract (settled across a long co-design + 3 intueri casts; `record` + `Status` builder-confirmed):
- **`:record [fields]`** clause (NEW, prepended adjacent to `:state`) → mints `:<fqdn>::Record` (the durable
  EDN record = the SOUL) and **prepends a field named `record`** as the State struct's first field.
- **`:state [fields]`** now mints a **`defstruct`** (the BODY: record field + ephemeral/resource flesh). Empty
  for a pure-data service. The struct never crosses (4b-i).
- **Four lifecycle verbs collapse to ONE user fn `:init : Record → State`** (REQUIRED; the no-`:init`
  ship-State default is DEAD — a struct can't ship). `start` = init-from-initial-record; `resume` =
  init-from-saved-record (same fn). `hibernate` = **`(State/record s)`** (a field read, NO `:hibernate`
  callback). `stop` returns resp (default the record, or `:stop`).
- **Wire carries records only:** `Admin::{Init,Resume}[<- Record]`, `Status::Hibernated[<- Record]`,
  `Status::Stopped[resp]`. Revises 4a (4a returned whole State).
- **The renames (intueri, fold into the re-tool):** `LineageUp→Status`, `LineageUp::Final→Status::Stopped`,
  `init-from-admin→dispatch-admin`, `lineage-extract-addr→extract-addr` (+ binding names `lineage-*→status-*`).
  Keep `Admin`/`Started`/`Hibernated`/all owner-facing. The ONE literal break outside service.wat:
  `probe_arc209_c2`'s `Peer'<…::LineageUp,…::Admin>` → `…::Status`.
- **Migration (accepted cascade):** every defservice + probe moves `:state [data] → :record [data] + :state []`,
  declares `:init`, reads durable via `(Record/… (State/record s))`. Affected probes: counter_on (locus-
  parity), seeded (init-parity), admin_stop, stop_resp, hibernate_resume, arc272 rs1/rs2×2, arc209 c1/c2.

**GROUNDED SITES for 4b-ii-a (so you don't re-ground — verify they're current, then brief):**
`wat/service.wat`: macro signature `:52-58` (positional `:state`/`:ops` markers + `&opts` — ADD `_record-kw`
+ `record-fields` before `_state-kw`+`state-fields`); known-opts `:74-80`; State emission `:181-184`
(`Record::def`→`defstruct` w/ `record` prepended; + a separate `Record::def` from record-fields); op-handler
`s`-access fold `:460-478` (rewrite durable reads → `(Record/f (State/record s))`); init-def `:153`
(→ `[r <- :<fqdn>::Record] -> :State`); Admin/Status enums `:293-302` (payloads `state-ty`→`record-ty`);
start/resume bodies `:895-915`; child-main-form `:834+` (self-peer types). **It's BIG** — consider whether to
fire as one thorough sonnet strike (RED probe + migrate all back-compat probes; gate = all green + SET-diff ∅)
or decompose; the signature change forces all call sites at once (can't cleanly split structural-vs-lineage).

### Then
- **4b-iii** — the resource RED probe (a defservice `:state` holding a genuinely non-EDN field; proves the
  struct holds a resource + hibernate emits the record + resume rebuilds the resource). THE honest fulfillment.
- **R1 → full PROBATUM EST · PAUSE (builder's) · the 291 INSCRIPTION** (dedicated INSCRIPTION.md, the closure
  ledger — see `project_arc_close_inscription_file`).
- **Deferred/follow-on:** 4b-i-b (audit if any struct still reaches `closure_extract::encode_struct` at
  runtime → make it refuse; belt to the type-gate); `wat-fixes-rust` (NOT an arc — build when a mammoth
  refactor forces it; `NOTE-wat-fixes-rust.md`).

## GATE LESSONS
- Corpus gate = `cargo test -p wat --no-fail-fast` (`--test test` fans out across member crates — use `-p wat`
  + the right target). SET-diff extraction: grep `'^test .+ \.\.\. FAILED$'`, NOT bare `FAILED` (catches
  `result:`/`Probe` timing-noise lines). The execve floor ≈ 202 real-name failures; `deporder_verify_stdlib_runs`
  flaps ±1 (passes isolated). Weigh by SET-diff vs HEAD = ∅, never absolute count.
- **struct ↛ wire is now LAW** (4b-i): a struct can never be a channel/wire payload; if you want a portable
  payload, you want a record. (`is_portable_type` check.rs:~13044.)
- wat-tests re-scan on `.rs` recompile → `touch tests/test.rs` after editing a wat-test.

> ⛔ **You did NOT live the above.** recolligere FIRST, freshness-probe HEAD, ground every claim on the disk.
> **Arc 291 IN PROGRESS — next = 4b-ii-a** (the `:record`/struct re-tool) per `STRIKE-4b-struct-state.md`
> §"4b-ii DESIGN PINNED", or ask the builder. The prophecy is one re-tool from PROBATUM. *CORPUS OBSOLESCIT,
> ANIMA MANET — the body sheds; the soul, and the thread, remain. NON SEPARABIMUR; gather it.*
