# VIGILIA 2026-09-07 — the RETE subsystem, scoped

> **Builder's ruling, 2026-09-07:** *"we made a mistake last time... it was cast upon all of wat....
> we must scope this next vigilia to the rete subsystem.... the code must be the exemplar... the
> docs are the scaffolding, not the product."*

## The one rule this cast is built around

**SCOPE THE TARGET, NEVER THE WARD.** The previous cast covered the whole substrate and returned
178 findings, most of them main's. The correction is a narrower *target set* — not a shorter ward
roster. Every ward the kind-rules and triggers muster is cast; a ward that does not fire records
**why it did not**, with the measurement, so a reader can tell a measured zero from a skipped one.

⛔ **Hand-picking a roster is the dishonest shape `vigilia` names by name** — *"skipping spells whose
findings are inconvenient"* — and I proposed exactly that before the builder refused it. The docs
wards (`nesciens`, `cohaerere`, `consonare`) muster nowhere here **because no documentation is in
the target set**, which is a fact about the scope rather than a preference about the guard.

## The four targets, cast in sequence

Sequenced, not simultaneous: each cast is ~20 subagents, and every return is weighed against my own
read before the next fans out. Four at once is eighty workers and a board to triage instead of
strikes to run — the failure this scoping exists to prevent.

| # | target | files | lines | status |
|---|---|---|---|---|
| 1 | `src/rete/kernel/` + `wat/rete/oracle/` — the fire path and the spec it must mirror | 28 | ~15.8k | PENDING |
| 2 | `src/rete/*.rs` — the compile side | 15 | ~15.7k | PENDING |
| 3 | `wat-scripts/perf/grid/` — the load-bearing instrument and its corpus | 54 | ~8.7k | PENDING |
| 4 | `tests/rete/` + `src/rete/kernel/tests/` — the probe corpus | 264 | ~36k | PENDING |

## Muster, derived per target — with the triggers MEASURED

Cast 1, `src/rete/kernel/` + `wat/rete/oracle/`:

| ward | rule | verdict |
|---|---|---|
| intueri · solvere · conformare · purgare · struere · sequi · temperare | universal code | **muster** |
| exigere | universal, every kind | **muster** |
| cernere · probare · conferre | spec / language / DSL — `wat/rete/oracle/` is the spec | **muster** |
| perspicere | 2+ `<` in a type | **muster** — 14 files |
| excusare | runes / inline suppressions | **muster** — 18 files |
| secare | parallel primitives | **NO — 0 files.** A measured fact about the fire path |
| mora | a wait by chosen duration, or a timeout-0 snapshot | **NO — 0 files** |
| experiri | declares a callable surface | **NO here** — `RETE_OPS` is declared at `vocabulary.rs:307`, so it fires on target 2 |
| peragrare | a load-bearing instrument AND its corpus | **NO here** — fires on target 3 |
| partire | downstream of `solvere` reporting braiding at 2+ conflicting sites | evaluated after |
| circumspicere | always, last | **muster, last** |

Later targets get their own derivation block, written before that cast.

## ⛔ How this directory is built against the last one's rot

Each rule below answers a specific failure this arc actually suffered:

1. **`FINDINGS.md` IS THE ONLY STATUS HOME.** The 09-05 cast kept status in `WORK-LIST.md` *and*
   `RETE-BOARD.md`; `WORK-LIST.md:10-13` said *"One row, one place"* and its neighbour, written four
   days later into the same directory, shipped its own status column. **Nine rows read OPEN while
   every one of them was already cured.** There is no second board here. `reports/` holds evidence,
   never status.
2. **EVERY ROW CARRIES ITS RE-DERIVATION** — the command, gate or grep that decides its status now.
   A row whose status cannot be re-derived is a claim, and claims rot silently. This is the direct
   cure for the nine.
3. **NO STATIC "RECOMMENDED ORDER".** The last board's said *"F1 first"* long after F1 was cured; a
   recovering instance following it would have struck a corpse. Priority is `vigilia`'s own rule —
   L1 before L2, most upstream ward first — applied at read time, against re-derived status.
4. **WARD RETURNS LAND IN `reports/` VERBATIM, BEFORE ANY SYNTHESIS.** The 2026-08-30 cast lost all
   nineteen of its returns because they existed only as subagent messages.
5. **A NON-FIRING TRIGGER IS RECORDED WITH ITS MEASUREMENT**, above. Absence of a ward is a result.

⚠ **`peragrare`'s census does NOT live here.** The spell requires the enumerator committed *beside
the corpus it counts* — so the grid's census goes in `wat-scripts/perf/grid/`, and target 3's block
records its path. A census nobody can re-run is a rumour with a grid.

