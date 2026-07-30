# BRIEF — one door for `RuntimeError`: a canonical constructor + a `kind()` accessor (stone B1)

> **Status: DRAWN, NOT STARTED.** Drawn 2026-07-29 as the third stone of the clippy-to-zero campaign,
> after stone A (`5b59f061`) took `result_large_err` 1640 → 659.
>
> **Home:** arc 109 (kill-std) — accumulated-cruft cleanup. Siblings: `BRIEF-evalbreak-width.md` (landed),
> `BRIEF-runtime-error-width.md` (**superseded — its arithmetic was wrong; do not use its numbers**).
>
> **This stone changes NO behaviour and NO widths.** It is the foundation that makes the width change
> (stone B2, one line) possible at all. Read "Why this is the stone" before assuming it is a detour.

---

## The work, in one paragraph

`RuntimeError { span, kind }` is constructed by **~1438 open struct literals** across the tree — 867 in
`runtime.rs` alone — and read through **188 direct `.kind` field accesses**. There is **no canonical
constructor and no accessor**. That absence is the real defect: it means *any* change to this type's
shape costs 1438 edits, which is exactly why its width has gone unfixed while 482 `result_large_err`
warnings stand against it. This stone builds the one door — `RuntimeError::new(span, kind)`,
`kind()`, `into_kind()` — makes the fields **private**, and sweeps every site through it. Widths do not
move; the clippy count does not move. What moves is that afterward, changing this type is a one-line
change forever.

## Why this is the stone (and not just boxing a field)

The cheap alternative — box one more field inside `PostconditionFailed` — was considered and rejected on
correctness, not effort:

- It lands `RuntimeError` at **120** *if* the next-widest variant is 64 and *if* the discriminant stays
  8 bytes. **Both are unmeasured**, and mis-predicting exactly this is what made
  `BRIEF-runtime-error-width.md` wrong: the tag cost 0 bytes before that stone's change and 8 after.
- Even if it lands, 120 leaves **8 bytes of headroom** — and the superseded brief itself named landing
  near the line a STOP, on the grounds that one future field breaks it. Shipping a fix already declared
  too thin is incoherent.
- It leaves the root untouched. The next fat field bills 1438 edits again.

Boxing `kind` behind a private field is the top rung: measured **`RuntimeError` 128 → 56**, 72 bytes of
headroom, and — because the field is private — the width becomes **invisible to every caller**, so no
future variant can re-breach and no future width change touches a call site. That is the difference
between catching this failure and making its whole class unrepresentable.

## THE ONE CONTRACT DECISION — private fields, three doors

```rust
pub struct RuntimeError {
    span: Span,                 // was pub
    kind: RuntimeErrorKind,     // was pub — stone B2 makes this Box<RuntimeErrorKind>
}

impl RuntimeError {
    /// The ONE door for construction.
    pub fn new(span: Span, kind: RuntimeErrorKind) -> Self { Self { span, kind } }
    /// The ONE door for reading the kind.
    pub fn kind(&self) -> &RuntimeErrorKind { &self.kind }
    /// The ONE door for taking the kind by value.
    pub fn into_kind(self) -> RuntimeErrorKind { self.kind }
    /// Span stays readable; it is not what B2 boxes.
    pub fn span(&self) -> &Span { &self.span }
}
```

**Privacy is the load-bearing half, not a style preference.** With `kind` public, B2's `Box` leaks into
every reader and every future width change is another 1438-site sweep. With it private, `Box` is an
implementation detail — `kind()` returns `&RuntimeErrorKind` whether the storage is boxed or not, so B2
becomes one line and stays one line forever. If you keep the fields public, this stone has not done its
job.

**Same doctrine as the `defclause` registration collapse** — one door, not N — and the same reason.

## Proven before briefing (do not re-derive)

`tests/value/probe_runtime_error_boxed_kind_edn.rs` — **committed, green.** It pins B2's one non-trivial
claim empirically: `Box<RuntimeErrorKind>::to_edn()` is byte-identical to `RuntimeErrorKind::to_edn()`,
because `RuntimeError`'s hand-written `ToEdn` (`src/runtime_error_edn.rs:64-83`) reads the field through
a plain method call and `wat-edn` carries a blanket `impl<T: ToEdn> ToEdn for Box<T>`
(`crates/wat-edn/src/lib.rs:217`). Its second test guards that the wrapper still reads the kind
*through the field*, so the forwarding is on the real path. You do not need to re-prove this.

## The measured migration surface

Every number measured this session, by grep over `src/ tests/ crates/`:

