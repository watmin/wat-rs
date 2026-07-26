# DESIGN — `RequestMalformed`: input sanitization at the service boundary

> **Builder-ruled 2026-07-25**, on seeing the DoS proven: *"we must have a request-malformed … the whitelist
> of what we accept is already explicit — a bad caller (malicious or dumb) cannot crash anything."*
>
> Framing is the builder's and it is the correct one: this is **input sanitization**, not "type enforcement."
> The oldest rule there is — *accept only the shapes you allow, before the input reaches any logic.*

## The vulnerability (PROVEN, orchestrator's own hands, both tiers)

A client sends well-formed EDN with a wrong-typed body under a correct tag:

```clojure
#dos.Bag/PutRequest {:items [1 2 3]}        ;; declared: items <- Vector<String>
```

The wire accepts it verbatim and hands the handler the mistyped value. A handler that merely *uses* the field
at its declared type — `(string::length (nth items 0))`, legal and correct against the declaration —
**detonates, and the service dies for everyone**:

```
"attacker good  => Ok"
"attacker BAD   => LOST (peer gone)"
victim: connect REFUSED — service is GONE          ← a second, innocent client cannot even connect
```

One frame, from any client, kills the service. No bug in the handler. **This is a denial of service.**

## Why it is open

- **Thread tier — no decode at all.** `ReactorClass::InMemory` (`src/runtime.rs:27585-27591`) passes the
  `Value` through crossbeam verbatim.
- **Process tier — decode is tag-driven, not target-driven.** `reconstruct_record`
  (`src/edn_shim.rs:2751-2765`) uses the declared fields for **names and order only**; the declared `fty`
  goes to `rewrap_option_field` alone and is **never compared to the decoded value**.
- The only inbound inspection is the `:max-request-bytes` **size** guard.

The decode is literally named `decode_trusted_wire`. That was honest when both ends were ours; a
`defservice` any client can `connect'` to is a different posture, and by the substrate's own end goal
(processes are networked file handles; fully distributed later) it becomes **false by construction**.

## The whitelist already exists; the validator already exists

- **Whitelist:** the op's declared request record — `items <- :wat::core::Vector<wat::core::String>` — is the
  accepted shape, per op, already authored. **Nothing new to declare.**
- **Validator:** `edn_to_typed_value` (`src/edn_shim.rs:1741`) walks a declared `TypeExpr`, recurses per
  element (`Vector<T>` arm, `:1892-1905`), rejects an `Integer` against `String`, and yields an
  `EdnCoerceError` carrying the offending path (`.items.[0]`). **Zero production callers** — its last was
  deleted by arc 258 Stone 258.5b on the trusted-wire premise.

## The shape — mirror the size guard EXACTLY

`wat/service.wat`'s `guarded-arm` (~`:1060`) is the precedent, and its placement solves the hard part:

```clojure
(:wat::core::let [n (:wat::core::string::length (:wat::edn::write req))]
  (:wat::core::if (:wat::core::i64::> n cap)
    ;; violation → send the NAMED variant, then RECURSE INTO SERVE (keep serving,
    ;; state unchanged); a gone client is not fatal either — every arm keeps serving.
    (:wat::core::match (:wat::kernel::send' … (Reply (RequestTooLarge n cap)))
      (Sent   (serve self l selectables state))
      (Closed (serve self l selectables state))
      ((Lost _c) (serve self l selectables state)))
    <the handler>))
```

**It is POST-DECODE, inside the generated dispatch arm, before the handler.** So a shape guard in the same
slot covers **BOTH TIERS for free** — which a Rust-side decode fix would not, since the thread tier never
decodes.

The shape guard is its sibling:

```clojure
;; size — enforced today
:RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
;; shape — the missing half
:RequestMalformed [path     <- :wat::core::Vector<wat::core::String>   ;; ["items" "[0]"] — segments
                   expected <- …
                   got      <- …]
```

and the caller is **forced** to face it — enum matches are exhaustive, no wildcard arm (arc 109
`NOTE-full-enum-match-mandatory-no-wildcard-arm`). Verbosity is the shield.

## What must be built