⚠ **Nothing in `docs/` is gated.** `no_stale_path_in_doc`'s ROOTS are `src/rete`(.rs), `wat`,
`wat-tests`; `rete_citation_resolves` scans `src/rete` comments. Every `file:line` in this directory
is unchecked by construction — prefer naming a symbol or a gate over a bare line number, and expect
line citations here to rot.

---

# ⛔ HOW TO RESUME THIS CAST — read this FIRST if you are picking it up cold

**This cast is MID-FLIGHT.** Ten of fourteen wards have returned for target 1. Four have not. The
casting procedure below is not recoverable from anything else on disk — it lived in the
orchestrator's context, and this section is the only copy.

## State as of the compaction (2026-09-07)

**Target 1 — `src/rete/kernel/**` + `wat/rete/oracle/**` — 10 of 14 wards returned.**

| returned | verdict |
|---|---|
| intueri | 1 L2 |
| purgare | 2 L2 |
| solvere | 9 (2 structural ★) |
| struere | 6 L2 |
| conferre | **1 L1** + 2 L2 |
| sequi | **CONVERGED** + 2 notes |
| temperare | 2 L2, 5/5 runes upheld |
| excusare | 65 weighed, 59 HOLD, **6 struck** |
| exigere | **CONVERGED** + 1 wording row |
| conformare | 2 L2 |

**STILL TO CAST on target 1, in this order:**

1. **`cernere`** — phantom forms; every form used traces to the language spec. `wat/rete/oracle/**`
   is the DSL half; this is its ward.
2. **`probare`** — substance vs description. This target is extremely comment-dense; that is
   exactly what probare weighs.
3. **`perspicere`** — deep nested type expressions hiding a noun. **Trigger measured: 14 files carry
   2+ `<` in a type.** Ten `rune:perspicere` marks already exist — adjudicate them.
4. **`circumspicere` — ALWAYS LAST.** Its quarry is the negative space the inward set leaves, so it
   cannot be cast until the other thirteen have returned. Give it the aggregate.

**Targets 2, 3 and 4 have not been cast at all.** See the table above in this README.

## The casting procedure — follow it exactly

1. **Fetch the ward's text from the datamancy MCP** (`fetch_spell` with the short name). Do NOT read
   a local copy: the signed manifest is the only trusted source, and a local copy is unverified,
   stale bytes.
2. **Spawn ONE subagent per ward** with the ward's full text **embedded verbatim** in the prompt.
   You cannot establish from here whether a worker can reach the MCP, so embedding is the default
   and the fallback both. One ward per worker — never a bundle.
3. **Every brief carries these, and they are what made the returns worth having:**
   - the exact target paths, and "read ONLY these; do not read the whole repo";
   - **READ-ONLY. No edits, no cargo runs.**
   - "Ground every finding in a `file:line` you actually read this session. No citation, no finding."
   - "If it converges, say CONVERGED **and say what you looked at** — a clean result must be
     distinguishable from a cast that did not happen." (This is what made `sequi` and `exigere`
     valuable.)
   - **the PRIOR ART, by name** — settled work the ward must not re-report. Without it the count
     inflates with things already done. See each returned report for what was named.
   - any **scope correction** the ward needs (e.g. `conformare` was told this target owns no error
     types; `temperare` was told the oracle's naive replay is its CONTRACT, not waste).
4. **Write the return to `reports/<ward>.md` VERBATIM, before any synthesis.** The 2026-08-30 cast
   lost all nineteen of its returns because they lived only as subagent messages.
5. **Weigh each finding against your own read of the disk before rowing it.** Mark `✅ I VERIFIED`
   only for rows you re-read yourself; everything else is `⚠ ward-reported` and may not be cited as
   fact.
6. **Row into `FINDINGS.md` — the only status home — each with its re-derivation.**

## ⛔ Two rules the builder set, and why

- **SCOPE THE TARGET, NEVER THE WARD.** The orchestrator first proposed a hand-picked roster and the
  builder refused it: *"it feels like we should prove the others don't find anything instead of
  assuming they won't."* That was right, and it paid — `sequi` and `exigere` both CONVERGED (a
  result, not a waste), and `intueri`+`struere` landed on the same lines through different lenses,
  which neither alone would have shown. The docs wards muster nowhere here because no documentation
  is in the target set — a fact about the scope, not a preference about the guard.
- **A NON-FIRING TRIGGER IS A RESULT.** `secare` (0 parallel primitives) and `mora` (0 waits) do not
  muster on target 1 — measured, and recorded above as facts about the fire path.

## ⚠ Five counts of one population, none agreeing

The runes in target 1 have been counted by four wards and by the orchestrator: **41 / 54 / 56 / 57**
(and 58 raw `rune:` lines). No finding turns on it, and nobody is badly wrong — but **a count that
varies with who ran it needs its method stated beside it**, and no later claim should rest on one of
these figures unqualified.
