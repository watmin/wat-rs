# BRIEF — thread type params through the protocol synthesis (`Op<K,V>` / `Reply<K,V>`)

> **The last blocker for cache Stone 2** (`:wat::cache::lru-svc<K,V>`). Builder-ruled option **(a)**, the
> parametric protocol, by four-questions (4×YES; concrete-messages failed Obvious, Simple AND Honest —
> it understates a guarantee it actually keeps and forces an unchecked narrow at the server).
>
> **This is a design extension, not a bug fix.** A previous rider correctly STOPPED here and said so.

## Where the three prior fixes got us

```clojure
7336464e   defservice takes <T>            ;; ::Record<T> ::State<T> ::Handle<T> ✓ and RUNS
10107da9   depth-aware type-arg split      ;; <K,V> + nested commas              ✓
9a5e6519   :messages compares BASE names   ;; a parametric message DECLARES      ✓
```

The surface declares fine. The **wire** still strips the parameter.

## The defect — GROUND TRUTH, re-verified this session (earlier citations were STALE)

**`src/types.rs:2510-2522`** — `synthesize_surface_protocol` (the fn starts at **`:2215`**) returns:

```rust
Ok(vec![
    TypeDef::Enum(EnumDef {
        name: op_name,
        type_params: vec![],        // ← Op is born NON-parametric …
        purity: Purity::Pure,
        variants: op_variants,      // ← … while its variant fields may reference K
    }),
    TypeDef::Enum(EnumDef { name: reply_name, type_params: vec![], … }),
])
```

So `Op`'s `Get` field is `GetRequest<K>` with **`K` unbound in the enum**.

**`wat/service.wat`** — message type NAMES are derived by **string concatenation** from
surface-base + op-pascal (`kebab->pascal-in <surface-kw> <op>`), a convention with **no channel for the
message's own type arguments**. Grep `kebab->pascal-in` and the `req-ty` / `resp-ty-str` bindings; the old
line numbers (852/1247/1250) moved when 109 files were swept today — **find them yourself, do not trust
a number in this brief.**

**Proven by `macroexpand`, not inferred** (do this first to see it):

```clojure
(defenum :cx-svc::Op :wat::enum::Pure :Get [req <- :Cx::GetRequest])   ;; <K> gone
(defn :cx-svc/get [c <- Peer'<Cx::Op,Cx::Reply>  req <- :Cx::GetRequest] …)
```

while its siblings `Record<K>` `State<K>` `Admin<K>` `Status<K>` `serve<K>` all carry `<K>` correctly.

**⚠ It is NOT a genericity problem.** It reproduces with a **monomorphic** service and the message
referenced at a **concrete** instantiation — `req <- :Cx::GetRequest<wat::core::String>` fails the same
way. The name derivation itself has no representation for a message that carries type arguments, ever.

## The work

1. **`synthesize_surface_protocol`** — `Op`/`Reply` inherit the surface's `type_params`; the variant field
   `TypeExpr`s keep referencing them, now bound.
2. **`wat/service.wat`** — the message-name derivation must carry type arguments instead of concatenating a
   bare name. Reuse the split helper the earlier strikes established (base + params, suffix on the base,
   params re-attached) rather than minting a fourth spelling of it.
3. **Thread the params** through the generated client fn, the serve-loop arms, and `Peer'<Op,Reply>`.

## ⚠ SAFETY PROPERTY — the whole floor rides on it

A surface with **no** type params must produce **byte-identical** output. Verify it, don't argue it. The
established bar in this series: run the HEAD-built binary and the patched one with
`--check --check-output edn` over the whole `.wat` corpus, diff, and dispose of every difference — some
pre-existing nondeterminism exists (hash-iteration order in `DuplicateDefine` `:name`), so prove that by
self-diffing HEAD against itself first.

## The gate

Land a `deftest` sibling of `wat-tests/service-parametric-two-params.wat`: a surface with **parametric
messages**, a `<K,V>` service satisfying it, stood up on the thread locus, `connect'`ed via `Handle/addr`,
**K and V pinned to DIFFERENT concrete types** (`K=String`, `V=i64`), and a real round-trip carrying typed
payloads — send actual `String`s, get actual `i64`s back, assert on the values.

Existing gates stay green: `wat-tests/service-parametric.wat`, `-two-params.wat`, and all nine concrete
defservices unchanged.

## STOP triggers

1. **If the EDN wire cannot carry the parametric payload** — the codec, the child-lineage `forms`, or the
   decode failing to enforce `K` — **STOP and report.** Do NOT weaken the gate to concrete messages to
   reach green. (Note: request sanitization now validates every inbound payload against its declared type,
   so a `K`-typed field IS checked at the boundary — `edn_to_typed_value` via `:wat::edn::validate`.)
2. If the blast radius exceeds `src/types.rs` + `wat/service.wat` + the new gate — STOP and report.
3. Process tier for parametric services is known-unverified and OUT of scope — do not chase it.

## Method

- `target/release/wat --check <f.wat>` is the fast per-file arbiter; **read its output, not `$?` through a
  pipe.**
- **`macroexpand` first** — when a macro's output is confusing, read what it EMITTED before theorising. It
  is how this defect was found.
- Scratch `.wat` → `wat-scripts/scratch-pad/` (loader-gated; must be GREEN).

## Gate

- The parametric-message round-trip `deftest` green.
- `cargo build --release` clean; `cargo nextest run --release` — **Summary line VERBATIM**. Floor:
  **4178 passed, 314 skipped**.
- FOREGROUND only. **Do NOT commit** — the orchestrator weighs by their own re-run and commits.

## Your report

The diff shape; how you verified the non-parametric path is byte-identical; the round-trip evidence quoted;
whether the wire genuinely carries and enforces `K` (evidence, not inference); the verbatim Summary line;
any STOP.
