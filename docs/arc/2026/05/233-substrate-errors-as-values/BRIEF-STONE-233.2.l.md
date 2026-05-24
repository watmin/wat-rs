# BRIEF — Arc 233 Stone 233.2.l — #[wat_value] proc-macro structural seal

## What we're doing

Mint `#[wat_value]` proc-macro in `crates/wat-macros/` and apply it to `pub enum Value` in `src/runtime.rs`. The macro structurally forbids future wrapping-style variants on Value (single `Box<Self>` / `Arc<Self>` / `Rc<Self>` / `Self` field) at compile time. Future authors who try to re-introduce the trap-door class get a compile error with a teaching diagnostic. Per FAILURE-ENGINEERING.md ✅✅✅: the META-class is closed at the highest possible layer.

This stone is the SEAL. Stone 233.2.k retired the variant; this stone makes re-introduction structurally impossible.

**Implementation surface:**

1. **`crates/wat-macros/src/wat_value.rs`** — new file:
   - `#[proc_macro_attribute] pub fn wat_value(args: TokenStream, input: TokenStream) -> TokenStream`
   - Parses `input` as `syn::ItemEnum`
   - Iterates variants; for each, skips if `#[wat_value(allow_wrapping = "<reason>")]` attribute present with non-empty reason string
   - Otherwise walks field types; rejects field types matching `Self` / enum-own-name / `Box<Self>` / `Arc<Self>` / `Rc<Self>` (including nested forms like `Box<Box<Self>>`)
   - Allowed: container types (`Vec<Self>`, `Option<Self>`, `Result<Self, Self>`, `HashMap<K, Self>`, etc.) — these don't shadow inner Self via match dispatch
   - On rejection: emit `compile_error!` with SUBSTRATE-AS-TEACHER diagnostic (names trap-door, recommends TrackedValue sibling alternative, documents opt-in syntax)
   - Returns input unchanged (strip per-variant `#[wat_value(...)]` attrs)

2. **`crates/wat-macros/src/lib.rs`** — export:
   ```rust
   mod wat_value;
   pub use wat_value::wat_value;
   ```

3. **`src/runtime.rs`** — apply to Value enum:
   ```rust
   use wat_macros::wat_value;
   
   #[wat_value]  // the structural seal
   pub enum Value {
       // ... all existing variants (post-233.2.k; no Tracked variant) ...
   }
   ```

4. **`crates/wat-macros/Cargo.toml`** — may need `[dev-dependencies] trybuild = "1"` for compile-fail tests (sonnet decides if trybuild is right vs alternative mechanism).

5. **`crates/wat-macros/tests/ui/*.rs`** — compile-fail fixtures for contracts 1/3/5 from sub-DESIGN:
   - `ui_wat_value_rejects_box_self.rs` (`+ .stderr` snapshot)
   - `ui_wat_value_rejects_arc_self.rs` (`+ .stderr`)
   - `ui_wat_value_rejects_self_direct.rs` (`+ .stderr`)
   - `ui_wat_value_accepts_opt_in.rs` (compile-pass — opt-in with reason works)
   - `ui_wat_value_rejects_alias_bypass.rs` (`+ .stderr` — alias of Box<Self> still rejected; OR documented as known limitation if alias resolution is out of scope per sub-DESIGN Decision 1)

6. **`crates/wat-macros/tests/wat_value_test.rs`** — runtime tests (alongside `tests/probe_stone_233_2_l_wat_value_seal.rs` in main crate):
   - Container enum compiles cleanly
   - Opt-in escape hatch enum compiles cleanly
   - Smoke: enum instances are constructable + matchable

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.l.md`** (commit `57eced2`) — sub-DESIGN; mechanic + rule + escape hatch + 5 contracts + Decisions 1/2/3 + four-questions verdict. **Authoritative for shape decisions.**

2. **`tests/probe_stone_233_2_l_wat_value_seal.rs`** (held uncommitted until 233.2.k ships; will be committed alongside BRIEF spawn) — FM 2-bis probe. 3 contracts that run as regular tests; verifies macro exists + applies to container enum + opt-in syntax accepted + real Value reachable. Compile-fail contracts live in trybuild fixtures per #5 above.

3. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.k.md`** — the prerequisite. Value::Tracked retired; #[wat_value] now applies cleanly.

