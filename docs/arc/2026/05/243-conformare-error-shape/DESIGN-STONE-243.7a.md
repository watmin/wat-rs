# DESIGN — Stone 243.7a — Box `RuntimeError` large variants (the `result_large_err` retrofit)

**Status:** NAMED + OPEN (the deferral target). Child of arc 243 (conformare). Named 2026-06-01 to give the warded-home `result_large_err` exemptions a legitimate OPEN-DEFERRAL target (excusare rules a deferral to the unnamed `243.7…` placeholder ILLEGITIMATE-AT-BIRTH; this stone is the named, in-chain, in-reach address).

## Why this stone exists (named now, struck later)

`clippy::result_large_err` fires ~605× substrate-wide: `RuntimeError` is a 33-variant enum returned **by value** in `Result<_, RuntimeError>` everywhere. The lint is **correct** — a large Err variant inflates every `Result` on the stack, even the happy path, because the return type must size to the largest variant.

The fix is NOT local. Every site returns the bare `RuntimeError` type; the lint is on the type's size, not on any one function. You cannot box "these 9 sites" — they are the same type as the other ~596, `?`-chained together. The only fix is at the **type definition**: box the large variants (or the whole error) so `RuntimeError` becomes small-by-value. That ripples to all ~605 return sites + every `?` boundary. THAT is this stone — a full RuntimeError-family retrofit, peer to 243.3 (TypeError) and 243.6 (CheckError).

It is correctly **deferred**, not done tonight: it is the 605-site type-level change, and doing it under pressure (to green two warded-home stamps) would be the tail wagging the dog. This DESIGN names it so the deferral that points here is honest.

## Scope (when struck)

- Box the large `RuntimeError` variants (audit which variants carry `Value` / `Arc<Function>` / `Vec<Value>` / large payloads; box those) OR wrap the whole error (`Box<RuntimeErrorInner>` newtype) — decide by four-questions at strike time + an FM 2-bis probe on the `?`-chain ergonomics.
- Cascade: every `Result<_, RuntimeError>` site stays source-compatible (boxing is transparent to `?` if the From-impls thread); verify the full workspace stays green via substrate-as-teacher.
- Retire EVERY `#[allow(clippy::result_large_err)]` and every `rune:excusare(OPEN-DEFERRAL → 243.7a)` the moment this ships — they convert to CLOSED-DEFERRAL and MUST be struck (excusare re-musters them).
- Likely its own `src/` home-carve per the rolling-audit pattern (RuntimeError is in the fat `runtime.rs`); may decompose into sub-stones. In-chain after 243.6 (CheckError) — the home-carve pattern matures first on the smaller error types.

## What this stone unblocks (the deferrals pointing here)

The instant 243.7a ships, these become CLOSED-DEFERRAL and are struck:
- `src/function/eval.rs` + `src/function/parse.rs` — 2 `result_large_err` sites (warded-home drift)
- `src/rust_deps/custodia.rs` (×4) + `src/rust_deps/marshal.rs` (×3) — 7 sites (warded-home drift)
- `src/runtime.rs:13073` — the bare reasonless `#[allow(clippy::result_large_err)]` (excusare: ILLEGITIMATE-AT-BIRTH; this stone is its legitimate home OR it gets struck+fixed here)
- the remaining ~590 substrate-wide sites (flat untrusted files — cleaned as the type retrofit lands)

## Cross-references

- arc 243 `DESIGN.md` line 57 (`243.7…` rolling audit — this NAMES the RuntimeError row)
- `datamancy.dev/excusare/SKILL.md` — OPEN-DEFERRAL requires a named target; this is it
- task #170 (the ward-drift this closes) + #167 (the adjacent `list_span` RuntimeError span-thread debt — same error type, likely same retrofit)
- `docs/CONFORMARE.md` — Pattern A (the shape RuntimeError retrofits toward)
