# Intueri Findings — the minted name families (`defservice`, variant accessors, spawn builders)

**Spell:** intueri (datamancy grimoire, fetched by the caster via `mcp__datamancy__fetch_spell`;
first 6 lines verified byte-identical against the orchestrator's own signed copy)
**Target:** a family of names — `wat/service.wat` (defservice mints), `src/declare/register.rs`
(variant + aggregate accessor mints), `wat/spawn.wat` (255.14 reference point), and the general
class "lowercase (namespace) parent joined to a member".
**Tree:** `main` @ `de6cba5ad`. Read-only cast; the caster edited nothing and probed nothing.
**Date:** 2026-09-23 · **Cast by:** orchestrator, via an Opus subagent.
⚠ The harness refused the caster's write of this file ("Subagents should return findings as text,
not write report files"); the orchestrator wrote it from the returned text. §"Orchestrator's weigh"
at the end is the orchestrator's, not the ward's.

## Spell verdict
**The spark lives in the types and dims in the functions.** Every minted type (`<b>::State`,
`::Record`, `::Admin`, `::Status`, `::Handle`, `::Op`) and type member (`::State/durable`,
`::Handle/addr`, `Enum.Variant/field`) keeps the house grammar — case tells kind, `/` joins a type
to its member. The minted *functions* are where it dies: three separators (`/`, `::`, `$impl`) each
do disjointness work and none says what the name is. One plumbing name lies (`dispatch-admin`),
four mumble, and the stop/hibernate/surface-forms prose describes a retired shape.
Class 4, crisp: every lowercase-parent `/` join in these rooms is one of exactly 25
interpolation sites in `wat/service.wat` (21 `<b>/…` + 4 `…::<op>/Request|Response`).
Nothing in `register.rs` or `spawn.wat` is in the class.

## Corrections to the brief (the code wins)
1. `:user::serve`/`:probe::serve` are hand-written defns (`tests/comms/probe_arc209_bound_listener.wat:10`,
   `tests/services/probe_arc170_c1_kwargs_bracket.wat:59`), not the minted `<b>::serve`. Only direct
   minted-serve call found: `tests/services/probe_arc209_c2_defservice_dispatch.wat:73`. Minted serve
   is otherwise reached by runtime name (`wat/service.wat:2609`). Instrument: token grep; blind to
   runtime-built names.
2. Variant accessors are minted in `register_enum_methods` (`register.rs:1313`) at `:1478`;
   `register_aggregate_methods` (`:927`) mints aggregate `Type/field` at `:1085`.
3. `grep -E 'interpolate "[^"]*/[^"]*"'` → 30 sites: 21 lowercase-service-parent, 4 lowercase-op-parent,
   5 type-parent. Class count 21 agrees with WEIGH 255.14.

## Level 1 (lies)
- **L1-A `<b>::dispatch-admin`** — `service.wat:1178-1179`, body `:1235-1246`, doc `:1070-1072`.
  Dispatches 2 of 6 Admin variants (Init/Resume); assertion-fails on Stop/Hibernate/AllowPeer/DenyPeer.
  Real Admin dispatch is the serve loop (`:1865-1880`). Doc tells the truth; name does not.
  **Proposed:** `<b>::boot` (or `boot-from-admin`).
- **L1-B the `<b>/…` separator** — `:2022`, `:2179`, `:2218`, `:2254/2258`, `:2300/2302`, `:2561`,
  `:2575`, `:2549-2573`. Under 255.4/255.14, `/` promises a type parent; `<b>` is a namespace
  containing types (`:592`, `:598`, `:999`), lowercase in every shipped service
  (`wat/kernel/services/stdio.wat:42`). Same lie 255.14 fixed in spawn — but here the `/` also
  carries disjointness from the `::` plumbing, so respelling alone is unsafe. **Proposed:** `::` per
  255.14, after solvere S1.
- **L1-C `<S>::<op>/Request|Response`** — `src/types.rs:4157`, `:4164`; refs `service.wat:1381`,
  `:1784`, `:2036`, `:2045`. `StdOut::write/Request` (`render.rs:3622`, `:3813`): parent is a method
  (lowercase, `stdio.wat:40`), member is a type. Unspellable shape (shares its faithful image with
  `StdOut::write::Request`). **Proposed:** `<S>::<op>::Request` / `::Response`. Note `<Op>Request`/
  `<Op>Response` are LAW (`types.rs:3578-3586`, `:3647`) — see S2.
- **L1-D surface-forms prose** — comments `service.wat:821, 921, 928, 2507, 2512, 2517-2521`,
  `types.rs:4060` say `::surface-forms`; code mints `/` (`service.wat:938`, `:2514`, `types.rs:3957`).
  Name speaks; prose lies. **Proposed:** fix the comments to `<S>/surface-forms`.
- **L1-E lifecycle prose** — `:1062` `Stopped [state]` vs `:1215` `[resp]`; `:1191` "Final" and
  `:1193` "Shutdown" do not exist (`:1204-1219`); `:2172-2175` "-> state-ty … returns state" vs
  `:2203` `resp-ty`, `:2193` `{:resp resp}`; `:2209-2211` "the WHOLE State (not a projection)" vs
  `:2241` `record-ty-ann`, `:1216` `snapshot <- record`, `:1874` projection, and `:1188` saying the
  opposite. **Proposed:** correct prose to the code.

## Level 2 (mumbles)
- **L2-A** `<b>::stop-project` / `hibernate-project` (`:711-715`, `:755-757`; used `:1868`, `:1874`) —
  `-project` is a disambiguator wearing a description (`:711` "distinct from <fqdn>/stop method").
  **Proposed:** `<b>::stop-reply`, `<b>::hibernate-snapshot` (the protocol's `resp`/`snapshot`).
- **L2-B** `<b>::extract-addr` (`:1180-1181`, `:1254-1258`) → **`<b>::started-addr`**.
- **L2-C** `<b>/start$impl{,-thread,-process}` + resume twins (`:2548-2573`, defs `:2646-2648`) —
  differ only by locus param (`Locus`/`ThreadOpts`/`ProcessOpts`, `:2586-2588`).
  **Proposed:** `start$locus`, `start$thread`, `start$process`.
- **L2-D** `<b>::Handle/handle` (`:2181`, field `:2868`) — prose calls it the lineage peer
  (`:2172-2173`). **Proposed:** field `lineage`, accessor `Handle/lineage`.
- **L2-E** one op, three spellings: `write/Request` (`types.rs:4157`), `WriteRequest` (LAW, `:3647`),
  `WRITE-MAX-REQUEST-BYTES` (`:4024`). **Proposed after L1-C:** `<S>::<op>::Request`, `::Response`,
  `::max-request-bytes`.
- **L2-F** (reference room) `ProcessOpts.env-fn <- String` (`spawn.wat:69`, WHY `:74-77`, builder
  `:134`, default expression `:128`). **Proposed:** `env-source` / `process::env-source`.

## What speaks (do not "fix")
`<b>::init`, `<b>::serve`, `<b>::service-forms`; the 7 spawn builders (`spawn.wat:124-168`, WHY
header `:103-111`); `Enum.Variant/field` — parent is a registered singleton type
(`register.rs:1405-1414`, receiver `:1462-1471`), so NOT in the lowercase-parent class by name; the
WEIGH's evidence is `ns_to_wat_path` (`render.rs:3608-3610`) destroying `.` — a conversion defect,
not a naming one (unmeasured: any lowercase variant names in the corpus); `::State/durable`,
`::Handle/addr`, `<S>/surface-forms`.

## Rune evaluations
None in the target rooms (`service.wat`, `spawn.wat`, `register.rs`, `types.rs`). Out of scope:
`wat/rete/oracle/fire.wat:54`, `src/resolve/walk.rs:60`. The `rune:lint(one-variant-separator,
type-path)` at `types.rs:4154-4156` is another ward's; it is true and silent on L1-C.

## Structure / Good UX
One namespace `<b>`, six populations, four devices:
| population | example | told apart by |
|---|---|---|
| user ops | `<b>/write` (`:2022`) | `/` |
| lifecycle, public | `<b>/start /stop /hibernate /grant /revoke /resume` | `/` — same as user ops |
| helpers | `<b>/start$impl-thread` | `$` |
| plumbing (runtime-named) | `<b>::init ::serve ::dispatch-admin ::extract-addr ::stop-project ::hibernate-project ::service-forms` | `::` |
| types | `<b>::State ::Handle ::Op` | case |
| type members | `<b>::State/durable ::Handle/addr` | Pascal parent + `/` |
1. The user/macro line is not where the separator draws it: `/` groups user ops with six macro
   lifecycle methods. `op-methods` (`:2022`) and `stop-method-name` (`:2179`) both mint `{b}/…`; no
   reserved-op-name guard in `service.wat`. Unprobed: whether a surface op `stop`/`start` fails loud.
2. After the symbol spelling only case and `$` still separate; `/` vs `::` collapse (`my.counter/init`).
3. The plumbing names are the ones that mumble (L1-A, L2-A, L2-B).
4. The lifecycle prose is the least trustworthy text in the file (L1-D, L1-E).

### Handed to solvere
- **S1** one namespace braids client, owner, and runtime audiences (`:2606-2613`) — layout decision.
- **S2** two names for one message type (alias `types.rs:4157` vs LAW `:3647`; alias rationale `:4137-4142`).
- **S3** nothing walls the class: `tests/lint/one_member_join.rs:51` is case-based, silent on lowercase parents.
- **S4** `<b>`'s case is the user's; `:my::Counter` would fake a Type/member join. Unmeasured.

## Disposition
5 L1 · 6 L2 · 0 runes. Naming-only (codemod, no ruling needed): L1-A, L1-D, L1-E, L2-A, L2-B, L2-C,
L2-D, L2-F. Needs solvere then builder: L1-B, L1-C; L2-E follows L1-C.

## Cross-references
`BRIEF-STONE-255.14-a-namespace-is-not-a-type.md`, `WEIGH-STONE-255.14-a-namespace-is-not-a-type.md`;
`wat/spawn.wat:103-122`; `docs/arc/2026/05/224-substrate-naming-honesty-audit/FINDINGS-INTUERI-CHECK.md`;
`wat/service.wat:75-84` (prior intueri cast, `CallCtx` → `Invocation`).

---

## Orchestrator's weigh — every load-bearing finding re-read against the disk

| finding | verified | how |
|---|---|---|
| correction 1 — `:user::serve`/`:probe::serve` are hand-written | ✅ | both lines read: `(:wat::core::defn :user::serve` / `(:wat::core::defn :probe::serve`. ⛔ **The brief's "~100 calls of the minted serve" was a name coincidence the orchestrator's token grep could not tell apart.** |
| correction 2 — variant accessors from `register_enum_methods` | ✅ | `register.rs:1313` is that fn; `:927` is the aggregate one. ⛔ **The brief sent the caster to the wrong room.** |
| `Enum.Variant/field` is a TYPE-parent join | ✅ | `register.rs:1405-1414` registers `constructor_path` as its own `TypeDef::Enum` singleton; `:1478` mints `format!("{}/{}", constructor_path, field_name)`. `render.rs:3610` `ns_to_wat_path` does `ns.replace('.', "::")`, destroying the variant `.`. ⭐ **A converter defect — a stone, not a naming ruling. This family leaves the builder's list.** |
| structure (1) — user ops share `{b}/…` with lifecycle methods, no guard | ✅ | `service.wat:2022` `"{b}/{op-str}"` and `:2179` `"{b}/stop"`; no reserved-name check in the file. |

### ⛔⛔ The caster's "unprobed" item, probed — a LIVE collision in today's spelling

The caster could not run a probe. The orchestrator did, from the 255.14 rider's working
`defservice` scratch probe (`wat-scripts/scratch-pad/255-14-defservice-minted-names.wat`), renaming
the `put` op to `stop`. ⚠ The first counter was broken (the probe prints one escaped string, so
line-counting saw 1) — caught because **the control returned 0 where the answer is known to be ≥1**;
recounted by occurrence:

| | `defn :probe::kv/stop` | `defn :probe::kv/put` |
|---|---|---|
| control (op `put`) | 1 — the macro's lifecycle stop | 1 |
| op renamed `stop` | ⛔ **2** | 0 |

Declared for real (not quoted):

| | result |
|---|---|
| control, declared | `rc=0` |
| op `stop`, declared | `rc=1` — ⛔ `DefRedefForbidden: redef of ':probe::kv/stop': name already bound … opt in via (:wat::config::set-redef! true)` |
| op `stop`, **with `set-redef! true`** | ⛔⛔ **`rc=0` — SILENT.** One `:probe::kv/stop` replaces the other. |

⭐ **Loud by default — but by the wrong wall, with advice that leads into the trap.** A user who
names an operation `stop` is told about "redef" and invited to enable it; doing so silently drops
either their operation or the service's lifecycle `stop`. The same applies to any op named `start`,
`hibernate`, `grant`, `revoke` or `resume`. **This exists today, independent of the conversion.**
`extirpare`: the cure is a wall at `defservice` that refuses a reserved op name with its own
diagnostic — better, a shape where a user op and a lifecycle method cannot share a name at all.