1. **A wat-callable shape validator.** The guard is generated *wat*, so it needs a wat verb; `edn_to_typed_value`
   is Rust with no wat surface. **Ground whether one exists before minting one** — check `:wat::edn::*` and
   whether arc 299's constraint system (`conforms`) already answers "does this value match this shape."
2. **`:RequestMalformed` as a mandatory sibling of `:RequestTooLarge`** on every serviceable op-response enum
   (ruling A's shape, checker-forced).
3. **The generated guard** in `guarded-arm`, before the handler, keeping the service alive on violation.

## Decomposition — prove the mechanism on ONE service before paying the cascade

- **Stone 1 (this brief's companion): the mechanism, end-to-end, on ONE service.** Validator + guard +
  variant, proven against the exact DoS probe: attacker sends the malformed frame, gets a
  `RequestMalformed` reply, **and the victim's later `connect'` still succeeds.** No corpus cascade yet.
- **Stone 2: the corpus rollout** — every response enum gains the variant; every caller's match gains the
  arm. Same shape as ruling A's rollout, and the same cascade cost. A map-reduce crusade fits here.

## Open — four-question at strike, do not assume

- **`expected`/`got` as `String` or as structured type forms?** A `TypeExpr` is EDN-expressible, so the
  builder's own prose-vs-structured rule says structure it — but that collides with the `format_type`
  question already noted for 296. Decide once, here, and use it in both places.
- **Per-op variant vs protocol-tier rejection.** `RequestTooLarge` is per-op (ruling A). `ServiceEvent::
  {Malformed,Rejected}` already exist — ground whether they reach the *client* as a matchable value or are
  owner-side only. If a protocol-tier path already reaches the client, the corpus cascade may be avoidable.
- **The incidental find, same class:** `RecvOutcome::Lost` declares `cause <- LociDiedError`
  (`src/types.rs:1231-1241`) but the generated forwarder builds it with a `Failure`
  (`src/runtime.rs:5443-5452`). It ships because the forwarder is AST-synthesized and bypasses the checker,
  and nothing at runtime validates a variant payload's type — an instance of this very hole inside the
  substrate's own code.

## RULED at Stone 1 — `expected`/`got` are STRINGS; `path` is STRUCTURED

Four questions, flat YES/NO. **Strings: YES / YES / YES / YES. Structured type forms: NO / NO / NO / NO.**

- **Obvious?** `expected ":wat::core::String"` / `got "Integer"` reads at a glance and is the same
  rendering every type diagnostic in the substrate already prints. A `TypeExpr` ADT the client must walk
  to recover a one-line fact is not obvious. **String YES, structured NO.**
- **Simple?** `check::format_type` is the ONE authoritative type renderer and already exists; zero new
  types. Structured needs a wat-side `TypeForm` ADT mirroring `TypeExpr` (Path/Parametric/Tuple/Fn/Var)
  plus a Rust→wat lowering — and `got` has nothing to lower FROM. **String YES, structured NO.**
- **Honest?** This is the decisive one. **`got` is not a type and cannot be made one.** The value that
  arrived came off an untyped wire; it has no declaration. The honest datum is its EDN SHAPE
  (`edn_shape_name` — "Integer", "Vector", "Map"). Structuring it as a type form would FABRICATE
  information. And an asymmetric pair (structured `expected`, string `got`) implies a comparison the
  substrate cannot make. **String YES, structured NO.**
- **Good UX?** The consumer is a 400-class refusal arm: log it, or reply it verbatim. Nobody computes on
  a rejected request's type. **String YES, structured NO.**

`path` goes the OTHER way and stays `Vector<String>` — segments (`["items" "[0]"]`) a caller can index
and walk. That is real data.

**The rule this settles, for arc 296's `format_type` question to inherit:** the prose-vs-structured rule
binds DATA THE PROGRAM COMPUTES ON. A rendering of a type for a human or a log is not that — `format_type`
is the one renderer and its output is a String field. Structure the coordinate; render the type.

## The acceptance bar

The DoS probe, inverted: the attacker's malformed frame returns a **named `RequestMalformed`**, and a
subsequent innocent client **connects and is served**. Both tiers. That is the whole point — a bad caller,
malicious or dumb, cannot crash anything.
