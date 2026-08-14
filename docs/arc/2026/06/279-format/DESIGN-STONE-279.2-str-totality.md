# DESIGN — STONE 279.2: `str` becomes TOTAL, and the third renderer is retired

> Drawn 2026-08-14 against HEAD `b2136b02`. Probe committed and RED before this document was written.

## Why — 279's own unfinished intent, not a new decision

`279/DESIGN.md:67` specifies the verb it was about to mint:

> *"renders ANY value unquoted (String→itself, i64→digits, bool→`true`/`false`, **…**)"*

What shipped (`runtime.rs`, `eval_str`) is a **five-arm match** — `String | i64 | f64 | bool | u8` —
that raises `TypeMismatch` on everything else. The `…` was never filled in. This stone fills it. No
prior ruling is being overturned; the DESIGN and the disk disagree, and the disk is the one that is
short.

The forcing consumer is `wat.string/join`. `(join "," [1 2 3])` can only render its elements if `str`
is total. With a partial `str`, `join` needs a **bound on a type variable** — a form wat does not
have — or it is a `join` that cannot join numbers. Make `str` total and the bound stops existing:
there is nothing left to constrain `T` by. (Builder, 2026-08-14: *"its either everything must have a
to-str call or we only accept strings — this mixed state shit is crazy."*)

## The finding that shapes the fix: there are THREE renderers, and one is already right

Measured live, 2026-08-14:

| renderer | domain | output for `nil` / `[1 2 3]` / `{:a 1}` |
|---|---|---|
| `eval_str` (5-arm match) | **partial** — raises | n/a — raises on all three |
| `show` → `value/observe.rs::render_value` | total | `()` · `[1, 2, 3]` · `{:a: 1}` |
| **the EDN encoder** (what `println` uses) | **total** | **`nil` · `[1 2 3]` · `{:a 1}`** |

`show` is a **third implementation** that duplicates the EDN encoder in Rust `Debug` shape. `()` is
Rust's unit leaking through a wat verb; `[1, 2, 3]` is comma-space; `{:a: 1}` has a **doubled colon**
and is nobody's syntax. The encoder also already handles the cases that looked hardest — an opaque
renders `#wat-edn.opaque/IOWriter nil`, a function renders `:wat.core/str`.

**So this is not "write a total renderer." It is "point the two wat verbs at the one that is already
correct."** Extirpare's shape: delete the duplicate rather than repair it.

## The ONE contract decision

> **`str` and `show` are the SAME rendering, differing at exactly one place: a TOP-LEVEL
> `Value::String` renders bare under `str` and quoted under `show`. Everywhere else — including a
> string NESTED inside a collection — they are byte-identical.**

`(str "abc")` → `abc` · `(show "abc")` → `"abc"` · `(str ["a"])` → `["a"]` · `(show ["a"])` → `["a"]`

This is Clojure's rule (`str` uses the readable form inside collections) and it is what makes `str`
something other than "show with the quotes stripped." The probe's
`str_keeps_nested_strings_quoted` row is the one that pins it.

## Out of scope — affirmative cuts, each with its reason

- **`ValueSnapshot::of`'s `:rendered` field stays on `render_value`.** `render_value` has exactly one
  external caller (`eval_show`) plus `ValueSnapshot` (`observe.rs:102`, `:135`) — the diagnostics
  renderer, which has its own constraints (a depth cap, and an unbounded-List guard recorded at
  `value/mod.rs:16`). Re-pointing diagnostics is a separate change with a golden-file blast radius
  that has nothing to do with `str`'s totality. `render_value` therefore SURVIVES this stone; only
  `eval_show`'s single call to it goes. Tracked as its own question, not deferred inside this one.
- **Map key ORDER is not normalized.** Builder's ruling, 2026-08-14: *"maps are unordered.... that's
  like... the whole point... we don't do string equality here, we do data equality."* The probe's map
  row uses a single key so it asserts SHAPE without asserting ORDER. If a golden elsewhere pins
  multi-key map rendering, that golden is the defect — see STOP-2.
- **`Seqable` and `wat.string/join` are stone 2.** This stone changes no signature and adds no
  protocol. `join` still takes `Vec<String>` when it lands.
- **The `wat.string/*` namespace rename is stone 3.** 1,617 sites, by codemod.
- **`format`'s macro grammar is untouched.** It gains totality for free — it expands to `str` calls
  (`279/REALIZATIONS.md:12`) — and that is the point, not a side effect to be guarded against.

## The four questions

- **Obvious?** YES — two verbs, one rendering, one documented difference. The alternative ("`str`
  handles five types and raises on the sixth") is what a reader currently cannot predict.
- **Simple?** YES — this DELETES a renderer's only wat-facing consumer rather than adding a fourth.
- **Honest?** YES — `str` stops being typed total (`str<T>`) while behaving partially. That gap is the
  precise defect: the signature already promises what the body refuses to deliver.
- **Good UX?** YES — `(str x)` works for every x a user can write, and the output is the same form
  they already see from `println`, so there is one rendering to learn instead of three.

## The probe (committed BEFORE this design, red at HEAD)

`tests/value/probe_arc279_str_totality.{rs,wat}` — 8 rows, **3 controls green / 5 reds failing**:

```
Summary  8 tests run: 3 passed, 5 failed
  FAIL  str_renders_a_keyword
  FAIL  str_renders_nil_as_nil_not_unit
  FAIL  str_renders_a_vector_in_wat_form_not_rust_debug
  FAIL  str_renders_a_map_in_wat_form
  FAIL  str_keeps_nested_strings_quoted
```

verbatim from the run:

```
:wat::core::str: expected String | i64 | f64 | bool | u8, got wat::core::Vector `[1, 2, 3]`
:wat::core::str: expected String | i64 | f64 | bool | u8, got wat::core::HashMap `{:a: 1}`
```

The controls are load-bearing (R59 `NISI FRANGAS, NIHIL PROBAS`): without a green
`control_str_renders_a_top_level_string_bare`, a red below could mean the harness is broken rather
than that `str` is partial, and the probe would prove nothing.
