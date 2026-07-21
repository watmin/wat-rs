# NOTE — an anonymous fn's identity is a STRUCTURED unit (its defining span), not a `<fn@file:line:col>` string

> **Deferred design decision (builder, 2026-07-21, arc 278).** Surfaced in the caller.2 gate's
> `AssertionFailure`, whose frame carried `:callee :<fn@tests/services/probe_arc278_emitted_from.wat:18:1>`.
> Builder: *"the `:<fn@…>` is doing a wonky way to express a structured unit of data… we need to kill this."*
> Kin to caller.2 itself (a call-site expressed as a forgeable keyword → the structured `:wat::kernel::Frame`);
> this is the same class, one layer down. Recorded per the arc-109 `NOTE-*.md` convention.

## The wart
An **anonymous** fn (no bound name) has its identity rendered as a **string**:
`format!("<fn@{}>", ast.span())` → `<fn@file:line:col>`. Grounded, 3 sites:
- `src/freeze.rs:422` · `src/freeze.rs:455` · `src/runtime.rs:20083` — all `FunctionBody::Wat(ast) => format!("<fn@{}>", ast.span())`.

That token packs a **structured unit** — the fn's *defining source location* (file, line, col) — into a stringy
blob a consumer must parse back out. It's the exact anti-pattern this arc keeps killing: structured data wearing
a string/keyword costume instead of a real value (records-are-EDN; a location is a `:wat::kernel::Frame`, not a
`#"…:l:c"`).

## The symptom that surfaced it (caller.2 gate)
Two related faces of the same under-structuring, both seen when `(:wat::kernel::call-site)` runs inside an
anonymous fn (a thread-peer test body, `wat/spawn.wat:272` invoking `<fn@…:18:1>`):
1. **In a captured `Frame`**, an anon fn's `symbol` comes back **`None`** — there's no structured name to carry
   (the caller.2 gate asserting `Frame/symbol` is `Some` therefore failed inside an anon-fn context).
2. **In display / assertion frames**, the same anon fn is rendered as the **`<fn@file:line:col>` string**.

So an anon fn's identity is `None` in one path and a stringified span in another — neither is a proper
structured value.

## The fix (deferred)
Give an anonymous fn a **structured identity** — its defining span as real fields, not a formatted string.
Options to weigh at draw time:
- Reuse **`:wat::kernel::Frame {file, line, symbol}`** (or a `{file, line, col}` core span record) as the anon-fn
  identity, so a frame's `:callee` / `symbol` for an anon fn is structured data (`symbol` = e.g. an
  `<anonymous>` marker, the location living in `file`/`line`), never a `<fn@…>` string and never a bare `None`.
- A dedicated fn-identity value if a new noun is warranted — **intueri cast owed** at draw (do not narrate a
  name here).
Kill the `format!("<fn@{}>", ast.span())` at the 3 sites in favor of the structured form.

## Status
**DEFERRED.** Arc-109-adjacent cleanup (stringy-representation → structured value). Kin to caller.2
(`emitted-from <- :wat::kernel::Frame`) and the general "structured data is a value, not a string" push.
Grounded: `src/freeze.rs:422,455`, `src/runtime.rs:20083`.
