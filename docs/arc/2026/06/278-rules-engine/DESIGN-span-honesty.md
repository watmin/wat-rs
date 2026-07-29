# DESIGN — span honesty: never point a wat author at Rust

> **Origin (arc 170 closure #6, 2026-07-28→29):** lifting the spawn ORIGIN onto `ps` labels needed
> the caller's source position. `Bracket` got it; `Service` came back naming `wat/core.wat:649` for
> every service. Chasing that one wrong label opened a defect class across the whole diagnostic path
> — `ALIVS ARGVIT`, the consumer surfaced the flaw.

## The class, in one sentence

A location that reaches a wat author must name **the `.wat` they wrote**. Three distinct mechanisms
were violating that, and they are NOT the same bug.

| # | mechanism | where | status |
|---|---|---|---|
| 1 | **substitution** — a Rust fn holds a real wat span and mints `rust_caller_span!()` anyway | 43 sites, 5 files | **FIXED** (`1d334431`) + walled |
| 2 | **template-stamping** — a macro rewrites the user's CALL and the emitted form carries the TEMPLATE's span | `src/macros/expand.rs` | strike in flight |
| 3 | **non-reproducible paths** — user-`.wat` spans are absolute; Rust/stdlib spans are relative | `crates/wat-reader/src/span.rs` | OPEN |

## 1 — substitution (closed)

`unused_span_justified` polices span **omission** (`_…span: &Span`, discarded) and its header states
the boundary is load-bearing: a USED param must not match. So substitution — where nothing is
discarded and the *wrong* span is USED — was unwalled. New wall:
`tests/lint/span_substitution_justified.rs`.

**What kept it alive was comments, not code.** `io.rs`/`runtime.rs` carried arc-138 notes — *"no span
— no AST context available at the point of failure"* — in fns whose signature had `list_span: &Span`.
A reviewer reading that has every reason to believe it. Hence a wall, not an audit.

**Known gaps in the new lint, stated not discovered:**
- counts per LINE, not per occurrence (`rete/collect.rs:71` held two calls, reported as one) — so the
  47 understated the population.
- keys on PARAMETER NAMES (`…span: &Span`). That is why it took two attempts to write, and why the
  sibling lint could never reach `_call_site` (ends in `site`, not `span`). **A Rust-origin span is
  structurally identifiable** — see §3 — and a wall built on that fact would have no
  name-shaped holes. The honest next sharpening.
- the SIBLING lint still names *"a `rust_caller_span!()` Rust line instead of their `.wat`"* as the
  HARM and lists `rust_caller_span!()` among its earned reasons. A rule cannot forbid a thing and
  accept it as its own justification. 2 existing runes cite it and must re-earn standing or become
  fixes.

## 2 — template-stamping (the root of the `Service` label)

`kwargs-lower` (`wat/core.wat`) rewrites `(svc/start …)` → `(svc/start$impl …)`. The emitted call
carries the template's span, so the only frame pushed names `wat/core.wat:649` — and the author's
line is **absent from the stack**, not buried. MEASURED; probes rule out every reader-side recovery
(`-1`, name-search, any selection policy): `wat-scripts/scratch-pad/probe-call-site-kwargs.wat`,
`probe-kwargs-stack-shape.wat`. Substitution destroys the location where it happens, so it must be
walled there.

**Consequence far past labelling:** `assertion-failed!` inside ANY kwargs fn reports the author's
failure as living in `core.wat:649`. Kwargs are the doctrine for user-facing forms.

**The hook already exists and was switched off.** `src/macros/expand.rs` calls
`restamp_unknown_spans(expanded, &call_site_span)` on every expansion; arc 298.2 hollowed it to an
identity fn and discarded the param, reasoning that *"synthetic nodes carry an honest Rust caller
location."* A Rust location is not honest to a wat author — that premise IS the defect.

**Rule being restored:** re-stamp any node whose `span().file` differs from the call site's with the
call site. User-spliced nodes (`~`/`~@`) already live in the caller's file and keep their more
precise spans. **Stated limitation:** a macro defined AND used in one file is not re-stamped (files
match) — the file is right, only the line may point at the template.

**Cascade expectation is SMALL, and here is why** — `crates/wat-reader/src/span.rs`:

```rust
impl PartialEq for Span { fn eq(&self, _: &Self) -> bool { true } }
```

Span equality is unconditionally true; spans contribute nothing to structural equality. Structural
AST/Value comparisons are BLIND to a re-stamp. Only a **rendered** location can flip (an `.edn`
golden carrying `:file`/`:line`, a diagnostic string). A large cascade would therefore be evidence
that something other than rendering changed — a signal to re-read, not to mass-update.

## 3 — path reproducibility (open)

Three path shapes, unequal:

| origin | observed `:file` | reproducible |
|---|---|---|
| Rust | `wat-rs/src/freeze.rs` | ✅ relative |
| baked stdlib wat | `wat/core.wat` | ✅ relative |
| user `.wat`, CLI-loaded | `/home/watmin/work/holon/wat-rs/…/probe.wat` | ❌ **absolute** |
| user `.wat`, harness-loaded | `tests/wat_lang/wat_core_cond_no_else.wat.bad` | ✅ relative |

**Corrected 2026-07-29** — an earlier draft of this table said "user `.wat` → absolute" flatly. That
is FALSE. The shape depends on **how the file was loaded**, not on whose file it is: the CLI
canonicalizes its entry path, while `startup_from_file` keeps the path it was handed verbatim. So
the reproducibility hazard is real but NARROWER than first written — it bites CLI-loaded paths. The
`cond` golden below is the counter-example that caught the overstatement.

`rust_caller_span!()` = `Span::new(Arc::new(format!("wat-rs/{}", file!())), line!(), column!())`.
The `wat-rs/` prefix is a **reproducibility device** (builder): absolute laptop paths cannot be baked
into goldens, so Rust paths are normalized to repo-rooted. That normalization was **never applied to
user `.wat` spans** — which is why `tests/process/wat_arc170_closure6_label_wall.rs` derives its
expected path at runtime via `canonicalize()` instead of baking it. The workaround was invented
without recognising it as this gap.

**⚠ §2 moves diagnostics from the stable column into the unstable one** — from `wat/core.wat:649`
(relative, golden-safe) to the caller's own file, absolute for a disk-loaded fixture. No golden may
bake an absolute path; derive at test time (precedent: the label-wall gate).

**Ruled (builder, 2026-07-29): DROP the `wat-rs/` prefix.** `file!()` already yields `src/runtime.rs`
and the `.rs` suffix is the honest discriminator (wat files are `.wat`); the prefix adds only the
doubling hazard — `format!("wat-rs/{}", file!())` has no guard, and `file!()` resolves ABSOLUTE in
some build configurations, yielding `wat-rs//home/…`. Not currently produced (measured: no `wat-rs//`
on disk) but latent. Three sites assert on the prefix *as a proxy for "is this a real Rust path"* —
`src/stdlib.rs:691`, `src/to_edn_derive_tests.rs:218`/`:328` — and become MORE honest as `.rs`-suffix
tests. ~15 files mention it, one `.edn` golden.

**Second structural tell, currently unused:** `Span::new` leaves `end: None`; the lexer sets a real
`end` for wat tokens. So `end.is_none()` marks a Rust-origin span independently of any path string —
the field-based provenance that a path prefix was standing in for.

## Order (each lands green + committed before the next)

1. §2 restamp — in flight.
2. Drop `wat-rs/` + the 3 assertions → `.rs` suffix.
3. §3 user-`.wat` path normalization (makes locations bakeable; retires the label gate's workaround).
4. Sibling lint's self-contradiction + its 2 runes.
5. Then arc 170 #1 — `wat --repl`, the closure condition.
