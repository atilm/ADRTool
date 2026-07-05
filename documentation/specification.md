# ADR Tool Specification (Derived from Usage Guide)

## 1. Purpose

The `adr` command-line application manages Architecture Decision Records (ADRs) as markdown files stored in a repository and version controlled with source code.

This specification is derived from `documentation/usage-guide.md` and is intended to drive implementation and testing.

## 2. Scope

In scope:
- Command help output (`adr -h`)
- Repository initialization (`adr init <adr-directory>`)
- New ADR creation (`adr new "<adr title>"`)
- ADR modification (`adr mod <id> -a`, `adr mod <id> -s <id-to-supersede>`)
- Table of contents generation (`adr toc`)
- Consistency checks executed during TOC generation and modifying operations

Out of scope:
- Any behavior not explicitly described in the usage guide

## 3. Terms and Data Model

- ADR directory: the directory where ADR markdown files and ADR support files are stored.
- Marker file: `.adr-directory`, created in a repository directory and used to locate the ADR directory.
- ADR template file: `<adr-directory>/.adr-template` used as the content template for new ADR files.
- ADR status file: `<adr-directory>/.adr-status` listing valid status keywords.
- ADR overview file: `<adr-directory>/adr-overview.md` listing ADR links and status-based formatting.
- ADR ID: positive integer used in file names and document metadata.
- ADR file naming format: `<id>-<adr-title>.md`.

## 4. Command Specification and Acceptance Criteria

## 4.1 `adr -h`

Behavior:
- Display documentation for available commands.

Acceptance criteria:
1. Given the user runs `adr -h`, when the command executes, then help text is printed and the process exits successfully.
2. Help output includes usage for at least `init`, `new`, `mod`, and `toc` commands.

## 4.2 `adr init <adr-directory>`

Behavior:
- Initialize ADR management files for the repository.

Required effects:
1. Create `.adr-directory` in the current working directory.
2. Write the path to `<adr-directory>` into `.adr-directory`, as a path relative to the marker file location.
3. Create the ADR directory itself.
4. Create `<adr-directory>/.adr-template`.
5. Create `<adr-directory>/.adr-status` with exactly:
   - `DRAFT`
   - `PROPOSED`
   - `ACCEPTED`
   - `ADOPTED`
   - `SUPERSEDED`
   - `EXPIRED`

Acceptance criteria:
1. Given a repository directory with no ADR setup, when `adr init docs/adr` is run, then `.adr-directory` exists in the current directory and contains a relative path pointing to `docs/adr`.
2. Given initialization succeeds, `docs/adr` exists and contains `.adr-template` and `.adr-status`.
3. Given initialization succeeds, `.adr-status` contains exactly the six status values listed above, one per line.
4. Given a non-init command is run from the marker directory or any subdirectory, the tool searches upward for `.adr-directory` and resolves the ADR directory using the path stored in that marker file.

## 4.3 `adr new "<adr title>"`

Behavior:
- Create a new ADR as draft using the next auto-incremented ID.

Required effects:
1. Determine next free ADR ID.
2. Create a file `<adr-directory>/<id>-<adr-title>.md`.
3. New ADR status is `DRAFT`.

Acceptance criteria:
1. Given existing ADR files with IDs `1`, `2`, and `4`, when `adr new "Decision X"` is run, then a new file with the next auto-incremented ID is created according to the defined ID policy (see open questions).
2. Given `adr new "Decision X"` succeeds, created ADR metadata status is `DRAFT`.
3. Given ADR directory cannot be resolved from `.adr-directory`, `adr new` fails with a non-zero exit code.

## 4.4 `adr mod <id> -a`

Behavior:
- Accept an ADR.

Required effects:
1. Set status of ADR `<id>` to `ACCEPTED`.
2. Update the ADR date.

Acceptance criteria:
1. Given ADR `<id>` exists, when `adr mod <id> -a` is run, then the ADR status is `ACCEPTED`.
2. Given ADR `<id>` exists, when command succeeds, then ADR date is updated.
3. Given ADR `<id>` does not exist, command fails with non-zero exit code.

## 4.5 `adr mod <id> -s <id-to-supersede>`

Behavior:
- Accept ADR `<id>` and supersede ADR `<id-to-supersede>`.

Required effects:
1. Set status of ADR `<id>` to `ACCEPTED`.
2. Set status of ADR `<id-to-supersede>` to `SUPERSEDED by <id>`.
3. Update date (scope of date update is ambiguous; see open questions).

