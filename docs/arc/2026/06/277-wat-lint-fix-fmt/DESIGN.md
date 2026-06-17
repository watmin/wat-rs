# Arc 277 — the self-hosted toolchain: `wat-lint` → `wat-fix` → `wat-fmt`

> **STATUS: STRIKE-READY (277.1).** Opened 2026-06-17. The language takes care of its own *form*: a
> linter that finds bad structure, a fixer that rewrites it comment-faithfully, a formatter that lays it
> out canonically — all wat, all run on wat, over the whole corpus. The maturity line (#95/#96 Omerta /
> Again We Rise) made total: wat doesn't just self-migrate syntax, it self-*lints* and self-*formats*.

## The trigger

`deporder` (arc 275), the tool built to find bad structure, was written *in* bad structure — a 13-deep
`if`/`=` ladder. The builder: *"what tool is missing to call us out on these bad forms?… is it time to
make our first linter?"* and then the full vision: *"we're making that linter and fixing every piece of
source code — the next tool is the formatter — we're going to use wat-fix to consume wat-lint results and
issue wat-fmt on them."* The bad form propagated from `fix.wat` into `deporder` because nothing caught it.
The linter is the catcher; the fixer is the cure; the formatter is the finish.

## The architecture (grounded — most of it already exists)

The decisive recognition: **a rule is `(form → Vector<edit>)`**, where `edit = (offset, old-len,
new-text)` + metadata (rule-name, severity, message). This is *exactly* the shape `fix.wat` already uses
(`fix-macro-param-types`, `rename-keyword-prefix` each walk the AST and emit edits; `fix-text-apply`
splices them right-to-left so comments + formatting survive byte-identical, located via `ast-span`). The
linter and the fixer are therefore **the same engine in two modes**:

