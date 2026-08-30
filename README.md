# Tracing Result

An ergonomic wrapper making it easy to emit tracing messages on errors.

## Example

```rust
// Warn and early return
let contents: String = fs::read_to_string(&path).and_warn("reading file")?;

// Debug and continue
let let contents: Option<String> = fs::read_to_string(&path).ok_or_debug("reading file");
```
