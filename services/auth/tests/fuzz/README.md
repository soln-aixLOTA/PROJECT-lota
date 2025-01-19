# Fuzz Testing Strategy

This directory contains different approaches to fuzz testing the authentication service:

## Directory Structure

```
tests/fuzz/
├── minimal/           # Minimal fuzz testing crate for core types
│   ├── src/          # Core type definitions and property tests
│   └── fuzz/         # Fuzz targets for core types
└── README.md         # This file
```

## Testing Approaches

1. **Minimal Fuzz Testing (`minimal/`)**
   - Pure Rust implementation of core types
   - No external dependencies except serde
   - Property-based testing with proptest
   - Basic fuzz targets for serialization/deserialization

2. **Main Fuzz Testing (`../../fuzz/`)**
   - Full service fuzzing
   - Includes all service features
   - Tests actual endpoints and handlers

## Running Tests

### Property Tests
```bash
# Run property tests for minimal crate
cd minimal && cargo test

# Run all property tests
cargo test
```

### Fuzz Tests
```bash
# Run minimal fuzz tests
cd minimal && cargo fuzz run fuzz_register

# Run main fuzz tests
cargo fuzz run fuzz_register
```

## Known Issues

- Sanitizer-based fuzzing (AddressSanitizer) may cause segmentation faults in containerized environments
- Some dependencies (e.g., ring) are incompatible with LLVM sanitizers
- Workaround: Use property-based testing for core functionality

## Adding New Tests

1. For core type testing:
   - Add new property tests in `minimal/src/lib.rs`
   - Add new fuzz targets in `minimal/fuzz/fuzz_targets/`

2. For service testing:
   - Add new property tests in `tests/`
   - Add new fuzz targets in `fuzz/fuzz_targets/`
