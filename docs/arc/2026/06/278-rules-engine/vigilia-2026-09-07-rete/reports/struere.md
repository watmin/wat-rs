## Struere Ward — Cast Report

> Written verbatim as returned. `&lt;`/`&gt;`/`&amp;` are HTML-entity artifacts of the agent's output.

**Scope covered (28/28 files, full read, read-only):** `src/rete/kernel/{arm,census,mod,node,outcome,session,insert,stratify}.rs`, `src/rete/kernel/fire/{mod,acc,delta,rules}.rs`, `src/rete/kernel/fire/pass/{mod,alpha,root_join,hash_join,filter,filter_after_join,join_after_filter,accumulate,production,round_census}.rs`, `wat/rete/oracle/{accum-pass,explain,fire,insert,pass,stratify}.wat`. Not converged — six findings, all Level 2 (mumble), none Level 1.

---

### Findings

**1. `src/rete/kernel/fire/mod.rs:1527` (`key_of`), `:1744` (`join_key_from_columns`), `:1822` (`key_of_el`)**
Construct: join-key resolution functions (`fn key_of`, `fn join_key_from_columns`, `fn key_of_el`).
Lens: composition-doesn't-hold / type-doesn't-enforce.
Level: 2.
`key_of` carries `// rune:struere(invariant-coupling)` at lines 1525-1526 ("missing key is a malformed network"), and panics via `unwrap_or_else(|| panic!(...))` at 1533-1534, 1539-1541, 1549. `join_key_from_columns` (1753-1755, 1756-1758, 1764-1766, 1767-1769) and `key_of_el` (1831-1833, 1834-1835, 1841-1842, 1843-1844) panic identically but carry no rune — their doc at 1818-1821 justifies the panics by claiming "the same invariant class as `driver_of`." That claim is inaccurate: `driver_of` (fire/mod.rs:282-298) and its sibling `rematch_compiled` (:388-404) answer the *identical* "setup should have compiled/produced this" invariant by returning `Result&lt;_, EvalBreak&gt;` — a matchable refusal — not by panicking. Worse, `build_gather_index`'s own doc (2044-2046) calls the missing-key case "structurally impossible for a well-formed network," but `fire/acc.rs` treats the *exact same hazard class* — a value proven only at the compile door, not at the import door — as reachable and answers it with `Result` refusals naming the door (`acc_var_i64`, acc.rs:83-131, esp. 92-102: "an imported network may name a var no condition binds"). The join-key family sits in the same file, walks the same import-reachable structures, and instead aborts the host process. This contradicts the outcome-wall architecture `kernel/outcome.rs` states as this crate's own design law ("a dynamic failure in this substrate is a value a caller must match, never a raise that unwinds past them").
Direction: type-tightening / composition-shape — either return `Result&lt;JoinKey, EvalBreak&gt;` from the three functions (mirroring `acc_var_i64`'s pattern), or add an honest `rune:struere(invariant-coupling)` to each site whose reason actually confronts the import-door counter-example already on record in this same file.

**2. `src/rete/kernel/fire/acc.rs:13-15` (`AccView`)**
Construct: `// rune:struere(host-constraint)` on the `AccView&lt;'a&gt;` struct.
Lens: type-doesn't-enforce (rune-verdict finding, not a code defect).
Level: 2.
The category `host-constraint` is defined by the ward as "the abstraction can't sit at the caller's level because the host language doesn't offer the mechanism." The stated reason — "`Bindings` is not implemented on `AccView` (would duplicate `BindView`)" — is a code-reuse/duplication-avoidance decision, not a missing-language-mechanism claim. The rune's category and its own reason don't match.
Direction: recategorize (likely `invariant-coupling`, matching the file's other correctly-tagged runes at 81-82, 157-158, 201, 214-215) or restate the reason to name the actual host limitation, if one exists.

**3. `src/rete/kernel/fire/pass/accumulate.rs:222-229` and `:308-314`, `src/rete/kernel/fire/pass/filter.rs:161-167`**
Construct: the `BindIntern { keys: &amp;mut wm.bind_keys, vals: &amp;mut wm.bind_vals, ids: &amp;mut wm.bind_val_ids, pool: &amp;mut wm.bind_pool }` literal, hand-spelled identically at three call sites.
Lens: wrong-level / composition-doesn't-hold.
Level: 2.
All three sites hold no competing borrow of `wm` that would block a constructor (unlike `FireCtx`, whose four-teen-field literal is deliberately NOT collapsed per its own documented borrow-split rationale in fire/mod.rs:84-102). The sibling `GatherIntern` type already solves this exact problem with `GatherIntern::from_wm(wm, alpha_id)`, used pervasively across these files, and `FireSession::bind_intern(&amp;mut self)` (session.rs:658-666) already performs this exact wiring — but is `#[cfg(test)]`-gated, leaving production call sites to repeat the four-field literal by hand.
Direction: abstraction-extraction — a production `BindIntern::from_wm(&amp;mut FireSession) -&gt; BindIntern&lt;'_&gt;`, mirroring `GatherIntern::from_wm`.

**4. `src/rete/kernel/fire/pass/alpha.rs:76-89` (`ClassPlan::observe`)**
Construct: `pub(crate) fn observe(&amp;mut self, class: &amp;str, i: u32, packed: bool) -&gt; bool`.
Lens: type-doesn't-enforce.
Level: 2.
The `bool` return conflates two different facts — "defer this fact, do not activate it here" vs. "not a leaf class, fall through to ordinary activation" — recoverable only from the doc comment, not the signature. This is the same "one `None`/`bool` hides two facts" shape the codebase itself already diagnosed and cured elsewhere in the same module family: `OperandSlot` (fire/acc.rs:133-153) replaced a conflated `Option&lt;usize&gt;` for exactly this reason.
Direction: type-tightening — a small two-variant enum (e.g. `Observed::Deferred | Observed::NotLeaf`) in place of `bool`.

**5. `src/rete/kernel/fire/pass/mod.rs:162` (`RoundScratch.pre_dispatched`), writer at `fire/pass/accumulate.rs:59-89`, reader at `fire/pass/filter.rs:60`**
Construct: the "no Test node dispatched twice against one parent delta" invariant, carried as a bare `HashSet&lt;i64&gt;` field two independent passes must remember to touch correctly.
Lens: composition-doesn't-hold.
Level: 2.
Each pass reads cleanly alone; the invariant holds only because `accumulate_pass` happens to run before `filter_pass` in `delta.rs`'s call order and both remember `pre_dispatched`. Nothing in the types stops a third caller, or a future pass, from dispatching the same Test id without consulting it. The codebase has an established cure for precisely this "two writers must not diverge" shape: `JoinLeftIndex`/`JoinRightIndex` (session.rs:225-306, 342-450) fold two maps that used to drift into one type whose single insertion door advances both invariants atomically, specifically because the old two-map shape caused real double-counting bugs (arc 278 A1/D2, documented at session.rs:207-224, 311-341).
Direction: composition-shape — an owned ledger type (e.g. `DispatchedTests`) with one insert-or-skip door, same shape as `JoinLeftIndex`/`JoinRightIndex`.

**6. `wat/rete/oracle/fire.wat:57-76` (`:wat::rete::walk-filter-ids`)**
Construct: fn whose name promises a filter-only walk but whose body (lines 70-74) composes `accumulate-pass`, `filter-pass`, and `hash-join-pass` per node.
Lens: Obvious? (signature/name honesty).
Level: 2 — already substantially self-disclosed. A pre-existing (non-struere) rune at fire.wat:54-56, `rune:intueri(naming)`, states plainly: "oracle populate-then-emit walker (acc+filter+hash-join); the name is the historical walk-sorted-ids split, not filter-alone. Rename would fork every oracle fire caller." Verified against the body (70-74): the claim is accurate.
Direction: none required — the existing doc-comment does the work a `struere` rune would; a formal `rune:struere(...)` tag alongside the `intueri` one would only be for cross-ward bookkeeping symmetry, not because anything is unaddressed.

---

### Runes encountered, with verdict

**`rune:struere(...)` (this ward's own vocabulary) — all in `src/rete/kernel/`:**
- `node.rs:83-85` `kind_of`, `invariant-coupling` — justified (closed 9-kind enum; a well-formed network node cannot be a 10th kind).
- `session.rs:20-21` `BindView`, `:61-62` `Token`, `:85-86` `BindSpan`, `:97-98` `Element`, `:616-617` `I64Row`, all `lifetime-coupling` — justified, each cites the specific DESIGN-STONE the fire-scoped pool contract rests on.
- `session.rs:109-110` `fact_at`, `:1686-1688` `pool_slice`, `:1716-1717` `match_slice`, `invariant-coupling` — justified; these panic under violation exactly as their tag claims, no gap between tag and behavior (contrast with Finding 1).
- `fire/mod.rs:1525-1526` `key_of`, `invariant-coupling` — reason is plausible in isolation but is contradicted elsewhere in the same file's own precedent (`driver_of`) and by `fire/acc.rs`'s explicit refusal-not-panic handling of the identical import-door hazard; see Finding 1.
- `fire/acc.rs:81-82` `acc_var_i64`, `:157-158` `operand_slot`, `:201` (`row_i64`), `:214-215` `slot_i64`, all `invariant-coupling` — justified and, taken together, a correct model of the ward's own principle (compile-door-only proofs refuse as values rather than panicking).
- `fire/acc.rs:13-15` `AccView`, `host-constraint` — category does not match its own stated reason; see Finding 2.

**Other wards' runes, recorded per instruction, no struere verdict rendered (not this ward's territory):**
- `rune:sequi(...)`, `rune:perspicere(...)`, `rune:circumspicere(...)` scattered through `arm.rs`, `census.rs` (test-only instrumentation and thread-local ambient-context tags) — left to `sequi`/`purgare`'s casts.
- `rune:lint(cited-name-absent)` — fire/pass/alpha.rs:93,169; fire/pass/hash_join.rs:21; fire/pass/production.rs:7; fire/rules.rs:167; fire/mod.rs:840,948 (`gather-walk-not-examining`).
- `rune:perspicere(intentional-structure)` — fire/pass/alpha.rs:284; wat/rete/oracle/pass.wat:245 (verified: the `Option` vs `Some(empty)` distinction is load-bearing, both branches consumed differently downstream).
- `rune:temperare(simplicity-win)` — fire/mod.rs:311-313,494-497; fire/pass/root_join.rs:53; fire/pass/filter.rs:57; fire/rules.rs:682.
- `rune:excusare(no-falsifier)` — session.rs:1834.

### Not flagged (considered and cleared)

- The repeated `FireCtx`/`RoundScratch` struct literals across `delta.rs` and every `fire/pass/*.rs` file: genuinely blocked by field-level borrow-splitting, documented at length (fire/mod.rs:76-102) including a dated note that a constructor was tried 2026-08-24 and reverted the same hour for a real compile error. Not a struere finding.
- `JoinLeftIndex`/`JoinRightIndex`/`BetaStore` (session.rs): exemplary struere — each folds a "two doors must agree" hazard into one type with a single mutating door, with the specific historical bug that motivated it cited in-place. No finding.
- The eight near-duplicate fold branches in `accum-pass.wat`'s `accumulate-pass-for-token`: verbose but each pure value-in/value-out, and the module header (accum-pass.wat:9-16) gives a real host-constraint reason (wat's parametric types can't unify the fold's differing payload types) — verified against the type system, holds up.
- All `insert.wat`, `fire.wat`, `explain.wat`, `stratify.wat`, `pass.wat` functions apart from Finding 6: pure value-flow, ordering dependencies stated in place rather than assumed.
- Various `.expect()`/`panic!` invariant sites in `fire/pass/hash_join.rs` (e.g. `:139`) matching the same "structurally impossible in a well-formed network" class as Finding 1, just without a rune either way — a documentation-consistency nit riding on the same root cause as Finding 1, not counted separately.
