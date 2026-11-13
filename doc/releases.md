# Releases

How a TargetScope release is numbered, built, installed and run. The tools are on the branch `releases` (folder `release/` and the file `VERSION`); the build dependencies are those of [development.md](development.md).

## The branch `releases`
Every part of TargetScope is developed on its own branch. The branch `releases` contains the merged state of all of them, plus the release tools and `Cargo.lock`, so a release always builds with the same dependency versions. Before a release, the current state of all branches is merged into it:
```
git checkout releases
git merge workspace documentation schemas common_rust common_cpp testkit launcher database_handler \
          camera_reader image_processor session_manager analyzer statistics exporter importer web_server system_tests
release/build.sh ...
```
A fix found while building a release is made on the branch of its part and merged again, never only on `releases`.

## Version numbers
Every release has the number `<release>.<majorfix>.<minorfix>`, e.g. `1.2.3`:

| Part | Raised when | Then |
|---|---|---|
| `release` | a new release with new features or changed behavior | `majorfix` and `minorfix` start at 0 again |
| `majorfix` | a fix of a serious error (data loss, a program doesn't start, wrong scores) | `minorfix` starts at 0 again |
| `minorfix` | a small fix (texts, layout, logging, a rare error with a workaround) | |

The number is written in four places, which always agree: `VERSION` (repository root), `version` in `[workspace.package]` of `Cargo.toml`, `project(... VERSION ...)` in `CMakeLists.txt`, and the installed file `<prefix>/lib/targetscope/VERSION`. Every Rust program reports it as `program_version` in its `ping_response` ([interfaces.md](interfaces.md#ping)). Release `0.x.y` is a development version.

### Raising the version
```
release/bump.sh release        # 0.1.0 → 1.0.0
release/bump.sh majorfix       # 1.0.0 → 1.1.0
release/bump.sh minorfix       # 1.1.0 → 1.1.1
```
`bump.sh` writes the new number into `VERSION`, `Cargo.toml` and `CMakeLists.txt` and prints it. It doesn't commit or tag; the release commit is tagged `v<version>` by hand after the build was checked.

## Building a release
```
release/build.sh [--without <program>,<program>] [--out <folder>]
```
1. Checks that `VERSION`, `Cargo.toml` and `CMakeLists.txt` have the same number; otherwise it stops.
2. Builds the Rust programs with `cargo build --release` (without the feature `failpoints`, so no failpoint and no test clock is compiled in, see [testing/overview.md](testing/overview.md#failpoints)).
3. Builds the C++ programs with CMake (`Release`, tests and failpoints off, folder `build-release/`) and copies them into the bundle. If `TARGETSCOPE_DEPS` is set ([development.md](development.md#environment)), its `lib` folder is the run path of the C++ binaries, so they find redis-plus-plus, hiredis and iceoryx2 there.
4. Assembles the bundle (below) in `<folder>` (default `dist/`), packs it as `targetscope-<version>-linux-<arch>.tar.gz` and writes its SHA-256 sum next to it (`.sha256`).

`--without` leaves programs out of the bundle (e.g. `--without camera_reader,image_processor` for a PC without camera, or while a program isn't finished). The bundled `launcher.toml` then has `enabled = false` for them; the launcher ignores dependencies on disabled programs ([launcher.md](programs/launcher.md#startup-sequence)). The launcher itself, `database_handler` and `web_server` can't be left out.

### Bundle
```
targetscope-<version>-linux-<arch>/
├── VERSION
├── README.md                   # this document
├── install.sh
├── uninstall.sh
├── run.sh
├── bin/
│   └── targetscope             # the launcher
├── lib/targetscope/            # bin_dir: the programs, VERSION, the web frontend
│   ├── VERSION
│   ├── database_handler, session_manager, analyzer, statistics, exporter, importer, web_server
│   ├── camera_reader, image_processor
│   └── static/                 # frontend, next to web_server (see programs/web_server.md)
└── share/targetscope/
    └── launcher.toml           # default configuration with the left-out programs disabled
```

## Installing
Needs at run time: `redis-server` (Redis or Valkey) in the `PATH`, and for the C++ programs OpenCV and the libraries of [development.md](development.md).
```
tar xzf targetscope-<version>-linux-<arch>.tar.gz
cd targetscope-<version>-linux-<arch>
sudo ./install.sh                     # into /usr/local
./install.sh --prefix ~/.local        # without root
```
`install.sh` copies `bin/targetscope` to `<prefix>/bin/` and `lib/targetscope/` to `<prefix>/lib/targetscope/` (an older version there is replaced completely), and checks that `redis-server` is found. The configuration of the user (`~/.config/targetscope/launcher.toml`, see [launcher.md](programs/launcher.md#usage)):
- doesn't exist: with the default prefix `/usr/local` nothing is written (the launcher writes its default on first start, which has `bin_dir = "/usr/local/lib/targetscope"`); with another prefix, the bundled `launcher.toml` is written with `bin_dir = "<prefix>/lib/targetscope"`
- exists: it's never changed; if its `bin_dir` isn't `<prefix>/lib/targetscope`, `install.sh` prints a warning
When run with `sudo`, the configuration of the user who called `sudo` is used (`SUDO_USER`).

Data, settings, images and logs (folder `~/.local/share/targetscope`, see [software_description.md](software_description.md#file-locations)) aren't touched by installing, updating or uninstalling. An update is installing the newer bundle; the database handler migrates the database on its next start ([database_handler.md](programs/database_handler.md#schema-migrations)).

## Running
After installing:
```
targetscope                    # starts everything; Ctrl+C stops it
```
Without installing, from the unpacked bundle:
```
./run.sh [launcher options]
```
`run.sh` writes `etc/launcher.toml` in the bundle on its first start (the bundled default with `bin_dir` = the bundle's `lib/targetscope`) and starts `bin/targetscope --config etc/launcher.toml`. It uses the same data folder as an installed TargetScope. If `TARGETSCOPE_DEPS` is set, its `bin` folder is put in front of the `PATH` (for a `redis-server` built there).

## Uninstalling
```
sudo ./uninstall.sh                   # or: ./uninstall.sh --prefix ~/.local
```
Removes `<prefix>/bin/targetscope` and `<prefix>/lib/targetscope/`; only if `<prefix>/lib/targetscope/VERSION` exists, so it never deletes a folder it didn't install. Data and configuration are kept.
