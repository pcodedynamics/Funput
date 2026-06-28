# Funput benchmarks

Reproducible, open measurements for Funput's speed (a lean Rust core) and
**round-trip coverage** of Vietnamese syllables. Everything here
runs from the `app/` directory with stock `cargo` — no hidden setup, so anyone can
verify the numbers.

> Numbers below were measured on an Apple M-series dev machine; latency is
> machine-dependent, so **re-run on your hardware**. Coverage is deterministic for
> a fixed corpus and Funput revision.

## B1 — Speed (latency / throughput / footprint)

[Criterion](https://github.com/bheisler/criterion.rs) microbenchmarks over a
realistic Vietnamese key sequence (tones, circumflex/horn/breve, the `đ` stroke).

```sh
cargo bench -p funput-core   --bench apply          # per-keystroke transform
cargo bench -p funput-engine --bench process_char   # full engine path (+ boundaries)
cargo bench -p funput-ffi    --bench latency         # end-to-end across the C FFI
```

Criterion writes an interactive HTML summary to
`target/criterion/report/index.html`. Individual benchmark reports live under
their corresponding directories in `target/criterion/`.

| Metric | Result |
|---|---|
| Compose latency (core `apply`) | **~0.23 µs / keystroke** (Telex), ~0.22 µs (VNI) |
| Compose throughput | **> 4 million keystrokes / second** |
| Full engine path (`process_char`, incl. boundary + English-restore) | ~1.5 µs / keystroke |
| **End-to-end across the C FFI** (`process_char` + read composed text back) | **~1.5 µs / keystroke** |
| `size_of::<Engine>` (per-field session state) | **112 bytes** |
| Release FFI shared lib (`libfunput_ffi.dylib`) | ~0.46 MB |

A human types a few keys per second; Funput answers each in **sub-microsecond**
time, i.e. millions of times faster than needed — composition is never the
bottleneck.

### What "end-to-end" measures (and what it doesn't)

The `latency` bench drives the **real C ABI a platform shell uses** for every key:
`funput_process_char` (returns the ~268-byte `FunputResult` POD by value) **plus**
`funput_buffer` (copies the composed text back out to render the marked text). This
is the layer the pure-core bench skips.

Key finding: it lands at **~1.5 µs/keystroke — essentially identical to the engine
alone**, so the FFI boundary (handle indirection + by-value result + UTF-32 copy)
adds negligible cost.

**Honest scope:** this is *keystroke → composed text available to the platform* —
Funput's full contribution. It does **not** include OS keystroke delivery (IMKit /
ibus / fcitx5 / the Windows hook) or the host app's own text render, which are not
Funput's code and cannot be measured reproducibly here. To measure true
keystroke-to-pixels you need per-platform instrumentation (e.g. a high-FPS capture
or an accessibility-API probe); that is out of scope for an automated benchmark.

Footprint check:

```sh
cargo test -p funput-engine -- --nocapture engine_struct_size   # prints size_of::<Engine>
ls -lh target/release/libfunput_ffi.dylib                       # after: cargo build --release -p funput-ffi
```

## B2 — Coverage (round-trip)

For each Vietnamese syllable in a corpus we **encode** it to the Telex/VNI
keystrokes that would produce it, type those back through the real engine, and
check we get the original syllable. A syllable is covered if it reproduces
under **either** tone style (`hòa` and `hoà` are both valid). Smart-restore is off
to isolate pure composition. The corpus is filtered to structurally valid
Vietnamese syllables, so acronyms (AIDS), symbols (Ar/As) and foreign words are
excluded.

```sh
# Default: the committed, MIT-clean sample corpus.
cargo run --release -p funput-cli -- coverage benchmarks/sample.txt --show-mismatches 10

# Headline: a large external word list (downloaded, not vendored — see below).
sh benchmarks/fetch-corpus.sh
cargo run --release -p funput-cli -- coverage benchmarks/.corpus/Viet74K.txt
cargo run --release -p funput-cli -- coverage benchmarks/.corpus/Viet74K.txt --json
```

| Corpus | Syllables | Telex | VNI |
|---|---|---|---|
| Viet74K (full) | 8,977 | **99.33%** | **99.70%** |
| `sample.txt` | 137 | **100%** | **100%** |

The handful of misses are genuine, explainable edges — not silent corruption:
- **Telex digraph collisions** in rare loanword rhymes (`boong`, `boóc`): `oo`→`ô`
  by Telex convention, so they need an alternative typing. VNI has no digraphs, so
  it scores higher here.
- **Malformed corpus entries** with stacked diacritics (e.g. two tones).
- A few `gi`-onset / glide cases.

## Data & licensing

`sample.txt` is our own, MIT-licensed. We **do not vendor** the large word list —
its license differs from Funput's MIT — so `fetch-corpus.sh` downloads
[Viet74K](https://github.com/duyet/vietnamese-wordlist) into `.corpus/`
(gitignored). This keeps the third-party corpus separate from the MIT repository.

## What this number means

This is a corpus **round-trip coverage** measurement, not a real-user accuracy
score: Funput's encoder generates one canonical key sequence per syllable and the
engine checks whether it can reproduce that syllable. It does not measure typing
habits, corrections, or false conversions in non-Vietnamese text. We publish
absolute numbers rather than head-to-head claims against closed-source IMEs.
