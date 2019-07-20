# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `--time` argument to `start` command

### Changed
- `Transitions` renamed to `Events`
- `week` command renamed to `chart`

## [0.1.1] - 2019-07-18
### Fixed
- Empty device files will no longer cause `augr` to crash
- Read from `.config/augr` instead of `.config/time-tracker`

## [0.1.0] - 2019-07-18
### Changed
- Renamed project from `time-tracker` to `augr`

### Added
- Convinent ways to specify date and time
- Multi device synchronization through any file synchronization service
- Termux support through cross compilation
- Control time range of `summary` and `week` commands with `--start` and `--end`
  options
- `tags` command to get a list of tags that have been used
- `summary` and `week` and week commands can be filtered by passing a list of
  tags to the command
- `week` command to get a visual overview of the past week
- `summary` command to see where your time has gone
