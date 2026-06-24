# BRIEF — arc 291 strike 4b-ii-a: the defservice State→struct re-tool (the keystone)

**You are a LEAF executor. Model: sonnet. Do NOT spawn subagents.** Work ONLY in
`/home/watmin/work/holon/wat-rs/`. If the work exceeds the rooms below or hits a STOP trigger, STOP and
report — do not improvise a workaround.

**THE CONTRACT IS ON DISK — read it first, whole:** `docs/arc/2026/06/291-defservice-durable-state/STRIKE-4b-struct-state.md`
§ **"4b-ii — CONTRACT EVOLVED"** (at the END of that doc). That section is the spec; this BRIEF is the
execution plan. Where they ever disagree, the CONTRACT wins — STOP and report the conflict.

## The work, in one paragraph

Today `defservice`'s `:state [fields]` mints the State as a **record** (`service.wat:181-184`) that ships
over the wire. This strike makes State a **struct** (the live body) holding a required **durable record**
(the EDN soul, the only thing that crosses). It re-tools the macro to an all-keyword-clause surface
(`:durable`/`:ephemeral`/`:ops`/`:init`/`:hibernate`/`:stop`), mints `:<fqdn>::Record` (durable) +
`:<fqdn>::State` (defstruct), makes `:init : Record→State` the constructor (defaulting only when there's no
body), turns `:hibernate`/`:stop` into projection hooks with defaults, threads **records** (not the struct)
through the wire payloads, and **migrates every defservice definer** to the new surface. The signature
change breaks all defservice sites at once — this is **one atomic strike**; ride the compile cascade to zero
(the fail-count is the progress meter). **KEEP the internal lineage names** (`LineageUp`/`init-from-admin`/
`lineage-extract-addr`) — their rename is the separate follow-on 4b-ii-b.

## Rooms — read in order (all in `wat/service.wat`, 963 lines; line numbers are HEAD anchors — verify before editing)

1. **Signature + opts-fold `:52-58` + `:67-118`.** Today: `[fqdn _state-kw state-fields _ops-kw ops & opts]`
   (positional markers) + a fold of trailing `opts` into `opts-map` (`known-opts` at `:74-80`; the foldl at
   `:91-111`). **Change to all-kwargs:** signature `[fqdn & clauses]`; fold `clauses` into one clause-map the
   same way `opts` is folded today, with `known-clauses = {durable, ephemeral, ops, init, hibernate, stop,
   record-parent}`. Read each: `ops` (REQUIRED — `macro-error` if absent), `durable-fields` (default empty
   vector node `[]`), `ephemeral-fields` (default `[]`), `init`/`hibernate`/`stop`/`record-parent` (optional).
2. **`state-ty` mint `:123-124`.** Keep `state-ty = :<fqdn>::State`. **Add** `record-ty = :<fqdn>::Record`.
3. **State emission `:176-184`.** Today: `state-record = (:wat::Record::def ~state-ty ~state-fields)` (or holon
   parent). **Replace with TWO defs:**
   - `record-def` = `(:wat::Record::def ~record-ty ~durable-fields)` (or `:wat::holon::Record::def` when
     `record-parent` is holon — the existing `state-parent` branch now governs the RECORD, not the struct).
   - `state-def` = `(:wat::core::defstruct ~state-ty [durable <- ~record-ty ~@<ephemeral-field-children>])`
     — prepend the 3 tokens `durable <- ~record-ty` to the ephemeral fields. (`defstruct` splices as a
     type-decl — `classify_type_decl`, `types.rs:1620` — exactly as the Record did.)