- **`wat-lint` = report mode.** Run the rules, collect the edits-as-**findings** (don't apply); emit
  structured EDN: `{rule, file, line, col, severity, message, fix?}`. Plus *report-only* rules that have
  no mechanical fix — `deporder`'s load-order check (rule-zero), `complectens`'s deftest-length/forward-ref
  (the 004 sketch's five), etc. Findings a human reads, a script greps, or a spell judges (the 004
  two-phase: mechanical findings → optional LLM verdict).
- **`wat-fix` = apply mode.** Run the rules, apply the edits via `fix-text-apply` — the engine that
  already shipped (`fix.wat`, arc 251.5). Comment-faithful, idempotent.
- **`wat-fmt` = the total layout pass.** AST → canonical text (the 003 sketch): two-space indent, parens
  stack at EOL, trailing newline, no trailing whitespace; comment preservation the hard part. A total
  function over the AST, round-trip stable, semantically preserving.
- **The pipeline (the builder's line):** `wat-lint` produces findings → `wat-fix` consumes the fixable
  ones and applies them → `wat-fmt` lays the result out canonically. One corpus runner, three stages.

**Where it lives:** `wat/` (stdlib), run via the CLI — exactly like `deporder` and `fix.wat`, NOT a
separate crate. This is the deporder-reframe of the 003/004 scratch sketches (which predated the
pure-wat-self-hosted-tool proof and assumed `crates/wat-lint/` + `crates/wat-fmt/`). The crate packaging
for *external vendors* is the separate arc-276 direction; the toolchain itself is stdlib wat.

## Grounded prior art (this repo)

- `wat/fix.wat` (34 KB) — the rule-as-edits engine: `fix-text` (parse → locate edits via `ast-span` →
  splice original text), `fix-text-apply` (`(offset, old-len, new-text)` edits), and rules
  (`fix-source`, `fix-macro-param-types`, `rename-keyword-prefix`). **The apply half of `wat-fix`
  already exists.**
- `wat/deporder.wat` (arc 275) — the AST-analyzer: read sources, walk forms (`structural?`, recursive),
  classify, emit findings. **The analyzer half of `wat-lint` already exists; deporder is rule-zero.**
- `wat-scripts/fixes/` — the CLI runners (`fix-macro-param-types.wat`, `rename-kernel-to-spawn.wat`) that
  invoke fix rules over the corpus via `:wat::io::read-file`/`write-file` + the CLI (#96's self-hosted
  runner pattern). **The corpus-runner half already exists.**
- `scratch/2026/05/004-wat-lint/` (LINT-RULES.md) — the linter design: findings shape, rules-as-wat-fns,
  the five complectens rules, the rune-suppression pattern. Reframe: pure-wat, not a crate.
- `scratch/2026/05/003-wat-fmt/` (STYLE-RULES.md) — the formatter design: confirmed style rules (2-space,
  parens-at-EOL, trailing newline, …), AST→text, comment preservation. Reframe: pure-wat.

## Contract decision (the one pinned interface)

**A rule is `(form → Vector<Finding>)`**, where a `Finding` carries the location (`file`, `line`, `col`,
and the `ast-span` offset for the fixer), `rule`, `severity` (`:error`/`:warn`/`:info` — the L1/L2/L3
scale), `message`, and an **optional** `fix` (the `(offset, old-len, new-text)` edit when the rule is
mechanically auto-fixable; absent for report-only rules like load-order or deftest-length). `wat-lint`
emits the findings; `wat-fix` applies the `fix` field of each via the existing `fix-text-apply`; `wat-fmt`
runs after. The findings format IS the contract that binds the three tools — designed fix-consumable from
line one.

## Decomposition

- **277.1 — `wat-lint` framework + the first structural rule (this strike).** A rule registry +
  `lint-source (sources) → Vector<Finding>`; the first rule is the exact bad form that triggered the arc:
  **`nested-if-=-ladder → HashSet/contains?`** (a chain of `(if (= x "a") true (if (= x "b") true …))` is
  a set membership in disguise — the `deporder`/`fix.wat` disease), auto-fixable (emits the edit). Fold
  `deporder`'s load-order in as rule-zero (report-only). Its own deftests + a CLI runner that lints the
  corpus and prints findings.
- **277.2 — promote the existing `fix.wat` rules + grow the set + the corpus fix.** The decisive
  shortcut: **`fix.wat`'s rules are ALREADY lint rules** — `fix-macro-param-types` is an argspec/defmacro
  rule (rule→edits→splice via `argspec-type-edits-walk` → `fix-text-apply`), `rename-keyword-prefix` and
  `fix-source` likewise. Promote them into the registry — each becomes a lint rule that *carries its fix*
  — rather than writing them fresh. Add more structural rules (nested-`if`→`cond`, redundant-`do`) + the
  004 complectens report-only rules. Then run `wat-fix` over **every `.wat` file** and clean the corpus
  (the builder's "fixing every piece of source code"). Each rule probe-gated, grown one at a time
  (the `fix.wat` discipline).
- **277.3 — `wat-fmt`.** The total AST→canonical-text formatter (003's confirmed style rules), comment-
  preserving, round-trip stable; CLI `wat --format`.
- **277.4 — the integrated runner.** `wat-lint` → `wat-fix` → `wat-fmt` as one corpus pass; the build
  gate (lint findings at `:error` fail the build, like `deporder`'s gate).

## Out of scope (rejected, not deferred)

- **No Rust crate** for the toolchain itself — it is stdlib wat run via the CLI (the deporder-reframe).
  External-vendor crate packaging is arc 276's lane, not this one.
- **No new edit engine** — `wat-fix` IS `fix.wat`'s `fix-text-apply`, reused. If a rule needs a richer
  edit, extend `fix.wat`, don't fork it.
- **No config file. Ever. No flavors.** `wat-fmt` has exactly ONE correct behavior, and we *find* it —
  start opinionated, course-correct against taste until it is right — we do not parameterize it. There
  are no toggles, no `.watfmt`, no style options, no "but my team prefers." Someone who wants different
  output **rolls their own formatter** with whatever options they want; the entire burden of a flavor
  sits on the one who wants it, and never as a surface in ours. The rigidity IS the power (the
  single-table doctrine: submit to one brutal constraint and it buys indefinite simplicity). One
  canonical form, *discovered*, not configured. Same stance for `wat-lint`'s rule set and `wat-fix`:
  one correct behavior; rune-based suppression only (conscious, not silent), never a config knob.

## The four questions

- **Obvious?** YES — `(lint-source …) → findings`, `(fix-source …) → rewritten`, `(fmt …) → canonical`;
  three verbs, one rule shape, the same `ast-span` engine underneath.
- **Simple?** YES — one rule type (`form → findings`), lint reports it, fix applies it, fmt is orthogonal;
  no new edit machinery (`fix-text-apply` reused).
- **Honest?** YES — the linter that catches bad form is built so it catches *its own* (the strange-loop
  lesson made structural); findings are mechanical (the verdict is the spell's, opt-in); the fix is
  comment-faithful (no silent corruption).
- **Good UX?** YES — `wat --lint` / `wat --fix` / `wat --format`, run on the corpus; a bad form added
  anywhere fails the gate with the file, line, rule, and the cure. The right path is the only path.
