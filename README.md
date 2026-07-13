# ADRTool
A cli tool written in rust to manage architecture decision records

See the [Usage Guide](documentation/usage-guide.md) for help.

## Builds and releases

Every push runs the GitHub Actions build workflow. It runs the test suite on
Linux and builds release binaries for Linux x86_64 and Windows x86_64.

To release a specific commit, create and push a version tag pointing to that
commit:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The release workflow creates a stable GitHub Release for the tag and publishes
these downloadable assets:

- `adr-linux-x86_64`
- `adr-windows-x86_64.exe`
- `adr-linux-x86_64.tar.gz`, containing `adr` and shell completions
- `adr-windows-x86_64.zip`, containing `adr.exe` and shell completions

The archives include completion scripts for Bash, Zsh, Fish, and PowerShell.
See the [Usage Guide](documentation/usage-guide.md) for installation commands.