4. **`docs/SUBSTRATE-AS-TEACHER.md`** — error-message-as-lesson doctrine. The macro's `compile_error!` output MUST teach (names trap-door class, recommends TrackedValue sibling alternative, documents opt-in syntax).

5. **`crates/wat-macros/src/lib.rs` + existing `#[wat_dispatch]` macro** — pattern reference for proc-macro structure (TokenStream parsing via syn, error reporting via Error / compile_error!).

## Implementation surface (detailed)

### Detection algorithm (Decision 1 from sub-DESIGN — syntactic scan)

```rust
fn is_forbidden_field_type(ty: &syn::Type, enum_name: &syn::Ident) -> bool {
    match ty {
        // Self / EnumName directly
        syn::Type::Path(type_path) => {
            let segments = &type_path.path.segments;
            if segments.len() == 1 {
                let seg = &segments[0];
                // Direct Self
                if seg.ident == "Self" || seg.ident == *enum_name {
                    return true;
                }
                // Box<X>, Arc<X>, Rc<X> wrapping forbidden inner
                if matches!(seg.ident.to_string().as_str(), "Box" | "Arc" | "Rc") {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                        for arg in &args.args {
                            if let syn::GenericArgument::Type(inner_ty) = arg {
                                if is_forbidden_field_type(inner_ty, enum_name) {
                                    return true; // recursive: Box<Box<Self>> etc.
                                }
                            }
                        }
                    }
                }
            }
            false
        }
        _ => false,
    }
}
```

(Sonnet refines per actual syn API.)

### Opt-in attribute parsing

For each variant, look for `#[wat_value(allow_wrapping = "reason")]`. If present with non-empty reason, skip the field-type check for that variant. The reason string is preserved by stripping the attr (parsed but not propagated to output) — its purpose is documentation in source, not runtime.

### Error message (SUBSTRATE-AS-TEACHER)

```rust
syn::Error::new_spanned(
    variant,
    format!(
        "#[wat_value]: variant `{}` has wrapping shape (single Box<Self> / Arc<Self> / Rc<Self> / Self field)\n\
         \n\
         Wrapping variants are forbidden because they silently mis-dispatch \
         pattern-match on Value::X(...): the inner Value::X gets shadowed.\n\
         This is the trap-door class arc 233 eliminated (see Stone 233.2.f apply fix; \
         Stone 233.2.j cascade; Stone 233.2.k variant retirement).\n\
         \n\
         If your use case GENUINELY requires wrapping, add\n\
             #[wat_value(allow_wrapping = \"your reason\")]\n\
         to this variant. The reason string is mandatory and documents WHY \
         the structural exception is justified.\n\
         \n\
         More often the right fix is a SIBLING TYPE outside Value \
         (e.g., wat::runtime::TrackedValue per Stone 233.2.h). \
         See docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.l.md \
         for the doctrine.",
        variant.ident
    )
).to_compile_error()
```

## What does NOT change

- **Value enum's existing variants** (post-233.2.k) — the macro is applied; doesn't add or remove variants
- **TrackedValue struct** — unchanged
- **eval / eval_inner / producers / Environment** — unchanged
- **All other arc 233 work** — unchanged (probes stay green)
- **wat-rs/src/runtime.rs lib tests** — unchanged baseline
- **holon-rs** — NOT touched

## Out of scope (affirmative scope-bounding)

