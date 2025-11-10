# Rust-Python Integration with PyO3 and Maturin

## Overview

**PyO3** is a Rust library that enables creating Python extensions in Rust. It provides bindings to write native Python modules with Rust's performance and safety guarantees.

**Maturin** is a build tool that simplifies the process of building and publishing PyO3-based Python packages.

## Project Setup

**Cargo.toml configuration:**
```toml
[package]
name = "mybpe"
version = "0.1.0"
edition = "2021"

[lib]
name = "mybpe"
crate-type = ["cdylib", "rlib"]

[dependencies]
pyo3 = { version = "0.23.3", features = ["extension-module"] }
```

**Key points:**
- `crate-type = ["cdylib"]` creates a dynamic library that Python can import
- The `extension-module` feature optimizes for Python module usage
- **Important:** With `extension-module`, you CANNOT use `cargo run` or build standalone Rust binaries
- The code can only be used as a Python module (via maturin)
- Adding `"rlib"` to crate-type won't help - linking will fail because Python C API symbols aren't available
- If you need both Python module AND Rust binary, you need separate build configurations without `extension-module`

## PyO3 Attributes

**Exposing Rust to Python:**

**`#[pyclass]`** - Makes a Rust struct available in Python:
```rust
#[pyclass]
pub struct Tokenizer {
    pub merges: StdHashMap<Pair, TokenId>,
    pub pattern: String,
    compiled_pattern: Regex,
}
```

**`#[pymethods]`** - Exposes methods to Python:
```rust
#[pymethods]
impl Tokenizer {
    #[new]
    pub fn new() -> Self {
        // Constructor called when Python does: Tokenizer()
    }

    pub fn train(&mut self, text: &str, vocab_size: u32) {
        // Callable from Python: tokenizer.train(text, 256)
    }

    pub fn encode(&self, text: &str) -> Vec<TokenId> {
        // Returns Python list: tokenizer.encode("hello")
    }
}
```

**`#[pymodule]`** - Defines the Python module:
```rust
#[pymodule]
fn mybpe(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Tokenizer>()?;
    Ok(())
}
```

## Important Patterns

**Separating public API from internal helpers:**
```rust
// Internal helpers - NOT exposed to Python
impl Tokenizer {
    fn merge_pair(&self, tokens: &[TokenId], pair: Pair, new_id: TokenId) -> Vec<TokenId> {
        // Private Rust method
    }
}

// Public Python API
#[pymethods]
impl Tokenizer {
    pub fn encode(&self, text: &str) -> Vec<TokenId> {
        // Calls internal helpers but exposed to Python
    }
}
```

**Only put Python-callable methods inside `#[pymethods]`**. Internal Rust helpers go in a separate `impl` block.

## Type Conversions

**Automatic conversions PyO3 handles:**
- `&str` ↔ Python `str`
- `String` ↔ Python `str`
- `Vec<T>` ↔ Python `list`
- `u32`, `i64`, etc. ↔ Python `int`
- `f64` ↔ Python `float`
- `bool` ↔ Python `bool`

**Python iterator handling:**
```rust
pub fn train_from_iterator(
    &mut self,
    py: Python<'_>,
    iterator: &Bound<'_, PyAny>,
    vocab_size: u32,
) -> PyResult<()> {
    for item in iterator.iter()? {
        let text: String = item?.extract()?;
        // Process text
    }
    Ok(())
}
```

**Key points:**
- `Python<'_>` provides GIL (Global Interpreter Lock) context
- `Bound<'_, PyAny>` represents a Python object
- `.iter()?` converts Python iterator to Rust iterator
- `.extract()` converts Python objects to Rust types
- `PyResult<T>` handles Python exceptions

## Building and Testing

**Development workflow with `uv` and `maturin`:**

```bash
# Build and install in development mode
uv run maturin develop

# After changes, rebuild
uv run maturin develop

# Run Python tests
uv run python test_mybpe.py
uv run pytest tests/test_rustbpe.py -v -s
```

**Development mode benefits:**
- Installs module directly in current Python environment
- Changes require rebuild but no reinstall
- Fast iteration cycle

## Common Pitfalls

**Error: Trait bound not satisfied for Python arguments**
- **Cause:** Trying to expose methods with Rust-only types in `#[pymethods]`
- **Solution:** Keep those methods in separate `impl` block without `#[pymethods]`

**Error: Module not found after building**
- **Cause:** Module not installed in current Python environment
- **Solution:** Run `uv run maturin develop`

**Error: cfg_attr issues with conditional compilation**
- **Cause:** Trying to conditionally enable PyO3 features
- **Solution:** For simple cases, just always use PyO3 attributes (simpler)

**Ownership issues with `&Vec<T>` parameters**
- **Better:** Use `&[T]` (slice) instead of `&Vec<T>`
- More idiomatic Rust and works with any contiguous sequence

**Error: Symbol(s) not found for architecture (linking errors with Python C API)**
- **Cause:** Using `extension-module` feature with `cargo run` or trying to build a standalone binary
- **Error example:** `"_PyModule_Create2", "_Py_Dealloc", etc. symbol(s) not found`
- **Solution:** Only use as Python module via `uv run maturin develop`, NOT with `cargo run`
- **Why:** The `extension-module` feature tells PyO3 to NOT link against Python (Python will load it dynamically)
- This is correct for Python extensions but prevents standalone Rust binaries

## Key Concepts

**The GIL (Global Interpreter Lock):**
- Python's global lock ensuring thread safety
- PyO3 manages GIL acquisition/release automatically
- `Python<'_>` parameter provides GIL token when needed

**Rust safety guarantees in Python:**
- No segfaults from memory errors
- No data races
- Type safety at compile time
- Performance of native code with safety of Rust

**Why Rust for Python extensions:**
- 10-100x faster than pure Python for CPU-bound tasks
- Memory safety without garbage collection overhead
- Fearless concurrency (can release GIL safely)
- Rich type system catches bugs at compile time

## Summary

PyO3 + Maturin provides a seamless workflow for writing high-performance Python extensions in Rust. The key is understanding which code goes in `#[pymethods]` (Python API) versus regular `impl` blocks (internal helpers), and how PyO3 automatically handles type conversions between Rust and Python.
