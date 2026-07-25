# DESIGN — errors are first-class EDN, all the way down (no structured value trapped in a String)

> Builder-ruled 2026-07-25. Surfaced by the cache-probe run: a startup failure rendered as
> `#wat.kernel.LociDiedError/StartupError ["#wat.runtime/UnknownFunction {…}"]` — a **structured error
> `edn::write`'d into a `String`** and re-serialized (double-encoded escaped EDN). *"we are meant to be edn
> all the way down — masking it in a string is unacceptable."* This is the no-hidden-failures LAW (R41/R55/R57)
> reaching the error-VALUE layer.

## The governing shape — what a correct error IS (the B principle, four-questions-ruled)

An error is the recursive floor **`{message, location, causes}`** (the `:wat::core::Error` surface, wat/core.wat:1782;
`:causes <- Vector<:wat::core::Error>`) **plus** its own variant-specific coordinate fields.

- **Leaf error** (one failure) → **always carries its point.** `:location` is NEVER nil — every failure is about
  code at a span; the genuinely Rust-raised, form-less cases fall back to `rust_caller_span!()` (the Rust file:line
  is still a coordinate). `:causes []`. Its coordinate detail rides in typed variant fields (`:got`/`:expected`/…),
  never re-rendered into `:message`.
- **Collection / wrapper error** (N failures) → **its sub-failures live in the floor `:causes`, each a located
  `Error`** — NOT a bespoke variant field. Its own `:location` = the covering form/file the pass ran on. The tree
  IS the failure structure.
- **`:message`** is a **one-line headline** (a count / a concise phrase); the **structure** (`:location` + the
  `:causes` tree) carries the navigable coordinates. A count standing in for the detail is the mask.

**Why (the four-questions, on the live `ResolveError::UnresolvedReferences`):** today it emits
`{:message "3 unresolved references" :location nil :causes [] :unresolved [<ref>…]}` — sub-failures shoved into a
bespoke `:unresolved` field while `:causes` sits empty and `:location` sits nil. It **fails all four**: not Obvious
(two places for sub-failures; the floor lies "empty"), not Simple (two mechanisms), not Honest (locations + sub-errors
exist but the floor denies them), not Good-UX (*the builder was confused by terse error blobs for months and couldn't
challenge them — that is the verdict*). The B shape (sub-failures in `:causes`, covering `:location`) wins all four.

**"No single primary span" is a rationalization, not a truth** — the child spans exist; the pass always runs on a
file. Nil is a coordinate we declined to carry, not a genuine absence. `:wat::core::Error.location` stays **non-Option**.

**How to challenge any terse blob (the durable test):** are the located sub-failures in `:causes` as real `Error`s,
or hidden behind a nilled floor / a bespoke field?

## The EDN-expressibility rule (prose vs structured — builder-ruled)

- **Is it EDN-expressible?** Can structured data carry it (coordinates: file:line, refs, types, spans)? → **EDN.**
- If not — genuine advisory prose ("do not use this because foo-reasons") that structured data can't capture → **String** (R53-legit).
- **Errors carry BOTH by design** (R3, the diagnostics are the corpus): EDN gives the machine coordinates; a prose
  `:message`/remedy gives the agent context — including intentional prompt-injection guidance. The prose `:message` is
  **not** a mask; the mask is only a *structured value flattened into text*.

## The registration miss (the enabling fix)

The 10 error enums (`RuntimeErrorKind` 32 · `CheckErrorKind` 29 · `TypeErrorKind` 18 · `MacroErrorKind` 12 ·
`ParseErrorKind` 10 · `ConfigErrorKind` 8 · `LoadErrorKind` 7 · `ReteCheckErrorKind` 4 · `ResolveError` 2 ·
`StdlibErrorKind` 1 = **123 variant-tags** across 10 namespaces, `src/error_ns.rs`) each `#[derive(ToEdn)]` a
`#wat.<ns>/<Variant> {…}` tag, but **none are registered as wat types** → STRICT decode (`edn_to_value`) hits
`UnknownTag` → string-wrap. The miss: these tags must be **first-class registered records** satisfying `:wat::core::Error`.

