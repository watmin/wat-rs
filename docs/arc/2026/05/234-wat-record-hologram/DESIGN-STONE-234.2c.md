# DESIGN — Arc 234 Stone 234.2c — runtime class-safety in per-field accessor bodies

**Status:** ACTIVE (2026-05-24 — orchestrator-authored sub-DESIGN; sonnet implements per BRIEF).

**Predecessor:** Stones 234.0, 234.1, 234.1.5, 234.2a (+CORRECTION), 234.2b, 234.5 SHIPPED. The 234.2b macro mints per-field accessors at `wat/Record.wat`; D10 of that sub-DESIGN named Stone 234.2c as the follow-up for runtime class-safety.

**Discipline:** sonnet writes substrate; orchestrator briefs + scores. Per `feedback_sonnet_writes_substrate`.

---

## The gap closed

234.2b shipped per-field accessors that delegate to `:wat::Record/field-at v <fi>` WITHOUT verifying that `v`'s `class_fqdn` matches the accessor's declared class. Today:

```
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::Record::def :myapp::Point [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::let
  [p (:myapp::Point 3 4)]
  (:myapp::Voltage/magnitude p))   ; ← TODAY: returns 3 (silently wrong)
                                   ;   234.2c: panics with clear message
```

Per `feedback_any_defect_catastrophic`: silent wrong-field-returned is a substrate defect. 234.2c plugs it.

Per D10 of 234.2b: two approaches were on the table:
- (a) wat-level `:wat::core::if` + `:wat::core::=` + panic-via-`Option/expect`
- (b) substrate-level per-class TypeDef registration enabling check-time class narrowing

Approach (b) depends on arc 232 defprotocol work surfacing a substrate-side type-narrowing primitive. **Arc 232 is gated by arc 234 closure per spawn-block winding discipline.** Therefore Stone 234.2c MUST use approach (a) — wat-level runtime check in accessor bodies.

This is honest scope: when arc 232.1 eventually ships and provides per-class TypeDef machinery, a future stone could LIFT the safety from runtime-wat into check-time-Rust. For now, runtime check is the right shape.

---

## Scope

ONE file changes: `wat/Record.wat`. The `:wat::Record::def` macro's per-field accessor body grows a class-equality guard before delegating to `:wat::Record/field-at`.

No substrate (Rust) changes. No new probes beyond the 234.2c probe. No changes to existing accessors, the constructor, or the predicate.

---

## Locked decisions

### D1 — Approach: wat-level runtime check (Option/expect)

Per-field accessor body wraps `:wat::Record/field-at` with a class-equality check. The check uses `:wat::core::Option/expect` to panic with a runtime-computed message when class_fqdn doesn't match.

### D2 — Accessor body shape after 234.2c

For a record-type `:myapp::Voltage` with field `magnitude` at position 0:

```
(:wat::core::defn :myapp::Voltage/magnitude [v <- :wat::Record] -> :wat::core::f64
  (:wat::Record/field-at
    (:wat::core::Option/expect -> :wat::Record
      (:wat::core::if
        (:wat::core::=
          (:wat::core::type v)
          "myapp::Voltage")
        (:wat::core::Some v)
        :wat::core::None)
      (:wat::core::string::concat
        ":myapp::Voltage/magnitude: expected receiver of class :myapp::Voltage, got class :"
        (:wat::core::type v)))
    0))
```

Mechanism:
- `(:wat::core::type v)` returns class_fqdn String (e.g., "myapp::Voltage" or "myapp::Point")
- `(:wat::core::=)` compares the actual class string vs the accessor's declared class string
- `(:wat::core::if ...)` branches: matching → `:wat::core::Some v`; non-matching → `:wat::core::None`
- `(:wat::core::Option/expect -> :wat::Record (Some v) <msg>)` unwraps Some(v) to v, OR panics with msg (per arc 108 + `expect_panic` at runtime.rs line 13629)
- The msg is a runtime `string::concat` that names BOTH the expected class (literal) AND the actual class (`:wat::core::type v` at error-time)
- `field-at` only fires on the unwrapped v (which is guaranteed-matching-class)

### D3 — Message includes expected + actual class

The panic message MUST name both:
- The expected class FQDN (literal string baked into the macro expansion at expand time)
- The actual class FQDN (computed at runtime via `:wat::core::type v`)

Format: `":<class>/<field>: expected receiver of class :<class>, got class :<actual>"`.

This makes the diagnostic immediately actionable — user sees the wrong call site + the type mismatch.

### D4 — Macro-expand-time work: class FQDN string literal

The macro expands `~(:wat::core::keyword/to-string fqdn)` to produce the literal class FQDN string for the equality comparison + the message prefix. Same pattern as 234.2b's predicate body (which already uses this pattern at the message-build site).

### D5 — Zero-field record case unchanged

Zero-field records (`:wat::Record::def :myapp::Tag []`) emit ZERO accessors. The accessor body extension only applies when accessors exist. No impact on zero-field records.

