# EXPECTATIONS — Arc 220 Stone 220.5 — `:wat::core::Char` atomization gap fix

Mode A target: 4/4 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `is_atomizable` extension | `src/check.rs:~3638` — one line added: `| ":wat::core::Char"` in the `matches!` arm next to the other primitives. Doc comment cites Stone 220.2's Hash impl at runtime.rs:846. |
| 2 | Probe 1 — `(:wat::holon::Atom \\N)` round-trip | `tests/wat_arc220_char_atomization.rs` — Atom holding a Char value; verify the resulting Value is structurally distinct from Atom holding an i64 leaf (cross-type Eq sanity check). |
| 3 | Probe 2 — `HashMap<Char, i64>` insert + lookup | Same file — char-frequency-tally pattern; assoc 2 entries, get back the values via `Some` Option. |
| 4 | All test suites + clippy green | `cargo build --release` 0 warnings. `cargo test --release --lib -p wat` PASS (827/0 baseline preserved). `cargo test --release --test wat_arc220_char` 10/10 PASS (Stone 220.2 unchanged). `cargo test --release --test wat_arc220_char_atomization` 3/3 PASS (new file; 3 probes). `cargo test --release --test wat_arc220_list` 23/23 PASS (sanity; Stone 220.4 unchanged). `cargo test --release -p wat-edn` 1/1 PASS (unchanged). `cargo clippy --release --all-targets -p wat-edn -- -D warnings` 0 warnings. |

(Note: row 2 listing covers Probes 1+2+3 collectively per the test file; only one row to keep the scorecard atomic per stone-discipline.)

## Independent prediction (calibration record)

**Target runtime:** 15-25 min Mode A
**Upper bound:** 35 min
**Confidence:** very high

**Rationale:**
- Smallest stone in arc 220 — 1 line in src/check.rs + 1 new test file with 3 probes
- All patterns established by Stones 220.2 (Char) + 220.4 (List) — sonnet has the test-file template
- Risk: existing tests assuming Char is NOT atomizable (STOP-1) — but unlikely since Char shipped recently with full Hash impl
- Calibration: 14 stones at-or-below band; Stone 220.3 (5 min) is the floor; 220.5 is comparable scope
- Substrate Hash impl exists (`Value::wat__core__Char(c) => c.hash(state)` at runtime.rs:846); predicate extension is the literal one-line completion of work Stone 220.2 deferred

**Per `feedback_stone_briefs_cite_prior_score`:** Stone 220.2 SCORE — 12/12 PASS in ~30 min; that stone added Char Value variant + Hash + Char/of constructor but did NOT extend `is_atomizable`. Stone 220.5 closes that gap with the literal one-arm addition + 3 probes proving the gate is now open. Band 15-25 reflects only the 1-line predicate change + 3 probe writes.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- INSCRIPTION + USER-GUIDE — Slice 5 (separate; runs after 220.5)
- List atomization — out of scope (List lowers to Bundle, not Atom)
- Wat-edn modifications — untouched
- Interop handshakes — not required (wat-edn surface untouched)
- New runes — no candidates
- Documentation beyond test comments + SCORE

## Honesty deltas accepted

- Probe wat syntax exact form — sonnet picks based on existing wat_arc220_char.rs / wat_arc220_list.rs patterns; load-bearing assertion is `(:wat::holon::Atom \\char)` works + HashMap<Char,V> works + HashSet<Char> works
- Test count: 3 probes is the minimum; sonnet may add additional regression coverage if it surfaces an interesting edge case (e.g., named char `\newline` as HashMap key)
- Exact line placement in `is_atomizable` — sonnet picks; recommendation is next to Uuid (both are recent typed primitives with Hash impls)

## Honesty deltas NOT accepted

- Skipping any of the 3 probes — STOP. All three are the contract: Atom-able, HashMap-key-able, HashSet-element-able.
- Modifying `Value::wat__core__Char` runtime arms — STOP. Stone 220.2 shipped them; Stone 220.5 only extends check-time gate.
- Touching wat-edn — STOP. Not in scope.
- Adding new runes — STOP. No candidates this stone.
- Wat-crate clippy gate — NOT applicable (arc 170 backlog per user direction).
- Running interop handshakes — NOT required this stone.
- Scope beyond the 1 predicate line + 3 probes + SCORE — STOP at the boundary.
