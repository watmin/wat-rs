# BRIEF — Stone P1: native working-memory + transient/freeze boundary

Single-hop **sonnet** Shadowdancer in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A RUST
stone (new `src/rete/kernel.rs`). Build, run the named tests, report verbatim. Another agent weighs.

## The work
Stand up the native, mutable `WorkingMemory` rep of a `:wat::rete::Session` and the lossless boundary that
converts a frozen `Session` value into it and back: `to_transient` + `to_persistent`. INTERNAL Rust only — no
wat surface, no mutation primitive exposed to the language. No firing (P2), no keyed joins (P3), no delta (P4).
Prove the seam is lossless with an in-crate round-trip unit test.

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-P1-native-wm.md` — the `WorkingMemory` struct, the two
   converter contracts, the pinned round-trip-identity contract, the unit-test spec, out-of-scope.
2. `wat/rete.wat:124-131` — the `:wat::rete::Session` record: 7 fields IN ORDER (network, rules, alpha-memory,
   beta-memory, production-memory, facts, next-id); memories are `:wat::core::PersistentMap`.
3. `src/rete/matcher.rs:40-72` — the rete-Rust import pattern + how a `Value::wat__Record { class_fqdn,
   struct_form }` is read (`struct_form.as_slice()`); `src/rete/collect.rs` — building an `rpds::VectorSync`
   (`new_sync()` + `push_back`) and the `crate::runtime::{...}` imports.
4. `src/collection/eval.rs:867-1010` — reading a `Value::wat__core__PersistentMap(m)` (rpds
   `HashTrieMapSync<Value,Value>`): `m.size()`, `m.iter()` yielding `(&Value, &Value)`, `m.get(&k)`; and how a
   `HashTrieMapSync` is built (`rpds::HashTrieMapSync::new_sync()` + `.insert(k, v)` returns a new map).

## The structure (DESIGN §what)
- `WorkingMemory { network: Value, rules: Value, alpha: HashMap<i64,Vec<Value>>, beta: …, production: …,
  facts: Value, next_id: i64 }` — `pub(crate)`, in `src/rete/kernel.rs`.
- `pub(crate) fn to_transient(session: &Value) -> Result<WorkingMemory, EvalBreak>`:
  - match `Value::wat__Record { class_fqdn, struct_form }` with `class_fqdn == "wat::rete::Session"` (else
    `RuntimeError::TypeMismatch`); read `struct_form.as_slice()` positions 0..7.
  - for each of the 3 memory `Value::wat__core__PersistentMap(m)`: build `HashMap<i64,Vec<Value>>` — iterate
    `m.iter()`, key `Value::i64(n) -> n`, value `Value::wat__core__PersistentVector(pv) -> pv.iter().cloned().collect()`.
    (A malformed key/value shape → TypeMismatch; do not silently drop.)
  - passthrough network/rules/facts (clone the `Value`) + `next_id` (read the `Value::i64`).
- `pub(crate) fn to_persistent(wm: WorkingMemory) -> Value`:
  - rebuild each memory: `HashTrieMapSync::new_sync()`, for each `(n, vec)` insert
    `(Value::i64(n), Value::wat__core__PersistentVector(VectorSync from vec))`; wrap `Value::wat__core__PersistentMap`.
  - rebuild `Value::wat__Record { class_fqdn: Arc::new("wat::rete::Session".into()), struct_form:
    Arc::new(vec![network, rules, alpha_pm, beta_pm, prod_pm, facts, Value::i64(next_id)]) }` — DECLARATION
    ORDER, matching the Session record.

## The unit test (the contract — write it FIRST, watch it RED, then make it GREEN)
`#[cfg(test)] mod tests` in `src/rete/kernel.rs`:
- A `WORLD` (Temperature/WindSpeed/ColdAndWindy records + the cold-and-windy `defrule`), `startup_from_source`.
- Build a fired session via the oracle, through `eval_in_frozen`:
  `(let [rules (collect-rules :weather) s0 (compile rules) s1 (insert s0 (Temperature 15 "Oslo"))
         s2 (insert s1 (WindSpeed 45 "Oslo"))] (fire-rules s2))` → returns the fired `Session` `Value`.
- `let wm = to_transient(&fired).unwrap(); let back = to_persistent(wm); assert_eq!(back, fired);`
- A second case: round-trip a freshly-`compile`d session (empty memories) → identity.
(`startup_from_source`/`eval_in_frozen`/`parse_one!` are reachable from an in-crate test — see how
`src/test_runner.rs` / existing `#[cfg(test)]` modules call them.)

## Builder directive: build missing deps, never hack around
Deps exist (rpds `HashTrieMapSync`/`VectorSync`, `Value` variants, the freeze/eval entry points). **If a
needed primitive is genuinely missing → STOP + name it.** Do NOT expose any transient/mutation op to wat.

## STOP triggers
1. A needed primitive is missing → STOP, name it.
2. You're tempted to expose `to_transient`/`to_persistent`/a mutation op as a wat primitive → STOP (P1 is
   internal Rust; the transient is sealed).
3. You reach for FIRING / keyed joins / delta / a bench → that's P2–P5; STOP.
4. Round-trip identity fails and the fix would change the Session SHAPE → STOP (the rep must be lossless
   against the existing Session, not a new shape).

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --lib rete::kernel 2>&1 | grep "test result"     # the round-trip unit test(s) GREEN
cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy 2>&1 | grep "test result"  # 1/1 (oracle intact)
cargo test --release -p wat --test probe_arc278_5b_collect_rules -- --include-ignored 2>&1 | grep "test result"  # 4/4
cargo test --release -p wat --lib 2>&1 | grep "test result"                  # 931+N / 36 (N = the new kernel tests; the 36 unchanged)
cargo test --release --test test 2>&1 | grep "test result"                   # 264/1 (UNCHANGED)
cargo test --release --test test_stdlib_load_order | grep result             # 1/0
cargo build --release 2>&1 | tail -2                                          # Finished; no NEW warnings beyond the known 25 (+0)
```
Report: the `WorkingMemory` struct + `to_transient` + `to_persistent` source verbatim; the unit test; all
outputs verbatim; any STOP hit. No git.

## Blast radius
`src/rete/kernel.rs` (new) + `src/rete/mod.rs` (`pub(crate) mod kernel;` + 1 note). NO wat changes. NO change
to the oracle / any prior stone. No git.
