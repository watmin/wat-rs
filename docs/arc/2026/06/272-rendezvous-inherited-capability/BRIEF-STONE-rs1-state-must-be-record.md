# BRIEF — Stone rs-1: a service's `:state` MUST be a record (the `assert-record!` check + migration)

> Single-hop sonnet Shadowdancer. Do NOT spawn sub-agents. Do NOT use git worktrees. Work ONLY in
> `/home/watmin/work/holon/wat-rs`. FIRST run `pwd`; if not there, `cd`. Use `git -C <that path>` for git.
> Commit NOTHING — the orchestrator weighs + re-runs the gate. Grounded against HEAD `03b47b93`.
> Full reasoning + contract: `DESIGN-STONE-record-state-final-return.md` § rs-1 (READ IT).

## The work (one paragraph)

A service's `:state` must be a **record** (base `:wat::Record`-derived, or `:wat::holon::Record`-derived) —
a structureless scalar like `:wat::core::i64` must be rejected at compile time. defservice is a macro that
monomorphizes the user's chosen `state-ty` into concrete `defn`s, so the type system sees a valid `i64` and
(correctly) accepts it; the constraint is on the macro's *argument* and must be stated to the checker as an
**emitted check-time form**. Build a new form `(:wat::type::assert-record! <type-keyword>)`: at CHECK time it
resolves the keyword against the `TypeEnv` and errors unless it is a record (base or holon-derived); at
RUNTIME it is a no-op returning `nil`. defservice emits it once for `state-ty`. Then migrate the 12 existing
`i64`-state services to a single-field record (the substrate-as-teacher cascade: the new check reds them; the
failures are the worklist).

## Build

### 1. The check (`src/check.rs`)
Recognize `:wat::type::assert-record!` in the `infer()` list-head dispatch (model the existing head-string
special cases — e.g. `if head_str == ":wat::core::match"` ~line 1041, and the keyword-arg special-cases
~3844/4326). For `(:wat::type::assert-record! <kw>)`:
- ⚠ **The single arg is a TYPE KEYWORD in non-value position. Do NOT `infer()` it as a value** — inferring
  `:wat::core::i64` as a value trips the "primitive type keyword in value position" guard
  (`is_primitive_type_keyword_in_value_position`, check.rs ~3270). Read the keyword STRING directly from the
  arg AST (like the match/keyword special-cases do).
- Resolve + validate exactly as `src/collection/infer.rs:378-381`:
  `is_subtype(kw, ":wat::Record", env.types()) || is_subtype(kw, ":wat::holon::Record", env.types())`.
- On FALSE → push a `CheckError` (conformare — reuse an existing `CheckErrorKind`; if none fits cleanly use
  the closest, e.g. a `TypeMismatch`/`MalformedDecl`-style with a clear message): **"a service's state must be
  a record (base or holon-derived); `<kw>` is not a record type"**. Span = the arg's span.
- The form's TYPE is `:wat::core::nil` (it produces nothing). Arity must be exactly 1 (else a clear error).

### 2. The runtime no-op (`src/runtime.rs`)
Add an eval arm for `:wat::type::assert-record!` near the other kernel/special-form head dispatch (e.g. by
`":wat::core::record?" => eval_record_q` ~line 3862). It returns `nil`. ⚠ **Do NOT eval the type-keyword arg**
(it is a type reference, not a value). The form rides the generated `do`, so it must eval cleanly to `nil`.

### 3. defservice emits it (`wat/service.wat`)
In the final `do` assembly (~597-606), add `(:wat::type::assert-record! ~state-ty)` as one spliced form
(alongside `~start-fn` / `~handle-record`). `~state-ty` is already a binder in scope. Place it so it is among
the emitted forms (it types to nil; it does not need to be first — the state record is registered by check
time regardless of order).

## Migrate the 12 `i64`-state services (the cascade)
After step 1 lands, `cargo test` will red every `:state :wat::core::i64` service. Migrate EACH to a
single-field record. **Worked pattern (mirror it):**

```
;; BEFORE:  :state :wat::core::i64   ... handler: (:wat::core::i64::+ s n)
;; AFTER:
(:wat::Record::def :my::counter::CounterState [count <- :wat::core::i64])
(:wat::service::defservice :my::counter
  :state :my::counter::CounterState
  :ops
  [(:Increment [s <- :State n <- :wat::core::i64] -> [value <- :wat::core::i64]
     (:wat::core::let [c (:wat::core::i64::+ (:my::counter::CounterState/count s) n)]
       (:wat::service::Outcome::Reply (:my::counter::CounterState c) (:my::counter::IncrementResponse c))))])
;; state0 at the call site:  (... /start (locus) (:my::counter::CounterState 0))   ; was `0`
;; rs-2 stop probes: the final state is now a CounterState; assert on (CounterState/count final) == 5,
;;   OR adjust the probe's expected Value to the record. Keep the test's INTENT (stop returns final state).
```

