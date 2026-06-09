# BRIEF — Strike 213.S: the WatAST ↔ EDN bridge (program-over-the-wire serializer)

Read `DESIGN-EXECVE-PROGRAM-OVER-WIRE.md` (same dir) first — esp. the SUPERSEDED note
at the top. This strike builds the serializer that the original design got wrong (it
reached for `watast_to_holon`, the VSA hologram encoder). Arc 257 made wat's AST node
set equal EDN's collection set (`List`/`Vector`/`Map`/`Set` all native), so the correct
serializer is now a straight structural `WatAST ↔ EDN` map — no holon, no tags.

## The work (one paragraph)
Mint `src/wat_edn_bridge.rs`: convert a `Vec<WatAST>` program to a single plain-EDN
frame and back, so a program can be catted over the wire as the EDN it already is. It
is a near-1:1 structural map between `WatAST` and `wat_edn::Value` (alias `OwnedValue`)
— every variant has a twin. Keyword `::`↔`.` translation is already solved; REUSE the
existing codec, do not hand-roll it.

## The mapping (pinned contract)
`WatAST` (post-257) → `wat_edn::OwnedValue`:
- `IntLit(n)`→`Integer(n)`, `FloatLit(x)`→`Float(x)`, `BoolLit(b)`→`Bool(b)`,
  `StringLit(s)`→`String(s)`, `NilLit`→`Nil`
- `Symbol(ident)`→`Symbol(Symbol::new(ident.as_str()))`
- `Keyword(":wat::core::foo")`→`Keyword(..)` — **REUSE the proven keyword encoder**
  (`edn_shim`'s `make_qualified_keyword` / the keyword arm of `value_to_edn`): split the
  wat keyword on its LAST `::` into (ns, name), build via `Keyword::try_ns(ns, name)`
  (which translates `::`→`.`), with the existing fallback for names containing `/`
  (e.g. `:wat::core::char/of`). Do NOT invent a new keyword codec.
- `List(items)`→`List(map)`, `Vector(items)`→`Vector(map)`
- `Map(pairs)`→`Map(pairs mapped)`, `Set(items)`→`Set(map)`

Inverse `OwnedValue → WatAST` (`edn_to_watast`): the structural mirror.
- `Keyword`→`WatAST::Keyword` via `ns_to_wat_path(ns, name)` (`edn_shim.rs:1327` — make it
  `pub(crate)` or reuse) to rebuild `:wat::core::foo`; non-namespaced → `:name`.
- `Tagged`/`Inst`/`Uuid`/`Char`/`BigInt`/`BigDec`: a program AST does not contain these at
  the source level — return a clean `BridgeError` (not a panic) if encountered, so the
  failure is honest. Span is not preserved (EDN carries none); use `Span::unknown()` on
  the way back — fine, freeze re-derives what it needs.

## Public surface
```rust
pub fn watast_to_edn(a: &WatAST) -> wat_edn::OwnedValue
pub fn edn_to_watast(v: &wat_edn::Value) -> Result<WatAST, WatEdnBridgeError>
/// Whole program = ONE frame: a Vector of the top-level forms.
pub fn program_to_edn(forms: &[WatAST]) -> String          // wat_edn::write(Vector[...])
pub fn edn_to_program(frame: &str) -> Result<Vec<WatAST>, WatEdnBridgeError>
```
Register the module in `src/lib.rs` (`pub mod wat_edn_bridge;`). `WatEdnBridgeError` is a
small honest error enum (UnsupportedEdnForm { shape }, KeywordDecode, ParseFrame) — no
`unwrap`/`panic` on the decode path.

## Blast radius
New file `src/wat_edn_bridge.rs`, one `pub mod` line in `lib.rs`, possibly making
`ns_to_wat_path` / `make_qualified_keyword` `pub(crate)` in `edn_shim.rs`. Touch nothing
else. Do NOT touch the holon path, `watast_to_holon`, or the comms wire (that rewiring is
a later 213 strike).

## STOP triggers
1. If a wat keyword cannot round-trip through the reused codec (e.g. a `/`-in-name case the
   existing helper doesn't cover), STOP and report the exact keyword — do not paper it with
   a String fallback that silently changes the form's meaning.
2. If `edn_to_watast` needs to handle a `Tagged`/`Inst`/etc. that a real program AST
   actually contains, STOP — that means the mapping is incomplete, surface it.

## Gate (the kill — this round-trip IS the disconfirming proof)
Add `tests/probe_arc213_program_edn_roundtrip.rs` (its own binary). A program that
exercises every collection + a destructure:
```
(:wat::config::set-capacity-mode! :error)
(:wat::core::defstruct :myapp::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :myapp::sum [p <- :myapp::Pt] -> :wat::core::i64
    (:wat::core::let [{:keys [x y]} p] (:wat::core::i64::+ x y)))
(:wat::core::defn :myapp::tags [] -> :wat::core::i64
    (:wat::core::let [m {:a 1 :b 2}  s #{:x :y :z}]
        (:wat::core::i64::+ (:wat::core::HashMap/length m) (:wat::core::HashSet/length s))))
(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)
```
Assertions:
- `program_to_edn(&forms)` contains **NO** `#wat-edn.holon` substring (PLAIN EDN — the
  whole point); and visibly contains `:wat.core/defn` form keywords and native `{ }` / `#{ }`.
- `edn_to_program(frame)` yields a `Vec<WatAST>` of the same length that
  `startup_from_forms` freezes **Ok** — identical to freezing the directly-parsed forms.
- A focused keyword round-trip unit test: `:wat::core::i64::+`, `:user::main`,
  `:wat::core::char/of` each survive `watast_to_edn`→write→parse→`edn_to_watast` unchanged.
- `cargo build --release` clean; `cargo test --release --workspace --no-run` 0 errors;
  the new probe GREEN. (Skip full-workspace EXECUTION — the arc-213 process tests deadlock;
  the bridge probe is pure/non-forking.)

## Expectations
| what | command | expected |
|---|---|---|
| compiles | `cargo test --release --workspace --no-run` | 0 errors |
| plain EDN, no tags | new probe | pass; frame has no `#wat-edn.holon` |
| program round-trips + freezes | new probe | pass |
| keyword codec | keyword unit test | `::`-deep + `/`-name survive |

Runtime estimate: 30–50 min. Return a SCORE (scorecard rows, the sample frame string,
honest deltas — esp. any keyword edge — files + line counts, STOP hits). Do NOT commit.
