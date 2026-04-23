# Arc 031 — sandbox inherits outer config

**Status:** opened 2026-04-23. Cut immediately after arc 030's
arg-order flip (name mode dims …) closed.

**Motivation.** The arg-order flip made one shape honest:
setter-order matches arg-order, capacity-mode commits before dims,
the deftest template emits the setters in that order. That closed
a COHERENCE gap. It also surfaced the next one.

A lab test file looks like this today:

```scheme
(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

(:wat::test::make-deftest :deftest :error 1024
  ((:wat::load-file! "wat/vocab/shared/time.wat")))

(:deftest :my::test body)
```

The outer setters on lines 1-2 commit the test binary's config. Then
`make-deftest` takes `:error 1024` AGAIN, and the inner `deftest`'s
template auto-injects ANOTHER pair of `(set-capacity-mode!)` +
`(set-dims!)` into every sandbox it produces. The same two values
declared three times in three places — once at the outer file, once
at the factory call, once per generated test. Two of those three
are redundant.

The user sets the config at the top of the file. That is the
honest place to say it. The sandbox should inherit. The factory
should drop `mode`/`dims` entirely. The deftest template should
stop auto-injecting the setters.

**Path B** (confirmed 2026-04-23): sandbox inherits outer committed
config by default; inner-form setters still win when present. The
substrate becomes one level simpler; every caller of the test
macros becomes simpler; the double-declaration goes away.

**Relationship to arc 027.** Arc 027 shipped scope inheritance for
the SOURCE LOADER (`:None` scope means "inherit caller's loader").
Arc 031 is the same move for the CONFIG. A sandbox is the same
kind of child-of-caller that a loaded file is; it should inherit
the same environment unless it declares its own.

---

## Semantics

**Before:** `:wat::kernel::run-sandboxed-ast` builds a fresh
`FrozenWorld` from the passed forms. Fresh = default `Config`. Any
inner form that doesn't `set-dims!` / `set-capacity-mode!` runs
against defaults. The deftest template auto-injects both setters
into every sandbox to work around this.

**After:** `:wat::kernel::run-sandboxed-ast` and
`:wat::kernel::run-sandboxed-hermetic-ast` accept the caller's
committed `Config` as the starting point. If the inner forms call
`set-*!`, the sandbox's freeze pipeline writes the new value the
same way it does today (setters in the entry position commit; the
entry file's last setter wins). If the inner forms don't, the
sandbox ends up with the caller's values.

**Deftest / deftest-hermetic:** drop `mode` and `dims` parameters.
Template emits just `,@prelude` and the `:user::main` define. No
auto-injected setters.

**make-deftest / make-deftest-hermetic:** drop `mode` and `dims`
parameters. The factory generates a defmacro whose template calls
the (now simpler) deftest. The factory itself takes only
`(name default-prelude)`.

**The outer file preamble stays exactly as it is today.** Users
write `(:wat::config::set-capacity-mode! :error)` +
`(:wat::config::set-dims! 1024)` at the top of their test file.
Those commit the test binary's config. Every sandbox created
during the tests inherits that commit.

---

## Why this is simple, not easy

*Simple* (Hickey): un-braids the outer config from each sandbox's
inner config. Today they're redundantly entangled — the outer must
match the inner or behavior diverges. After arc 031 they're one
value with one declaration site.

*Not easy*: the substrate change is small (~40 lines in the
sandbox entry), but the callsite sweep is large. Every
`deftest` / `deftest-hermetic` / `make-deftest` / `make-deftest-hermetic`
callsite drops two arguments. Arc 030's flip touched 16 files for a
symmetric 76-swap sweep; arc 031's drop touches the same files with
~76 two-token removals.

Honest work. Not clever work. Tedium is the safety.

---

## Reserved prefixes — unchanged

No new primitives. No new `:wat::*` paths. The existing four
`:wat::test::*` macros keep their names; their signatures
shrink by two parameters each. `:wat::kernel::run-sandboxed-ast`
keeps its name and shape; its default behavior changes from
"fresh config" to "inherit caller's config." Callers who relied
on "fresh config" — none today — would need to pass explicit
`set-*!` forms in the input.

---

## Non-goals

- **No Rust API change.** `Harness::run` and the test-runner path
  continue to work as today. The config-inheritance is a wat-level
  improvement; Rust callers construct `FrozenWorld` with explicit
  config and get explicit config.

- **No per-sandbox config-scope stack.** Sandboxes aren't nested
  config scopes; they're fresh worlds that happen to inherit one
  initial value. A deeply-nested sandbox tree still has just one
  inherited baseline (the outermost committed config).

- **No silent override swap.** If the inner forms set config, the
  sandbox's committed value is what the inner set. Same as today.
  The only change is what happens when the inner forms DON'T set.
