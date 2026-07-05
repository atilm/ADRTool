# Usage Guide for the ADR tool

The cli application `adr` helps you to manage architecture decision records in 
markdown files that can be revision controlled alongside the source code.

## To show documentation on available commands

```
adr -h
```

## To initialize ADRs

In the root directory of your repository execute

```
adr init <adr-directory>
```

This will:

* Create a file `.adr-directory` in the current working directory which contains the path
  to the `<adr-directory>` relative to its own location
  * When any other `adr` command is executed the application will search the current working directory
    and all parent directories for `.adr-directory` and reconstruct the location of the `<adr-directory>`
* Create the `adr-directory` itself
* Create the file `<adr-directory>/.adr-template` which the application uses to create new adr files
  You can modify the template, but you must not modify the header lines which contain place-holders, because
  the application relies on the line structure to perform consistency checks.
* Create the file `<adr-directory>/.adr-status` with the following content
  ```
  DRAFT
  PROPOSED
  ACCEPTED
  ADOPTED
  SUPERSEDED
  EXPIRED
  ```
  It lists the key words which are valid options for adr status. You may remove key words
  so that the application's consistency checks will not accept them, but you cannot add new ones.

## To add a new draft ADR

```
adr new "<adr title>"
```

This will create a new adr file `<adr-directory>/<id>-<adr-title>.md` with an auto-incremented id.
The adr's status will be DRAFT.

## To accept a new ADR

```
adr mod <id> -a
```

This will set the status of the adr with the given id to ACCEPTED and
update the date.

## To accept an ADR superseding an old ADR

```
adr mod <id> -s <id-to-supersede>
```

This will

* set the status of the adr with `<id>` to ACCEPTED.
* set the status of the adr with `<id-to-supersede>` to SUPERSEDED by <id>
* update the date

## To regenerate the overview document

```
adr toc
```

This command is executed automatically whenever the application is used with a modifying command.
But you can execute it explicitly after you made manual changes to an ADR.

This will generate the overview document `<adr-directory>/adr-overview.md`.
It lists links to all ADRs:

* ADRs in status DRAFT and PROPOSED will be formatted bold to mark them as ToDos
* ADRs in status SUPERSED will be formatted with strike-through and the superseding ADR will be mentioned
* ADRs in status EXPIRED will be formatted with strike-through and marked as "expired"
* ADRs labels will be shown

The command will also perform consistency checks:

* Are there missing IDs in the sequence of existing IDs?
* Are the IDs mentioned inside the document matching the IDs in the file name?
* Do the meta data lines exist and have valid values (title, date, status, author, labels)

