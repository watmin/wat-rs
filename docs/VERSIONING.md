# Versioning — `C.S.D`

wat's version is **three independent timestamps**, dotted: **`C.S.D`** — **C**ontract . **S**caffolding . **D**ependencies.

Each positional is an **ISO-8601 UTC instant with every non-digit removed** — `YYYYMMDDHHMMSS`:

```
2020-01-02T03:04:05Z   →   20200102030405
```

So a version looks like:

```
20260628T....Z  .  20260628T....Z  .  20250101T....Z
     ↓                   ↓                  ↓
20260628120000  .  20260628173000  .  20250101000000
   = C                = S                 = D
```

## The three clocks — what each measures

| pos | name | advances when you change… | the question it answers |
|---|---|---|---|
| **C** | **Contract** | **userland-facing tooling** — public functions, constants, argspecs, return values; anything a *user program binds to* | "when did the surface my code depends on last move?" |
| **S** | **Scaffolding** | the stuff that **makes the contract work but isn't bound to** — internals, tests, fixtures, doc updates | "when was the implementation/test/doc behind that surface last touched?" |
| **D** | **Dependencies** | a **Cargo dependency** was added/removed/bumped | "when did our external dependency set last change?" |

A positional is the **timestamp of the most recent change of that kind** — three separate "as-of" dates, not a sequence number.

## The carry-forward rule

Each clock advances **independently**, to "now," only when *its* kind of change is made. **A kind that didn't change keeps its prior value** (it is re-used / carried forward).

- Change a **public contract** → bump **C** to now. You almost always also touched the implementation/tests, so **S** bumps too. You probably did **not** bump a Cargo dep, so **D carries forward** unchanged.
- Change only **internals / tests / docs** → bump **S** only; **C** and **D** carry forward.
- Bump a **Cargo dependency** → bump **D** only; **C** and **S** carry forward (unless the bump also moved the contract or scaffolding).

So the common relation is **S ≥ C** (scaffolding is touched at least as recently as the contract — a contract change rides with a scaffolding change, and scaffolding-only changes push S past C), with **D** floating freely.

## How to read one

- **C** old, **S** recent → the surface is stable; we've been polishing/testing behind it (safe to upgrade — your bindings don't move).
- **C** recent → the userland surface moved; check what you bind to.
- **D** recent → our dependency set shifted; relevant for supply-chain / build reproducibility (see arc 295, signed code).

Monotonic by construction: each positional only ever increases (a later UTC instant is a larger number), so version comparison is plain lexical/numeric per-positional.

## Why not semver

Semver's `MAJOR.MINOR.PATCH` encodes *compatibility intent* as opinion; `C.S.D` encodes *what-changed-and-when* as fact. The three axes here are orthogonal kinds of change (surface / internals / deps), each with its own honest timestamp — you read the version and know exactly which layer moved and on what date, with no judgment call about "is this a minor or a patch." (The C.S.D values are valid integers, so they can sit in a Cargo `version = "C.S.D"` field directly.)
