# EXPECTATIONS — Arc 220 Stone 220.4 — `:wat::core::List<T>`

Mode A target: 14/14 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `Value::wat__core__List(Arc<LinkedList<Value>>)` variant added | `src/runtime.rs:~618` after Char variant |
| 2 | 5 runtime.rs arm sites (Char precedent) | PartialEq same-type + Hash (sequence-Hash; see #3) + type_name (`"wat::core::List"`) + structural-eq same-type + render (EDN parens form) |
| 3 | Cross-type sequence-Hash (NEW novel surface) | Outer Hash impl modified: Vec + List share sequence-Hash (helper function approach β recommended); both use SEQ_TAG constant + iterate + hash each Value; Hash invariant preserved (List(1,2,3) == Vector(1,2,3) → same hash) |
| 4 | Cross-type Eq arms | PartialEq + structural-eq gain `(Value::Vec(a), Value::wat__core__List(b))` + reverse arm using shared `sequence_eq` helper; per EDN spec §282-289 |
| 5 | closure_extract List arm | `src/closure_extract.rs:~1493` — List captures as `(:wat::core::List/of item1 item2 ...)` variadic form |
| 6 | Dispatch arms — length + empty? | `list_length_inner` + `list_empty_q_inner` in runtime.rs; dispatch entries `:wat::core::List/length` + `:wat::core::List/empty?` registered per arc 146 |
| 7 | Dispatch arms — first/rest/conj/contains?/get | Polymorphic paths at `runtime.rs:4525` (first), `:4537` (rest), `:4741` (conj), contains?/get sites extended to handle List. conj on List = PREPEND (Clojure semantic; distinct from Vector conj = APPEND) |
| 8 | `:wat::core::List/of` variadic constructor | `eval_list_of` in string_ops.rs following `eval_char_of` precedent; dispatch entry `":wat::core::List/of" => crate::string_ops::eval_list_of(...)` in runtime.rs:~4570 |
| 9 | HolonRepresentable<LinkedList<T>> impl | `src/comms/mod.rs` mirrors HashSet impl pattern (line 142+); encodes as `Bundle(vec![T_holon, ...])` |
| 10 | edn_shim bridge 3 sites | Parse direction × 2 + write direction × 1 mirrors Char/Uuid Edn::List ↔ Value::wat__core__List |
| 11 | Rust integration tests | `tests/wat_arc220_list.rs` — construction via `'(1 2 3)` literal + `List/of` constructor + cross-type Eq with Vector + cross-type HashMap key + first/rest/conj-prepend + length/empty?/contains?/get + EDN round-trip |
| 12 | wat-source test | `wat-tests/holon/list_round_trip.wat` — assert-eq! exercises using both literal + constructor + EDN round-trip |
| 13 | Interop shape matrix `:list-3` probe | shape_matrix.rs + shape_matrix_reader.rs + consume_shapes.clj + produce_shapes.clj — bidirectional `Value::List` of 3 ints |
| 14 | All test suites + clippy + handshakes green | `cargo build --release` 0 warnings. `cargo test --release --lib -p wat` PASS (count += new List tests). `cargo test --release -p wat-edn` 344/344. `cargo clippy --release --all-targets -p wat-edn -- -D warnings` 0. interop-tests cargo build + clippy clean + 4 handshakes PASS (orchestrator-side per 6th-stone permission wall precedent) |

## Independent prediction (calibration record)

**Target runtime:** 90-150 min Mode A
**Upper bound:** 180 min
**Confidence:** medium-high

**Rationale:**
- Largest single substrate addition in arc 220 — variant + Hash/Eq cross-type + dispatch arms × 7 + constructor + HolonRepresentable + bridge + tests + interop probe
- 6 mechanical items (Char-precedent): variant, runtime arms, closure_extract, edn_shim bridge, constructor, HolonRepresentable
- 2 novel surfaces: cross-type sequence-Hash (Vec hash modification) + cross-type Eq arms
- 7 dispatch arm extensions: length, empty?, first, rest, conj, contains?, get
- Substrate-pre-grep dense: Char arm sites mapped, dispatch fn names located (vector_length_inner, vector_empty_q_inner, eval_vec_rest, eval_positional_accessor), HolonRepresentable HashSet pattern verbatim available
- Risk: Vec hash modification cascades to existing tests (STOP-1; mitigated by deterministic hash algorithm)
- Risk: dispatch arm extension surfaces more polymorphic ops than expected (STOP-3; bounded by 7 named ops)
- Calibration band conservative; Stone 220.2 shipped 12-item / 12-file work in 30 min sonnet; Stone 220.4 is ~50% larger surface

**Per `feedback_stone_briefs_cite_prior_score`:** Stone 220.2 SCORE — ~30 min sonnet for 12 files / 8 substantive items. 220.4 has ~14 items + 7 dispatch extensions + 1 novel Hash strategy. Band 90-150 reflects scope; below-band possible if 3-part structure unlocks parallelism.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- INSCRIPTION + USER-GUIDE — Slice 5
- BigInt / BigDec wat-core types — deferred per DESIGN
- Performance optimization
- HolonAST schema extension
- New runes
- New public surface beyond `:wat::core::List/of` + variadic + `'(...)` literal (Slice 3 ships the literal)
- Touching wat-edn substrate

## Honesty deltas accepted

- Sequence-Hash strategy: (α) merged arm OR (β) helper function — sonnet picks the cleaner Rust; SEQ_TAG value (0xA5 in BRIEF) is arbitrary; sonnet may pick any byte-distinct-from-discriminants
- conj semantics: List/conj prepends, Vector/conj appends — per Clojure precedent. Sonnet may pick the dispatch site (modify existing `conj` polymorphic handler vs add new arm)
- Test fixture choices — sonnet picks illustrative cases
- HolonRepresentable<LinkedList<T>> exact bounds — sonnet matches the HashSet trait bound pattern (T: HolonRepresentable + Send + 'static; no Hash + Eq needed since LinkedList doesn't require them)
- Variant placement in enum — sonnet picks (after Char vs alphabetical)
- Cross-type HashMap test exact form — sonnet picks; the load-bearing assertion is "same key per Hash invariant"

## Honesty deltas NOT accepted

- Skipping the cross-type Eq (item #4) — STOP. EDN spec compliance demands it.
- Skipping the cross-type sequence-Hash modification — STOP. Hash invariant would break List/Vector HashMap interop.
- Skipping conj-prepend semantic — STOP. Clojure precedent. List/conj prepends; Vector/conj appends.
- Adding NEW runes — STOP.
- Touching wat-edn substrate — STOP. wat-edn handles List at the Value::List level natively.
- Wat-crate clippy gate — NOT applicable (arc 170 backlog per user direction).
- Skipping interop handshakes silently — STOP. Either run them OR mark "pending orchestrator-side verification" per 6th-stone precedent.
- Touching Slice 3's `'` reader macro OR Slice 2's Char surface — both shipped; this stone consumes them.
- Scope beyond the 13 substantive items — STOP at the boundary.
