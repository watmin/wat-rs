# SCORE — the last comma lives in a SYMBOL

**Verdict: Mode A. Every row green, weighed by my own hand on a quiescent tree.**
Brief: `BRIEF-STONE-the-last-comma-lives-in-a-symbol.md`. Rider: sonnet, background, edit-only.

## The scorecard — re-run independently, not read off the report

| # | what | expected | my own measurement |
|---|---|---|---|
| 1★ | the binder is accepted | `(launch :- [S R St Sh Lu] …)` parses; registers bare | ✅ `combine :- [A B]` over `(:t::C)` `5` `"ten"` → **`5`** |
| 2★★ | the comma is refused in a symbol | lex error naming the COMMA | ✅ verbatim below |
| 3★★ | a legal symbol still lexes | `:wat::kernel::Peer'`, `foo/bar`, `a<b` | ✅ all three, same run |
| 4★ | dispatch round-trips | a value comes back | ✅ same as row 1 — `defsurface` + `extend-type` + call |
| 5 | the floor | green | ✅ **`4881 tests run: 4881 passed, 19 skipped`** (78.6s, `scripts/floor.sh`, my invocation) |
| 6 | clippy | 0 under `-D warnings` | ✅ 0 |

**Row 2, verbatim** (`./target/release/wat`, freshly built by me — not the MCP server, which is stale):

```
#wat.core.ReadOutcome/Malformed [#wat.parse/Lex {:message "lex error: lex error at byte 5:
comma inside symbol body retired (arc 109 \"the last comma lives in a symbol\", closing the
arc 271 carve-out): a comma can never appear in a symbol body, at any depth. A multi-param
generic method name like `mk<S,R>` must use the `:-` binder form instead — `(mk :- [S R]
[args] -> ret)` in place of `(mk<S,R> [args] -> ret)`." …}]
```

**Row 3, same run, immediately after** — this is the row that makes row 2 mean anything:

```
#wat.core.ReadOutcome/Forms [((foo/bar))]
#wat.core.ReadOutcome/Forms [((a<b))]
#wat.core.ReadOutcome/Forms [((:wat.kernel/Peer'))]
```

**And the DUAL, which every wall in this arc must preserve** — measured in the same run:

```
(:wat::core::Vector :- [:wat::core::i64] 1, 2, 3)   →  [1 2 3]
(:wat::core::read-string "(a, b, c)")               →  ((a b c))
```

`<` `>` `/` `'` stay legal symbol characters; a comma at angle-depth 0 still BREAKS a symbol as
ordinary EDN whitespace. The wall fires only at `angle_depth > 0`. It refused the comma and
nothing else.

## The census — imposed, not derived

Five instruments lied about one population in this campaign (`294/SEAM.md`). So this one was not
grepped. **I ran the wall itself over every `.wat` and `.wat.bad` in the repository — 1798 files:**

```
SYMBOL-COMMA     0 files                                    ← the population is EXHAUSTED
KEYWORD-COMMA    tests/types/probe_arc232_…wat.bad          ← the intentional negative control
                 docs/arc/2026/05/130-…/complected-2026-05-02/{substrate,test}.wat
                                                            ← archived snapshots inside docs/, not loaded corpus
```

`INCENDIMVS VT VIDEAMVS` — the wall answers *are you inside this surface*, exhaustively, which is
the question a grep never answers. Zero symbol-commas survive. The construct is gone.

## STOP-3 fired, and the rider handled it correctly

A **fifth** site existed: `tests/types/probe_arc232_generic_method_type_application.wat.bad:6`, a
`mk<S,R>` in a `defsurface :features` — the identical shape, sitting in a `.wat.bad` that every
census correctly excludes as a negative fixture.

The first floor went **RED**, and the rider did the right things in the right order: it did **not**
re-run, it captured the failing arm verbatim, and it named the mechanism before touching anything.

```
thread 'probe_arc232_generic_method_type_application::the_callable_turbofish_is_refused_by_the_reader'
panicked at tests/types/probe_arc232_generic_method_type_application.rs:36:5:
must be refused for the COMMA specifically … got: … "comma inside symbol body retired …`mk<S,R>`…
```

