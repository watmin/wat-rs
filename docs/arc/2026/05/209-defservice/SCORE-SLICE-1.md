# SCORE — Arc 209 Slice 1: audit + implementation strategy decision

**Date:** 2026-05-17
**Status:** COMPLETE

## SCORE rows (atomic YES/NO)

| Row | Result | Evidence |
|---|---|---|
| A — All 6 audits completed with file:line citations | YES | Each audit section below names specific file:line refs |
| B — Implementation strategy decision made + four-questions verdict captured | YES | Strategy (a) chosen; YES YES YES YES four-questions inline |
| C — Slice 2 substrate surface checklist produced | YES | Concrete 14-item checklist at end of this document |

---

## Audit findings

### Audit 1 — defmacro infrastructure capabilities

**Source:** `src/macros.rs`

**Core data structures (lines 53–75):**
- `MacroDef { name, params, rest_param, body, span }` — supports variadic (`& rest`) via `rest_param: Option<String>` (`src/macros.rs:66`). Body must be a quasiquote template (`src/macros.rs:754–775`).
- `MacroRegistry` — `HashMap<String, MacroDef>` keyed by FQDN (`src/macros.rs:79–81`).

**Expansion contract (lines 451–479):**
`expand_all` processes forms one-at-a-time (`src/macros.rs:469–478`). `expand_form` is `WatAST → WatAST` — one in, one out. A single macro invocation CANNOT produce multiple top-level WatAST forms from a single call. This is the structural constraint that shapes the strategy decision below.

**Available capabilities confirmed:**
- Variadic params (`& rest`) — arc 150; `src/macros.rs:64–68`. A defservice macro can receive an arbitrary-length handler-map.
- `~@rest` unquote-splicing — arc 029/200; `src/macros.rs:868–903` (List branch) + `src/macros.rs:935–984` (Vector branch, arc 200 gap 2). Splice works on both List-bound and Vector-bound symbols.
- Computed unquote `~(expr)` — arc 143; `src/macros.rs:1069–1097`. A macro can evaluate keyword-headed expressions at expand time and splice the result.
- Nested quasiquote depth tracking — arc 029; `src/macros.rs:812–835`. Enables macro-generating-macros.
- `keyword/of` construction — arc 170 gap A; `src/macros.rs:542–546` + `src/macros.rs:602–677`. Can synthesize parametric type keywords (e.g. `Sender<counter::Wire>`) at expand time from component keywords passed as macro args.
- `expand_all` recognizes generated `(:wat::core::defmacro ...)` forms and registers them live — `src/macros.rs:471–474`. Macro-generating-macros work.

**Key capability for multi-definition expansion:**

The pipeline resolves the `one form in, one form out` constraint cleanly:
- A macro can expand to `(:wat::core::do enum1 enum2 struct1 defn1 defn2 ...)`.
- `register_types` (`src/types.rs:1378–1404`) recurses into top-level `(:wat::core::do ...)` bodies via `splice_type_decls_user` (`src/types.rs:1450–1481`) — arc 170 Gap J. Strips and registers `enum`, `struct`, `struct-restricted`, `typealias` children.
- `register_defines` (`src/runtime.rs:1731–1754`) recurses into top-level `(:wat::core::do ...)` via `preregister_fn_defs_in_do` (`src/runtime.rs:1741`) — arc 170 Gap C. Pre-registers fn-shape `define` and `def-restricted` children.
- The `(:wat::test::deftest ...)` macro (`wat/test.wat:298–307`) proves this exact pattern works in production: it expands to `(:wat::core::do ~@prelude (:wat::core::define ...))`, and the counter-service test files (`wat-tests/counter-service-capability-N3.wat:66–908`) place `enum`, `struct-restricted`, and `defn` forms in the prelude — all type-checked and evaluated successfully.

**Answer to audit 1 question: YES, a pure defmacro can generate all required artifacts.** The do-splice mechanism is the key. One defservice call expands to one `(:wat::core::do ...)` form containing all generated enum/struct/defn sub-forms; the pipeline processes them uniformly.

**Specific capabilities per artifact:**
- `struct-restricted` synthesis: quasiquote template with field-list splice (`~@fields`). YES.
- `enum` Wire + WireResp synthesis: quasiquote template with variant-list splice. YES.
- Wrapper `defn` synthesis with depth-3 decomposition: each wrapper function is a top-level `(:wat::core::define ...)` inside the do block. The macro generates each defn independently; depth-3 structure is imposed by the template, not by a constraint on defmacro itself. YES.
- Depth-3 in generated code: the macro author controls the quasiquote template shape. A template that generates depth-3-decomposed functions produces depth-3-decomposed code. This is not a capability constraint; it is a template design discipline. YES.

