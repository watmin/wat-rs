# NOTE (arc 300) — reader parity: the `'` prime bug + unknown-tag eagerness — ON DECK AFTER STONE C

**Filed 2026-07-03. A POINTER + a decided direction, queued.** Surfaced by the builder hand-probing
`#foo/bar {:kw 1 :kw' 2}` while Stone B built. Two `clojure.edn`-parity gaps of OPPOSITE polarity, both
grounded against the running oracle this session. Queued **after Stone C (rational arithmetic)**, before
the 278 rete NEXT-2 Clara matrix. `AD ORACVLVM NON AD LIBRVM` — clj is the oracle; non-parity is illegal.

## Grounded (clj 1.12.4 vs wat-edn, run 2026-07-03)

```
                          clojure.edn (oracle)          wat-edn
:kw'                      OK  :kw'                       ERR  UnexpectedByte(39)     ; ' = 0x27
{:kw 1 :kw' 2}           OK  {:kw 1, :kw' 2}            ERR  (same — the ')
#foo/bar 2               ERR "No reader function..."    OK   Tagged(foo/bar, 2)
```

## Gap 1 — the `'` prime: `clj:OK / wat:ERR` = a wat BUG (too strict). No decision — just fix.

Same class as `:/`, `##Inf`, Unicode symbols, ratios: the **EDN spec DOC** omits `'` from the symbol
grammar, but **`clojure.edn` (the oracle) accepts it** (`foo'` / `:kw'` is idiomatic "prime"). wat
implemented the doc → non-parity. Fix: `crates/wat-edn/src/lexer.rs` must admit `'` (0x27) in
symbol/keyword bodies. Grow the ward corpus with `:kw'` / `foo'` (it was silent — that is why the builder
found this by hand, not the ward).

## Gap 2 — unknown-tag eagerness: `clj:ERR / wat:OK` = wat too LENIENT. Decided: reject-by-default.

`clojure.edn` rejects `#foo/bar …` because **without the registered reader it does not know the shape
contract the tag promises** — it refuses rather than fabricate. wat's eager-accept **guesses that any
payload satisfies the tag** — accepting requirements-on-shape it has no right to accept (builder:
*"the user could have expected requirements on shape and we just happily accept them… feels wrong"*).

**Backfire:** shape contract lost (`#foo/bar {:kw 1 :garbage 9}` and `#foo/bar "x"` accepted identically);
typo-masking (`#myapp/Persn` reads as a valid generic `Tagged`; clj catches it); asymmetric round-trip
(wat emits tagged EDN clj then refuses — breaks the one-reader/parity thesis).

**The ward exemption was the daemon in "superset" clothing** — `#myapp/Foo {:x 1}` is exempted as a
"spec-blessed 'read any and all edn' superset," but that leans on the DOC (the spec's *optional* `:default`
handler), not the ORACLE (`clojure.edn`'s actual default = reject). Flip the exemption from `wat:OK` to
**must-be-ERR**.

### Why reject-by-default is NOT lossy — defrecord ships the batteries

**Builder (2026-07-03): *"when we declare a record we install tags for the user… our defrecord comes
batteries included."*** `defrecord` auto-installs the record's `#ns/Name` tag reader (shape contract and
all) — exactly clj's model. So legitimate tags read via **declaration = registration**; only *undeclared*
tags error. This resolves the "what depends on eager reading?" caveat: **nothing should** — the registered
path (built-ins `#inst`/`#uuid` + defrecord-installed record tags) covers every legitimate tag; eager
accept-anything is pure liability. `Value::Tagged` stays for REGISTERED handling; eager-accept is deleted.

## The strike (when it comes up — after C)

1. **Prime** (bug): wat-edn lexer admits `'` in symbol/keyword bodies. + corpus rows.
2. **Tags** (parity): unknown/unregistered tag → clean reader ERROR (match `clojure.edn`); keep the
   registered path (built-ins + defrecord-installed readers). Flip the ward's `#myapp/Foo` exemption to a
   required-ERR row. Confirm defrecord's tag-install path is the registration hook.
3. Grow the clj-oracle corpus until it stops finding divergences (loop-until-dry) — these two prove it was
   incomplete.

## Refs
- The clj-oracle differential ward: `crates/wat-edn/tests/clj_oracle_parity.rs` (+ `clj_oracle/` corpus,
  golden, regen.clj). `#myapp/Foo` is its one standing exemption — to flip.
- `crates/wat-edn/src/{lexer,parser,value}.rs` — the prime + tag rooms.
- defrecord tag-installation path — to locate when the strike is drawn (the registration hook).