```
construction — mechanical, becomes RuntimeError::new(span, kind)
  1034   same-line   RuntimeError { span: X, kind: Y }
   233   multi-line  RuntimeError {\n span: …,\n kind: … }
    48   kind-first  RuntimeError { kind: Y, span: X }      ← argument ORDER must not be swapped
 ~1438   total construction sites (867 of them in src/runtime.rs)

reading — becomes .kind() / into_kind()
   188   direct .kind field accesses
    36   match … .kind          → match err.kind()  (or &*, post-B2)

NOT mechanical — needs restructuring, ~18 sites
    18   PATTERNS destructuring through kind, e.g.
         matches!(err, RuntimeError { kind: RuntimeErrorKind::TypeMismatch { .. }, .. })
         Files: src/io.rs, src/freeze.rs, src/runtime.rs, src/function/eval.rs,
                src/function/parse.rs, src/rust_deps/marshal.rs,
                tests/diagnostics/probe_arc237_stone4_rich_errors.rs,
                tests/diagnostics/probe_arc243_stone7b_signal_split.rs

zero struct-update sites — verified: every `..` in a RuntimeError brace is pattern
rest-syntax, never `..Default`. Nothing to migrate there.
```

**The 18 patterns are the only real thinking in this stone.** They cannot survive B2 as written —
stable Rust cannot pattern-match through a `Box` (`box_patterns`/`deref_patterns` are nightly and are
**not** enabled here; verified). Rewrite each to match on the accessor instead:

```rust
// before
matches!(err, RuntimeError { kind: RuntimeErrorKind::TypeMismatch { .. }, .. })
// after — works identically before AND after B2 boxes the field
matches!(err.kind(), RuntimeErrorKind::TypeMismatch { .. })
```

Do this in **B1**, while the field is still unboxed, so the pattern work is verified green *before* the
boxing lands. That ordering is the whole reason this is two stones.

## Read in order — the rooms

1. **`src/value/signal.rs:102-105`** — the struct. The definition + the new `impl` block go here.
2. **`src/runtime_error_edn.rs:64-83`** — the hand-written `ToEdn`. It reads `self.kind` / `self.span`
   from *inside* the defining crate, so privacy does not break it. Confirm, do not change.
3. **`src/value/signal.rs:107-114`** — the multi-span convention. Read before touching span at all.
4. **`src/runtime.rs`** — 867 construction sites. The bulk; scriptable (see the sketch).
5. **The 18 pattern sites** — listed above with their files. Hand work, one at a time.
6. **`tests/value/probe_runtime_error_boxed_kind_edn.rs`** — the committed probe. Note it constructs
   `RuntimeError { span, kind }` directly; it lives in `tests/`, so **it must migrate to
   `RuntimeError::new` too**. It is a consumer like any other.

## Implementation sketch

The bulk is a mechanical, uniform rewrite — a small throwaway script is the right tool for it, in the
established pattern (a prior sweep did 215 sites with a ~24-line state-tracker). `wat-fix` does **not**
apply; that is the `.wat` codemod and these are `.rs`.

```
1. Add the impl block + flip the fields private.  Build → the compiler now names every site.
2. Script the same-line form:  RuntimeError { span: A, kind: B }  ->  RuntimeError::new(A, B)
   and the kind-first form:    RuntimeError { kind: B, span: A }  ->  RuntimeError::new(A, B)
   ★ the kind-first 48 are the trap: emit (A, B), never (B, A).
3. Hand the multi-line 233 (a script over a brace-spanning literal is where silent corruption
   lives — prefer the compiler's error list, file by file).
4. Rewrite the 18 patterns to match on .kind().
5. Sweep .kind reads -> .kind() and the 36 match sites.
6. Delete the script before the diff is final.
```

Cascade discipline: the fail-count **is** the progress meter — a wide structural change makes many
sites fail at once, each error naming the next site. Watch it waterfall to zero. Never stash-and-revert.

## The gate — a wall that goes RED first

```rust
// tests/value/probe_runtime_error_one_door.rs
#[test]
fn runtime_error_has_exactly_one_construction_door() {
    // Structural: RuntimeError's fields are private, so a struct literal outside its
    // defining module cannot compile. That is what makes stone B2 a one-line change
    // instead of a 1438-site sweep. This test asserts the door WORKS; the compiler
    // asserts the door is the ONLY one (a literal in any other module is an error).
    let e = RuntimeError::new(wat::rust_caller_span!(), RuntimeErrorKind::UserMainMissing);
    assert!(matches!(e.kind(), RuntimeErrorKind::UserMainMissing));
    let k = e.into_kind();
    assert!(matches!(k, RuntimeErrorKind::UserMainMissing));
}
```

**Prove it is a real wall, not a gate that happens to pass (R59 `NISI FRANGAS, NIHIL PROBAS`):** before
finishing, add a `RuntimeError { span: …, kind: … }` literal to a test module, confirm it **fails to
compile**, then delete it. Report that you did this and what the error said. A privacy wall nobody tried
to breach is a claim, not a wall.

## Blast radius

`src/value/signal.rs` (definition + impl), and every construction/read site across `src/`, `tests/`,
`crates/`. **No function signature changes. No width changes. No `RuntimeErrorKind` variant changes. No
`#[allow]` anywhere.** Do not touch `clippy.toml`. Do not box anything — boxing is stone B2 and lands
separately.

