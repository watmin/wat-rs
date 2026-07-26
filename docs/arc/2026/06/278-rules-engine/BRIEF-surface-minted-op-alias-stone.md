# BRIEF — the surface-minted op alias: kill the vacuous `<T>` on message declarations

> **The scout answered YES** (`BRIEF-surface-minted-op-alias-scout.md`, verified by the
> orchestrator's own runs with a disconfirming control in every arm, including a real
> `:nature :Peer'` corpus surface). This is the stone that spends the answer.
>
> **Feasibility is settled — do NOT re-prove it.** Build the feature.

## What ships (the user-visible change, and it is the whole point)

```clojure
;; TODAY (1ac85d96) — the surface's params must be spelled on every message, even vacuously
(defsurface :Box<T> :nature :Peer'
  :messages [(defrecord :Box::PutRequest<T> [item <- :i64])]      ;; <T> unused. forced.
  :features [(put [self <- :Box<T>  req <- :Box::PutRequest<T>] -> :Box::PutResponse<T>)])

;; AFTER — spell the params you actually use
(defsurface :Box<T> :nature :Peer'
  :messages [(defrecord :Box::PutRequest [item <- :i64])]         ;; bare. honest.
  :features [(put [self <- :Box<T>  req <- :Box::PutRequest] -> :Box::PutResponse)])
```

A message that genuinely uses a param keeps it (`GetRequest<K,V>` with `probes <- Vector<K>`).
The rule becomes *spell what you use* — which is what a reader assumes.

## Why it works (settled by the scout; stated so you do not re-derive it)

The macro's only knowledge of the surface is the `:satisfies` keyword (`wat/service.wat:244`
`proto-str`), so it **rebuilds** each message name by concatenation and cannot know the message's
real arity — hence the forced spelling. Rust at surface registration holds the surface's
`:features`, which **declare** each request/response type outright. So Rust mints one alias per op
with a uniform name; the macro names the alias and stops guessing.

The scout proved a `TypeDef::Alias` minted during `register_types` resolves for a later
declaration in the SAME pass, monomorphic and parametric, on a real `:Peer'` surface —
`expand_alias` (`src/types.rs:4269`): `:4273-4276` (`type_params.is_empty()`) and `:4279-4292`
(the `Parametric` arm, `len == len`, substitution executed).

## Read in order

1. **`src/types.rs:~2752`** — the surface-registration site. The scout's ~19-line mint is **in the
   working tree right now** and is the correct shape for `/Request`. Extend it: mint **`/Response`
   too**, from each method's declared return type. Name pattern `<Surface>::<op>/Request` /
   `<Surface>::<op>/Response`; `type_params` = the surface's; target = the `TypeExpr` exactly as
   `:features` declares it.
2. **`wat/service.wat`** — the derivation sites that must name the alias instead of concatenating.
   Grounded live: **`:894`** (`"{b}::{v}Request{p}"`), **`:1157`** (`"::{variant-pascal}Request"`),
   **`:1375`**/**`:1377`** (`"{b}::{op-pascal}Request{p}"` / `Response`). Re-ground before editing —
   line numbers move.
3. **`src/types.rs:~2295-2310`** — the message-params lock `1ac85d96` added (the located
   `MalformedDecl` that FORCES a message to spell the surface's params). It must **go**: the whole
   point of this stone is that the spelling is no longer required. Delete the lock and its error
   text; keep every other `MalformedDecl` in that file untouched.
4. **`wat-tests/service-parametric.wat`** + **`-two-params.wat`** — revert their messages to the
   bare spelling `1ac85d96` forced them off of. `git show 1ac85d96^:wat-tests/service-parametric.wat`
   shows the honest form.
5. **`wat-tests/service-parametric-messages.wat`** — leave the declarations alone. `K` and `V` are
   genuinely used there; it must stay green unchanged, proving the params-you-use case still works.

## The RED gate (write it first; it must fail before the stone and pass after)

A `deftest` sibling: a parametric `:nature :Peer'` surface whose messages are declared **bare**,
a `<T>` service satisfying it, stood up and round-tripped on **both loci**. At HEAD that is a
located error from the lock; green when the stone lands.

## STOP triggers — rejection criteria; report and ship nothing

1. **If naming the alias from the macro fails at any of the four derivation sites** — STOP and
   report which site and the diagnostic. Do not fall back to concatenation at that one site; a
   half-converted derivation is the two-sources-one-truth defect this stone exists to remove.
2. **If deleting the message-params lock makes any existing gate go red** — STOP and report the
   gate and the error. That would mean something else depends on the forced spelling.
3. **If the blast radius exceeds `src/types.rs` + `wat/service.wat` + the listed test files** —
   STOP and report before spending it.

## Method

- `target/release/wat --check <f.wat>` is the fast per-file arbiter (~0.2s); read its printed
  output, never `$?` through a pipe.
- **`macroexpand` first** when a macro's output confuses you — read what was EMITTED before
  theorising. A generated name is built by string concatenation, which is exactly where generics
  get silently mangled.
- Run every command in the **FOREGROUND** to completion. Never launch a build or test in the
  background and return — a returned "waiting for the monitor" is not a report.

## Gate

- The new RED gate green, **both loci**.
- `wat-tests/service-parametric-messages.wat` green **unchanged**.
- `:894`/`:1157`/`:1375`/`:1377` no longer concatenate a message name.
- `cargo build --release` clean; `cargo nextest run --release` — the **Summary line, verbatim**.
  Floor: **4180 passed, 314 skipped** (expect +1 or +2 for the new gate).
- **Do NOT commit.** The orchestrator weighs by their own re-run and commits.

## Your report

The diff shape; which of the four sites now name the alias; the gate's before/after quoted; the
verbatim Summary line; any STOP.
