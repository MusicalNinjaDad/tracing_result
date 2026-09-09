# tracing_result changelog

## [v0.0.1]

### New features

- Provide `trait Trace` to `add or_...()` & `and_...()` to `Result`
- Provide `TracingResult` which emits tracing events on `?`

### Known issues / TODOs

- [#5](https://github.com/MusicalNinjaDad/tracing_result/issues/5) `impl Trace` for all `Try`-types
- [#5](https://github.com/MusicalNinjaDad/tracing_result/issues/5) allow chaining e.g. `.and_info("").or_warn("")?`
- [#3](https://github.com/MusicalNinjaDad/tracing_result/issues/3) don't duplicate emissions when `?` inside a block returning `TracingResult`
- [#1](https://github.com/MusicalNinjaDad/tracing_result/issues/1) provide details of actual callsite