Files (each defines a service with `:state :wat::core::i64`):
`tests/probe_arc209_c1_defservice_op_enum.rs`, `…_c2_defservice_dispatch.rs`, `…_c3_defservice_client_face.rs`,
`tests/probe_arc209_locus_agnostic_start.rs`, `…_naming_conversion.rs`, `tests/probe_arc265_acronym_registry.rs`,
`tests/probe_arc272_6b_defservice_on_process.rs`, `tests/probe_arc272_rs2_thread_stop_returns_final_state.rs`,
`tests/probe_arc272_rs2_process_stop_returns_final_state.rs`, `tests/probe_arc272_rs2_crash_surfaces_to_client.rs`,
`wat-tests/service-locus-parity.wat`.
Keep each test's existing assertions/intent — only the state shape changes (scalar → 1-field record), with
handlers wrapping/unwrapping the field and call-site `state0` wrapped in the record constructor.

### The rs-1 probe (`tests/probe_arc272_rs1_state_must_be_record.rs`)
- `scalar_state_is_rejected`: **REMOVE the `#[ignore]`** — it must now go GREEN (the check rejects i64). KEEP
  its `:state :wat::core::i64` (it is the gate proving rejection — do NOT migrate it to a record).
- `record_state_is_accepted`: already green; leave it.

## Rooms (read in order)
1. `DESIGN-STONE-record-state-final-return.md` § rs-1 (the contract + reasoning).
2. `src/collection/infer.rs:370-419` (the EXACT `is_subtype` record-or-derived pattern to mirror).
3. `src/check.rs` ~1041-1090 (head-string special cases) + ~3251-3300 (`check_form`/`infer` entry +
   the primitive-type-keyword-in-value-position guard) + ~3840-3900/4320-4450 (keyword-arg special-cases —
   how a keyword arg is read literally, NOT inferred as a value).
4. `src/runtime.rs` ~3860 (`record?` eval arm — the dispatch neighborhood for the no-op).
5. `wat/service.wat:597-606` (final `do` — the emit site) + `:40-50` (Outcome/state-ty binder context).
6. `tests/probe_arc272_rs1_state_must_be_record.rs` (the gate) + one migrated probe as the pattern.

## STOP triggers (halt + report — rejection criteria, not permission to ship less)
1. STOP if `is_subtype`/`env.types()` are NOT reachable from the check site you choose — report where you
   looked (the DESIGN claims they are; if wrong, the orchestrator re-plans).
2. STOP if a clean `CheckErrorKind` for "not a record" does not exist AND inventing one would touch the
   conformare error taxonomy broadly — report; do not bolt on a sloppy variant.
3. STOP if migrating a service to a record state requires changing what a test fundamentally PROVES (beyond
   scalar→record state shape) — report that file; do not silently weaken a test.
4. STOP if the emitted `assert-record!` form breaks defservice expansion or the `do` assembly (e.g. type-decl
   splicing) — report.

## Gate (orchestrator re-runs)
- `cargo build --release -p wat` → clean.
- `cargo test --release -p wat --test probe_arc272_rs1_state_must_be_record -- --include-ignored --test-threads=1` → 2 GREEN (`#[ignore]` removed).
- `cargo test --release -p wat --test probe_arc272_rs2_thread_stop_returns_final_state --test probe_arc272_rs2_process_stop_returns_final_state --test probe_arc272_rs2_crash_surfaces_to_client -- --include-ignored --test-threads=1` → all GREEN.
- `cargo test --release -p wat --test probe_arc209_c1_defservice_op_enum --test probe_arc209_c2_defservice_dispatch --test probe_arc209_c3_defservice_client_face --test probe_arc209_locus_agnostic_start --test probe_arc209_naming_conversion --test probe_arc265_acronym_registry --test probe_arc272_6b_defservice_on_process -- --test-threads=1` → all GREEN.
- `cargo test --release -p wat --test test -- counter 2>&1 | grep "test result"` → locus-parity deftests GREEN.
- `cargo test --release -p wat --lib -- --test-threads=1 | grep "test result"` → 929/36 (zero new).
- `cargo test --release -p wat --test nursery -- --test-threads=1 | grep "test result"` → ~893/4 baseline.

Report: exact files+lines changed; how the check arm reads the type keyword (not as a value) + validates via
is_subtype; the runtime no-op; the defservice emit; the per-file migration pattern you applied; the pasted
gate results from YOUR OWN runs; any STOP hit.
