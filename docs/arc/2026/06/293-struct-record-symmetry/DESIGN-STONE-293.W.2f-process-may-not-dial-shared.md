# DESIGN — STONE 293.W.2f: a process may not dial a shared-memory address

## Why this stone

2e minted `address-wire?`. Live MCP then rebuilt the illegal circuit:

```wat
(:usr::echo/start :locus (:wat::spawn::thread) :record (:usr::echo::Record))
(:wat::kernel::address-wire? (:usr::echo::Handle/addr :usr::eh))  ;; false
(:wat::bracket::map (:wat::spawn::process) ["a"] :usr::work :echo :usr::eh)
```

The mouth said **false**. The checker said **legal**. The child died:

`unsupported substrate tag #wat-edn.opaque/RustOpaque` at `dial-runner` (`wat/bracket.wat:478`).

A process may never dial a thread. That is not a missing feature. It is an illegal
program that must fail **at check time**. 2e named the fact. This stone makes the
type stop lying.

## The algorithm (pinned)

One question, already answered at runtime by `portable_form` / `address-wire?`.
The type of an Address must carry that answer so `connect` / Setup / the map
expansion can unify against it.

1. **Marker types** `:wat::kernel::Shared` and `:wat::kernel::Wire` — phantoms,
   used only as the third type argument. Not values. Not a third hinge.
2. **`Address<S,R,T>`**. Runtime entity unchanged (`Address { inner }`).
   - 2-arg `Address<S,R>` still means **T unknown** (old fixtures, abstract Locus).
     Unifies with both Shared and Wire (T unbound).
   - Thread `listener` → `Address<S,R,Shared>`.
   - Process `listener` → `Address<S,R,Wire>`.
   - Abstract `Locus` `listener` → `Address<S,R,T>` with T fresh.
3. **`Bound` / `Launched` address fields** carry T when the locus is concrete.
4. **`Handle<T>`** with `addr <- Address<Op,Reply,T>`.
5. **`/start` must not erase T.** Today it is one kwargs defn
   `[& [locus <- Locus …]] -> Handle`. That is the erasure.
   It becomes a **defclause** on `ThreadOpts` → `Handle<Shared>` and
   `ProcessOpts` → `Handle<Wire>`. Same body (still `Locus/launch`).
   A residual `[locus <- Locus] → Handle` clause (T unknown) may exist so
   abstract-locus callers still check; it does **not** cover the acceptance
   test (that test passes a literal `(process)` / `(thread)`).
6. **`connect` still accepts any T** (a thread may dial a process; a thread
   may dial a thread). The illegal act is not `connect` in a thread world.
7. **The raise is at the process runner's door.** `bracket::map` / `each`,
   when the locus AST is a ProcessOpts constructor (`process`,
   `process/runner-count`, …), emits an `ann-form` (or a tiny
   `require-wire-address` intrinsic that unifies against `Address<?,?,Wire>`)
   on each kwargs handle's address / `TypedCapability/coord` result.
   `Handle<Shared>.addr` is `Address<_,_,Shared>` ↛ `Address<_,_,Wire>`.
   **TypeMismatch. The program is illegal.**

`is_pure_type`: `Address<_,_,Shared>` is impure (in-locus resource).
`Address<_,_,Wire>` stays pure (it already crosses as `SocketAddressWire`).
2-arg `Address<S,R>` stays pure (unknown T — do not break Coords records).

## Acceptance test (this exact program)

The FM 2-bis probe is the MCP circuit, minimized. **RED at HEAD:** it
type-checks (startup Ok). **GREEN:** startup is a CheckError naming the
shared-memory address / Wire mismatch. A thread `map` of the same handle
stays GREEN (legal).

## Error contract

The check error is a `TypeMismatch` (or `MalformedForm` if you mint
`require-wire-address`) that names **Wire** vs **Shared** (or `Address` +
not-a-wire). It does **not** say `RustOpaque`. It does **not** wait for
the child. Prefer the existing `TypeMismatch` shape over a new error kind
unless a new kind is forced.

## Out of scope — REJECTED

- Changing `connect` so a thread cannot dial a process.
- Making `peer-wire?` accept an Address (2e already minted `address-wire?`).
- 255 registry hoist.
- Closing abstract-`Locus` start (`[locus <- Locus]`) — residual, documented.
- Remote locus (still uncut).

## Rooms

- `src/check.rs` `infer_listener_prime` ~9353, `bound_type` ~9477,
  `infer_connect_prime` ~9482, `infer_address_wire` ~10858, `is_pure_type` ~12737.
- `src/runtime.rs` `eval_listener_prime` / `eval_connect_prime` / `eval_address_wire`
  (runtime entity unchanged; check types only unless a new intrinsic is minted).
- `wat/spawn.wat` `Bound` ~278, `Launched` ~291.
- `wat/service.wat` `addr-ty` ~760, `handle-fields` ~2301, `start-fn` ~2209/2237.
- `wat/core.wat` Coords Peer→Address swap ~919-940 (2-arg Address stays valid).
- `wat/bracket.wat` `defmacro map` ~838 (the ProcessOpts-AST raise).
- 2e probe `tests/comms/probe_arc293_W2e_address_wire.*` must stay GREEN.

## The farm — wat-fix, not a hand rewrite

Adding T to `Address` / `Bound` / `Handle` is a **structural corpus rewrite**.
The weapon is **wat-fix** (`wat/fix.wat` + `wat-scripts/fixes/*.wat`). Not
hand-edits. Not python. Not sed.

- Framework: `wat/fix.wat` — `read-string` → `with-children` / comment-faithful
  text edits (`rename-keyword-prefix` is the closest primitive; this strike
  needs a **type-arg append** on parametric keywords, recorded as a new
  `:wat::fix::…` verb or a one-shot walk in the fix script).
- Recorded migration: `wat-scripts/fixes/address-transport-arity.wat`
  (new). Header: *"Self-hosted fix-wat codemod: no hand-editing of .wat files."*
- Drive: `printf '["pathA" …]\n' | cargo wat ./wat-scripts/fixes/address-transport-arity.wat`
  List EVERY path. Idempotent (re-run = 0 changes). Dry-run on a copy first, `diff`.
- **STASH-DANCE** (`wat/fix.wat` header): this ships WITH a checker change that
  makes the old 2-arg form illegal. Do not hand-edit instead.

  1. Land the NEW fix verb in `wat/fix.wat` (old checker still accepts 2-arg Address).
  2. `git stash push` the `src/check.rs` / `src/runtime.rs` (and any other rust)
     that rejects 2-arg.
  3. `cargo build --release` — old checker + new verb.
  4. Run the fix over the whole corpus (`wat/`, `wat-scripts/`, `tests/`, `wat-tests/`).
  5. `git stash pop`.
  6. `cargo build --release` — new checker; corpus is 3-arg.

Default rewrite of a 2-arg `Address<S,R>` / `Bound<S,R>` is
`…,wat::kernel::Wire` (the old "it crosses" meaning, now explicit). Thread
`listener` sites that must be Shared are the checker (infer_listener) plus
the `Bound`/`Handle` declarations that are *parametric in T*, not a second
guessy pass.

## Calibration

This is a cascade + a recorded wat-fix. Band 90–180 min. STOP at 240 min
if kwargs+defclause cannot express the start split — that is a substrate
extension, not a guess. STOP if you start hand-editing more than the
declaration sites + the fix script.
