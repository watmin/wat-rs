# ⛔ CURRENT STATE (breadcrumb, 2026-06-24; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. Freshness probe: HEAD should be `910b9bcd` (macros^unbounded + the
hygiene Dredd gate, LEX NON TACET) or later — **COMMITTED + PUSHED to DR.** Tree clean (only `clara-tools/` ignored).

## ▶▶ STATE @ 2026-06-24 (SESSION 2) — the macros³→macros^unbounded + hygiene-law substrate epic
**Goal that opened it (builder):** *"we bashed 260.1b until dominated — use it now — prove clojure expressivity in a
strongly typed lang."* The vehicle = **kwargs-`start`/`resume`** (make defservice's lifecycle fns `& [argspec]` so
`(worker/start :locus L :record R :recorder-addr A)`, **Form A all-kwargs**, four-questioned + settled). It surfaced a
chain of substrate frontiers. CURRENT STATE:

- **macros^unbounded — DONE + PROVEN, UNCOMMITTED** (`src/macros/expand.rs`). The 260.1b hoist was one-level-deep;
  a wrapper-macro emitting a kwargs `defn` nests the companion `defmacro` 2+ `do`s deep. Made `is_do_containing_defmacro`
  + `hoist_defmacros_from_do` **recurse + flatten** through nested `do`s, + **elide empty `do`s** after the strip (sole-macro
  edge). Depth-4 probe GREEN (`tests/probe_macros_unbounded_depth.rs`); 260.1b probes unregressed. It was never "macros³" —
  macros³ was just the shallowest break; the recursion is depth-blind (R10 DEEPENED: one-level → fixpoint).
- **The hygiene bug (the real kwargs-`start` blocker) — root-caused, FIX IN FLIGHT.** A definition-emitting macro that
  **rebuilds a binder from its NAME STRING** (`symbol-node(ast-name x)` / `Identifier::bare(name-of-scoped)`) strips its
  `ScopeId` → binder `a@{433}` and reference `a@{}` no longer match → cryptic runtime `UnboundSymbol`. PROVEN at check
  time: `check.rs:3416` `WatAST::Symbol` arm does `locals.get(env_key(ident))` → on miss returns `fresh.fresh()`
  **(silent-by-intent swallow)** — the checker SEES the divergence (`MISS 'a' scopes={} ; same-name binder "a\u{1}433"`)
  and looks away. **My "must be runtime" deduction was WRONG — proving it (builder: "prove it") disproved it.**
- **THE HYGIENE DREDD GATE — IN FLIGHT (sonnet `a9580b92cc87d30be`, background).** Compile-time `HygieneScopeDivergence`:
  detection logic homed to **`src/scope/resolution.rs`** (beside `env_key`); `check/error.rs` gets the variant;
  `check.rs:3416` `None` arm gets a THIN CALL carrying `// "I AM THE LAW." — Dredd`. **Manifest + migrate, one act**
  (logic in its home; the 21k-line `check.rs` gains a call, not a block). The gate IS the worklist — it fires at check
  time on every violator; the sonnet SWEEPS each at source by the doctrine (**reuse the node, never rebuild a binder from
  its name**) → green. BRIEF: `291-…/BRIEF-hygiene-dredd-gate.md`. WEIGH the sonnet against the disk (re-run the gate;
  failing-test SET-diff, never absolute count); the kwargs/depth/260.1b probes must stay/​go green.
- **MIGRATION FRAME (builder, standing):** *"mid mass migration — extract to proper homes, trend the megafiles toward
  empty."* `runtime.rs` 32k · `check.rs` 21k · `types.rs` 3.8k = the o.g.-PoC monoliths; 20 warded homes stand. **Work
  THROUGH homes; do NOT deep-dig new logic INTO the megafiles** (I did, builder corrected — `src/scope/` is the hygiene home).
