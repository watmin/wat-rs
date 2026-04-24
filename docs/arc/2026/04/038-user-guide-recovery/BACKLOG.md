# Arc 038 — BACKLOG

Status markers:
- **ready** — gap is small, region known, edit is obvious in shape.
- **obvious in shape** — clear once the prior slice lands.
- **foggy** — needs a re-read of the touched region first.

Implementation slices group gaps by USER-GUIDE.md section to minimize re-touch of the same lines. Each slice is a small set of `Edit` calls; never a full-file rewrite.

---

## Slice 1 — §1 Setup overhaul

**Status: ready.**

Highest-impact section. Three arcs collapse here:

- **arc 028** — load forms moved from `:wat::core::load!` (single form taking iface keywords) to root-level `:wat::load-file!` / `:wat::load-string!` / `:wat::digest-load-file!` / `:wat::signed-load-file!` (six honest forms). Eval forms similarly: `:wat::eval-edn!` / `:wat::eval-file!` / `:wat::eval-digest-string!` / `:wat::eval-signed-string!`.
- **arc 027** — `loader: "wat"` option on `wat::main!` / `wat::test!`. The current §1 mentions multi-file trees but predates the explicit loader-option syntax landing. Update the example to show the option.
- **arc 037** — multi-tier dim-router. `set-dims!` retired; replaced by `set-dim-router!`. The current §1 minimum entry shows `(set-dims! 10000)`; update to either remove (zero-config default) or show `set-dim-router!` override.

Affected subsections: "Setup — your first wat application crate", "Multi-file wat trees — entry vs. library", "Capability boundary — the Loader", "What the macro actually emits".

## Slice 2 — §6 Algebra forms

**Status: ready.**

Five additions to the algebra surface enumeration:

- **arc 023** — `coincident?` joins `presence?` in measurements (the dual predicate; cosine `(1-c) < noise-floor` direction).
- **arc 026** — `eval-coincident?` family (4 forms: bare + edn + digest + signed). Goes after `coincident?`.
- **arc 032** — `:wat::holon::BundleResult` typealias documented as the canonical Result return for `Bundle`.
- **arc 033** — `:wat::holon::Holons` typealias documented as the canonical `Vec<HolonAST>` shape (the type used by every `encode-*-holons` vocab function in the lab).
- **arc 034** — `ReciprocalLog` joins idioms section. N=2 is the smallest reciprocal pair; pattern is `(ReciprocalLog N value) → (Log value (/ 1 N) N)`.

Affected subsections: "The three measurements" (extend to four if we count coincident? — or recompute the count), "The ten wat-written idioms" (extend to eleven).

## Slice 3 — §10 Caching paths

**Status: ready.**

- **arc 036** — wat-lru namespace promoted from `:user::wat::std::lru::*` to `:wat::lru::*`. Path strings update across the section. Examples remain shape-stable; only namespace prefixes shift.

Affected subsections: "Caching — LocalCache vs CacheService", "LocalCache — per-program hot cache", "CacheService — shared across programs".

## Slice 4 — §13 Testing updates

**Status: ready.**

Two arcs:

- **arc 031** — sandbox inherits the caller's Config. Test macros (`deftest` / `make-deftest` etc.) no longer take `mode` + `dims` parameters. Outer preamble carries them; tests inherit.
- **arc 029** — `make-deftest` factory documented as the idiomatic shape for test files with shared loads/helpers. Default-prelude carries common setup; bare-name `(deftest :name ...)` calls per test.

Affected subsections: "Convention", "Writing a test — `deftest`", "Fork/sandbox tests", "When to use hermetic". Most existing prose stays; signatures shift.

## Slice 5 — §4 Functions (macros + length)

**Status: ready.**

Three additions:

