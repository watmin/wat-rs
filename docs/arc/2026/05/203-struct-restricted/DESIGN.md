# Arc 203 — `struct-restricted` (capability-restricted struct constructor + per-accessor whitelists)

**Status:** OPEN 2026-05-16.

**Pedigree:** Arc 198 shipped function-level + value-binding-level access control (`def-restricted` / `defn-restricted` / `#[restricted_to(...)]`). Arc 203 extends the same mechanism to **structs** — both the auto-synthesized constructor (`T/new`) and each per-field accessor (`T/<field>`). One walker (arc 198's `walk_for_def_restricted_call`) covers everything; one HashMap (`defined_value_restrictions`) is the source of truth.

## Motivation

Capability-based security via secret-witness (issued opaque types whose constructor is private to the issuer) requires substrate-level mint protection. arc 198 covers the function case; struct constructors auto-synthesized by `register_struct_methods` (src/runtime.rs:1879) need a parallel declaration surface that registers their restrictions at struct-decl time.

**First consumer:** Counter actor's ServiceWithProvisioning demo (task #338) uses a `Counter/Client` capability struct — issued by server, held by user, validated on Deprovision. Per-field restrictions distinguish "server reads to validate" fields from "user reads to talk" fields without forcing the consumer to compose two structs.

## Settled form

```scheme
(:wat::core::struct-restricted :Name
  [<constructor-whitelist-prefixes>...]            ;; slot 1 — explicit whitelist for Name/new
  ([<wlist>] field <- :T, ...)                     ;; slot 2 — restricted attrs (variadic; each has own whitelist)
  (field <- :T, ...))                              ;; slot 3 — public attrs (variadic; no whitelist)
```

Four positional slots after head:

1. **Name** — type keyword (e.g., `:Counter/Client`)
2. **Constructor whitelist** — Vector of keyword prefixes governing `Name/new`
3. **Restricted-attrs section** — List of variadic `[wlist] name <- :T` triples; each restricted field gets its own whitelist registered against the synthesized `Name/<field>` accessor
4. **Public-attrs section** — List of variadic `name <- :T` pairs; no restriction registered for these accessors

**No inheritance.** Every whitelist explicit at declaration. Empty restricted-section `()` means "all fields public except the constructor is still restricted." Empty public-section `()` means "everything restricted including all reads."

### Worked example — Counter/Client

```scheme
(:wat::core::struct-restricted :Counter/Client
  [:counter/]                                                  ;; only :counter/* can mint Client/new
  ([:counter/] server-id <- :wat::core::keyword                ;; only :counter/* can read server-id (validates issuance)
   [:counter/] client-id <- :wat::core::keyword)               ;; only :counter/* can read client-id (registry key)
  (in!  <- :wat::core::Sender<Counter/UserReq>                 ;; any caller can read in! (user needs to talk)
   out! <- :wat::core::Receiver<Counter/UserResp>))            ;; any caller can read out! (user needs to listen)
```

### Degenerate cases

```scheme
;; All-restricted struct (every read + the constructor restricted to same whitelist)
(:wat::core::struct-restricted :Secret
  [:internal/]
  ([:internal/] secret-1 <- :T1
   [:internal/] secret-2 <- :T2)
  ())

;; Constructor-only restriction (mint protection; all fields readable by holder)
(:wat::core::struct-restricted :Token
  [:auth/]
  ()                                                           ;; no restricted attrs
  (id      <- :wat::core::keyword
   payload <- :wat::core::Bytes))
```

## Four questions verdict (atomic; per `feedback_four_questions_yes_no`)

| | Verdict | Why |
|---|---|---|
| Obvious | YES | Two-section form is a clear visual delimiter; per-line whitelist makes policy local to declaration; mirrors arc 198's "name + whitelist" shape, extended one level |
| Simple | YES | N attrs with restrictions = N uniform registrations against the same arc 198 HashMap (per `feedback_simple_is_uniform_composition`); mechanism is unchanged from arc 198 — only the declaration surface is new |
| Honest | YES | Heterogeneous visibility expressed directly at declaration; no bundling unrelated policies under one whitelist; no forced composition gymnastics; mint and read decisions independently expressed |
| Good UX | YES | Domain author declares the struct the way the domain shapes — some fields are sensitive, some aren't; expresses capability patterns without composition workarounds |

YES YES YES YES.

## Substrate touchpoints

Verified during DESIGN drafting (FM 1 + FM 9 + FM 13):

- **Arc 198 storage:** `CheckEnv.defined_value_restrictions: HashMap<String, Vec<String>>` (src/check.rs:1637); mirrored on `SymbolTable` per arc 198 slice 2 Stone 1
- **Arc 198 walker:** `walk_for_def_restricted_call` (src/check.rs:3152+); iterates call sites and matches enclosing fn FQDN against the whitelist; fires `CheckError::DefRestrictedCallerNotAllowed` on mismatch — **reused unchanged for struct case**
- **Struct decl recognition:** `:wat::core::struct` keyword head at src/check.rs:5260; runtime-side `register_runtime_defs_form` at src/runtime.rs:2224+ detects struct shape at src/runtime.rs:2410
- **Struct accessor synthesis:** `register_struct_methods` (src/runtime.rs:1879) creates `Type/new` Function + `Type/<field>` Functions per declared struct, inserts into `sym.functions` — **arc 203 extends this path** to also populate `defined_value_restrictions` per the declared whitelists
- **`def-restricted` shape parser:** `infer_def_restricted` (src/check.rs:7478) parses the prefix vector and validates whitelist entries are keywords — **arc 203 mirrors this pattern** for each per-field whitelist + the ctor whitelist

## Out of scope (affirmatively named)

- **Rust-side complement:** arc 198's `#[restricted_to(...)]` already covers Rust-defined wat-visible constructors emitted as `eval_*_new` fns. No new Rust-side mechanism needed for arc 203. The asymmetry is honest: wat-defined structs auto-synthesize accessors via `register_struct_methods` and need a new declaration surface; Rust-defined ones already have one.
- **Inheritance / implicit defaults:** every whitelist explicit; no "ctor inherits union of field wlists" magic; no "implicit empty section" — sections must be present even when empty.
- **Per-field write restrictions:** N/A (wat values are immutable; there are no writes to restrict).

## Slicing

Per arc 198 calibration lesson — bounded stones beat one-shot multi-piece changes (`feedback_iterative_complexity`). Refined 2026-05-17 post-slice-1: split original "slice 2 consumer integration" into a minimal capability proof + full ServiceWithProvisioning proof, per user direction "the least amount of oneshotting we can entertain."

### Slice 1 — substrate primitive minting (SHIPPED 2026-05-17 at `26c9298`)

Parser arm + check.rs validation + runtime registration extension + minimal proof tests. SCORE: 6/6 PASS. Honest delta: type-declaration forms flow through `parse_type_decl` (types.rs) at register_types step 5; no `infer_struct_restricted` in check.rs needed (the BRIEF assumption was wrong; SCORE corrects it).

### Slice 2 — minimal Counter/Client capability proof (NEXT)

Minimal first consumer of struct-restricted: a Counter actor that ISSUES `:counter::Client` capability values to its caller via the restricted constructor. Single user, single state, simple round-trip. Proves struct-restricted works in real consumer context (not just isolated unit tests).

**Scope:**
- New wat-tests file `wat-tests/counter-client-capability-proof.wat`
- Counter actor declared via spawn-thread; mints `:counter::Client` via restricted constructor; hands to caller (test body)
- Caller uses `:counter::Client/in!` + `:counter::Client/out!` (public accessors) to talk
- `:counter::Client/server-id` + `:counter::Client/client-id` (restricted accessors) — verified server's own code reads them; caller cannot
- Positive test: round-trip Increment + Get succeeds; capability used successfully
- Negative test (compile-time): a hand-rolled defn outside `:counter/` prefix attempting `:counter::Client/new` → `DefRestrictedCallerNotAllowed`

**Predicted runtime:** 30-60 min sonnet.

**Dependencies:** Slice 1 (substrate primitive shipped); arc 091 `uuid::v4` for the server-id + client-id generation.

**Out of scope:** Provision/Deprovision admin protocol; multiple users; HandlePool registry; per-channel select. That's slice 3.

### Slice 3 — ServiceWithProvisioning thread-tier (task #338 proper)

Full ServiceWithProvisioning demo with:
- Two separate channel types (admin `AdminPeer<AdminReq, AdminResp>` + per-user `Sender<UserReq>+Receiver<UserResp>`)
- Server-side dispatch with `:wat::kernel::select` across admin-rx + N user-rxs (dynamic registry)
- Admin sends Provision → server mints `:counter::Client` (using slice 2's pattern) + adds user-rx to select set + returns Client
- Admin sends Deprovision client-id → server drops registry entry; user's recv sees Disconnect
- User self-drop: drops Sender → server's recv on that user-rx sees Disconnect → server cleans up registry entry automatically
- Tests prove the full multi-user lifecycle: spawn → provision N users → all talk concurrently → deprovision some → server keeps going → final Stop returns Final state

**Predicted runtime:** 90-120 min sonnet (larger; richer protocol; multi-user state).

**Dependencies:** Slice 2 (capability pattern proven in single-user case).

### Slice 4 — closure paperwork

INSCRIPTION + 058 changelog row + USER-GUIDE entry + cross-reference to arc 198 as the precursor; pre-INSCRIPTION grep (FM 11) for deferral language.

**Predicted runtime:** 30 min orchestrator-side.

**Dependencies:** Slices 1 + 2 + 3 shipped.

## Connection to prior arcs

- **Arc 198** — direct precursor: same HashMap, same walker, same prefix-matching rules; arc 203 adds one declaration surface
- **Arc 170 INTERSTITIAL § 2026-05-16 (deeper) service-with-provisioning addendum** — the ServiceWithProvisioning pattern the demo proves
- **Arc 170 (open)** — task #338's ServiceWithProvisioning thread-tier proof was queued as the workhorse demo; arc 203 makes the capability struct honest

## Cross-references

- `docs/arc/2026/05/198-defn-restricted/INSCRIPTION.md` — the precursor pattern
- `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` § 2026-05-16 (deeper) — the service-with-provisioning context
- `src/check.rs:3152+` (`walk_for_def_restricted_call`) — the walker arc 203 reuses
- `src/runtime.rs:1879` (`register_struct_methods`) — the synthesis point arc 203 extends
- `src/check.rs:7478+` (`infer_def_restricted`) — the shape-validation pattern arc 203 mirrors

The substrate refuses; the user does the work; we ship the hard part because that's what we do.
