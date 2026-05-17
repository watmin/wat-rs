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

### Worked example — counter::Client (per slice 2 proven shape)

```scheme
(:wat::core::struct-restricted :counter::Client
  [:counter::]                                                          ;; only :counter::* can mint Client/new
  ([:counter::] server-id <- :wat::core::String                         ;; only :counter::* can read server-id (validates issuance)
   [:counter::] client-id <- :wat::core::String)                        ;; only :counter::* can read client-id (registry key)
  (peer! <- :wat::kernel::ThreadPeer<counter::Response,counter::Request>)) ;; user reads to talk/listen (bundles channels)
```

**Naming note (per slice 2 honest delta 2):** Arc 198's `caller_matches_prefix_list` requires whitelist entries to end in `::` for namespace-prefix matching; entries not ending in `::` are exact-FQDN matches. Capability-issuing modules using struct-restricted must use `::` separator in their function names (e.g., `:counter::spawn`, not `:counter/spawn`) so the `[:counter::]` whitelist matches via prefix.

**Field shape note (per slice 2 honest delta 3):** Bundling the channel pair as a single `:wat::kernel::ThreadPeer<I,O>` field (instead of separate `Sender<O>` + `Receiver<I>`) is the cleaner consumer pattern. The wrappers (`:counter::get` etc.) use `Thread/println peer!` + `Thread/readln peer!` directly without per-call ThreadPeer construction. This ALSO sidesteps the known ProcessPeer/ThreadPeer field-type naming defect at the consumer surface (substrate-internal `:rust::crossbeam_channel::*` field-type naming for ThreadPeer/ProcessPeer is a separate concern; arc 204 territory).

**uuid::v4 note (per slice 2 honest delta 1):** Random IDs come from `:wat::telemetry::uuid::v4` (under the `wat-telemetry` dep), not `:wat::measure::uuid::v4`. Return type is `:wat::core::String` (canonical 8-4-4-4-12 hyphenated hex), not `:wat::core::keyword`. Slice 3's BRIEF must declare the telemetry dep.

### Degenerate cases

```scheme
;; All-restricted struct (every read + the constructor restricted to same whitelist)
(:wat::core::struct-restricted :secret::Secret
  [:secret::]
  ([:secret::] secret-1 <- :T1
   [:secret::] secret-2 <- :T2)
  ())

;; Constructor-only restriction (mint protection; all fields readable by holder)
(:wat::core::struct-restricted :auth::Token
  [:auth::]
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

---

## Post-3e expansion — arc 203 becomes "the one pattern" enforcement arc (settled 2026-05-17)

User direction post-slice-3e: *"we do not close out arc 203 until all services we vend follow the one and only pattern for parallelism and concurrency... so this means we go block that 'everyone must follow the one pattern' on error propagation for both being delivered - i think we are eliminating the remainder of the 170 issues now by doing this... all deadlocks are eliminated by following the rules... we show our users how to behave for guaranteed success."*

Arc 203 expands its scope: not just minting the substrate primitive + proving the pattern, but ENFORCING the pattern across every service the substrate vends. The two Counter demos become the canonical user-facing documentation; all existing services align with the pattern; closure ships only when alignment is complete.

### Revised slicing (post-3e)

| Slice | Status | What |
|---|---|---|
| 1 — substrate primitive | SHIPPED `26c9298` | `:wat::core::struct-restricted` form + parser + check + registration |
| 2 — Counter/Client single-user proof | SHIPPED `e8101d8` | First consumer; ThreadPeer composition |
| 3a — server dispatch foundation | SHIPPED `d4d76b4` | N=1, Wire enum, select |
| 3b — dynamic Provision/Deprovision | SHIPPED `15cf7a8` | N=3 multi-user, Vector registry, auto-cleanup |
| 3c — capability struct wrappers (thread) | SHIPPED `e7aa671` | Admin + Client struct-restricted |
| 3d — process variant (stdio multiplexed) | SHIPPED `45a1727` | Wire + WireResp over single stream |
| 3e — server-id validation (secret-witness live) | SHIPPED `cd6f261` | AccessDenied; forge demonstration |
| **3f — error propagation pattern (Result-bearing wrappers)** | OPEN | Counter demos: `Result<T, :counter::ServiceError>`; honest typed errors (no String escape) |
| **3g — apply pattern to wat-lru CacheService** | OPEN | Refactor `crates/wat-lru/wat/lru/CacheService.wat` to use struct-restricted Client capability + Result-bearing wrappers |
| **3h — apply pattern to HologramCacheService** | OPEN | Same pattern, holon-lru consumer |
| **3i — apply pattern to stdio services** | OPEN | `wat/kernel/services/{stdin,stdout,stderr}.wat`: substrate-side orchestrator holds Admin; threads hold per-thread Client (forge-resistance load-bearing — currently a thread could forge Add/Remove for other thread-ids) |
| **3j — closure paperwork** | OPEN | INSCRIPTION + 058 changelog row + USER-GUIDE entry pointing at all canonical artifacts |

### Honest typed errors (slice 3f core decision)

`(SubprocessDied (chain :String))` would be DISHONEST — chains are structured EDN/data, not strings. The honest shape uses substrate-provided typed errors:

```scheme
(:wat::core::enum :counter::ServiceError
  (AccessDenied)                                              ;; server rejected server-id
  (PeerDied    (cause :wat::kernel::ThreadDiedError))         ;; thread-tier peer dropped
  (ServerDied  (cause :wat::kernel::ProcessDiedError))        ;; process-tier subprocess died (carries typed panic-chain)
  (Disconnected))                                             ;; clean recv-returned-None