- **Application to HolonAST / WatAST / other enums** — separate stones if needed
- **Type alias semantic resolution** (Decision 1 of sub-DESIGN — opt-in escape hatch covers the corner; alias bypass is a known limitation documented in macro's user docs)
- **Lint-level enforcement on USER code** (defrecord/defservice are wat-level mechanics; users don't add to substrate Value enum)
- **Conversion of `#[derive(...)]` macros** — orthogonal
- **holon-rs** — STOP-4
- **HARD CUT** — no parallel macro names or deprecation aliases

## Verification flow

```bash
cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 | tail -5    # 3/3 PASS post-stone
cargo build --release -p wat 2>&1 | tail -5                                      # 0 errors (real Value compiles with #[wat_value])
cargo build --release -p wat-macros 2>&1 | tail -5                               # 0 errors
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                  # ≥ 827 passed; 0 failed
cargo test --release -p wat-macros 2>&1 | tail -3                                # all wat-macros tests pass (incl. trybuild)
cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 | tail -3   # 5/5 PASS (regression guard)
cargo test --release --test probe_stone_233_2_j_producer_migration 2>&1 | tail -3 # 5/5 PASS
cargo test --release --test probe_eval_signature_returns_tracked_value 2>&1 | tail -3 # 3/3 PASS
cargo test --release --test probe_tracked_value_mint_contract 2>&1 | tail -3     # 6/6 PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3 # 8/8 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"      # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                          # empty
```

## STOP triggers (REJECTION criteria)

- **STOP-1:** unexpected compile errors NOT tracing to the macro work
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** **120 min elapsed** (per sub-DESIGN calibration: 45-90 Mode A; 120 STOP — smaller stone)
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning above 54
- **STOP-6:** scope creep — applying #[wat_value] to other enums (HolonAST, WatAST); changing detection algorithm to semantic-resolution; minting enum-level escape hatch
- **STOP-7:** probe still has failures post-stone (any of 3 contracts not PASS)
- **STOP-8:** existing arc 233 probes regress
- **STOP-9:** cascade exceeds time-box — apply partial-state-grading per `feedback_partial_state_grading`

Per FM 2-bis: STOP triggers are REJECTION criteria; never permission-to-defer.

## Trap-door audit

- **Macro detection algorithm catches Value::Tracked shape** — verified by inspection in sub-DESIGN; trybuild fixtures empirically confirm
- **Macro allows container variants** (Vec<Value>, Option<Value>, Result<Value, Value>, etc.) — confirmed by sub-DESIGN; probe 1 + probe 2 in `probe_stone_233_2_l_wat_value_seal.rs` exercise this
- **Opt-in escape hatch's reason string is mandatory non-empty** — enforced by macro parsing; trybuild fixture verifies empty reason is rejected
- **Type-alias bypass is a documented limitation** — Decision 1 of sub-DESIGN; opt-in covers the corner; macro user docs note "if you alias a forbidden type, you bypass the seal — consider whether you want to"
- **Error message includes file:line:variant + recovery hint** — span-anchored via syn::Error::new_spanned; recommends TrackedValue sibling alternative
- **Macro applies to real Value enum without breaking baseline** — STOP-2 enforces; trybuild fixture #4 verifies

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no parallel macro names; no enum-level escape hatch; no semantic-resolution algorithm
- Per `feedback_inscription_immutable`: SCORE is a NEW file (`SCORE-STONE-233.2.l.md`)
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification.
- **THIS IS THE FINAL SEAL STONE in the j → k → l chain.** After this stone, Value::Tracked cannot be re-introduced without explicit ceremonial opt-in. The trap-door class is annihilated at both the current substrate AND future-authoring-time layers.
- The probe at `tests/probe_stone_233_2_l_wat_value_seal.rs` IS the success criterion for the runtime contracts. trybuild fixtures cover compile-fail contracts.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.l.md` — sub-DESIGN (commit `57eced2`)
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.k.md` — prerequisite (variant retirement)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.k.md` — prerequisite SCORE (forthcoming)
- `tests/probe_stone_233_2_l_wat_value_seal.rs` — FM 2-bis probe (held uncommitted; ships alongside BRIEF spawn)
- `crates/wat-macros/src/lib.rs` — host crate; pattern reference from #[wat_dispatch]
- `docs/SUBSTRATE-AS-TEACHER.md` — error-message-as-lesson doctrine
- `scratch/FAILURE-ENGINEERING.md` — ✅✅✅ standard driving this stone
- `feedback_partial_state_grading` — discipline if STOP-3 fires