## STOP triggers — REJECTION criteria. Ship nothing further and report.

1. **STOP-1: privacy breaks the crate's own `ToEdn`/`Display`/`WatError` impls in a way a same-crate
   access cannot fix.** Those live in `src/runtime_error_edn.rs` and `src/value/signal.rs`, inside the
   defining crate, so field access should remain legal. If some impl lives *outside* the crate and
   genuinely needs the field, **STOP** — report which, and the one-door design needs re-drawing (a
   `pub(crate)` field is a different contract and is not yours to choose).
2. **STOP-2: a construction site cannot use `new` because it needs partial/deferred construction**
   (builds a `RuntimeError` in pieces, or mutates `kind` after the fact). Report the site. Do not add a
   setter or re-`pub` the field to route around it.
3. **STOP-3: the structured error EDN moves.** `probe_arc298_3_runtime_derive_identical` and
   `probe_arc237_stone4_rich_errors` are the arbiters. Note honestly that many of their tests are
   `#[ignore = "296-recapture-pending"]` and already fail at HEAD for an unrelated stale-golden reason,
   so a filtered green there is weak evidence — say which tests carried your claim.
4. **STOP-4: the floor moves for a reason you cannot name.** The floor is `cargo nextest run --release`
   at **4187 passed / 262 skipped** (4185 at stone A, plus the 2 committed probe tests). Your own gate
   makes 4188. Any *other* change to the count is a STOP, not something to reconcile.
5. **STOP-5: the argument order is wrong at any kind-first site.** `RuntimeError::new(span, kind)` —
   the 48 `kind:`-first literals are the one place a mechanical rewrite can silently transpose two
   arguments of different types. If the compiler does not catch a transposition somewhere (it should —
   `Span` and `RuntimeErrorKind` are unrelated types), say so, because that means the types are not
   protecting you and the sweep needs a stricter check.

## Gates the rider runs

- `cargo build --release --all-targets` → no new warnings. One pre-existing `unused_comparisons` in
  `tests/value/probe_arc216_stone5a_value_hash.rs:347` is at HEAD; it is not yours.
- The new gate: green, plus the deliberate-breach check above.
- `cargo nextest run --release` → **4188 passed**. Read the ANSI-stripped **Summary** line by hand;
  never a piped exit code (`| tail` returns `tail`'s exit).
- The clippy count, unchanged, by JSON (a bare `grep -c` is unreliable — incremental caching suppresses
  re-emission for unchanged crates):
  ```
  touch src/value/signal.rs
  cargo clippy --release --workspace --all-targets --message-format=json \
    | grep -c '"code":"clippy::result_large_err"'
  ```
  Expect **659, unchanged**. This stone fixes no warnings; a change here means something unintended moved.

## Expectations — written before the strike

| what | how it is checked | expected |
|---|---|---|
| construction sites through `new` | `git grep -c 'RuntimeError {'` outside `signal.rs` | **0** |
| fields private | a literal in another module | **compile error** (deliberately proven) |
| `result_large_err` | clippy JSON count | **659, unchanged** |
| `RuntimeError` width | the width gate | **128, unchanged** |
| structured error EDN | STOP-3's arbiters | unmoved |
| floor | `nextest --release` Summary | **4188 passed** |
| signatures changed | `git diff` | **zero** |

**Runtime prediction:** 90–150 min — the bulk is mechanical but large, and the 18 patterns are real work.
**Trap-door risks, in order:** (1) the 48 kind-first literals transposing arguments; (2) the 233
multi-line literals, where a brace-spanning script silently corrupts; (3) a pattern site rewritten to
`.kind()` that changes binding-by-move to binding-by-reference and no longer compiles downstream.

## Stone B2 — what this unlocks (do NOT do it in this stone)

One line: `kind: RuntimeErrorKind` → `kind: Box<RuntimeErrorKind>`, plus `Box::new` in `new` and `*self.kind`
in `into_kind`. **Measured: `RuntimeError` 128 → 56.** Clears the **482** `RuntimeError`-attributed
warnings; the width gate's ceiling drops from its regression bound to `<= 120`. Because the field is
private, no call site changes. Expect a dividend on `StartupError` too — `impl From<RuntimeError> for
StartupError` (`src/freeze.rs:723`) suggests it holds a `RuntimeError` inline, so its 21 warnings and
160-byte width may fall out for free; **measure, do not assume** — unmeasured prediction is what broke
the first brief in this family.

## Out of scope

- **Boxing anything** (stone B2).
- **`TypeError` 152/103 warnings, `LoadError` 160/44, misc 9** — separate types, separate stones.
- **The 229 non-`result_large_err` lints** — their own stone, briefed in pieces.
- **Arming `-D warnings`** in `.github/workflows/ci.yml:41-44` — the campaign's last act. Zero is a
  moment; the wall is what makes it permanent.
