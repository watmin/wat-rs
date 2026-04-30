# Realizations — Arc 109 (kill-std)

Observations that surfaced during the work and are worth keeping
even though they didn't fit the INVENTORY's structural sections.

## The transitional-form pattern

The incremental approach for each FQDN family looks like:

1. **Additive (1a-shape).** Parser/checker accept BOTH the old bare
   form AND the new FQDN form. Internal representation stays the
   bare form (implementation detail; never escapes to user code).
   No consumer breaks. Tests stay green.

2. **Sweep (1b-shape).** Mechanical replacement of every bare
   reference in `.wat` sources to FQDN. Tests stay green at every
   step (because the additive phase still accepts both spellings).

3. **Retire (1c-shape).** Parser/checker errors on the bare form
   with a *self-describing* error pointing at the FQDN
   replacement:

   > `bare ":i64" retired in arc 109 — use ":wat::core::i64"`

   Test failures (broken fixtures, missed sites) are now
   self-correcting: the error message tells the next reader (or
   delegated agent) exactly what to type.

The pattern keeps the test suite green at every checkpoint, lets
us spot-and-correct as we migrate, and surfaces missed sites
through cargo's own error reporting rather than through silent
behavioral drift.

## The redirect helper is point-in-time

The retirement error itself — the match arm in
`src/types.rs::parse_type_inner` that matches bare `:i64`/`:f64`/
`:bool`/`:String`/`:u8` and returns
`TypeError::MalformedTypeExpr` — is a **transitional form**. It
exists only during the migration window.

After the full FQDN migration completes:

- Every consumer (substrate stdlib, tests, lab, examples) has
  already been swept to the FQDN form.
- No live caller — internal Rust or external wat — uses the bare
  form anymore.
- The error message has nothing to teach.

At that point, the redirect arm can be **removed entirely**. The
parser's plain-path branch goes back to its pre-arc-109 shape —
it just constructs `TypeExpr::Path(format!(":{}", s))` — and the
substrate is honest end-to-end: bare paths under
`:wat::core::*`-shaped names are user namespace (not substrate),
exactly as the doctrine says.

The same retirement logic will appear and disappear once per FQDN
family that arc 109 migrates:

- Slice 1c: `:i64` / `:f64` / `:bool` / `:String` / `:u8` retirement.
- Slice 2c (after 2's parametric heads land): `Vec` / `Option` /
  `Result` / `HashMap` / `HashSet` retirement.
- Slice 4c: `Some` / `:None` / `Ok` / `Err` retirement.
- Slice 5c: `:else` retirement.
- Slice 7: existing-name retirement (the `:wat::core::try` /
  `option::expect` / `result::expect` aliases from arc 108).
- etc.

Each retirement exists for one purpose: surface missed sites with
a self-describing error. Once the substrate has been fully swept,
the retirement code is dead weight and gets removed.

## What this realization means in practice

- **Don't be precious about the retirement code.** It's
  scaffolding. Write it for clarity (the error message IS the
  fix); ship it; remove it when its job is done.
- **The error message is the spec for the migration.** A
  retirement that says `"this name is gone"` without naming the
  replacement is hostile. A retirement that says `"use this
  exact name instead"` is collaborative — the migration delegates
  itself to whoever's watching the test output (human or agent).
- **The final state has no redirect arms.** When arc 109 closes
  for real, `parse_type_inner` looks the same as before arc 109
  except the user-facing form is FQDN. The substrate didn't grow
  permanent migration plumbing — it grew the right names and
  dropped the redirect once those names landed.

## When redirects DO survive

Aliases shipped as user-facing convenience (e.g.
`:trading::treasury::Service::PaperStateEntries` aliasing the
verbose `:Vec<(i64,Option<...>)>` shape) are NOT migrations —
they're permanent typealias decls the substrate's typealias
system supports. Don't conflate the two:

- **Typealias** (substrate-supported, permanent): one name in,
  same name out at unification time. Both forms are equally
  honest — the alias is a documentation and ergonomics device.
- **Migration redirect** (transitional, temporary): old name in,
  parse-time error out. Exists only to teach. Ripped out when
  the migration closes.

A migration redirect that survives past its window has rotted —
it's a permanent alias the substrate is silently supporting,
which violates the FQDN-is-the-namespace doctrine.

## The expect tooling is a bridge

Arcs 107 and 108 shipped `:wat::core::option::expect` and
`:wat::core::result::expect` directly because of proof_004's
silent-disconnect-cascade hang. The arc's INSCRIPTION explicitly
deferred the broader call-site sweep:

> Does NOT migrate other call sites that COULD use expect (e.g.,
> `Service/batch-log`, `Stream`'s ack loops). Each call site is a
> separate decision per author.

That deferred sweep is arc 110. The substrate's deadlocks have
been silent-swallow disconnects, not logical errors:

- A worker thread panics (e.g., the proof_004 reporter's
  `Atom`-on-Struct bug).
- The worker's channel-end disconnects.
- The producer's next `(:wat::kernel::send tx ...)` returns
  `:None` (silent — `:wat::kernel::Sent` IS `:Option<()>`).
- The producer ignored the result (`((_s :wat::kernel::Sent)
  ...)`), so it keeps going.
- The next `recv` it issues blocks forever — the peer is gone but
  this thread doesn't know it.

`:wat::core::option::expect` turns silent disconnect into a
panic-with-message at the exact call site. The bridge is
load-bearing during the FQDN migration: **before** we retire
bare names slice-by-slice, we want every comm path to surface
peer-death honestly. That way, when a slice goes wrong (an
escaped bare path, a missed sweep), the failure is loud and
diagnosable instead of an indefinite hang in CI.

The sweep is **not** uniform — it distinguishes shapes:

- **Worker recv loops** —
  `(:wat::core::match (:wat::kernel::recv rx) -> :T
     ((Some v) recurse) (:None terminate))` —
  KEEP. `:None` IS the legitimate end-of-work signal per
  `SERVICE-PROGRAMS.md`'s shutdown contract.

- **Client send/recv with `_`-bound `Sent` or `Option<T>`** —
  `((_s :wat::kernel::Sent) (:wat::kernel::send tx msg))` —
  WRAP. Silent swallow is the bug.

- **Stream-stage `match send`** — case-by-case. Some legitimately
  exit on downstream-closed; some are silent.

The realization fits the same incremental pattern as the FQDN
sweep itself: bridge tools (107/108) → broad sweep (110) →
slice retirements (109's remainder). Each layer surfaces
problems that would otherwise stay invisible.

## Cross-references

- `INVENTORY.md` — the FQDN inventory + slicing strategy.
- `feedback_fqdn_is_the_namespace.md` (memory) — the doctrine
  that motivates the cleanup.
- `arc/2026/04/107-option-result-expect/INSCRIPTION.md` — the
  bridge tool's first shape (`:wat::std::*` helpers).
- `arc/2026/04/108-typed-expect-special-forms/INSCRIPTION.md` —
  the bridge tool's final shape (`:wat::core::*` special forms
  with `-> :T` at HEAD); explicitly defers the broader sweep.
- `docs/SERVICE-PROGRAMS.md` § "The lockstep" — why worker
  recv-loops legitimately exit on `:None`.
