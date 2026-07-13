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
- Display documentation for available commands and subcommands.

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
5. The template looks like [Template-File](.adr-template.md)
6. Create `<adr-directory>/.adr-status` with exactly:
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
5. Given `.adr-directory` already exists, when `adr init docs/adr` is run, then `.adr-directory` is overwritten
6. Given `<adr-directory>/.adr-template` or `<adr-directory>/.adr-status` already exist, when `adr init docs/adr` is run, these files are not overwritten

## 4.3 `adr new "<adr title>"`

Behavior:
- Create a new ADR as draft using the next auto-incremented ID.

Required effects:
1. Determine next free ADR ID.
2. Pad the IDs with zeros so that they have 3 places. E.g. `003`
3. Create a file `<adr-directory>/<id>-<adr-title>.md` with content of the `.adr-template` where the following place holders have been replaced
   - `ID` 
   - `TITLE`
   - `YYYY-MM-DD`
   - `STATUS`
4. New ADR status is `DRAFT`.
5. Spaces in the given title are replaced with dashes `-`

Acceptance criteria:
1. Given existing ADR files with IDs `1`, `2`, and `4`, when `adr new "Decision X"` is run, then a new file with the ID `5` is created, because the ID policy is `max + 1` .
2. Given `adr new "Decision X"` succeeds, created ADR metadata status is `DRAFT`.
3. Given ADR directory cannot be resolved from `.adr-directory`, `adr new` fails with a non-zero exit code.
4. Given existing ADR files with IDs `1`, `2`, and `4`, when `adr new "Decision X"` is run, then a new file `0005-Decision-X.md` is created.
5. Given an ADR title with characters which cannot be used in file names, then the application exists with an error.

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
2. Set status of ADR `<id-to-supersede>` to `SUPERSEDED` and append as additional information `by <id>` in the same line.
3. Update date of only the ADR `<id>` not of ADR `<id-to-supersede>`

Acceptance criteria:
1. Given both ADRs exist, when `adr mod <id> -s <id-to-supersede>` is run, then ADR `<id>` status becomes `ACCEPTED`.
2. Given both ADRs exist, ADR `<id-to-supersede>` is marked as superseded by `<id>` in a format consistent with metadata and TOC rendering rules.
3. Given both ADRs exist, when `adr mod <id> -s <id-to-supersede>` is run, the date inADR `<id>` is updated and the date in  `<id-to-supersede>` remains the same
4. Given one or both IDs do not exist, command fails with non-zero exit code.

## 4.6 `adr toc`

Behavior:
- Regenerate `<adr-directory>/adr-overview.md`.
- This generation is also triggered automatically by modifying commands (i.e. new and mod).

Required overview rendering:
1. Include links to all ADRs.
2. Sort links to ADRs by ID.
3. Render ADRs with status `DRAFT` and `PROPOSED` in bold. No additional marker text needed.
4. Render superseded ADRs with strike-through and mention the superseding ADR.
5. Render `EXPIRED` ADRs with strike-through and mark as `expired`.
6. Show ADR labels.

Required checks during TOC generation:
1. Detect missing IDs in sequence of existing ADR IDs. Missing IDs result in warnings.
2. Validate IDs inside ADR documents match IDs in file names. Mismatches result in warnings.
3. Validate metadata lines exist and values are valid for: title, date, status, author, labels. Findings should result in warnings.
4. Format of the meta data is the following
   ```
   # ADR`ID` - `TITLE`

   * Date: `YYYY-MM-DD`
   * Status: `STATUS`
   * Author: `AUTHOR`
   * Labels: `comma-sperated list of labels`
   ```

Acceptance criteria:
1. Given ADR files exist, when `adr toc` runs successfully, then `adr-overview.md` is regenerated.
2. Given draft/proposed ADRs exist, their entries are bold in the overview.
3. Given superseded ADRs exist, their entries are strike-through and include a superseding ADR reference.
4. Given expired ADRs exist, their entries are strike-through and marked `expired`.
5. Given ADRs include labels, labels appear in overview entries.
6. Given metadata or ID consistency issues exist, command reports them according to tool error/warning policy (policy details unresolved; see open questions).

## 5. Global Resolution Rules

1. For all commands except `init`, ADR directory resolution is performed by searching the current directory and then parent directories for `.adr-directory`.
   1. When no `.adr-directory` can be found this is an error and should be reported to the user.
   2. When the path in `.adr-directory` does not exist, this is an error and should be reported to the user.
2. The path stored in `.adr-directory` is interpreted as relative to the directory containing `.adr-directory`.
3. If no valid `.adr-directory` can be resolved, the command fails.

## 6. Error Handling and Exit Codes

Minimum behavior:
- On unrecoverable command failure, exit with non-zero status.
- On successful command completion, exit with zero status.
- On command completion with only errors, exit with zero status.

- Format warnings in yellow
- Format errors in red
