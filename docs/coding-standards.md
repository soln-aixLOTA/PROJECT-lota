1|# LotaBots Coding Standards
2|
3|This document outlines the coding standards to be followed across all LotaBots projects. Adhering to these standards ensures code consistency, readability, and maintainability.
4|
5|## General Principles
6|- **Readability:** Code should be easy to understand and follow. Use meaningful names for variables, functions, and classes.
7|- **Consistency:** Follow the established style and conventions within the project.
8|- **Maintainability:** Design code that is easy to modify and update. Avoid overly complex logic.
9|- **Performance:** Write efficient code, considering performance implications.
10|- **Security:** Follow secure coding practices to prevent vulnerabilities.
11|
12|## Language-Specific Standards
13|
14|### Rust
15|- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/README.html).
16|- Use `cargo fmt` for automatic code formatting.
17|- Run `cargo clippy` to catch common mistakes and improve code quality.
18|- Document public APIs using `///` doc comments.
19|- Handle errors explicitly using `Result` and avoid `panic!` in library code.
20|
21|### Python
22|- Follow [PEP 8](https://peps.python.org/pep-0008/).
23|- Use `flake8` and `pylint` for linting and style checking.
24|- Write docstrings for functions and classes.
25|- Use type hints for improved code clarity and maintainability.
26|
27|### Go
28|- Follow the conventions established by `gofmt`.
29|- Use `go vet` to identify potential errors and issues.
30|- Document public APIs.
31|- Handle errors explicitly.
32|
33|### JavaScript/TypeScript
34|- Follow the [Airbnb JavaScript Style Guide](https://github.com/airbnb/javascript).
35|- Use ESLint for linting.
36|- Write JSDoc comments for functions and classes.
37|
38|## Version Control
39|- Use meaningful commit messages.
40|- Follow a consistent branching strategy (e.g., Gitflow).
41|- Code reviews are mandatory for all changes.
42|
43|## Error Handling
44|- Implement proper error handling to prevent crashes and provide informative error messages.
45|- Use logging to track errors and debug issues. (See `docs/error-handling.md`)
46|
47|## Security Best Practices
48|- Follow secure coding practices as outlined in `docs/security.md`.
49|- Avoid hardcoding sensitive information.
50|- Validate all user inputs. 