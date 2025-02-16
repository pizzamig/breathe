# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- ROADMAP: updated
- Removed total progression bar (for now)
- Removed async (useless complexity)
- Update all crates
- .gitignore: added \*.swp files (vi backup)
- Replace StructOpt with clap
- Adopt anyhow

## [0.2.4]
### Added
- pattern: Add new pattern 6-3-6 (Biohack your brain)

## [0.2.3] 2021-05-23
- crates: update dialoguer to 0.8
- crates: update to indicatif 0.16.1
- duration: add an option to specify a different duration of the exercise

## [0.2.2] 2021-02-12
### Fixed
- progress bar: the issue has never been fixed. Multiprogress bars has poor
  async support, only one event trigger the redrawn, the next ones can get postponed.
  For this reason, this fix is giving priority to the breath progress bar.

## [0.2.1] 2021-02-02
### Fixed
- progress bar: fix refresh issues

## [0.2.0] 2021-01-30
### Added
- ROADMAP: Add a roadmap file
- .dockerignore: skip the target folder when running docker build
- patterns: add more breathing patterns to the config toml example file
- pattern: add description of breathing patterns

### Changed
- layout: length of both bars set to 80

### Fixed
- breathe bar: fix the double/missing progress

## [0.1.1]
### Changed
- prompt: make Y the default answer

## [0.1.0]
### Added
- Initial working version
- config: toml-based configuration file
- license: add license
- cosmetics: output improvement
- docker: add docker build support
- list: add the ability to list the supported breathing patterns
- patterns: add 4-4-4-4 patterns