```

Wrappers return `:wat::core::Result<:T, :counter::ServiceError>`. Callers pattern-match on Ok/Err; Err variants carry typed cause data (not stringified).

`:wat::kernel::ThreadDiedError` (arc 060) and `:wat::kernel::ProcessDiedError` (src/types.rs:632) are substrate-provided. ProcessDiedError's `Panic` variant carries the structured chain via the existing accessors `/message` + `/to-failure` (src/runtime.rs:4687, 18545).

### Why this is "the one pattern"

The canonical wat service implements (in order):
1. **Privacy** — struct-restricted capability hides server-id, client-id, and channel ends; users hold opaque values
2. **Capability mint protection** — only the issuing namespace can construct Admin / Client
3. **Behavioral protocol routing** — Wire enum with Admin/User variants; server matches on receipt
4. **Secret-witness validation** — server validates incoming wire payload's server-id; AccessDenied on mismatch
5. **Honest error propagation** — Result-bearing wrappers with typed-data errors (no String escapes)
6. **Lifecycle discipline** — Provision/Deprovision via admin channel; auto-cleanup on user Disconnect; Stop with drain-and-join

Services that follow this pattern: cannot be impersonated; cannot have id-forgery succeed; cannot have callers stuck panicking on transient errors; cannot deadlock under the rules-enforced-at-substrate (per arc 117/126/202 walkers + the pattern's structural discipline).

### Connection to arc 170 closure

User: *"i think we are eliminating the remainder of the 170 issues now by doing this... all deadlocks are eliminated by following the rules."*

Arc 170's substrate work (typed channels, ProcessPeer, drain-and-join, structural walkers, deadlock detection) provided the SUBSTRATE for "the one pattern." Arc 203 ships the pattern itself. Together they form the foundation: arc 170 makes the rules possible; arc 203 makes the rules concrete + applies them to every vended service.

Future services follow the canonical pattern by copying from the Counter demos and adapting the per-domain bits. Future substrate work that introduces new transport tiers (remote per `:wat::kernel::run-remotes`) extends the pattern uniformly.


---

## Post-3f pivot — arc 203 blocked on new protocols arc (settled 2026-05-17)

User direction post-3f-spawn: *"203 is blocked on the protocol arc proving they work and then we unbind back to 170."*

Realization mid-session: what arc 203 hand-rolled IS Clojure's protocols pattern (independent convergence — Wire enum = protocol's operation list; dispatch loop = the implementations; `:counter::*` wrappers = the protocol's call surface; struct-restricted Admin+Client = typed views into the protocol). The natural next step is a substrate meta-form that abstracts the repetition.

### Revised dependency chain

| Slice / Arc | Status | What |
|---|---|---|
| 3a-3e | SHIPPED | Hand-rolled pattern proven (substrate primitive + Counter demos at both tiers + capability + secret-witness) |
| **3f** | IN FLIGHT (sonnet `aeb4fe6...`) | Error propagation (Result-bearing wrappers, typed ServiceError) |
| **NEW arc (TBD number)** | BLOCKS arc 203 closure | `defservice` substrate primitive — meta-form that auto-synthesizes Wire enum + capability structs + dispatch loop + wrappers from a user-supplied protocol declaration. Substrate validates all required handlers present at freeze time; PANIC if missing. Per Clojure protocols convergence |
| 3g — wat-lru CacheService refactor | BLOCKED on new arc | Becomes a USE of `defservice` (not hand-rolled refactor) |
| 3h — HologramCacheService refactor | BLOCKED on new arc | Same |
| 3i — stdio services refactor | BLOCKED on new arc | Same |
| 3j — closure | BLOCKED on 3g/3h/3i | INSCRIPTION + 058 + USER-GUIDE |
| Arc 170 closure | BLOCKED on arc 203 closure | The bracket-combinator family's actual user (arc 203) closes; arc 170 then closes |

### The `defservice` shape (sketch — refined when the new arc opens)

```scheme
(:wat::service::defservice :counter
  :admin    {Provision   [initial :i64]                     -> :counter::Client
             Deprovision [client  :counter::Client]         -> :wat::core::nil
             Stop        []                                  -> :wat::core::nil}
  :user     {Get         []                                  -> :i64
             Increment   [n :i64]                            -> :i64
             Reset       []                                  -> :i64}
  :state    :i64
  :handlers {<keyword-map of operation-name → handler-fn>})
