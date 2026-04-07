# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]

### Added

### Changed

### Fixed

## [0.3.0] - 2026-04-07

### Added

- Added `find_program_address_const` utility ([#364](https://github.com/exotic-markets-labs/typhoon/pull/364)).
- Added `token2022` feature to `typhoon-token` ([#356](https://github.com/exotic-markets-labs/typhoon/pull/356)).

### Changed

- Removed `borsh` dependency ([#365](https://github.com/exotic-markets-labs/typhoon/pull/365)).
- Reworked arguments handling ([#363](https://github.com/exotic-markets-labs/typhoon/pull/363)).
- Reworked account traits ([#349](https://github.com/exotic-markets-labs/typhoon/pull/349)).
- Replaced `Interface` with `Program` ([#355](https://github.com/exotic-markets-labs/typhoon/pull/355)).
- Replaced `InterfaceAccount` with `Account` ([#352](https://github.com/exotic-markets-labs/typhoon/pull/352)).
- Simplified lamports addition using `wrapping_add` ([#366](https://github.com/exotic-markets-labs/typhoon/pull/366)).
- Simplified `SplAccessor` implementation ([#361](https://github.com/exotic-markets-labs/typhoon/pull/361)).
- Updated dependencies (core and benchmarks) ([#367](https://github.com/exotic-markets-labs/typhoon/pull/367), [#360](https://github.com/exotic-markets-labs/typhoon/pull/360)).
- Updated `bytemuck` feature dependencies ([#362](https://github.com/exotic-markets-labs/typhoon/pull/362)).

### Fixed

- Improved address comparison logic for system `program_id` ([#359](https://github.com/exotic-markets-labs/typhoon/pull/359)).
- Improved error handling in context and IDL parsing ([#358](https://github.com/exotic-markets-labs/typhoon/pull/358)).
- Renamed `SignerNoCheck` to `UncheckedSigner` ([#357](https://github.com/exotic-markets-labs/typhoon/pull/357)).
- Removed unused fields in CLI templates ([#354](https://github.com/exotic-markets-labs/typhoon/pull/354)).

## [0.2.2] - 2026-02-27

### Changed

- Enhanced constraints documentation ([#347](https://github.com/exotic-markets-labs/typhoon/pull/347)).
- Cleanup of `AccountGenerator` internals ([#346](https://github.com/exotic-markets-labs/typhoon/pull/346)).
- Removed unused template file ([#350](https://github.com/exotic-markets-labs/typhoon/pull/350)).

### Fixed

- Activated `borsh` feature in `solana-address` ([#351](https://github.com/exotic-markets-labs/typhoon/pull/351)).

## [0.2.1] - 2026-02-17

### Fixed

- `typhoon-context` dependency ([#334](https://github.com/exotic-markets-labs/typhoon/pull/334)).
- Expose `address_eq` to prelude ([#335](https://github.com/exotic-markets-labs/typhoon/pull/335)).
- Error `NodeNotFound` if the fn is not an instruction ([#336](https://github.com/exotic-markets-labs/typhoon/pull/336)).
- Anchor IDL generator ([#337](https://github.com/exotic-markets-labs/typhoon/pull/337)).
- CPI invoke to accept `ExactSizeIterator` for remaining accounts ([#338](https://github.com/exotic-markets-labs/typhoon/pull/338)).
- Reexport the default allocator ([#339](https://github.com/exotic-markets-labs/typhoon/pull/339)).
- Codama program metadata ([#342](https://github.com/exotic-markets-labs/typhoon/pull/342)).
- IDL to only include needed types ([#343](https://github.com/exotic-markets-labs/typhoon/pull/343)).

## [0.2.0] - 2026-02-09

### Added

- CLI (`new`, `add` subcommands) ([#305](https://github.com/exotic-markets-labs/typhoon/pull/305)).
- `typhoon-traits` crate ([#315](https://github.com/exotic-markets-labs/typhoon/pull/315)).
- `const` discriminator calculation ([#322](https://github.com/exotic-markets-labs/typhoon/pull/322)).
- Reworked error logging ([#317](https://github.com/exotic-markets-labs/typhoon/pull/317)).
- Initial book chapters ([#323](https://github.com/exotic-markets-labs/typhoon/pull/323)).

### Changed

- Codama version updated ([#327](https://github.com/exotic-markets-labs/typhoon/pull/327)).
- Create account method refactored ([#320](https://github.com/exotic-markets-labs/typhoon/pull/320)).
- Package descriptions updated ([#319](https://github.com/exotic-markets-labs/typhoon/pull/319)).
- Context CPI optimized.
- MSRV bumped to 1.87 ([#318](https://github.com/exotic-markets-labs/typhoon/pull/318)).
- Pinocchio bumped to 0.10.0 ([#316](https://github.com/exotic-markets-labs/typhoon/pull/316)).
- Bytes writing: use `copy_nonoverlapping` instead of classic write ([#309](https://github.com/exotic-markets-labs/typhoon/pull/309)).
- More efficient bytes writing ([#314](https://github.com/exotic-markets-labs/typhoon/pull/314)).
- Benchmark table headers updated to reflect correct naming ([#312](https://github.com/exotic-markets-labs/typhoon/pull/312)).

### Fixed

- Unnecessary check in `Mut` removed ([#326](https://github.com/exotic-markets-labs/typhoon/pull/326)).
- Seed in `escrow` example ([#328](https://github.com/exotic-markets-labs/typhoon/pull/328)).
- Seeds and seeded usability ([#324](https://github.com/exotic-markets-labs/typhoon/pull/324)).
- CI workflow ([#330](https://github.com/exotic-markets-labs/typhoon/pull/330)).
- cargo-audit ([#329](https://github.com/exotic-markets-labs/typhoon/pull/329)).
- Macro error message typos ([#310](https://github.com/exotic-markets-labs/typhoon/pull/310)).

## [0.1.0] - 2025-12-16

### Added

- Initial release.

[Unreleased]: https://github.com/exotic-markets-labs/typhoon/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/exotic-markets-labs/typhoon/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/exotic-markets-labs/typhoon/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/exotic-markets-labs/typhoon/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/exotic-markets-labs/typhoon/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/exotic-markets-labs/typhoon/releases/tag/v0.1.0
