# tracing_result changelog

## [v0.1.0]

### Breaking changes

- renamed `TracingResult<T,E>` to `TracedResult<T,E>`
- removed `Event` & `TracingConfig` from public API

### New features

- added alias `TracedOption<T>`
- added `or_info` & `and_info`

### Technical

- improved docs

## [v0.0.3]

### New features

- Core type is now `Traced<T> where T: Try` (so also works with Option)
- added `impl Transform<O> for Traced<T> where T: Try<Output = O>`
- Together this allows `get_foo().or_warn("foo not present").and_then(|f| f.get_bar()).or_warn("bar not present")`
- `TracingResult<T,E>` kept as a type alias

## [v0.0.2]

### New features

- Provide extraction functions (`unwrap`, `ok`, `err`, etc.)

## [v0.0.1]

### New features

- Provide `trait Trace` to `add or_...()` & `and_...()` to `Result`
- Provide `TracingResult` which emits tracing events on `?`

### Known issues / TODOs

- [#1](https://github.com/MusicalNinjaDad/tracing_result/issues/1) provide details of actual callsite
