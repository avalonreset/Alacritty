Scripts
=======

## Flamegraph

Run the release version of Alacritty while recording call stacks. After the
Alacritty process exits, a flamegraph will be generated and it's URI printed
as the only output to STDOUT.

```sh
./create-flamegraph.sh
```

Running this script depends on an installation of `perf`.

## ANSI Color Tests

We include a few scripts for testing the color of text inside a terminal. The
first shows various foreground and background variants. The second enumerates
all the colors of a standard terminal. The third enumerates the 24-bit colors.

```sh
./fg-bg.sh
./colors.sh
./24-bit-colors.sh
```

## Third-Party License Notices

When redistributing binaries, keep `THIRD_PARTY_NOTICES.html` up to date.

```sh
./generate-third-party-notices.sh
```

On Windows (PowerShell):

```powershell
.\generate-third-party-notices.ps1
```
