# ⛔ CURRENT STATE (breadcrumb, 2026-06-24; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. Freshness probe: HEAD should be the breadcrumb commit
(`curare: night-pause …`) or later. **Tree is DIRTY at pause — uncommitted 4b-ii-a WIP (see the IN-FLIGHT
block below); do NOT discard it.** All COMMITTED work is pushed (GitHub = DR).

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

### ⚠⚠ IN-FLIGHT @ NIGHT-PAUSE 2026-06-24 — read THIS before touching the tree
- **The tree is DIRTY: uncommitted 4b-ii-a WIP** — `wat/service.wat` (the macro re-tool to the new
  `:durable`/`:ephemeral` struct-State surface) + 6 migrated `wat-tests/service-*.wat`. **Do NOT lose it /
  do NOT `git checkout` it.** The macro is **PROVEN sound**: `counter_on` + `seeded` pass BOTH tiers through
  the new surface. **REMAINING for 4b-ii-a:** migrate the ~12 `.rs` defservice-defining probes (they still
  write the old `:state` form → won't expand against the new macro), then full gate + SET-diff. Then
  4b-ii-b (lineage rename) → 4b-iii (resource probe). Contract: `STRIKE-4b-struct-state.md` §"4b-ii —
  CONTRACT EVOLVED" + `BRIEF-4b-ii-a.md`.
- **DETOUR SHIPPED (`6c9a351c`) — `wat-reader` leaf + real-parser test discovery.** A malformed `.wat` used
  to SILENTLY drop its tests (the hand-rolled lexer in `crates/wat-macros/src/discover.rs` diverged from the
  real parser). Now discovery IS the real parser (`crates/wat-reader` leaf = span+identifier+ast+lexer+parser;
  both `wat` and `wat-macros` dep on it, re-exported under old paths); a bad file fails LOUD with a
  `#wat.test/DiscoveryFailed {:file :path :line :col :error}` EDN tagged-literal (`compile_error!`,
  file·line·col precise). Hand-rolled lexer ANNIHILATED. Full story: `291-…/DETOUR-wat-reader-discovery.md`.
  GATE NOTE: trust **forced clean builds** (`cargo clean -p <c> && cargo build`) — cargo cache + stale
  rust-analyzer diagnostics whipsawed the weigh this session.

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

### ▶▶ NEXT — 4b-ii-a (the keystone re-tool). **CONTRACT: `STRIKE-4b-struct-state.md` §"4b-ii — CONTRACT EVOLVED" (the END of the doc).**
The contract evolved HARD in co-design 2026-06-24 (intueri cast + 6 refinements). Read the EVOLVED section —
the older "DESIGN PINNED" section above it is superseded (marked, kept for the reasoning path). Headline:
- **All-kwargs surface** `(defservice :fqdn & clauses)`, order-independent. Clauses: **`:durable [fields]`**
  (optional, → `:<fqdn>::Record`, the durable identity that CROSSES) · **`:ephemeral [fields]`** (optional,
  → `:<fqdn>::State` defstruct = `{durable + body}`, NEVER crosses) · **`:ops`** (required) · **`:init`**
  (`Record→State`; required IFF `:ephemeral` non-empty, else default `(State/new d)`, **macro-error** if a
  body exists and `:init` absent) · **`:hibernate`** (`State→::Record`, return type FORCED; default
  `(State/durable s)`) · **`:stop`** (`State→:Resp`, user-declared type; default the record).
- **Naming:** clauses name the AXIS (`:durable`/`:ephemeral`); types keep the KIND (`::Record`/`::State` — the
  4b-i wire-boundary law; intueri's `::Durable` OVERRIDDEN). struct field = `durable`. Hooks = uniform trio.
- **Fixed `start`/`resume` arity** — caller ALWAYS supplies the record (empty-durable → `(Record/new)`).
- **Composition:** the user's `:stop` `:Resp` type + helpers live OUTSIDE the block (normal top-level forms).
- **Decomposition (refined):** **4b-ii-a** = the macro re-tool to this surface + migrate ALL ~17 definers
  (incl. `probe_arc272_rs1_state_must_be_record` — premise INVERTS to "state is a struct"; rewrite+rename it).
  **KEEP internal lineage names** (`LineageUp`/`init-from-admin`/`lineage-extract-addr`) → renamed in **4b-ii-b**
  (mechanical fix-wat: `LineageUp→Status`, `Final→Stopped`, `init-from-admin→dispatch-admin`,
  `extract-addr`). Then **4b-iii** = resource RED probe (the honest fulfillment + the `:hibernate` override test).

**GROUNDED SITES for 4b-ii-a (re-grounded 2026-06-24 — verify current, then brief):** `wat/service.wat` (963 ln):
macro signature `:52-58` (today `_state-kw state-fields _ops-kw ops & opts` POSITIONAL — re-tool to `[fqdn & clauses]`
+ a clause-map fold, extending the existing opts-fold at `:74-111`); State emission `:181-184` (today mints a
**Record** `:wat::Record::def ~state-ty ~state-fields` — change to: a `Record::def` from `:durable` + a `defstruct`
`:<fqdn>::State` with `durable` field prepended); `:init` default + node `:130-153`; `:stop` proj `:159-174`;
Admin/LineageUp enums `:293-302` (payloads `ship-ty`/`state-ty` → record-ty); serve hibernate arm `:589-592`
(`(send' self (Hibernated state))` → `(Hibernated (State/durable state))`); `/hibernate` method `:752-764`
(ret `state-ty`→record-ty); `/resume` `:899-914` (snapshot `state-ty`→record-ty); start-params/body `:875-890`;
child-main-form `:834-849`; service-forms-def `:857-873` (emit BOTH the Record::def AND the State defstruct).
`defstruct` ctor = `Type/new` positional, accessors `Type/field` (grounded: `Launched/new`, `Bound/address`);
`classify_type_decl` (`types.rs:1620`) includes `defstruct` → State-struct splices like the Record does today.
**It's BIG + indivisible** (signature change breaks all defservice sites at once) → one thorough sonnet strike,
ride the compile cascade to zero, gate = all green + SET-diff ∅. NEXT ARTIFACT = draw 4b-ii-a's BRIEF.

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