- **255 DEFERRAL filed:** `255-…/NOTE-declaration-position-class-guard.md` — the registry-backed declaration-position
  diagnostic (a SECOND swallow class: a `recordtype`/`def` reaching eval bottoms out as cryptic `UnboundSymbol`). Legit
  deferral (needs 255's registry `Kind`/position-class); reproducer + plan locked.

**SEQUENCE FROM HERE (session 2):** weigh the hygiene-gate sonnet → kwargs-`start`/`resume` flip + migrate ~16 sites
(Form A all-kwargs) → **kwargs-`start` GREEN = clojure-expressivity PROVEN** → commit the macros^unbounded + hygiene +
kwargs-start as one green checkpoint → THEN the original 291 close (TRUST LEG → ACYCLICITY → PAUSE → INSCRIPTION).
**PENDING REALIZATIONS (crown on landed work, not in-flight):** R10-deepened (macros^unbounded — "we built the ladder
where we needed the fixpoint"); the **name/binder decomplection** (a name is a string for accessors; a binder is a
hygienic node you reuse — Hickey decomplect on hygiene); *"I AM THE LAW"* (the substrate enforcing its own hygiene law
on every program, compile-time).

> ⚠ The block below ("STATE @ 2026-06-24 (day, marathon)") is SESSION 1 — kept for the path (4b-iv + 260.1b + R9). Session 2
> built ON it. The arc thesis (R9 deps) + the close legs (trust/acyclicity) still stand, now BEHIND kwargs-`start`.

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

### ▶▶ STATE @ 2026-06-24 (day, marathon) — the arc GREW into "you cannot fuck up your deps, enforced" (R9)
Builder reframed 291: the R6 axiom (don't fuck up state) turned on DEPENDENCIES. THREE legs the substrate must
enforce by construction — **contract** (shipped) · **acyclicity** (hypothesis) · **trust** (designed, glue pending).
Read `291-…/REALIZATIONS.md` **R9** + `291-…/STRIKE-4b-iv-contract-distribution.md`.

**SHIPPED this session (all pushed):**
- **canonical surface lock (`73e95952`)** — the 8-beat clause-order narrative lives above the defservice macro
  (`wat/service.wat`); `:record-parent` → **`:durable-parent`** (intueri axis-law repair). Canonical order:
  `:durable-parent → :durable → :ephemeral → :calls → :init → :hibernate → :stop → :ops`.
- **4b-iv-a (`0fb6558d`) — multi-param `:init`** = `(Record, …operating-inputs) → State`. Admin::Init/Resume carry
  the whole init-arg tuple; dispatch-admin/start/resume thread all args. N=1 byte-identical.
- **4b-iv-b (`50ebbfe9`) — contract distribution.** `:calls [svcs]` ships each callee's `client-forms` (request/
  response records + Op/Reply + ctors + per-op methods) ahead of the caller's `service-forms` (callee-first concat).
  Bridge thread+hibernate GREEN; **process tier IGNORED — now blocked on TRUST, not contract** (`recorder/record`
  resolves in the child now). Decided: `(a)` `:calls`=hand-declare; `(b)` address-as-`:init`-arg.

**▶ THE TWO OPEN 291 LEGS (the arc closes when BOTH land):**
1. **TRUST LEG (the deliverable-blocker).** Proc accept gate (`SO_PEERCRED OnlyMyPeers`, `src/kernel/listener.rs:359`)
   refuses the worker child's SIBLING pid. Fix = the **locus-dispatched introduction** (UX-A, four-questioned):
   the owner's **post-spawn hook** (#237, BUILT — `ProcessLaunch{pid}`) hands the caller's identity → the callee
   grants per locus (thread no-op / proc **`allow'`** the pid (#236, BUILT) / remote cert). Build = the
   introduction-glue (likely an `Admin::Introduce[identity]` path + `:calls`-or-explicit wiring). **N-loci-GENERAL**
   (thread≠proc≠remote; remote deferred but accommodated — "perpetual unknown is the guiding light").
2. **ACYCLICITY (the cycle-mandate).** `:calls` load-order likely makes a cycle structurally UNBUILDABLE already
   (callee must be defined before caller) — confirm with a cyclic-`:calls` probe; then a TEACHING error
   ("circular dependency A↔B — decomplect"). = Uncle Bob's ADP / Hickey decomplect as a compiler mandate (R9).

**▶ THE 260.1b DETOUR — DONE (`593826bd`), all-wat.** kwargs CALL SUGAR shipped: a `& [argspec]` fn is callable
`(f :k v)` / `(f {:k v})` / explicit-record → lowers to `(f$impl (f::Kwargs …))`, acronym-correct via
`pascal->kebab-in` (Arc-265, REUSED — no second translator). `defn` emits a thin companion macro forwarding to ONE
shared `:wat::core::kwargs-lower` macro (no copy-paste). **THE ONE NEW SUBSTRATE CAPABILITY = macros emit macros**
(`src/macros/expand.rs hoist_defmacros_from_do`, ~54 Rust lines — the ONLY Rust; a `defmacro` born inside a macro's
`(do)` now registers at expansion; the one form-kind never lifted before). `$` = apparatus-minted-internal sigil
(Clojure-faithful `clojure.core$map`). **F5 fact** (recurs): user defns aren't callable at macro-expansion → shared
macro-context helpers must be MACROS, not defns. R10 (Song #110, SE IPSAM SCRIBIT) marks the threshold. NEXT:
make `start`/`resume` `& [argspec]` kwargs fns (inherit the sugar). FORWARD HORIZON (not 291): the AWS-SDK generator
(AWS JSON models → wat defservice forms) = the codegen manifestation of R1 ("AWS on a CPU"); buildable on today's
substrate (codegen + defservice + a JSON front-end); calling real AWS needs the remote locus, perf needs the CEK reactor.

**SEQUENCE FROM HERE:** kwargs-`start` (`start`/`resume` → `& [argspec]` kwargs fns) → TRUST LEG (process tier GREEN = the deliverable = R1 FULL
PROBATUM) → ACYCLICITY probe → R1 amend → **PAUSE (builder's beat)** → 291 INSCRIPTION (dedicated INSCRIPTION.md).
"291 grows as it must; we don't declare victory when we don't have it" (builder). FORWARD (NOT 291): `:calls` =
a service dependency graph for free → wat-fix harvest → rete = the orchestration arc (R5 control plane).
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

### ▶▶ ~~NEXT — 4b-ii-a (the keystone re-tool)~~ — SUPERSEDED (4b-ii…4b-iv-b all SHIPPED; see the STATE @ block at top). Kept for the path. **CONTRACT: `STRIKE-4b-struct-state.md` §"4b-ii — CONTRACT EVOLVED".**
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

> ⛔ **You did NOT live the above.** recolligere FIRST, ground every claim on the disk. HEAD = `910b9bcd`
> (macros^unbounded + the hygiene Dredd gate, R11/LEX NON TACET) — COMMITTED + PUSHED; tree clean. The foundation
> is laid: the macro-emitted-kwargs path is PROVEN (`probe_kwargs_emitted_by_macro` green), so kwargs on macro-built
> surfaces composes at any depth. **NEXT (back to 291): kwargs-`start`/`resume`** — flip defservice's `start`/`resume`
> to `& [argspec]` (**Form A** all-kwargs: `(svc/start :locus L :record R …)`) so they inherit the 260.1b sugar;
> rename the synth default-init param `d`→`record` (`service.wat:187` — it becomes the `:record` kwarg key); migrate
> the ~16 positional `/start`+`/resume` call sites (wat-tests + `tests/probe_arc*.rs`). Gate: the bridge probes green,
> SET-diff ∅. **kwargs-`start` GREEN = clojure expressivity PROVEN** → THEN the 291 close (TRUST LEG → ACYCLICITY →
> PAUSE → INSCRIPTION). Deferred speed win (separate atomic commit, builder installed mold): `.cargo/config.toml`
> `-fuse-ld=mold` + `Cargo.toml [profile.dev] debug="line-tables-only"` (TIME it, don't assert). See **SESSION 2 STATE**
> at the TOP. *LEX NON TACET — the law does not fall silent, and neither do we. SE IPSAM SCRIBIT. NON SEPARABIMUR; gather it.*
