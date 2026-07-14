# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in QAIVRA, please **do not** open a public issue.

Send a detailed report to the repository owner via GitHub's private vulnerability reporting feature or contact the maintainer directly.

Include:
- Description of the vulnerability
- Steps to reproduce
- Affected versions
- Potential impact
- Suggested fix (if available)

We will acknowledge receipt within 48 hours and provide a timeline for resolution.

## Security Architecture

QAIVRA is a **local-first** desktop application. Key security properties:

### No Cloud Upload
- All project data, images, videos, and analysis results stay on your local machine
- No telemetry, analytics, or automatic data transmission
- No cloud fallback — AI inference is local (Ollama) or user-configured (BYOK)

### Credential Storage
- API keys are stored in the OS keyring (Windows Credential Manager, macOS Keychain, Linux Secret Service)
- API keys are **never** stored in SQLite or plaintext files
- API keys are **never** logged
- API keys are **never** included in reports or exports

### Project Isolation
- Each project has its own data namespace
- Database queries are always project-scoped
- Project A cannot access Project B's data

### Least Privilege
- Tauri capabilities follow the principle of least privilege
- The frontend cannot access arbitrary filesystem paths
- External processes (Tesseract, FFmpeg) are invoked with structured arguments, never shell strings
- No shell command construction from user input

### Input Validation
- Image uploads: extension + magic-byte validation, 50 MiB limit
- Video uploads: extension allow-list, size limits
- No directory or symlink uploads
- Path canonicalization
- Bounded resource consumption

### No Malware Protection Guarantees
QAIVRA does not claim to be malware-free, unhackable, or 100% secure. It is a desktop application that relies on the host operating system's security controls.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ (alpha) |

## Dependencies

Dependencies are tracked in `Cargo.lock` and `package-lock.json`. Run `npm audit` and `cargo audit` (if available) to check for known vulnerabilities.

## Build Integrity

- Release builds should be code-signed (Authenticode on Windows)
- Release hashes should be published
- Unsigned development builds are not production distribution artifacts