**Register via the derive, not by hand** (derive-is-the-wall, R26 — structure IS the schema; 123 hand `defrecord`s is
the rot to avoid). The work-unit is **~10 enums**, not 123 records: enhance `#[derive(Edn)]` to (a) compose the
`:wat::core::Error` floor keys (`message`/`location`/`causes`), (b) map the error field types it rejects today
(STOP-2: `Box<ValueSnapshot>`/`Span`/`&'static str`/`Vec`/`Option`/nested errors), (c) handle recursive `causes`; then
flip the 10 enums `ToEdn → Edn` and all 123 variants auto-register — and every future variant self-registers.

## The three-move campaign

1. **B — shape correction** (targeted; most of the 123 leaf errors are already B-shaped — only the ~handful of
   collection/wrapper errors + the ~few `location() → Nil` returns need the hand-fix: sub-failures → `:causes`,
   covering `:location`).
2. **Registration via the enhanced `Edn` derive** (bulk — the 10 enums → 123 `Error`-satisfying records).
3. **Collapse the string-wraps** (the audit's MASK list — StartupError · RuntimeError · ServiceEvent::Lost ·
   test-harness): emit `error_edn()` structured into an `:wat::core::Error` carrier (R57 pattern); **widen
   `loci_died_error_from_reason`'s guard** (runtime.rs:23283) — it only accepts `wat.kernel.LociDiedError`-tagged
   reasons today, so a `#wat.runtime/…` cause still falls to the opaque `Panic{message}` even after registration;
   change to "any tagged value that STRICT-decodes to an `Error`-satisfying record."
   - **Left as legit prose (NOT masks):** `MainSignature`/`BadReturn`/`SigmaFn` (genuine `FlatMessage` prose);
     the send'/close'/accept'/connect' outcome walls' "THAT-not-WHY" transport reasons; the 28 `Display`/`Debug`
     impls + wire emitters. (`RecvOutcome::DecodeError`/`MalformedForm`/`EvalError`-projection: verify the read-error
     type isn't a structured `WatError`; else leave as prose.)

## THE ACCEPTANCE GATE (builder's north star) — the cache-probe error rendered correctly

The RED gate for the whole campaign, and stone 1's proof: a process-tier startup failure reproducing the cache probe
(a `:wat::kernel::println` typo → `RuntimeError::UnknownFunction`, wrapped in `StartupError`). Assert the emitted
`LociDiedError/StartupError` chain is a **fully-structured, navigable EDN tree** — the cause is a real
`#wat.runtime/UnknownFunction` **record** (typed, STRICT-decoded), with its own `:message` headline, a real
`:location`, its coordinate fields (`:path`), and `:causes` — **zero escaped-EDN-in-a-String anywhere**. Assert on
STRUCTURE (`assert_edn_eq!` / field extraction), never `.contains()` on a Debug string. Fails RED now (string-wrapped);
green when the mechanism lands.

## Strike order

- **Stone 1 — prove the cache-probe path end-to-end** (the acceptance gate + the hardest boss). Register the
  `RuntimeError` enum (32 variants — it carries the hairy field types, so proving it de-risks the derive for all the
  rest) as `Error`-satisfying records via the enhanced derive; structure `StartupError`'s cause (R57); widen the
  guard; write + green the acceptance RED gate. This proves registration + R57 + the guard on the *real* error.
- **Stone 2 — bulk-register the remaining 9 enums** via the now-proven derive (`ToEdn → Edn` flips).
- **Stone 3 — B shape-fix** the collection/wrapper errors (`UnresolvedReferences` + siblings) + the `location()→Nil` cases.
- **Stone 4 — collapse the other string-wraps** (`ServiceEvent::Lost`, the test-harness) via the same mechanism.

Each stone: DESIGN/RED-probe/BRIEF → rider → **weighed by the orchestrator's OWN `cargo nextest run --release`**
(Summary line, never a piped exit), FOREGROUND-only, commit on my weigh. `:wat::core::Error.location` stays non-Option.
