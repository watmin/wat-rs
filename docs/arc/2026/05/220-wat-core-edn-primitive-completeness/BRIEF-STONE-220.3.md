# BRIEF — Arc 220 Stone 220.3 — `'` reader macro (form-start quote)

**Stone scope (sonnet portion):** add `'` reader macro at form-start position; rewrites to existing `(:wat::core::quote X)` special form per `src/runtime.rs:4450` + `src/special_forms.rs:243`. Mechanical copy of the backtick (`` ` ``) precedent at `src/lexer.rs:281-292` + `src/parser.rs:286`.
**Type:** Sonnet Mode A.
**Time budget:** 15-25 min target; 35 min STOP.
**Depends on:** Stone 220.2 (`dd84fcf` — Char shipped).
**Calibration:** 12 stones at-or-below band; this is smallest yet (4 sites). Band 15-25.
**Unblocks:** Slice 4 (`:wat::core::List<T>` — consumes `'(1 2 3)` syntax in tests).

## User direction 2026-05-22

> *"we have (:wat::core::quote ...) but we need single ' in the system for edn"*
> *"'(1 2 3) and (defn foo' [] ...) are both legal in clojure — and they need to be legal in wat"*

Clojure handles both lexically by position. wat already handles `foo'` (arc 171 keyword-body discriminator) — this stone adds `'` at form-start as the missing piece.

## Pre-flight verified (orchestrator-grep'd 2026-05-22)

### Perfect precedent — backtick reader macro

**Token enum addition** at `src/lexer.rs:113-115`:

```rust
/// Quasiquote `` ` `` reader macro. Parser rewrites to
/// `(:wat::core::quasiquote X)` wrapping the following form.
Quasiquote,
```

**Lexer top-level emit** at `src/lexer.rs:281-292`:

```rust
// Quasiquote reader macros — `` ` ``, `~`, `~@`.
b'`' => {
    i += 1;
    tokens.push(SpannedToken { token: Token::Quasiquote, span: span_at(i) });
}
```

(Note: the exact code at :281-292 includes the `~` and `~@` cases too; I've shown only the backtick branch for clarity. Read the actual block to see the full match arm structure.)

**Parser dispatch** at `src/parser.rs:286`:

```rust
Token::Quasiquote => self.parse_reader_macro(":wat::core::quasiquote", span),
```

**`parse_reader_macro` helper** at `src/parser.rs:306+`: synthesizes `(<head> X)` wrap-list form. Reusable across all reader macros.

**Test pattern** at `src/parser.rs:810-870`:

```rust
fn quasiquote_wraps_following_form() {
    assert_eq!(
        parse_one("`foo"),
        list(vec![kw(":wat::core::quasiquote"), sym("foo")])
    );
}

fn quasiquote_over_list() {
    // `(a b c) → (:wat::core::quasiquote (a b c))
    assert_eq!(
        parse_one("`(a b c)"),
        list(vec![
            kw(":wat::core::quasiquote"),
            list(vec![sym("a"), sym("b"), sym("c")]),
        ])
    );
}
```

### Existing `:wat::core::quote` special form (downstream of this stone)

- `src/special_forms.rs:243` — `insert(&mut m, ":wat::core::quote", &["<expr>"]);` (registers the special form)
- `src/runtime.rs:4450` — `":wat::core::quote" => eval_quote(args, list_span),` (eval dispatch)
- `src/runtime.rs:9871` — `/// (:wat::core::quote <expr>) — capture an unevaluated AST.` (doc)

The eval path EXISTS. This stone only adds the `'` SYNTAX SUGAR that translates to this existing form at parse time.

### Lexical position discipline (arc 171 preservation)

`'` has two legal positions in wat (both legal in Clojure):

| Position | Meaning | Where in lexer |
|---|---|---|
| Form-start (top-level token boundary) | Reader macro → `(:wat::core::quote X)` | NEW: this stone adds case |
| Inside keyword body (after leading `:`) | Arc 171 discriminator separator (e.g. `:wat::core::op'2`) | EXISTING: handled by `lex_keyword` body absorption |

The lexer's existing keyword-body handling absorbs `'` inside `lex_keyword` — those bytes never reach the top-level token-emit dispatch. The new top-level `'` case only fires outside keyword body context. **Both stay independently working** per Clojure precedent.

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope (sonnet)

Execute 4 mechanical edits + tests:

### 1. Add `Token::Quote` variant

`src/lexer.rs:~115` — add new variant after `Quasiquote`:

```rust
/// Quote `'` reader macro. Parser rewrites to
/// `(:wat::core::quote X)` wrapping the following form.
/// Arc 220 Slice 3 (Clojure precedent — `'(1 2 3)` form-start).
/// Distinct from arc 171's keyword-body `'` discriminator (which is
/// absorbed by `lex_keyword` and never reaches this top-level token).
Quote,
```

### 2. Lexer top-level emit on `b'\''`

`src/lexer.rs:~281-292` area — add a new branch alongside the backtick / unquote / unquote-splicing cases:

```rust
b'\'' => {
    i += 1;
    tokens.push(SpannedToken { token: Token::Quote, span: span_at(i) });
}
```

Verify (via existing tests at `src/lexer.rs:881+` arc 171 section) that `:foo'2` keyword-body cases still tokenize as a single Keyword. The arc 171 absorbtion happens inside `lex_keyword` BEFORE the top-level dispatch sees `'`.

### 3. Parser dispatch

`src/parser.rs:286` area — add new branch alongside `Token::Quasiquote`:

```rust
Token::Quote => self.parse_reader_macro(":wat::core::quote", span),
```

### 4. Tests — mirror the quasiquote test pattern

`src/parser.rs:~810` area (next to `quasiquote_wraps_following_form`, `quasiquote_over_list`, etc.) — add quote counterparts:

```rust
#[test]
fn quote_wraps_following_form() {
    assert_eq!(
        parse_one("'foo"),
        list(vec![kw(":wat::core::quote"), sym("foo")])
    );
}

#[test]
fn quote_over_list() {
    // '(a b c) → (:wat::core::quote (a b c))
    assert_eq!(
        parse_one("'(a b c)"),
        list(vec![
            kw(":wat::core::quote"),
            list(vec![sym("a"), sym("b"), sym("c")]),
        ])
    );
}

#[test]
fn quote_does_not_disturb_keyword_body_apostrophe() {
    // Arc 171 invariant: `'` inside keyword body stays absorbed.
    // `:foo'2` is a single keyword token, NOT a quote of `(:foo 2)`.
    assert_eq!(
        parse_one(":wat::core::op'2"),
        kw(":wat::core::op'2")
    );
}
```

### Verification (must run before SCORE)

1. `cargo build --release` — workspace clean
2. `cargo test --release --lib -p wat` — PASS with new test count (delta: +3 from new quote tests)
3. `cargo test --release -p wat-edn` — 344/344 (unchanged)
4. `cargo clippy --release --all-targets -p wat-edn -- -D warnings` — 0 warnings (wat-edn untouched; matches arc 218 stone discipline)
5. **wat-clippy intentionally NOT gated** — pre-existing arc 170 backlog stays visible per user direction 2026-05-22 ("constant reminder")
6. Quick lex_keyword regression check — `cargo test --release --lib -p wat keyword_apostrophe` (the arc 171 test family at `src/lexer.rs:881+`) PASSes unchanged

**Interop handshakes NOT required for this stone** — parser-only change; no wat-edn or interop-tests files touched. (If you DO touch any interop-tests files for some reason, run the 4 handshakes per `feedback_wat_edn_touch_runs_interop_tests`.)

**Write `docs/arc/2026/05/220-wat-core-edn-primitive-completeness/SCORE-STONE-220.3.md`** mirroring SCORE-STONE-220.2 shape.

## STOP triggers

- **STOP-1 (arc 171 keyword-body `'` test breaks):** if any `keyword_apostrophe_*` test fails, the top-level `'` case is consuming inside-keyword-body bytes — fix by ensuring lex_keyword absorbs `'` BEFORE the top-level dispatch
- **STOP-2 (unexpected parser test breaks):** if existing parser tests using `'` (in test fixtures, source comments, etc.) break, surface the unexpected `'`-consumer
- **STOP-3 (35 min elapsed):** wall-clock STOP

## Out-of-scope

- `:wat::core::List<T>` — Slice 4
- INSCRIPTION / USER-GUIDE — Slice 5
- Any wat-edn modifications
- New runes (no candidates this stone)
- New public surface beyond the `'` reader macro syntax