**The diagnosis is right and I verified it independently.** That fixture carries TWO violations: a
collateral symbol-comma on line 6 and its actual subject, the keyword turbofish, on line 13. The new
symbol wall fires FIRST, so the fixture's own negative control — `assert!(msg.contains("comma inside
keyword body retired"))` — was being answered by the wrong wall. Migrating only line 6 restores the
fixture to testing its subject; line 13 and the keyword wall are untouched. Confirmed by reading the
fixture and its `.rs` after the change.

★ **The lesson worth keeping: a multi-violation negative fixture tests whichever violation the lexer
reaches first.** Adding a wall silently re-points such a fixture at the new wall, and it still goes
green — just green for the wrong reason. This one went red only because the assertion names the
MECHANISM rather than matching the whole diagnostic.

## Findings — three, and the first is a deviation from the brief

**1. The brief said "teach it the binder INSTEAD"; the door now takes BOTH.** ⚠

`split_method_name_type_params` is unchanged and still parses the inline `name<T>` spelling
(single param — no comma, so the new wall does not reach it). The door dispatches on
`name_raw.contains('<')` first, and only then looks for `:-`. So the method-name slot has **two
spellings for one thing**, which is precisely what this arc's thesis rejects:
*`:-` is the parameterization operator, and it is the ONLY one.*

Measured — the population keeping the second spelling alive is **four sites**:

```
wat-scripts/probes/arc-170/probe-locus1-generic-surface-method.wat:9      (make<T> …
tests/types/probe_arc293_4e_pre_iii_extend_impl_inherits_types.wat:13     (make<T> …
tests/types/probe_arc293_4e_pre_ii_generic_surface_method.wat:15          (make<T> …
tests/types/probe_arc293_4e_pre_ii_generic_surface_method.wat:1           (comment)
```

Keeping it was the *safe* call under an unmeasured population, and the rider's scope was the four
comma-bearing sites. But the number is four, and the follow-on is small: migrate them, delete
`split_method_name_type_params`, and the inline angle form leaves the method-name slot the same way
it left the type slot in ③. **Filed as the next stone.**

**2. The binder peel keeps γ-i's silent discard — now in two slots.** The `filter_map` drops any
vector element that is not a non-reference `Symbol`, so `:- [S 3]` silently yields `[S]`. This is
copied *verbatim* from `src/function/metadata.rs::peel_type_binder`, so it is the house shape, not
a defect the rider introduced. The rider actually tightened one arm — a non-Vector after `:-` now
raises `MalformedDecl` where γ-i silently un-peels the binder. The discard is worth closing at
BOTH sites at once, since a slot with two implementations is two slots.

**3. The retired spelling survives in PROSE, at scale.** ⚠ FM 14's Bucket B.

```
411 comment lines across 139 .wat files      (e.g. wat/cache.wat: `Lru<K,V>`, `Cache<K,V>`, `lru-svc<K,V>`)
591 comment lines across src/ + crates/
```

Not a wall breach — comments do not lex. But it is exactly the leftover class arc 162 was opened
for (*"i wasn't happy seeing left overs in the source"*), and a blind sweep would be **wrong**: some
of these lines RECORD the retirement and must keep the old spelling (Bucket C). This needs FM 14's
A/B/C/D classification, not a codemod fired from a grep count.

## What the rider owned that I could not reconstruct

- **Which reader, and how it confirmed it.** `crates/wat-reader` compiles the corpus; `src/lexer.rs`
  is a bare `pub use`; `crates/wat-edn` is independent and reached only through `wat_edn::write` /
  `#[derive(ToEdn)]` / structured errors — confirmed by walking every `wat_edn::` use site in `src/`.
  The previous strike cost a whole round on exactly this, and the brief asked it to report *how* it
  confirmed, not *that* it did. It did.
- **The RED arm, captured before diagnosis.** See STOP-3 above.
- Its own note that the `.wat.bad` collateral is invisible to any `.wat.bad`-aware census — including
  its own STOP-3 sweep. That is the honest delta.

## Deltas

- Predicted 4 sites; **5** (STOP-3, correctly handled).
- Predicted "teach the door the binder"; delivered "teach the door the binder **as well**" — finding 1.
- No commit, no stash, empty index, 6 files touched. The boundary held.

---

**This closes the comma.** `<K,V>` is unexpressible; the wire escape is deleted; `_`'s language-wide
reservation is gone; and now no comma can enter a symbol body at any depth. Zero survivors in 1798
files. Commas remain EDN whitespace between values, which is all they ever were.