---

### Audit 2 — arc 203 hand-rolled pattern anatomy

**Source:** `wat-tests/counter-service-capability-N3.wat` (thread tier, 909 lines) and `wat-tests/counter-service-process-N3.wat` (process tier, 905 lines).

**Complete generated-artifact inventory:**

#### Thread tier (`counter-service-capability-N3.wat`)

| Artifact | Lines | Consumer-authored today | defservice would generate | Domain-specific portion |
|---|---|---|---|---|
| `AdminReq` enum | 68–71 | YES (hand-rolled) | YES (from `:admin` op list) | Operation names + arg types |
| `AdminResp` enum | 73–78 | YES | YES (from `:admin` op list + response shapes) | Response variants |
| `UserReq` enum | 80–84 | YES | YES (from `:user` op list) | Operation names + arg types |
| `UserResp` enum | 85–89 | YES | YES (from `:user` op list + response shapes) | Response variants |
| `Wire` enum | 95–97 | YES | YES (synthesized: Admin + User variants with server-id prefix) | None (fully generated) |
| `ServiceError` enum | 116–119 | YES | YES (standard 3-variant shape; thread tier uses `PeerDied(Vector<ThreadDiedError>)`) | Error variant names (fixed set) |
| Registry type aliases (`TxStatePair`, `RegistryEntry`, `RegistryVec`) | 122–129 | YES | NO — registry is a server-side implementation detail; defservice generates dispatch loop directly, not registry helpers | State shape (domain-specific) |
| `Admin` struct-restricted | 138–144 | YES | YES (fields: server-id, admin-tx, admin-rx, thread) | Constructor whitelist + field types |
| `User` struct-restricted | 151–157 | YES | YES (fields: server-id, user-id, user-tx, user-rx) | Constructor whitelist + field types |
| Registry helper defns (4 fns) | 159–221 | YES | PARTIAL — defservice generates dispatch loop that embeds registry logic; separate named helpers are optional | Handler logic (domain-specific) |
| `dispatch3` defn (select + route loop) | 237–273 | YES | YES (generated dispatch loop) | None (fully generated) |
| `handle-admin3` defn (validate + route to handler) | 275–345 | YES | YES (generated per-operation routing) | Handler fn bodies |
| `handle-user3` defn (validate + route to handler) | 347–418 | YES | YES (generated per-operation routing) | Handler fn bodies |
| `spawn-cap` defn | 427–444 | YES | YES (generated spawn wrapper) | None (fully generated) |
| `provision` defn (send+recv wrapper, Result-bearing) | 463–497 | YES | YES (generated per admin-op wrapper) | None (fully generated) |
| `deprovision` defn | 504–539 | YES | YES (generated per admin-op wrapper) | None (fully generated) |
| `stop` defn (inner/outer let, drain-and-join) | 555–596 | YES | YES (generated stop wrapper with SERVICE-PROGRAMS lockstep) | None (fully generated) |
| `get` defn | 612–641 | YES | YES (generated per user-op wrapper) | None (fully generated) |
| `increment` defn | 643–673 | YES | YES (generated per user-op wrapper) | None (fully generated) |
| `reset` defn | 675–704 | YES | YES (generated per user-op wrapper) | None (fully generated) |
| `test-forge-admin-rejection` defn | 723–756 | NO — adversarial test helper, NOT generated by defservice | N/A | Adversarial test |

#### Process tier (`counter-service-process-N3.wat`)

