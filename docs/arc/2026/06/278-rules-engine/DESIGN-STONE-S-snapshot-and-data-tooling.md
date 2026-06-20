# DESIGN — Stone S: snapshot + data-tooling (the revive/diagnose closing capability)

Status: **DECISIONS LOCKED, not yet built.** Banked this session (2026-06-19) from the clara-tools /
clara.rules.durability comparison. The realization that grounds it is `REALIZATIONS.md` R5 (the snapshot is
deferred computation). Build order: this stone follows the **40k cheap-support-repr perf strike** and the
**EXPLAIN renderer** — it shares their substrate (the support chain / `token.matches`).

## Why
The builder's AWS pipeline triaged misfiring DDoS rules and derived autoscaling params by fetching the exact
prod state from S3, reviving it on a dev machine, and mutating facts / swapping rules to see the system evolve.
We reproduce that loop — and, because the engine is pure-by-compiler, get it leaner than the Clara original.

## The contract (locked decisions)

### 1. The snapshot blob = `{facts, rules}` — nothing more
Working memory is a deterministic pure function of `(facts × rules)` (`rete.wat:885-886, 976, 1006-1008`;
restricted-pure RHS `matcher.rs:319, 377-391`). So the durable artifact carries only the INPUT tier. Every
DERIVED item Clara's `durability.clj` serializes (alpha/beta/accum/production memory, the activation agenda, the
object-identity sharing graph, internal token/element objects) is **regenerated on re-fire** and therefore NOT
stored. Revive = read → compile → fire. (Clara had to store the derived state because its RHS is arbitrary
`eval`'d code and can't be safely re-fired — `compiler.clj:434-462, 1494`. Ours can.)

- **Version-stable by construction:** the blob holds zero engine internals — only domain facts + authored
  rules — so it does not break across engine changes. Clara's durability is EXPLICITLY EXPERIMENTAL /
  version-fragile (`durability.clj:9-11`) precisely because it serializes internals.
- **Provenance is regenerated, not stored:** "how did we derive these" lives in `token.matches`, rebuilt by the
  join passes every fire. EXPLAIN reads the re-fired session; it is never shipped in the blob.
- **Optional, opt-in only:** a `:derived` golden-baseline (the fired output) for diff/audit ("did my re-fire
  reproduce what prod derived"). NOT part of the core snapshot; the round-trip determinism probe makes it
  skippable. Default: exclude.

### 2. The durable rule form = structured `:lhs`/`:rhs` (Clara-parity), leaf exprs raw
The durable/snapshot rule is the **structured production data**, mirroring Clara's `{:lhs :rhs}`
(`schema.cljc:61-84`): `Rule {:name, :lhs [Condition…], :rhs [insert…]}`, where each `Condition` carries named
`:type` / `:binds` with the **raw leaf exprs preserved inside `:tests`** (clause-level structuring, leaf exprs
stay raw — exactly Clara's `{:type :constraints}` split).

- **Surface s-expr (`defrule` / `'` / `` ` ``) becomes authoring-input + render-on-demand**, NOT the stored
  form. Parse happens at author time (defrule → structured), like Clara.
- **Why structured durable** (revises the earlier "raw-surface is source of truth" pin): it serves replay +
  programmatic exploration + ML directly, catches malformed conditions at parse (typed record, wrong shape
  unrepresentable — `[[feedback_no_magic_that_lets_llm_fake_correctness]]`), and is the form the builder's AWS
  blob actually carried. It loses nothing — the raw leaf exprs live inside the structure.
- New pieces: a typed `Condition` / `Bind` record; a `parse-rule` (surface → structured); `compile` consumes
  structured (parse-once → perf + early validation); a structured→surface renderer (the sugaring pretty-printer
  below) for re-stashing mutated rules and human reading.

### 3. The data-tooling surface (pure data, no UI)
Grok `~/work/holon/clara-tools` for the render SHAPE, not to copy. The provenance unit is the **token**
(`engine.cljc:20-24` ≡ our `kernel.rs:326`). Clara's data model: flat `Explanation` per activation → fact-keyed
`:fact->explanations` → recursive dual-adjacency fact-graph (`fact_graph.cljc:65-96`). Ours regenerates the
equivalent on re-fire.
- `snapshot` / `revive` — `{facts, rules}` ↔ EDN ↔ live fired session.
- `explain <fact>` — walk a derived fact's support backward to inputs (the why-tree). Needs `token.matches`
  (kept) + the fact→producing-token link (the 4c cut — re-introduce; same substrate the streaming engine wants).
- Forward walk = impact / blast-radius / load-fabrication capacity.
- "which gate misfired" — the per-condition `ConditionMatch` is free from structured conditions; the *failed*-gate
  view is the thing Clara stops at and we add.

## Banked alongside (deferred, on disk so not lost)
- **quote/quasiquote → `'`/`` ` `` lint rule** — write as a *traditional* (if/cond) lint rule first; flip to a
  rete rule once the engine closes. Educational intent: prefer the short form unless there's a good reason. Needs
  a **sugaring pretty-printer** (quote-family heads → `'`/`` ` ``/`~`/`~@` + recursively-sugared arg) — `write-forms`
  emits longhand by design (Clojure-faithful: `watast_to_edn` `wat_edn_bridge.rs:100-101`). That pretty-printer
  is the SAME structured→surface renderer this stone needs (§2) — build once.
- **Lint suppression = point-in-code metadata rune** — `{:wat-lint.disable [:quote-expansion]}`. The builder's
  explicit choice of decorate-the-code over a config file; it is the rune-suppression mechanism the doctrine
  already allows (NO config file, ever).

## The reveal that reframes 278's purpose (REALIZATION candidate when it lands)
The engine's first serious application is the **reborn linter**: today's lint rules are if/cond chains; the
builder wants them mapped to **rete rules** the linter actions — for richer expressions without gnarly if/cond
chains, and the refactor freedom to thread new alpha nodes without rewriting the linter argspec. The rules
engine is being built *to rewrite the linter*. (Dogfood; the VSA⋈rete fabric's exact-match half, applied to
wat's own source.)

## Foundation probe (write FIRST when this stone opens)
`rule → EDN → rule → compile → fire → assert same derived set as the original`. Proves the revive contract +
the determinism that makes the `:derived` baseline optional. If it goes red, the failing form-node names the
exact gap — extirpate that class.
