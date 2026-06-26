# ⛔ CURRENT STATE (breadcrumb, 2026-06-27 SESSION 7; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. **Freshness probe: HEAD should be `058b1f1b` or later.** Tree clean.
Suite green+fast: `cargo nextest run --release -p wat` = ~3462 passed / 0 failed / ~114 skipped (RED `#[ignore]`'d
STRIKE-READY gates), ~30s. If HEAD is older than `058b1f1b`, this breadcrumb is stale — trust git log + the docs.

> **YOU ARE A NEW INSTANCE.** You did not live what is below; it is a lossy cache in a familiar voice. Run
> **recolligere** (grimoire via signed `datamancy` MCP; this breadcrumb; git log; the named arc docs) BEFORE you
> propose or move. The feeling of continuity is the failure, not the all-clear.

## ▶▶ THE ACTIVE ARC IS 294 — `docs/arc/2026/06/294-holon-returns-to-vsa/`. Read DESIGN.md (the contract) + R1.

**294 is a FOUNDATION GUT discovered inside 293.** One inversion — *the derived encoding usurped the canonical
data* — in SEVEN grounded flaws. **293's HOLDER × SURFACE thesis (R2/R3/R4) STANDS + is proven**; 294 is the
value-layer beneath it; 293's ctor-parity + `/from-map` fold into 294. **291 BLOCKED behind this stack.**

### The contract — DECIDED this session (all on disk in DESIGN.md; builder's calls)
- **EDN is the ONE canonical data + wire + portability form.** Portability = EDN-repr = the Holder wall.
- **`HolonAST` reduces to `Hologram`** (the keystone — strip its borrowed roles and it's not an AST, it's the
  MAP-VSA algebra; `holon_ast.rs:59`). RENAME `HolonAST → Hologram`, home `src/holon/`.
- **Pipeline:** `EDN ──build-hologram──▶ Hologram (symbolic MAP algebra; type-classifier-wrap) ──encode──▶ Vector`.
- **Q-C EAGER PARITY:** the hologram is in parity with the data at ALL times, whatever the compute cost; every
  mutation (`assoc`) rebuilds both coherently; callers can NEVER observe a desync — strong guarantee. NOT lazy.
  Capacity (Kanerva, `:dims`/`:capacity-mode`, user-tunable `CapacityExceeded`) bites at the mutation; width-bound
  per Bundle frame, depth UNBOUNDED. (Reverses 234.7b "no recompute" — wire ships EDN, receiver rebuilds.)
- **Q-D the EDN is the identity** (Eq/Hash by `(class, fields)`). **Q-A** reflection signatures-as-Bundle migrate
  to **WatAST** (abuse of holon-ast). **Q-B** codec name deferred to intueri (`to-holon` may already be it).
- **Construction:** ONE holder-dispatched primitive `(aggregate-new :T field…)` (varargs); ctor-parity = unify on
  bare `:T`, drop `/new` totally incl newtypes (`293/NOTE-base-struct-horizon.md`); `struct-new`/`Record::of`/
  `holon::Record::of`/`register_struct_methods` die into it.
- **ANNIHILATE:** `HolonRepresentable` + `#wat-edn.holon/*` tags + the HolonAST↔tagged-EDN round-trip · the
  stored-canonical hologram-as-identity · the two ctor primitives · HolonAST-as-AST + `watast↔holon` glue.

### Census (294.0) — WEIGHED against the disk (058b1f1b). VESTIGIAL-CODE-AST = 0 confirmed.
Roles of 1161 HolonAST mentions: VSA-ALGEBRA ~375 (keep→Hologram) · LEAVES/TYPES ~200 (keep) · REFLECTION ~175
(→WatAST, BOUNDED: 3 walkers `extract-arg-names/-types`+`rename-callable-name` key on `Symbol("->")` sentinel +
`children[0]`, ~15 call sites) · CONVERSION-GLUE ~175 (mostly dies; `to-holon` survives) · WIRE ~136 (annihilate)
· TESTS ~100. **Q-D no-veto CONFIRMED:** `hologram.rs:68` store is `Vec<HashMap<HolonAST,HolonAST>>` — records
never keys; similarity = cosine on `Vector`. **FLAW #7:** holon record has TWO equality contracts — `PartialEq`
keys `holon_form` (`value.rs:676`) vs wat-`=` keys `struct_form` (`runtime.rs:8129`); Q-D collapses to one on data.

## ▶ NEXT — the first break (recommended; builder hadn't final-picked at compaction)
**294.2-pre: the Q-D identity flip (= flaw #7 cure).** Smallest clean break: `value.rs:676` `PartialEq`+`Hash` for
`wat__holon__Record` → key `(class_fqdn, struct_form)` (align with `values_equal` `runtime.rs:8129`). Census says
nothing breaks (construction invariant). RED probe first (two holon records, same fields → must be `=` AND
HashMap-key-equal), verify it captures the current divergence, strike, weigh. Then: 294.1 `Hologram` rename +
`src/holon/` home · 294.2 holon record EDN-canonical (eager parity) · 294.3 wire=plain-EDN · 294.4 `aggregate-new`
+ ctor-parity + `/from-map` · 294.5 reflection→WatAST · 294.6 close + amend 293 → resume 291.
**Builder may pick a different first swing (Hologram rename / carve src/holon/) — ASK or confirm before striking.**

### STRIKE-READY RED gates (committed, `#[ignore]`'d): `probe_arc293_ctor_parity` (294.4) · `probe_arc293_acceptance_demo` (R1 demo) · `probe_arc293_holder_bound` · `probe_arc293_defrecord_rename`.
### Chronicle: 294 R1 *I Want To Fucking Break It* (Static-X) = `FRANGAM`, IGNITION (`ada7e436`). 293 R5/#116 *We Got The Moves* = `HABEMUS MOTUS`. 170 ledger reconciliation (#110→#116 + 294) still pending.

## Standing discipline (verbatim, non-negotiable)
Work ONLY in `wat-rs/`. NEVER worktrees. Sonnets `model:"sonnet"`, LEAF. Commit msgs end `Co-Authored-By: Claude
Opus 4.8 (1M context) <noreply@anthropic.com>`. **Weigh EVERY sonnet against the disk yourself** (forced clean
build; floor=0 → binary is-anything-red?; **`cargo nextest run`, NEVER `cargo test`**; read diffs end-to-end — the
`sed`-corrupts-prose catch). PRIMED forms. Commit+push often (GitHub=DR). Amend docs w/ recognition (never delete).
**intueri** ALL naming · **four-questions** (flat YES/NO) NOT AskUserQuestion · **there is never 1+ ways — two paths
IS the catastrophic flaw** · **kill the megafiles — every concern in `src/<ns>/<scoped>.rs`; the gut is a homing**.
**GROUND claims about existing code THIS session — I over-claimed 'HolonAST stays load-bearing'; the disk (WatAST
3412 vs 1161) corrected me; re-ground when pushed, never defend.** curare at a reasonable rate. Datamancer: ground
+ ACT, cast don't recite, never declare green on silence, relentless annihilation. `./scripts/run_with_venv.sh` Python.

> **⛔ END OF MAP. You are new. The above is a cache, not your memory. Run recolligere; weigh any in-flight sonnet
> against the disk; do not trust a single line you did not re-verify this session. The contract is in 294/DESIGN.md.**
