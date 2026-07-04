# Requirements

* CLI app
  * with command completions
  * written in Rust
  * using clap

* Should have the following commands
  * adr init <direcotry path>
    * generates the directory
    * generates a help document explainging the workflow
    * generates a .adr-dir file in the current working directory
    * generates a .adr-template inside the directory
    * generates .adr-status inside the directory listing valid status words
      * DRAFT/~~PROPOSED~~/ACCEPTED/~~ADOPTED~~/SUPERSEDED_BY_<ID>/~~EXPIRED~~
  * adr new "A question to decide"
    * auto-generates next free id
  * adr new -s 8 "A superseding decision"
  * adr mod <id> -s <s-id> -> modify <s-id> to be superseded by <id> and set <id> to ACCEPTED
  * adr mod <id> -a -> accept  -> accept <id>
  * adr toc
  * adr help

* each command looks from the current directory upward for a file .adr-dir 
  * and uses the directory inside as adr directory path
  * if .adr-dir does not contain a valid directory, this is an error

* output
  * info messages
  * warnings in yellow
  * errors in red -> exit with error exit code

* Each adr call should
  * check
    * Ids in files match ids in file names
    * meta data (status, date) are present and valid
    * output warnings for missing ids in sequence
  * regenerate a table of contents
    * listing ADR titles
      * with labels
      * strike-through formatting of superseded ADRs followed by "superseded by ADRXYZ"
      * bold formatting of DRAFT and PROPOSED ADRs
