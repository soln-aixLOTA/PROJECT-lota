use std::process::{Command, Stdio};
use std::time::Duration;
use std::io::Write;
use anyhow::{Result, anyhow};

const TIMEOUT_SECONDS: u64 = 5;
const ALLOWED_IMPORTS: [&str; 4] = ["math", "sympy", "numpy", "scipy"];

/// Python code verifier that safely executes mathematical code
pub struct PythonVerifier {
    python_path: String,
}

impl PythonVerifier {
    pub fn new(python_path: String) -> Self {
        Self { python_path }
    }

    /// Verify if the Python code executes successfully
    pub fn verify(&self, code: &str) -> Result<bool> {
        // Validate imports
        if !self.validate_imports(code) {
            return Ok(false);
        }

        // Prepare the code with safety wrapper
        let wrapped_code = self.wrap_code(code);

        // Execute the code with timeout
        let output = Command::new(&self.python_path)
            .arg("-c")
            .arg(&wrapped_code)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .timeout(Duration::from_secs(TIMEOUT_SECONDS))
            .output()?;

        // Check execution status
        if !output.status.success() {
            return Ok(false);
        }

        // Validate output format
        self.validate_output(&String::from_utf8_lossy(&output.stdout))
    }

    /// Validate that only allowed imports are used
    fn validate_imports(&self, code: &str) -> bool {
        let imports = code.lines()
            .filter(|line| line.trim().starts_with("import") || line.trim().starts_with("from"));

        for import in imports {
            let allowed = ALLOWED_IMPORTS.iter()
                .any(|&allowed| import.contains(allowed));

            if !allowed {
                return false;
            }
        }
        true
    }

    /// Wrap code with safety measures and error handling
    fn wrap_code(&self, code: &str) -> String {
        format!(
            r#"
import sys
from io import StringIO
import contextlib

# Redirect stdout
output = StringIO()
with contextlib.redirect_stdout(output):
    try:
        {}
    except Exception as e:
        print(f"Error: {{str(e)}}", file=sys.stderr)
        sys.exit(1)

print(output.getvalue())
            "#,
            code
        )
    }

    /// Validate that the output meets expected format
    fn validate_output(&self, output: &str) -> Result<bool> {
        // Basic validation - ensure output exists and isn't error message
        if output.trim().is_empty() || output.contains("Error:") {
            return Ok(false);
        }

        // Could add more specific validation here based on expected output format
        Ok(true)
    }
}

impl super::CodeVerifier for PythonVerifier {
    fn verify_code(&self, code: &str) -> bool {
        self.verify(code).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_math_code() {
        let verifier = PythonVerifier::new("python3".to_string());
        let code = r#"
import math
x = math.sqrt(16)
print(x)
"#;
        assert!(verifier.verify(code).unwrap());
    }

    #[test]
    fn test_invalid_import() {
        let verifier = PythonVerifier::new("python3".to_string());
        let code = r#"
import os  # Not allowed
print("test")
"#;
        assert!(!verifier.verify(code).unwrap());
    }

    #[test]
    fn test_syntax_error() {
        let verifier = PythonVerifier::new("python3".to_string());
        let code = r#"
print("unclosed string
"#;
        assert!(!verifier.verify(code).unwrap());
    }
}