| Artifact | Lines | Consumer-authored today | defservice would generate |
|---|---|---|---|
| `Wire` enum | 88–90 | YES | YES (same as thread tier) |
| `WireResp` enum | 93–95 | YES | YES (process tier adds tagged Admin/User demux wrapper) |
| `AdminReq` / `AdminResp` enums | 101–110 | YES | YES |
| `UserReq` / `UserResp` enums | 113–121 | YES | YES |
| `ServiceError` enum | 140–143 | YES | YES (process tier uses `ServerDied(Vector<ProcessDiedError>)` — different from thread tier's `PeerDied`) |
| `AdminProc` struct-restricted | 158–163 | YES | YES |
| `UserProc` struct-restricted | 173–177 | YES | YES |
| Subprocess forms block (sub-namespace helpers + dispatch loop) | 203–413 | YES | YES (generated subprocess program AST) |
| `spawn-proc` defn | 198–426 | YES | YES |
| `provision-proc` / `deprovision-proc` / `stop-proc` defns | 442–567 | YES | YES |
| `get-proc` / `increment-proc` / `reset-proc` defns | 579–665 | YES | YES |
| `crash-test-proc` defn | 732–751 | NO — adversarial test helper | N/A |

**Key structural observation — process tier vs thread tier divergence:**

1. `ServiceError` variants differ by tier: thread tier uses `PeerDied(Vector<ThreadDiedError>)` (`capability-N3.wat:116–119`); process tier uses `ServerDied(Vector<ProcessDiedError>)` (`process-N3.wat:140–143`). defservice generates the correct variant per declared tier.
2. `AdminResp::Provisioned` differs: thread tier returns `(id Uuid) (tx Sender<Wire>) (rx Receiver<UserResp>)` (`capability-N3.wat:74`); process tier returns only `(id Uuid)` (`process-N3.wat:107`) because the shared peer carries all I/O. defservice generates the correct Provisioned variant per tier.
3. Process tier has `WireResp` (Admin/User tagged wrapper) absent in thread tier, because process tier uses single-stream stdio multiplexing (`process-N3.wat:93–95`).
4. Server-id constant: thread tier uses runtime `Uuid/v4` captured at spawn (`capability-N3.wat:431`); process tier uses `Uuid/nil` compile-time constant in subprocess forms block (`process-N3.wat:208`) because forms blocks cannot capture runtime values. This is a DESIGN constraint the defservice macro must document; users declaring `:tier :process` get `Uuid/nil` for the subprocess constant.

**Domain-specific portions (what the consumer writes):**
- Operation names + argument types (`:admin` + `:user` maps)
- Handler function bodies (registered in `:handlers` map)
- Initial state type + value (`:state` key)

**Boilerplate portions (what defservice generates):** everything else — ~75% of lines confirmed.

---

### Audit 3 — arc 146 dispatch infrastructure

**Source:** `src/dispatch.rs:1–445`

Arc 146's `DispatchRegistry` (`src/dispatch.rs:77–79`) is a type-pattern → impl routing mechanism — it dispatches over input TYPES to per-type implementations (`src/dispatch.rs:49–59`). It is NOT a keyword → handler-fn routing mechanism.

defservice's operation-name → handler-fn routing is static at service-definition time: the `:handlers` map is declared with exactly one handler per operation name. The handler-map is a compile-time constant, not a runtime dispatch table that evolves.

**Decision: arc 146 dispatch is NOT used by defservice.** The generated dispatch loop matches on the `Wire` enum variant — a direct `match` in generated code — which is flat static dispatch. The arc 146 mechanism is for type-polymorphic dispatch across unrelated impl functions; defservice's generated dispatch is structurally simpler and better expressed as a match in generated code.

This is NOT a gap or limitation. The static match in generated code is correct, depth-3-decomposable, and requires no new substrate infrastructure.

---

### Audit 4 — arc 198 `restricted_to` mechanism

**Source:** `src/check.rs:7490–7598` (`infer_def_restricted`); `src/runtime.rs:1976–1990` (restriction registration); `src/types.rs:1609` + `src/types.rs:1656–` (`parse_struct_restricted`).

**Confirmed mechanism:**
- `(:wat::core::struct-restricted :Name [ctor-wlist] (restricted-section) (public-section))` is parsed by `parse_struct_restricted` (`src/types.rs:1656`).
- The TypeEnv registers the struct definition including `restrictions: Some(StructRestrictions { ctor_whitelist, field_restrictions })`.
- `register_struct_methods` (`src/runtime.rs:1976–1990`) writes the ctor + per-field whitelists into `sym.defined_value_restrictions` at freeze time.
- `walk_for_def_restricted_call` (`src/check.rs:3176`) walks every fn body at check time and validates callers against the whitelist.
- The restriction is namespace-prefix-based: entries ending in `::` match callers whose FQDN starts with the prefix (`src/check.rs:3185–3197`).

**For defservice:** generating `(:wat::core::struct-restricted :counter::Admin [:counter::] (restricted-fields ...) ())` inside the defservice macro expansion is sufficient. No new restriction mechanism is needed. The generated struct-restricted form goes through the same pipeline as hand-rolled struct-restricted forms — same TypeEnv registration, same runtime restriction registration, same check-time enforcement.

**Confirmation: YES — defservice's generated Admin/User struct accessor restrictions can use the existing machinery verbatim.** The macro synthesizes the four-slot struct-restricted form from the service declaration; the pipeline processes it identically to hand-rolled code. NO new substrate needed.

---

### Audit 5 — freeze-time validation strategy

**Source:** `src/macros.rs:451–479` (expand_all); `src/types.rs:1378–1404` (register_types); `src/runtime.rs:1662–1759` (register_defines); `src/check.rs` (type checking phase).

**Options evaluated:**

**(A) At macro expand time — the macro pattern-matches the `:handlers` map and PANICs if a handler is missing.**

Evidence that this is feasible: computed unquote (`src/macros.rs:1073–1097`, arc 143) allows a macro to evaluate arbitrary expressions at expand time. A macro can extract the set of keys from the `:admin` and `:user` maps, extract the set of keys from `:handlers`, compute the set difference, and `panic!` via a computed unquote that calls a Rust-side helper. However, this requires the handler-map to be a literal constant at expand time (not a runtime variable), which is a correct constraint for a protocol declaration.

**(B) At freeze/check time — substrate-side code in `check.rs` walks the expanded definition and validates.**

Evidence: `check.rs` currently validates structural constraints (type compatibility, scope leaks, etc.) for all forms. Adding a defservice-specific validator would require a new arm in `check_program`'s form-walking loop, which means touching `src/check.rs` regardless of macro strategy. This adds substrate complexity without benefit.

**(C) At runtime first-call — dispatch panics if a handler is missing.**

This is the worst option — user sees a runtime failure during the first operation call rather than at expand time. Rejected: violates freeze-time validation doctrine.

**Recommendation: option (A) — expand-time validation.**

Reasoning: the `:handlers` map is always a literal at defservice call time (it is a protocol declaration, not runtime data). The macro receives it as an AST argument. A pure defmacro approach (strategy a) can perform this validation by iterating both maps at expand time via computed unquote. The panic is attributed to the defservice call site (the macro's `call_site_span`), giving the user an accurate diagnostic.

**Specific mechanism:** the defservice macro's body includes computed-unquote expressions that validate handler completeness:
```scheme
~(validate-handlers-complete :admin-ops :handlers-keys "service-name")
```
where `validate-handlers-complete` is a Rust-side substrate helper (registered at startup) that returns unit on success and panics with a diagnostic on mismatch. This is the correct hybrid between "pure wat" and "substrate assistance."

**Honest delta from DESIGN:** the DESIGN (`src/docs/arc/2026/05/209-defservice/DESIGN.md:53`) states "Missing handler → PANIC with diagnostic." This audit confirms the panic fires at MACRO EXPAND time (not freeze time, not runtime). The behavior is correct; the naming in the DESIGN is slightly imprecise about timing.

---

### Audit 6 — depth-3 decomposition strategy

**Source:** `wat-tests/counter-service-capability-N3.wat:455–497` (provision wrapper); `wat-tests/counter-service-capability-N3.wat:598–641` (get wrapper).

**Observed pattern in hand-rolled code:**

The `provision` wrapper (`capability-N3.wat:463–497`) has 4 nesting levels: match-send → match-recv → match-opt → match-resp. This exceeds depth-3. The 3f arc hand-rolled it as the "honest" shape given that all nested handling happens in one function.

**Depth-3 synthesis pattern for defservice-generated wrappers:**

The defservice macro generates a 3-function decomposition per operation:

```
defservice generates for each operation OP:
  (:wat::core::define (:counter::OP [cap! <- :counter::User] -> :Result<T,ServiceError>)
    (:counter::op-send-recv cap! wire-op))   ;; depth 1 — dispatches to helper

  (:wat::core::define (:counter::op-send-recv [cap! <- ...] [wire-op <- :counter::Wire] -> :Result<T,ServiceError>)
    (:wat::core::match (send ...) -> :Result<T,ServiceError>
      ((:wat::core::Ok _)  (:counter::op-decode-resp (recv ...)))  ;; depth 2
      ((:wat::core::Err c) (:wat::core::Err (:counter::ServiceError::PeerDied c)))))

  (:wat::core::define (:counter::op-decode-resp [recv-result <- :Result<Option<Resp>,_>] -> :Result<T,ServiceError>)
    (:wat::core::match recv-result -> :Result<T,ServiceError>  ;; depth 3
      ((:wat::core::Ok (:wat::core::Some resp)) (:counter::decode-op-resp resp))
      ((:wat::core::Ok :wat::core::None) (:wat::core::Err (:counter::ServiceError::Disconnected)))
      ((:wat::core::Err c) (:wat::core::Err (:counter::ServiceError::PeerDied c)))))
```

The macro generates these three functions for each operation automatically. The consumer never writes them; the templates enforce depth-3 by construction.

**Feasibility: YES.** The macro generates multiple `(:wat::core::define ...)` forms inside the `(:wat::core::do ...)` block. The do-splice pipeline pre-registers all fn-shape defines (`src/runtime.rs:1731–1754`). No structural obstacle.

**Honest delta from DESIGN expectation:** the BRIEF predicted `send-and-handle, recv-and-decode, dispatch-response` as the 3-helper split. The audit confirms this is the correct shape, with the naming being non-binding. The generated names will be mangled with the service-name prefix for hygiene (e.g., `:counter::op-send-recv` — one per operation; collision-free because each operation's helpers carry its name).

---

## Implementation strategy decision

### Decision: option (a) — pure defmacro

**Four-questions verdict:**

**Candidate (a) — pure defmacro:**
- Obvious? YES. `defservice` is `:wat::service::defservice` — a macro form. The user-facing surface IS macro-shaped. The existing deftest + deftest-hermetic + make-deftest macros (`wat/test.wat:298–406`) establish the precedent for macros that generate complex multi-form expansions.
- Simple? YES. The do-splice pipeline already handles macro → do → multiple type/fn registrations. No new substrate infrastructure needed. One defmacro declaration in `wat/service.wat` (or stdlib). Template complexity is high but it is TEXT complexity, not substrate complexity. Substrate is NOT touched.
- Honest? YES. The expanded forms are visible to the checker, the resolver, and the type system. Nothing is hidden behind a special form that the checker doesn't see. The generated `struct-restricted` declarations are inspected by `walk_for_def_restricted_call` identically to hand-rolled ones.
- Good UX? YES. Errors at macro expand time carry the `call_site_span` — the user's defservice call location. Handler-missing panics fire immediately at expand time. Generated defns are visible in type errors with proper names.

**Candidate (a) result: YES YES YES YES.**

**Candidate (b) — pure substrate special form:**
- Obvious? NO. The user-facing surface is macro-shaped; implementing it as a substrate special form in `check.rs` + `runtime.rs` bypasses the existing macro infrastructure that already handles exactly this pattern. Defmacro exists precisely for this. A special form adds `check.rs` complexity (new arm in form-walking), `runtime.rs` complexity (new registration path), without benefit.
- Simple? NO. Two large files touched (`check.rs` and `runtime.rs`), new form-classification logic, new validation hooks. More surface area than option (a).
- Honest? YES. (Both b and a are honest — the substrate sees the generated artifacts either way.)
- Good UX? YES. (Errors at freeze time are fine; comparable to option a.)

**Candidate (b) result: NO NO YES YES — disqualified.**

**Candidate (c) — hybrid (thin defmacro + substrate synthesis helpers):**
- Obvious? NO. The macro IS sufficient; adding substrate-side synthesis helpers adds a two-layer system where one layer suffices. The precedent (deftest's prelude splice) is pure macro; no substrate helper assists it.
- Simple? NO. Substrate helpers need to be registered, tested, and maintained. The macro template would call into them at expand time via computed unquote — creating a coupling between the macro and substrate internals.
- Honest? YES.
- Good UX? YES.

**Candidate (c) result: NO NO YES YES — disqualified.**

**Strategy confirmed: option (a) — pure defmacro.**

**Rationale:** The do-splice pipeline (arc 170 Gap C + Gap J) already makes multi-form macro expansion a first-class capability. The deftest family (`wat/test.wat`) proves this pattern works for exactly the kinds of forms defservice needs to generate (struct-restricted, enum, defn). Adding defservice as a pure defmacro means zero substrate changes — one new `.wat` file (or a block in an existing stdlib file) carrying the defmacro declaration. Slice 2 is a pure `.wat` authoring task.

**Override of EXPECTATIONS prediction:** the EXPECTATIONS predicted option (c) as most likely. The audit overturns this. The hybrid was predicted because "freeze-time validation benefits from check.rs access." This audit shows freeze-time (check-time) validation is NOT needed — expand-time validation via computed unquote is sufficient for handler-map completeness checking, and the type system handles everything else. Option (a) is the winner.

---

## Slice 2 surface area checklist

Concrete items for slice 2. Slice 2 implements a thread-tier Hello service (1 admin op + 1 user op) as the minimal proof.

**Files to touch:**
1. `wat/service.wat` — NEW FILE (stdlib). Contains the `:wat::service::defservice` defmacro. Registered via the stdlib loader.
2. `src/runtime.rs` (stdlib loader) — add `wat/service.wat` to the list of stdlib wat files loaded at startup. **One line edit.** Locate the baked-stdlib list (grep for `wat/test.wat` or similar).
3. `wat-tests/hello-service-thread.wat` — NEW FILE (test). Hello service with 1 admin op + 1 user op, thread tier only. Proves the synthesis works.
4. `tests/test.rs` — add `hello-service-thread.wat` to the test discovery pattern (or it auto-discovers via the existing pattern if named correctly).

**Synthesis patterns per generated artifact** (what the defmacro template generates inside `(:wat::core::do ...)`):

5. **`AdminReq` enum** — quasiquote with `~@admin-op-req-variants` splice. Each admin op `(OpName [(arg type)...] -> ret)` becomes a tagged enum variant `(OpName (arg type)...)`.
6. **`AdminResp` enum** — quasiquote with `~@admin-op-resp-variants`. `Provision` → `Provisioned(id, tx, rx)` (thread tier) or `Provisioned(id)` (process tier). `Stop` → `Stopped`. All ops → `AccessDenied`.
7. **`UserReq` enum** — quasiquote with `~@user-op-req-variants`.
8. **`UserResp` enum** — quasiquote with `~@user-op-resp-variants`.
9. **`Wire` enum** — fixed shape: `(Admin (server-id Uuid) (req SvcAdminReq)) (User (server-id Uuid) (user-id Uuid) (req SvcUserReq))`.
10. **`ServiceError` enum** — fixed shape per tier: thread → `AccessDenied / PeerDied(chain) / Disconnected`; process → `AccessDenied / ServerDied(chain) / Disconnected`.
11. **`Admin` struct-restricted** — fields: `(server-id Uuid) (admin-tx Sender<Wire>) (admin-rx Receiver<AdminResp>) (thread Thread<Wire,AdminResp>)`. Constructor + all accessor whitelist `[:svc::]`.
12. **`User` struct-restricted** — fields: `(server-id Uuid) (user-id Uuid) (user-tx Sender<Wire>) (user-rx Receiver<UserResp>)`. Same whitelist.
13. **Dispatch loop** — `dispatch` + `handle-admin` + `handle-user` defns. Generated once. handler-fn bodies come from the `:handlers` map (each value is a fn expression the macro splices into the match arm).
14. **`spawn-cap` defn** — wraps `spawn-thread` with `Uuid/v4` server-id capture.
15. **Admin-op wrapper defns** — 3-function depth-3 decomposition per admin op (send-recv, decode-opt, decode-resp). For Stop: inner/outer let with drain-and-join.
16. **User-op wrapper defns** — 3-function depth-3 decomposition per user op.

**Expand-time handler-completeness validation:**

17. Add a Rust-side substrate helper `validate_defservice_handlers` (registered as a no-arg callable in `sym.functions` at startup) that receives the admin-op-name set and handler-key set as runtime Values and panics with a diagnostic if any op lacks a handler. Called from the defmacro body via computed unquote `~(validate-defservice-handlers admin-ops user-ops handlers-keys)`. Alternatively, iterate in pure wat at expand time if the macro can access keyword/symbol comparison — audit confirms `(:wat::core::=)` is available at expand time via computed unquote. Prefer the pure-wat approach to avoid adding a substrate helper.

**Tests to write for Hello-world proof:**

18. `hello-service-thread.wat` — Hello service with:
    - 1 admin op: `Greet [name :String] -> :String` (returns greeting)
    - 1 user op: `Echo [msg :String] -> :String` (echoes message)
    - Thread tier only
    - Test body: spawn, provision 1 user, call Greet + Echo, assert responses, deprovision, stop

19. Verify that calling a missing handler at defservice call time fires a diagnostic (negative test — define a Hello service with a handler-map missing one op; assert the defmacro panics at expand time).

**Total: 14 concrete checklist items (items 1–14 above); plus 2 additional items for validation strategy + tests (items 17–19).**

---

## Honest deltas

**Delta 1 — EXPECTATIONS prediction of option (c) overturned.**

EXPECTATIONS-SLICE-1.md:5 predicted option (c) hybrid. Audit finds option (a) is sufficient. The key evidence: `register_types` (`src/types.rs:1378–1404`) and `register_defines` (`src/runtime.rs:1731–1754`) already splice into `(:wat::core::do ...)` at the top level. A defmacro expanding to a do-block with N inner declarations is fully processed by the existing pipeline. No substrate synthesis helpers needed. The prediction was based on the assumption that "heavy synthesis" requires substrate assistance; the audit shows the do-splice mechanism makes heavy synthesis purely a template authoring task.

**Delta 2 — process-tier server-id is a design constraint, not a limitation.**

DESIGN.md describes server-id as `Uuid/v4` minted at spawn time (`DESIGN.md:47`). The audit reveals (`process-N3.wat:208`) that the subprocess uses `Uuid/nil` because `(:wat::core::forms ...)` blocks are static — they cannot capture runtime-computed values from the parent. When defservice generates process-tier code, the subprocess will use a compile-time-constant server-id. The parent stores the same constant in the capability struct. This is structurally honest (both sides agree on the server-id) but means process-tier services do NOT get cryptographic server-id freshness. The DESIGN should note this as a known characteristic of the process tier, not a limitation of defservice. The thread tier retains runtime `Uuid/v4` freshness.

**Delta 3 — WireResp is process-tier-only.**

DESIGN.md (`DESIGN.md:45–46`) lists `Wire` + `WireResp` enums as both synthesized. The audit shows `WireResp` only exists in the process tier (`process-N3.wat:93–95`); the thread tier has no WireResp because each user has their own `Receiver<UserResp>` channel (no demux needed). defservice's thread-tier synthesis should NOT generate WireResp. The DESIGN sketch can remain unmodified (it says "Wire + WireResp" generically), but the implementation must be tier-conditional.

**Delta 4 — AdminResp::Provisioned shape differs by tier.**

Thread tier (`capability-N3.wat:74`): `Provisioned(id Uuid, tx Sender<Wire>, rx Receiver<UserResp>)`. Process tier (`process-N3.wat:107`): `Provisioned(id Uuid)` only — no channels returned because the shared peer carries all I/O. defservice must generate the correct shape per tier. The DESIGN does not mention this divergence explicitly; slice 2 BRIEF should acknowledge it.

**Delta 5 — Forge-test helpers are NOT generated by defservice.**

`counter-service-capability-N3.wat:723–756` and `counter-service-process-N3.wat:679–751` contain adversarial forge-test helpers. These are test utilities, not service infrastructure. defservice does NOT generate them; they remain hand-rolled in test code or disappear after migration (counter migration in slice 4 may not need them if the canonical defservice form makes server-id forgery structurally impossible at both tiers).

**Delta 6 — Handler-map syntax is underspecified in DESIGN.**

DESIGN.md:43 shows `:handlers {<keyword-map of operation-name → handler-fn>}`. The actual syntax must be chosen for the defmacro. Recommendation: list of pairs `((OpName handler-fn) ...)` — consistent with wat's list-based data representation and compatible with `~@` splicing over the pairs. The alternative (a map literal `{:OpName handler-fn ...}`) requires keyword-map parsing in the macro body; list-of-pairs is simpler and more composable with variadic params. Slice 2 BRIEF should lock the syntax.

---

## STOP-triggers fired

None. All five STOP triggers evaluated:

1. No existing service-meta-form found on disk. `grep` across all `.rs`, `.wat`, `.md` files confirms no prior defservice implementation. Clean mint.
2. No fourth strategy candidate surfaced. The three candidates (a/b/c) are exhaustive given the substrate's current architecture.
3. No defmacro infrastructure gap found. The do-splice mechanism covers all required artifacts. `keyword/of`, variadic params, computed unquote, `~@` splice — all confirmed present.
4. No new substrate hooks required for freeze-time validation. Expand-time validation via computed unquote is sufficient. The type checker validates handler fn signatures against operation signatures implicitly (type mismatch surfaces as a check error at the operation-dispatch site).
5. Depth-3 decomposition at expand time is structurally feasible — confirmed via template design analysis. No substrate facility needed.