4. **`:init` node + default `:130-153`.** Today: default identity `(fn [s <- ~state-ty] -> ~state-ty s)`;
   `init-def` emitted. **Change:** the `:init` param type is now `~record-ty`. Conditional default:
   - `:init` present → use it (its declared param is `[d <- ~record-ty]`, return `~state-ty`).
   - absent AND `ephemeral-fields` empty → default `(fn [d <- ~record-ty] -> ~state-ty (~<state-ty>/new d))`
     (struct ctor `:<fqdn>::State/new`; synthesize the `d` binder via `symbol-node` for hygiene).
   - absent AND `ephemeral-fields` non-empty → `(:wat::core::macro-error "<fqdn>: :ephemeral declares fields
     but no :init — the macro cannot construct ephemeral fields; provide :init : Record → State")`.
   - `ship-ty` (`:148`, the init param's type, used for `Admin::Init`) now resolves to `~record-ty`.
5. **`:stop` node `:159-174`.** Today default identity `(fn [s] -> ~state-ty s)`. **Change the default** to the
   record projection: `(fn [s <- ~state-ty] -> ~record-ty (~<state-ty>/durable s))` so `resp-ty` defaults to
   `~record-ty`. A user-provided `:stop` keeps its own declared `resp-ty` (any EDN type — unchanged extraction
   at `:168`). `stop-project-def` machinery unchanged otherwise.
6. **`:hibernate` node + projection (NEW — mirror `:stop`).** Add, near the `:stop` block: read `:hibernate`
   from the clause-map; default `(fn [s <- ~state-ty] -> ~record-ty (~<state-ty>/durable s))`; return type is
   **forced** to `~record-ty` (if a user `:hibernate` declares any other return type, `macro-error`). Emit
   `hibernate-project-def` = `(defn ~hibernate-project-name ~hib-params -> ~record-ty ~hib-body)`.
7. **Admin enum `:293-297`.** `:Init [seed <- ~ship-ty]` (now `~record-ty`) ✓ falls out of step 4. `:Resume
   [snapshot <- ~state-ty]` → `[snapshot <- ~record-ty]`.
8. **LineageUp enum `:299-302`.** `:Hibernated [snapshot <- ~state-ty]` → `[snapshot <- ~record-ty]`.
   (`:Final [resp <- ~resp-ty]` unchanged — `resp-ty` now defaults to record-ty per step 5.)
9. **`init-from-admin` `:316-329`.** Resume arm today: `((~admin-resume-kw snapshot) snapshot)` (identity).
   **Change to** `((~admin-resume-kw snapshot) (~init-name snapshot))` — resume now rebuilds the struct via
   `:init` from the saved record (Init and Resume both route through `:init`). Init arm unchanged.
10. **serve hibernate arm `:589-592`.** `(send' self (~lineage-hibernated-kw state))` → `(send' self
    (~lineage-hibernated-kw (~hibernate-project-name state)))`. (Stop arm `:585-588` already projects via
    `~stop-project-name` — unchanged; the projection default now returns the record.)
11. **`/hibernate` method `:752-764`.** Return type `~state-ty` → `~record-ty`; match binder unchanged.
12. **`/resume` `:899-914`.** `snapshot <- ~state-ty` → `~record-ty`.
13. **service-forms-def `:857-873`.** Today splices `~state-record` first. **Change to** `~record-def`
    `~state-def` (both, record first — the struct field references the record type).
14. **Final `do` `:945-963`.** Same: `~state-record` → `~record-def` `~state-def`.
15. **Untouched, confirm:** `start-params`/`start-body` (`:875-890`) — start's 2nd param is the `:init` param =
    now the record, so start already takes `(locus record)` with zero edit. Handle record (`:916-931`),
    serve-op-arms (`:454-558` — binds `s`=the struct; op bodies are USER code, macro does NOT rewrite them),
    request/response/Op/Reply folds, constructors, client methods, the lineage keyword mints — all unchanged.
    **KEEP all `LineageUp`/`lineage-*`/`init-from-admin`/`lineage-extract-addr` names** (4b-ii-b renames them).

## PART B — migrate every defservice definer (the cascade)

The migration recipe per definer (`:state [data]` services are all pure-data today — `:ephemeral` stays `[]`,
`:init` DEFAULTS, so most need NO `:init`):

1. `:state [fields]` → `:durable [fields]  :ephemeral []` (all current probes are pure-data → empty ephemeral).
2. **op bodies:** durable read `(<fqdn>::State/count s)` → `(<fqdn>::Record/count (<fqdn>::State/durable s))`;
   build next state `(<fqdn>::State c)` → `(<fqdn>::State/new (<fqdn>::Record c))`.
3. **`start` call sites:** `(<svc>/start locus (<svc>::State 0))` → `(<svc>/start locus (<svc>::Record 0))`.
4. **existing `:init`** (service-init-parity, service-hibernate-resume): today `(fn [seed <- :i64] -> :State
   (State seed))`. Pure-data → **DROP `:init`** (it now defaults); `start` ships the record directly
   (`(<svc>::Record <seed>)`). The "seeded" semantics now live in start taking the record.
5. **existing `:stop`** (service-stop-resp, service-hibernate-resume return `:i64`): keep the `:stop` fn but
   update its body to read through the record: `(State/count s)` → `(Record/count (State/durable s))`.
6. **`hibernate`/`resume` call sites** (service-hibernate-resume): `hibernate` now returns the **record**, and
   `resume` takes the **record** — `snap` is a `<svc>::Record`; the round-trip is unchanged in shape.

### Worked reference — `wat-tests/service-locus-parity.wat` migrated (copy this shape)

```clojure
(:wat::service::defservice :wat-tests::counter
  :durable [count <- :wat::core::i64]                          ;; was :state [count <- :i64]
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s
       (:wat-tests::counter::GetResponse
         (:wat-tests::counter::Record/count (:wat-tests::counter::State/durable s)))))
   (:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [c (:wat::core::i64::+
                           (:wat-tests::counter::Record/count (:wat-tests::counter::State/durable s)) n)]
       (:wat::service::Outcome::Reply
         (:wat-tests::counter::State/new (:wat-tests::counter::Record c))
         (:wat-tests::counter::IncrementResponse c))))])
;; deftest start sites:  (:wat-tests::counter/start (:wat::spawn::thread) (:wat-tests::counter::Record 0))
```

### The full migration set (~17 files — grep `defservice` to confirm none missed)

**wat-tests (5):** `service-locus-parity.wat` (counter_on), `service-init-parity.wat` (seeded),
`service-admin-facet.wat` (admin_stop), `service-stop-resp.wat` (stop_resp), `service-hibernate-resume.wat`
(hibernate_resume). Also check `timer-env-grab-parity.wat` (grep showed it touches defservice).
**tests/*.rs that DEFINE a defservice** (wat embedded in string literals — hand-edit, wat-fix can't reach
them): `probe_arc272_rs1_state_must_be_record.rs`, `probe_arc272_rs2_process_stop_returns_final_state.rs`,
`probe_arc272_rs2_thread_stop_returns_final_state.rs`, `probe_arc272_rs2_crash_surfaces_to_client.rs`,
`probe_arc272_6b_defservice_on_process.rs`, `probe_arc209_c1_defservice_op_enum.rs`,
`probe_arc209_c2_defservice_dispatch.rs`, `probe_arc209_c3_defservice_client_face.rs`,
`probe_arc209_locus_agnostic_start.rs`, `probe_arc209_spawned_marker.rs`, `probe_arc209_naming_conversion.rs`,
`probe_arc209_handle_protocol.rs`, `probe_arc265_acronym_registry.rs`, `probe_diagnostic_c3_macro_emits_record_def.rs`.
(Some only CALL methods, not define — those may need only the `start`/accessor updates. Let the compiler tell
you which: grep + fix what breaks.)

### The one inverted-law probe — `probe_arc272_rs1_state_must_be_record.rs` (REWRITE, don't just patch)

Its premise INVERTS: "`:state` mints a **record**" → "`:state`/`:ephemeral` mints a **struct**; the **`:durable`
record** is the EDN soul." Rewrite it: rename the file's intent to "state is a struct, the durable record is
the soul"; `field_vector_state_mints_base_record` → assert `:durable [fields]` mints `:<svc>::State` (a struct)
holding a `:<svc>::Record`; the holon test (`:record-parent :holon`) now parents the **`:durable` Record** (so
`(record? (State/durable s))` is the holon check, not `(record? s)` — `s` is a struct, `record?` on it is
false). **KEEP** the still-valid tests: `bare_type_keyword_state_is_rejected` (a scalar in `:durable` is still
rejected) and `unknown_trailing_option_is_rejected` (unknown clause still errors). Amend the `//!` header with
recognition of the inversion (arc 291 4b made state a struct).

## STOP triggers (halt + report; do NOT improvise)

1. **STOP if a macro site doesn't match the BRIEF** (e.g. the opts-fold can't cleanly extend to all-kwargs, or
   `defstruct` field-prepend won't splice) — report the exact site + checker/macro error. Do NOT bolt a
   workaround onto the macro.
2. **STOP if the cascade spreads beyond defservice definers/callers** — the blast radius is the ~17 files +
   `wat/service.wat`. If a NON-defservice file goes red, report it (it's a surprise the design didn't predict).
3. **STOP if you find yourself renaming `LineageUp`/`init-from-admin`/`lineage-extract-addr`** — that's
   4b-ii-b, NOT this strike. Keep those internal names exactly as they are.
4. **STOP if a probe's migration needs a genuine design call** (e.g. rs1's holon-parent semantics, or a probe
   asserting macro internals that no longer hold) — report it rather than guessing the intent.
5. You are a LEAF. Do NOT spawn subagents. If the change is bigger than these rooms, STOP and report.

## Blast radius

`wat/service.wat` (the macro) + the ~17 defservice definer/caller files (wat-tests + tests/*.rs). **NO other
`wat/*.wat`** (do NOT touch `wat/spawn.wat` — `launch` is record/struct-agnostic; the lineage protocol types
flow through unchanged). **NO `src/*.rs`** (4b-i already shipped `is_portable_type(Struct)→false`; the
type-gate is in place — a struct on the wire is already rejected). Pure wat-macro + wat/wat-in-rust migration.

## Expectations (scorecard — written before the strike; the orchestrator re-runs each)

| what | command | expected |
|---|---|---|
| counter round-trips through struct-State, both tiers | `cargo test -p wat --test test counter_on` | green |
| seeded (start takes a record) | `cargo test -p wat --test test seeded` | green |
| owner-only stop | `cargo test -p wat --test test admin_stop` | green |
| stop→resp projection | `cargo test -p wat --test test stop_resp` | green |
| **hibernate→kill→resume** (the keystone, now struct-State) | `cargo test -p wat --test test hibernate_resume` | green (==10) |
| rs1 rewritten green | `cargo test -p wat --test probe_arc272_rs1_state_must_be_record` | all green |
| arc272 rs2 / arc209 probes migrated | `cargo test -p wat --test probe_arc272_rs2_process_stop_returns_final_state` (+ siblings) | green |
| struct never on the wire (still enforced) | `cargo test -p wat --test nursery channel_of_all_edn_struct` | 1 passed |
| no new workspace regressions | orchestrator: `cargo test -p wat --no-fail-fast`, SET-diff vs HEAD | **∅** (deporder flap aside; floor ≈ 202) |

Runtime prediction: 60–120 min (the macro re-tool + ~17 migrations). Trap-doors: (a) the all-kwargs clause-fold
(the signature is the structural pivot — get the fold + defaults right first, then the rest cascades); (b) the
`defstruct` field-prepend (`durable <- ~record-ty` before the ephemeral children — build the field vector with
`with-children`/quasiquote-splice as the existing field-vectors do); (c) rs1's inversion (a rewrite, not a
patch). Commit nothing until the full gate is green and the SET-diff vs HEAD is ∅.
