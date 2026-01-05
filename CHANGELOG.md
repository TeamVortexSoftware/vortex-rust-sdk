# Changelog

All notable changes to the Vortex Rust SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.3] - 2025-01-29

### Added
- **AcceptUser Struct**: New preferred format for accepting invitations with `email`, `phone`, and `name` fields
- **AcceptInvitationParam Enum**: Type-safe parameter handling for multiple input formats
- Enhanced `accept_invitations` method to support both new User format and legacy target format via `impl Into<AcceptInvitationParam>`

### Changed
- **DEPRECATED**: Legacy `InvitationTarget` format for `accept_invitations` - use `AcceptUser` instead
- Internal API calls now always use User format for consistency
- Added `eprintln!` deprecation warnings when legacy target format is used

### Fixed
- Maintained 100% backward compatibility - existing code using legacy target format continues to work
- Used `Box::pin` for async recursion to handle array of targets correctly
