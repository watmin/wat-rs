# NOTE — a live angle-minter survives in `wat/core.wat`, and my census could never have seen it

**Filed 2026-08-23**, from R3's STOP-3 flag during the prose sweep, confirmed by my own reading.

## The survivor

`wat/core.wat:736`, `kwargs-defn`'s companion-name machinery:

```clojure
binder-tp  (:wat::core::if has-binder
             (:wat::core::string::concat "<"
               (:wat::core::string::concat
                 (:wat::core::string::join "," …binder names…) ">")))
```

Its own comment, three lines above, states the intent plainly: *"the binder rendered as a `<T,U>`
string SUFFIX — the exact shape `name-tp` already takes from a `<T,U>`-spelled name, so every
downstream `{b}::Kwargs{p}` / `{b}$impl{p}` interpolation is unchanged by construction."*

It feeds `keyword/from-string` at `core.wat:835` (`{b}::Kwargs{p}`) and `:949` (`:{b}$impl{p}`).
**Both doors are walled.** So a `defn` that is BOTH parametric and kwargs mints `:f::Kwargs<T>` and is
refused — correctly, but from inside a macro, with a message about a name the author never wrote.

Measured: **no such `defn` exists in the corpus.** That is the only reason the floor is green, and the
only reason every census returned zero.

⚠ And `core.wat:798` documents the combination as intended and ordinary: *"A kwargs defn MAY be generic
(`:my::svc/start<T>` — **every parametric `defservice`'s auto start/resume is exactly this**)."* Whether
that claim is still true after the last mint stone reshaped `defservice` is itself unverified — but
either the comment is stale, or the path is reachable and simply untaken.

## ★ Why no census I ran could have found it

Every angle census in this campaign was a **runtime** instrument — the wall flipped to log-and-continue,
a floor run, read the screams. Row 7 of the last mint stone said *expect 0*, and it got 0, and that was
**true**.

**A runtime census sees what EXECUTES. It cannot see a minter on a path nothing currently takes.**
`binder-tp` is not dead code and not unreachable; it is *dormant*, waiting on a feature combination the
corpus happens not to use. Every measurement I took was honest and every one was blind to it.

That is a sharper form of the error this arc has repeated: not *"I scoped the check from a list instead
of the rule"* but *"I chose an instrument whose blind spot exactly matched the population I most needed
to see."* `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`.

The static instrument was available and cheap — a grep for `concat "<"` in `wat/` — and I never ran it
because the dynamic one had already answered zero.

## What is owed

- **`binder-tp` emits the binder as a FORM**, not a `<T,U>` string suffix — the same move
  `proto-tp` and `fqdn-tp` already took in `wat/service.wat`. Its companion-name interpolations
  (`{b}::Kwargs{p}`, `{b}$impl{p}`) need the treatment those got.
- **A STATIC census to pair with the dynamic one.** Every `string::concat "<"` / `interpolate` building
  a name in `wat/`, whether or not any current path reaches it. The dynamic census stays — it catches
  what static reading misses — but neither alone is a census.
- **Resolve `core.wat:798`'s claim.** If a parametric `defservice`'s start/resume really is a parametric
  kwargs defn, this is reachable today and the corpus is merely lucky.

## Scope

Out of the prose sweep's scope — R3's remit was comments, and it correctly flagged rather than edited
dense macro internals. Not tracked elsewhere; this NOTE is the record.

Kin: `NOTE-the-sibling-angle-strips-my-census-missed.md` (the Rust-side twin of the same blindness),
`NOTE-the-loader-gate-is-scoped-by-extension.md` (a check scoped to a spelling, not a property).
