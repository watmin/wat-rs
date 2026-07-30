# BRIEF — one door for `TypeError` + `LoadError` (clears the last 168 `result_large_err`)

> **Status: DRAWN, NOT STARTED.** Drawn 2026-07-29 as stone C of the clippy-to-zero campaign, after
> A (`5b59f061`), B1 (`8acbf23a`) and B2 (`5b4d8d75`) took `result_large_err` 1640 → 171.
>
> **Home:** arc 109 (kill-std).
>
> **This is a REPEAT of a proven template, not a new design.** `RuntimeError` is the worked exemplar —
> `src/value/signal.rs:107-152` (the door) plus B2's boxed private field. Copy that shape. The two types
> here are *structurally identical* to what `RuntimeError` was, and their surfaces are ~40× smaller.

---

## The work, in one paragraph

`TypeError` and `LoadError` are the same shape `RuntimeError` was — `pub struct X { pub span: Span, pub
kind: XKind }`, no constructor, open literals everywhere. Give each the same one door
(`new` / `kind` / `into_kind` / `span`, fields **private**), sweep its sites through it, then box the
private `kind`. That clears **103 + 44 = 147** warnings directly and — measured below — takes
`StartupError`'s remaining **21** with it for free, because `StartupError`'s width is driven by these two.
Total: **171 → ~3**.

## The measured chain — why StartupError needs no work of its own

Measured this session (`size_of`, my own runs):

```
StartupError = 160   (an ENUM; its width tracks its widest inline payload)
  payloads:  ParseError 88 · ConfigError 104 · LoadError 160 ←DRIVER · MacroError 96
             TypeError 152 ←DRIVER · ResolveError 24 · CheckErrors 24

TypeError = 152 = span 48 + TypeErrorKind 104
LoadError = 160 = span 48 + LoadErrorKind 112
```

`StartupError` at exactly 160 — the same as `LoadError` — means the variant tag is niche-packed and costs
nothing today. Once `TypeError` and `LoadError` are 56, the widest payload becomes **`ConfigError` at
104**, so `StartupError` lands at 104 (niche-packed) or 112 (with an 8-byte tag). **Both are under 128**,
which is why this prediction is safe where an earlier one in this family was not: it does not straddle
the threshold. **Still measure it — do not assume.** A prior brief in this family predicted 120 and got
128, and that is the whole reason the campaign has a measure-first rule.

This also explains a delta from stone B2: boxing `RuntimeError.kind` gave `StartupError` **no** dividend,
because `StartupError` never held a `RuntimeError` — it holds `LoadError`/`TypeError`. That was flagged
as an unmeasured guess at the time and it did not pay off; this chain is the grounded version.

## THE ONE CONTRACT DECISION — copy `RuntimeError`'s door exactly

For **each** of the two types, mirror `src/value/signal.rs:107-152`:

```rust
pub struct TypeError {
    span: Span,                  // was pub
    kind: Box<TypeErrorKind>,    // was pub TypeErrorKind
}

impl TypeError {
    pub fn new(span: Span, kind: TypeErrorKind) -> Self { Self { span, kind: Box::new(kind) } }
    pub fn kind(&self) -> &TypeErrorKind { &self.kind }
    pub fn into_kind(self) -> TypeErrorKind { *self.kind }
    pub fn span(&self) -> &Span { &self.span }
}
```

**Privacy is the load-bearing half.** With the field public, the `Box` leaks into every reader and the
next width change is another sweep. Private, `kind()` returns `&TypeErrorKind` whether the storage is
boxed or not — which is why B2 was three lines and cost **zero** call sites on a type constructed 1438
times. If you leave the fields public, this stone has not done its job.

**Do it in one stone, not two.** B1/B2 were split because `RuntimeError` had 1438 literals and 49 pattern
sites, so the pattern work needed proving green before the boxing landed. Here the surfaces are 22 and 15
literals; splitting would cost more than it protects. Land the door and the box together, per type.

## The measured migration surface

```
TypeError   (src/types/error.rs)
    22   construction literals    TypeError { span: …, kind: … }
    25   files mention the type

LoadError   (src/load.rs)
    15   construction literals    LoadError { span: …, kind: … }
    10   files mention the type

StartupError (src/freeze.rs:570)  — an enum; ZERO {span,kind} literals. Expected to need NO edit;
                                    verify its width after C1+C2 and report the number.
```

**A caution about counting `.kind` reads:** a bare `git grep -c '\.kind'` returns the same figure for
every type because it cannot tell which type a receiver has — it is a junk number, and this brief
deliberately does not quote one. Let the **compiler** enumerate the read sites once the fields are
private; that list is exact and it is the only one to trust. (Two earlier greps in this campaign produced
exactly this kind of bogus count, so the caution is earned.)

## Read in order — the rooms

1. **`src/value/signal.rs:107-152`** — the WORKED EXEMPLAR. `RuntimeError`'s private fields, its four
   doors, and B2's boxed-`kind` doc comment explaining why the box is invisible. Copy this shape twice.
2. **`src/types/error.rs`** — `TypeError`'s definition. Its `impl` block goes here.
3. **`src/load.rs`** — `LoadError`'s definition, same.
4. **`src/freeze.rs:570-590`** — `StartupError`'s variants. **Read only**; you should not need to edit it.
5. **`tests/value/probe_runtime_error_width.rs`** — the width-gate shape to mirror, including *why* its
   ceiling is 120 and not 128.
6. Each type's `ToEdn` / `Display` / `WatError` impls. **Note:** field privacy in Rust is **module**-scoped,
   not crate-scoped — a sibling module cannot reach a private field. B1 learned this the hard way when
   `runtime_error_edn.rs` (a sibling of `value::signal`) needed the accessors. Expect the same here and
   use the doors; it is not a blocker.

