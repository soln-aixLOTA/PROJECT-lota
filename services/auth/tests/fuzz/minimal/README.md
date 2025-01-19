# Minimal Fuzz Testing Crate

This is a minimal fuzz testing crate for the LotaBots authentication service. It focuses on testing core data structures and serialization/deserialization without the complexity of the full service dependencies.

## Structure

```
minimal/
├── src/              # Core type definitions and property tests
├── fuzz/             # Fuzz targets
├── Cargo.toml        # Crate configuration
└── README.md         # This file
```

## Features

- Pure Rust implementation of core types
- Property-based testing with proptest
- Fuzz testing of serialization/deserialization
- No external dependencies except serde

## Running Tests

### Property Tests
```bash
cargo test
```

### Fuzz Tests
```bash
cargo fuzz run fuzz_register
```

## Adding Tests

1. Add property tests in `src/lib.rs`:
   ```rust
   #[cfg(test)]
   mod tests {
       proptest! {
           #[test]
           fn test_name(input in strategy()) {
               // Test logic
           }
       }
   }
   ```

2. Add fuzz targets in `fuzz/fuzz_targets/`:
   ```rust
   fuzz_target!(|data: &[u8]| {
       // Fuzz test logic
   });
   ```

## Test Coverage

Current test coverage includes:
- Username validation
- Email format validation
- Password requirements
- JSON serialization/deserialization
- Invalid input handling
