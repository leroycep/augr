# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased
### Added
- `summary --refs` will list an Event's reference in the tags
- The `tag` command will add tags to previous events
- The `set-start` command will change the time an event starts

### Changed
- Reduced cloning and redundant work to make `augr` 10x faster
    - Results may vary ;)

## [0.2.0] - 2019-08-11
### Added
- `--time` argument to `start` command
- `summary --show-ends` will display when each event ended alongside the
  duration
- `--config` has been added to specify where the config file is
- User docs have been placed in <GUIDE.md>
- The `augr-core` has been split out of `augr-cli` and been made into its own
  crate
- The `import` subcommand has been added to migrate data from the previous
  format

### Changed
- File format has changed to a patch based format
    - This is a breaking change!
- `Transitions` renamed to `Events`
- `week` command renamed to `chart`
- `augr` will now attempt to sync before commands are run, instead of after

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