## Implementation sketch

Per type: add the impl block, flip the fields private and box `kind`, build, and let the compiler name
every site. At 22 and 15 literals this is hand work — **no script**. A script is what corrupted three
files during B1 (a UTF-8 byte-offset vs code-point bug); at this scale it buys nothing.

Pattern sites destructuring through `kind` — e.g. `matches!(e, TypeError { kind: TypeErrorKind::X { .. }, .. })`
— must become `matches!(e.kind(), TypeErrorKind::X { .. })`. Stable Rust cannot match through a `Box`
(`box_patterns`/`deref_patterns` are nightly and not enabled here). The compiler finds each one.

## The gate — walls that go RED first

Add to `tests/value/probe_runtime_error_width.rs` (it already houses this campaign's width walls):

```rust
#[test]
fn type_error_stays_narrow() {
    assert!(size_of::<TypeError>() <= 120,
        "TypeError is {} bytes (ceiling 120; clippy::result_large_err fires at >= 128)",
        size_of::<TypeError>());
}
#[test]
fn load_error_stays_narrow() { /* same, LoadError */ }
#[test]
fn startup_error_stays_narrow() { /* same, StartupError — the one that should fall out for free */ }
```

**RED first, all three** — 152 / 160 / 160 today. Verify that before changing production code. The
ceiling is **120, not 128**: clippy fires at `>= 128`, grounded on this tree — while `RuntimeError` sat at
exactly 128, all 482 of its warnings still stood. If you write `<= 128` you have rebuilt the vacuous gate
this campaign already shipped once (R59 `NISI FRANGAS, NIHIL PROBAS`).

## Blast radius

`src/types/error.rs`, `src/load.rs`, their construction/read sites, and the width gate. **No signature
changes. No `#[allow]`. No `clippy.toml`.** Do not edit `src/freeze.rs` — `StartupError` should fall out;
if it does not, that is STOP-2, not an invitation.

## STOP triggers — REJECTION criteria. Ship nothing further and report.

1. **STOP-1: a construction site needs partial or deferred construction** (builds the error in pieces, or
   mutates `kind` afterward). Report the site. Do not add a setter or re-`pub` the field.
2. **STOP-2: `StartupError` does not come under 128** after both types land. Report its measured width
   and the measured width of every payload, so the real driver is named. Do **not** start boxing
   `StartupError`'s variants on your own judgement — that is a different stone with its own contract.
3. **STOP-3: any type's EDN moves.** These carry `ToEdn`; the blanket `impl<T: ToEdn> ToEdn for Box<T>`
   (`crates/wat-edn/src/lib.rs:217`) makes boxing transparent **only** where the impl reads the field by a
   plain method call, as `RuntimeError`'s does. If either type's `ToEdn` reads its kind some other way —
   a `via`, a match on the field, a manual field-by-field build — **STOP** and report which, because the
   byte-identical claim does not automatically transfer. `tests/value/probe_runtime_error_boxed_kind_edn.rs`
   is the exemplar of how that claim gets pinned.
4. **STOP-4: the floor moves for a reason you cannot name.** The floor is `cargo nextest run --release` at
   **4189 passed / 262 skipped**. Your three gate tests make 4192. Any *other* change is a STOP.

## Gates the rider runs

- `cargo build --release --all-targets` → no new warnings. One pre-existing `unused_comparisons` in
  `tests/value/probe_arc216_stone5a_value_hash.rs:347` is at HEAD; not yours.
- The three new gates: RED before, green after.
- `cargo nextest run --release` → **4192 passed**. Read the ANSI-stripped **Summary** line by hand; a
  piped `| tail` returns `tail`'s exit code.
- The clippy count and its per-type histogram, by JSON (a bare `grep -c` is unreliable — incremental
  caching suppresses re-emission for unchanged crates):
  ```
  touch src/types/error.rs src/load.rs
  cargo clippy --release --workspace --all-targets --message-format=json \
    | grep -c '"code":"clippy::result_large_err"'
  ```
  Expect **171 → ~3**. Also report the count grouped by the type clippy names, extracting it from each
  diagnostic's child message matching ``try reducing the size of `([^`]+)` ``, so each bucket is checkable
  on its own.

## Expectations — written before the strike

| what | how it is checked | expected |
|---|---|---|
| `TypeError` width | new gate | 152 → **56** |
| `LoadError` width | new gate | 160 → **56** |
| `StartupError` width | new gate | 160 → **≤ 112**, with no edit to `freeze.rs` |
| `result_large_err` total | clippy JSON | **171 → ~3** |
| `TypeError`-attributed | JSON histogram | 103 → **0** |
| `LoadError`-attributed | JSON histogram | 44 → **0** |
| `StartupError`-attributed | JSON histogram | 21 → **0** |
| other lints | same | ~234, and report any movement with its cause |
| floor | `nextest --release` Summary | **4192 passed** |
| signatures changed | `git diff` | **zero** |

**Runtime prediction:** 40–70 min — two small repeats of a landed template.
**Trap-door risks:** (1) a `ToEdn` that does not read its kind by a plain method call (STOP-3); (2) sibling-module
privacy, expected and handled by the doors; (3) `StartupError` having a second fat payload the chain above
did not predict — measured as `ConfigError` 104, so the margin is 16–24 bytes rather than 0, but measure.

## Out of scope

- **The 234 non-`result_large_err` lints** — stone D, briefed in pieces. The 18 `mutable_key_type` want
  reading before anyone is briefed on them.
- **Boxing `StartupError`'s variants** — only if STOP-2 fires, and then as its own stone.
- **Arming `-D warnings`** (`.github/workflows/ci.yml:41-44`) — the campaign's last act, and the point of
  all of it. Zero is a moment; the wall is what makes it permanent.
