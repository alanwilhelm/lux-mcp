# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of Lux MCP
- 9 reasoning tools with different capabilities
- Support for GPT-5, O3, O4, and standard models
- Sequential thinking tools (manual and AI-powered)
- Traced reasoning with metacognitive monitoring
- Bias detection with dual-model verification
- Interactive planner with file awareness
- Direct file reading capability for all tools
- Session management with 3-hour TTL
- Comprehensive documentation
- Environment configuration helper tool

### Fixed
- Token limit handling for gpt-4o (16384 tokens)
- Temperature parameter handling for GPT-5 and O3 models
- Environment variable backward compatibility

### Security
- Secure API key handling via environment variables
- No hardcoded secrets in codebase
- Read-only file access for security

## [0.1.0] - TBD

Initial public release.