```

Substrate auto-synthesizes:
- `:counter::Wire` + `:counter::WireResp` enums (Admin/User tagged)
- `:counter::ServiceError` enum (standard variants: AccessDenied, PeerDied, ServerDied, Disconnected)
- `:counter::Admin` + `:counter::Client` capability structs (struct-restricted)
- Server dispatch loop (select + route + validate server-id + handler dispatch)
- Client-side wrappers (Result-bearing)
- Per-tier transport adapter (thread = crossbeam; process = stdio multiplex)

Substrate validates at freeze time: every operation in the protocol has a registered handler; signatures match. Missing handler → PANIC with diagnostic.

### Why this is the right architecture

Arc 203's hand-rolled pattern proved the SHAPE works. Repeating it per service (cache, holon-cache, stdio) by hand is N× the boilerplate with N× the surface for inconsistency. The meta-form abstracts what's repeated:
- User writes operations + handlers
- Substrate writes everything else
- New services follow the pattern by construction

Per `feedback_simple_is_uniform_composition`: N identical compositions IS simple; abstracting them into one form is the simplest possible composition.

Per the Clojure-protocols convergence pattern from INTERSTITIAL § 2026-05-16 (Erlang/OTP arrival): when independent design walks into a place a great has been, that's the validation signal that the engineering is on a known-good path.

### Arc 170 unblock

Once arc 203 closes (after the new arc + 3g/3h/3i ship), arc 170's bracket combinator chain (D3 + Stones E/F/G/H) can close because arc 203 demonstrated the actual user pattern that justified the bracket primitives in the first place. The full closure chain: new arc (protocols) → arc 203 (apply protocols to all vended services + close) → arc 170 (close on demonstrated user pattern + bracket primitives complete).