### D6 — Predicate unchanged

The predicate `:myapp::is-Voltage?` already does class equality via `(:wat::core::= (:wat::core::type v) "myapp::Voltage")`. It returns bool. 234.2c does NOT change the predicate — predicate is for explicit class-checking by callers; accessor safety is for implicit class-checking when callers skip the predicate.

### D7 — Constructor unchanged

The constructor `:myapp::Voltage` calls `:wat::Record::of` which validates internally. No safety gap there. 234.2c does NOT touch the constructor.

### D8 — Substrate Rust unchanged

No changes to `src/runtime.rs`, `src/check.rs`, `src/stdlib.rs`, or any other Rust file. Pure macro extension.

### D9 — HARD CUT — no escape-hatch unsafe accessors

234.2c is the v1 safety. There is NO "unsafe accessor" variant that skips the class check for performance. If a future stone needs unchecked access for hot loops, that's a separate decision (Stone 234.X with explicit DESIGN — not silently added).

The ~3-5 wat operations per accessor call is acceptable runtime overhead for the safety. If profiling reveals it as a bottleneck in real code, the substrate-level approach (D10's option b, gated on arc 232) is the right optimization.

### D10 — When arc 232.1 ships, LIFT the safety to check-time

Per 234.2b D10 + the discipline of "name the named follow-up": when arc 232.1 (defprotocol with per-class TypeDef registration) eventually ships, a future arc could LIFT the runtime check into check-time. The accessor signature would narrow `[v <- :myapp::Voltage]` (not `[v <- :wat::Record]`); the check-time would catch wrong-class calls; the runtime body would skip the check.

Affirmative scope cut for now: 234.2c ships runtime safety. The lift is FUTURE WORK named when arc 232.1 lands — NOT a deferral in 234.2c's INSCRIPTION.

---

## Trap-door audit

### T1 — `:wat::core::Option/expect`'s msg arg accepts runtime expressions

`expect_panic` at runtime.rs line 13629 evaluates the msg_ast as a String. The msg arg CAN be a runtime-computed `(:wat::core::string::concat ...)` expression. Pattern proven by existing usage (none specifically for dynamic msg, but the eval path is uniform).

### T2 — Option/expect signature: `(Option<T>, String) -> T`

Per arc 108. The Some path unwraps; the None path panics with msg. Our use: `(Option<:wat::Record>, String) -> :wat::Record`. Mechanism works.

### T3 — Conditional wrapping: `(:if ... (Some v) :None)` produces Option<T>

The if-expression must have both branches type-check to the same Option<T>. `:wat::core::Some v` is `Option<:wat::Record>` (since v is `:wat::Record`); `:wat::core::None` is `Option<T>` for any T. They unify at Option<:wat::Record>. Per arc 109 slice 1h.

### T4 — `:wat::core::None` vs `:wat::core::Option/None`

Per arc 109 + the existing macro, the FQDN is `:wat::core::None` (variant constructor). Confirm via grep in stdlib + existing usage.

### T5 — Multi-field expansion preserves the pattern

For N-field records, N accessors are emitted. Each accessor's body independently wraps its own `field-at` call with class check. No cross-accessor sharing; each is self-contained.

### T6 — Zero-field records emit zero accessors (no impact)

Per D5. The 234.2b probe 6 (`:myapp::Tag []`) currently PASS. 234.2c doesn't change zero-field behavior; probe 6 must stay green.

### T7 — Predicate-pattern usage (defensive check-then-access)

Users may pattern: `(:wat::core::if (:myapp::is-Voltage? v) (:myapp::Voltage/magnitude v) <fallback>)`. When predicate is false, accessor isn't called; no panic fires. The probe verifies this pattern works post-234.2c.

### T8 — Panic message format

The literal expand-time string MUST exactly match the runtime computed format:
- prefix: `":myapp::Voltage/magnitude: expected receiver of class :myapp::Voltage, got class :"` (the colons + accessor name + class name baked in)
- suffix: `(:wat::core::type v)` (the actual class FQDN at error time, no leading colon since type returns the bare FQDN per Stone 234.0 D4)

NOTE: `:wat::core::type` returns FQDN WITHOUT leading colon (per Stone 234.0's D4 and 234.2a's SCORE D5 finding). So the suffix needs a leading colon prefix `":"` in the concat to produce the canonical `:myapp::Voltage` form. Or omit the colon if the format reads cleanly without.

Honest trap-door: the message format above embeds the colon as a literal in the suffix prefix string `"got class :"`. If we want `":myapp::Voltage"` in the message, the concat is: `"... got class :" + actual_class_fqdn_without_colon`. Verify the resulting message reads cleanly.

---

## What the FM 2-bis probe must demonstrate

`tests/probe_arc234_stone2c_accessor_class_safety.rs` — contracts (5):

1. **Correct-class accessor returns value** — define `:myapp::Voltage`; construct `(:myapp::Voltage 5.0)`; call `(:myapp::Voltage/magnitude v)`; verify returns 5.0. (regression: 234.2b probe 2 equivalent — must stay GREEN under 234.2c)
2. **Wrong-class receiver panics** — define `:myapp::Voltage` + `:myapp::Point`; construct a Point instance; call `(:myapp::Voltage/magnitude point-instance)`; verify the eval panics (catch via `std::panic::catch_unwind`).
3. **Panic carries informative message** — same as #2 but verify the panic's AssertionPayload message contains BOTH "myapp::Voltage" (expected) AND "myapp::Point" (actual). Downcast the panic payload via `panic.downcast_ref::<wat::assertion::AssertionPayload>()`.
4. **Multi-field accessor (3 fields) — each independently checks class** — define `:myapp::Triple [a <- i64  b <- String  c <- bool]`; define another `:myapp::Other [x <- i64]`; construct an Other; call `:myapp::Triple/b` on Other → panic.
5. **Predicate-gated pattern works** — define `:myapp::Voltage` + `:myapp::Point`; in a let, bind a Point; use `(:if (is-Voltage? p) (:Voltage/magnitude p) -1.0)`; verify returns -1.0 (the fallback, NOT a panic — the predicate's false arm guards the accessor call).

**Initial state (before sonnet ships):** 5/5 FAIL or partial:
- Probe 1 PASSES (current 234.2b accessor works on matching class)
- Probes 2, 3, 4 currently DON'T PANIC (silent wrong-field-returned); they FAIL because the test asserts a panic
- Probe 5 PASSES (predicate-gated pattern works; the accessor isn't called)

So initial state is likely 2-3/5 PASS. Post-stone: 5/5 PASS.

---

## STOP triggers (rejection criteria)

- **STOP-1** — unexpected compile errors not tracing to the macro extension
- **STOP-2** — lib tests baseline regresses below 827
- **STOP-3** — 60 min elapsed (small stone; tight cap)
- **STOP-4** — `holon-rs` touched
- **STOP-5** — Rust changes (D8 — pure macro extension; only `wat/Record.wat` modified)
- **STOP-6** — scope creep: substrate-level type narrowing, unchecked accessor variants, predicate changes, constructor changes
- **STOP-7** — the new probe doesn't flip to 5/5 PASS
- **STOP-8** — 234.2b regression guard regresses
- **STOP-9** — any prior arc 234 regression guard regresses (234.0, 234.1, 234.1.5, 234.2a, 234.5)
- **STOP-10** — clippy warnings exceed 54

Each STOP is REJECTION criteria, not permission slot.

---

## What this unblocks

- **Stone 234.6** — migration sweep + `:wat::holon::defrecord` retirement. Migrated callers get class-safe accessors automatically.
- **Stone 234.3** — polymorphic record-y verbs benefit from the accessor-safety pattern being established (assoc, record->map can mirror the runtime check).
- **Future arc 232.1 → check-time narrowing lift** — D10 names this; when defprotocol's per-class TypeDef lands, the runtime check can lift to check-time.

---

## Calibration prediction

**Target runtime:** 20–40 min Mode A
**Upper bound:** 60 min (STOP-3 hard cap)
**Confidence:** high — wat-side macro extension is well-precedented; the Option/expect + if + string::concat pattern is straightforward; 234.2b's macro body already has similar patterns.

**Rationale:**
- Macro extension: ~10-15 lines inside the per-field accessor `:wat::core::let` body
- Probe is committed pre-spawn: no probe authoring time at sonnet
- Compile cycles: 1-2 rounds expected (extension is well-localized)
- SCORE writing: ~10 min

**Calibration precedents:**
- Stone 234.2a-CORRECTION (~25 min): check.rs change of similar scope
- Stone 234.2b (~78 min): full macro authoring (much larger)
- Stone 234.2c estimate: ~25-35 min predicted; band's middle

**Risks:**
- **`:wat::core::None` literal form** — confirm via lib search + existing usage
- **Option/expect msg with runtime concat** — should work per `expect_panic` analysis but empirically unproven
- **Panic payload downcast in probe** — uses `wat::assertion::AssertionPayload`; verify visibility from probe Rust code

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — predecessor (where D10 deferred class-safety to this stone)
- `wat/Record.wat` — the macro (target of extension)
- `src/runtime.rs::expect_panic` (line 13629) — panic mechanism behind Option/expect
- `src/runtime.rs::eval_option_expect` (line 13522)
- `crates/wat-?/src/assertion.rs` (or src/assertion.rs) — AssertionPayload struct for probe downcast
- `tests/wat_bundle_capacity.rs` line 178 — `std::panic::catch_unwind` pattern precedent
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs; sonnet writes
- `feedback_any_defect_catastrophic.md` — the doctrine driving this stone (silent wrong-field-returned is catastrophic)
- `feedback_no_known_defect_left_unfixed.md` — STOP triggers are rejection, not deferral; D10 follow-up named for future arc 232 lift