Acceptance criteria:
1. Given both ADRs exist, when `adr mod <id> -s <id-to-supersede>` is run, then ADR `<id>` status becomes `ACCEPTED`.
2. Given both ADRs exist, ADR `<id-to-supersede>` is marked as superseded by `<id>` in a format consistent with metadata and TOC rendering rules.
3. Given one or both IDs do not exist, command fails with non-zero exit code.

## 4.6 `adr toc`

Behavior:
- Regenerate `<adr-directory>/adr-overview.md`.
- This generation is also triggered automatically by modifying commands.

Required overview rendering:
1. Include links to all ADRs.
2. Render ADRs with status `DRAFT` and `PROPOSED` in bold.
3. Render superseded ADRs with strike-through and mention the superseding ADR.
4. Render `EXPIRED` ADRs with strike-through and mark as `expired`.
5. Show ADR labels.

Required checks during TOC generation:
1. Detect missing IDs in sequence of existing ADR IDs.
2. Validate IDs inside ADR documents match IDs in file names.
3. Validate metadata lines exist and values are valid for: title, date, status, author, labels.

Acceptance criteria:
1. Given ADR files exist, when `adr toc` runs successfully, then `adr-overview.md` is regenerated.
2. Given draft/proposed ADRs exist, their entries are bold in the overview.
3. Given superseded ADRs exist, their entries are strike-through and include a superseding ADR reference.
4. Given expired ADRs exist, their entries are strike-through and marked `expired`.
5. Given ADRs include labels, labels appear in overview entries.
6. Given metadata or ID consistency issues exist, command reports them according to tool error/warning policy (policy details unresolved; see open questions).

## 5. Global Resolution Rules

1. For all commands except `init`, ADR directory resolution is performed by searching the current directory and then parent directories for `.adr-directory`.
2. The path stored in `.adr-directory` is interpreted as relative to the directory containing `.adr-directory`.
3. If no valid `.adr-directory` can be resolved, the command fails.

## 6. Error Handling and Exit Codes

Minimum behavior:
- On unrecoverable command failure, exit with non-zero status.
- On successful command completion, exit with zero status.

Formatting of warnings/errors, warning-only conditions, and exact exit code taxonomy are undefined in the usage guide.

## 7. Open Questions and Inconsistencies to Resolve Before Implementation

1. Superseded status naming inconsistency.
- `init` status list contains `SUPERSEDED`.
- `mod -s` behavior says status becomes `SUPERSEDED by <id>`.
- TOC section uses `SUPERSED` (likely typo).
- Decision needed: canonical metadata representation for superseded ADRs.

2. ID auto-increment policy is not explicit.
- `adr new` says auto-incremented ID, but does not define whether to use `max + 1` or smallest missing positive integer.

3. Date update scope in `mod -s` is ambiguous.
- Text says `update the date` without defining whether one ADR or both ADRs must be updated.

4. Initialization overwrite behavior is undefined.
- If `.adr-directory`, ADR directory, `.adr-template`, or `.adr-status` already exist, expected behavior is not specified.

5. ADR file title slug rules are undefined.
- No normalization rules for spaces, punctuation, casing, unicode, or duplicate resulting filenames are defined.

6. ADR metadata format is not fully specified.
- Required fields are listed, but exact line format, parser rules, and allowed date format are not defined.

7. Missing marker or invalid marker behavior details are incomplete.
- It is clear this is an error, but message format and distinction between missing marker vs invalid target path are undefined.

8. TOC ordering is unspecified.
- Not defined whether entries are sorted by ID, date, status, or file system order.

9. Severity of consistency check findings is unspecified.
- Missing ID sequence might be warning or error; guide does not define whether command should fail.

10. Scope of automatic TOC regeneration needs explicit command list.
- Guide says modifying commands trigger TOC automatically, but does not explicitly list whether this includes only `new` and `mod`, or any future modifiers.

11. Behavior of `adr -h` output depth is unspecified.
- Not defined whether subcommand help (for example `adr mod -h`) is required.

12. Terminology for "ToDos" is presentation-only.
- Bold formatting for `DRAFT` and `PROPOSED` is defined, but whether additional marker text is needed is undefined.

## 8. Recommended Next Step

Before implementation, produce a short "decisions" addendum resolving all open points above. After that, these acceptance criteria can be converted directly into integration tests.
