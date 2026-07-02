# Arc 299 — Entropic Values — DESIGN

**Thesis (R1, ENTROPIA MENSVRA PVRITATIS):** entropy is the measure of purity. Isolate entropy from
"impurity" (which fused *effect* and *entropy*), make it a derivable axis, and the test-measurement
mode derives itself — `.edn` equality for the pinned, `.wat` conformance for the entropic. 296 is the
first consumer: it emits entropic values and must measure them.

**The doctrine of the build (the builder's frame): the kill is easy; the escape is the fight.** Making a
change is a keystroke; getting the whole workspace *back to green* after it is the crawl out. So every
stone below carries an **escape plan** — the mapped return to green — not just a strike. We prove the
mechanism *and the escape discipline* on the winnable rooms first, then take the hard room with the gear
proven.

## Decomposition — stones, ordered by escape difficulty (winnable first)

| stone | what | escape (return to green) |
|---|---|---|
| **299.1** | the entropic **measurement**, proven on uuid-v4 (random ⇒ equality impossible ⇒ conformance) | **ADDITIVE** — new uuid accessors + new tests; near-zero cascade. The winnable room. |
| **299.2** | the entropy **spectrum**: `now` (windowed) + pid (pinned); the orchestrator-bounds pattern | additive — a pid verb + tests; small cascade |
| **299.3** | refine `Purity` → `Pure \| Effectful \| Entropic`; tag the entropy sources; derive entropy transitively | **HARD — 23 files** (src + crates). The dungeon crawl out; plan it when we reach it. |
| **299.4** | the mode **auto-derives** from the entropy tag (`.edn`/`.wat` unrepresentably-wrong) + structured verdict (conformance failure = a 296 diagnostic) | moderate — rides 299.3's axis |
| **299.5** | **296 consumption** — `.wat` conformance for the entropic reds + the 111 runes; R6 lint → true zero | the recapture's real close |

Rationale for the order: 299.1–2 are additive (winnable escapes) and prove the architecture; 299.3 is
the 23-file cascade (the hard escape) and only earns its keep once the measurement it enables is proven;
299.4 is 299.3's payoff; 296 consumes at 299.5. You don't lead with the hardest escape.

---

## Stone 299.1 — the entropic measurement, proven on uuid-v4

**Why.** The whole inversion — Rust orchestrates (generates the entropy), wat measures (conformance) — is
unproven. uuid-v4 is the poster child: 122 random bits ⇒ the value can never be pinned ⇒ equality is
impossible ⇒ you *must* measure conformance to the v4 spec. Prove it end-to-end here, on a winnable escape,
before anything wide.

**What it delivers.**
1. Two uuid entropic-value **foundations** (accessor primitives):
   - `(:wat::core::Uuid/version u:Uuid) -> :wat::core::i64` — the version nibble (4 for v4).
   - `(:wat::core::Uuid/rfc4122-variant? u:Uuid) -> :wat::core::bool` — variant ∈ {8,9,a,b}.
2. A co-located **`.wat` measure clause** — `probe_299_uuid_v4_measure.wat`:
   ```clojure
   (:wat::core::defn :probe::measure [u <- :wat::core::Uuid] -> :wat::core::bool   ; arg-type = structure, FREE
     (:wat::core::and
       (:wat::core::= (:wat::core::Uuid/version u) 4)          ; version nibble = 4
       (:wat::core::Uuid/rfc4122-variant? u)))                 ; RFC-4122 variant
   ```
3. The **Rust bridge** — `probe_299_uuid_v4_measure.rs`:
   ```rust
   let world = startup_beside(file!());
   let generated = wat_edn::new_uuid_v4();                     // RUST generates the entropy
   let verdict = eval_in_frozen(&world, &format!("(:probe::measure #uuid \"{generated}\")"));
   assert!(verdict-is-true(&verdict), "uuid-v4 failed conformance: {generated}\n{verdict}");
   ```

**The ONE contract decision (pinned):** the measure clause returns a plain **`:wat::core::bool`**. The
*structured verdict* (a conformance failure emitting a 296 diagnostic, the defclause `:ensure` integration)
is **OUT of scope → 299.4**. Rust asserts the bool and prints the offending uuid on failure. (Minimal,
provable; the richer verdict rides the axis in 299.4.)

**Files (blast radius — additive):** `src/string_ops.rs` (two `eval_uuid_*` fns), `src/runtime.rs` (two
dispatch arms), `src/check.rs` (two type sigs); NEW `tests/value/probe_299_uuid_v4_measure.{rs,wat}`.
Nothing existing depends on the new verbs → the cascade is near-zero.

**Out of scope = rejected (named, not deferred):** the `Purity` refinement (299.3); the structured verdict
(299.4); time/pid measurements (299.2); the `.edn`/`.wat` auto-derivation (299.4); 296 consumption (299.5).

**The disconfirming probe (probe-first):** a test that runs the exact bridge against a clause calling
`:wat::core::Uuid/version` — it must fail with *"unknown verb `:wat::core::Uuid/version`"*, proving the
ONLY missing piece is the accessor (generate, `eval_in_frozen`, `#uuid` arg-binding all already work).
Committed before the strike as the worked reference the executor copies.

### The escape plan (the return to green)

The strike is additive, so the escape is short and mapped:
1. the two dispatch arms compile; the two type sigs check.
2. the probe flips from RED (unknown verb) → GREEN (accessor exists).
3. the new measure test greens.
4. **the WHOLE disk stays green** — full `cargo nextest run`, read the *Summary line*, not a grep
   (feedback_weigh_the_whole_disk_not_grepped_green). The one real risk: an existing test that asserts the
   *set* of `Uuid/*` verbs or an "unknown verb" negative — the summary catches it; fix by updating that
   invariant, not the strike.

This is the winnable room. We prove the architecture AND the escape discipline here, then carry both into
299.3's 23-file crawl.