- **arc 029** — nested quasiquote `,,` deep-splice mention in macro section. Used by `make-deftest` factory; useful pattern documented for users writing factories.
- **arc 030** — `:wat::core::macroexpand` and `:wat::core::macroexpand-1` primitives. Add a "Debugging macros" subsection.
- **arc 035** — `:wat::core::length` is polymorphic over HashMap/HashSet/Vec (matches arc 025's container surface).

Affected subsections: existing sections on `lambda`, possibly add a sub-section for macros (the current 467a3d4 doc may not have one).

## Slice 6 — Container surface (new subsection or near §11)

**Status: obvious in shape after Slice 5.**

- **arc 025** — `get` / `assoc` / `conj` / `contains?` are polymorphic over HashMap/HashSet/Vec, with semantically-forced illegal cells:
  - `assoc` on HashSet is illegal (use `conj`).
  - `conj` on HashMap is illegal (use `assoc`).
  - All four work on Vec (positional get; assoc-by-index; conj appends; contains?-by-index).

Drop the table from arc 025's INSCRIPTION into a new "Containers" subsection. May fit under §6 (algebra primitives) or extend §11 (stdio is too narrow); decide at slice time. Low-risk: pure addition.

## Slice 7 — §1 / §12 Sigma defaults + dim-router knobs

**Status: foggy until Slice 1 lands** (depends on §1's structure post-Slice-1).

- **arc 024** — `presence-sigma` and `coincident-sigma` config knobs. Default sigmas are functions of dims (`presence_sigma(d) = floor(sqrt(d)/2) - 1`, `coincident_sigma = 1`). User overrides via wat lambdas.
- **arc 037** — explicit `set-dim-router!` example with a custom router function.

Brief subsection in §1 overrides; full detail in arc 024/037 INSCRIPTIONs.

## Slice 8 — Appendix forms table refresh

**Status: foggy until Slices 1-7 land.**

The appendix is the cumulative form-by-form reference. Audit it last so it reflects the slice sequence's additions:

- New algebra: `coincident?`, `eval-coincident?` family, `ReciprocalLog`, `BundleResult`, `Holons`.
- New core: `macroexpand`, `macroexpand-1`, polymorphic `length` / `get` / `assoc` / `conj` / `contains?`.
- New config: `set-dim-router!`, `presence-sigma`, `coincident-sigma`. Retired: `set-dims!`.
- New load/eval root forms (arc 028).
- New caching paths (arc 036).

Audit, not rewrite. Add rows; correct rows that drifted; never re-table the whole thing.

## Slice 9 — INSCRIPTION + cross-references

**Status: obvious in shape.**

Standard close: `INSCRIPTION.md` summarizing what shipped per slice, with commit refs. Update `docs/README.md` arc index. 058 FOUNDATION-CHANGELOG row in the lab repo. Update `wat-rs/README.md` "What's next" if it carried a USER-GUIDE-related promise.

---

## Cross-cutting

- **Verification after each slice:** `wc -l docs/USER-GUIDE.md` (should grow modestly per slice; sudden multi-thousand-line jumps are a smell), `grep -nE '^#{1,3} '` for header sanity, optional spot-read of the touched region.
- **Commit per slice.** Keeps the audit trail clean and gives us cheap rollback if any single edit re-introduces a poison.
- **Push per commit.** Standing rule: gitlog is our public stream of consciousness.

## Sub-fogs

- **Is there a §6 idioms count?** If yes, "ten" → "eleven" with ReciprocalLog. Confirm at Slice 2 read time.
- **Does §13 already mention `make-deftest`?** Possibly partially via arc 022 era. Confirm at Slice 4 read time.
- **Does §11 (stdio) reference Console paths that shifted?** Likely no — Console paths are `:wat::std::*` which didn't move. Confirm at Slice 8 read time.

## Out-of-scope reminders

- Other docs in commit `5b5fad8` are NOT touched here. If a future read flags any as poisoned, that gets its own arc (039+).
- We do not restructure or reorder sections. We extend.
- We do not re-derive any arc's claims. INSCRIPTIONs are source of truth; this arc cites them.
