# npm-bumpall

Utility to bump npm packages, by default to the latest minor version.

Main feature is also updating the `package.json`, rather than just updating the version in the lockfile (e.g. like how `npm update` works)

## Usage
_Execute the binary_
```bash
~/repos/npm-bumpall/rust/target/release/npm-bumpall
```

### Options

`--help` | `-h` - print help to the terminal

`--dry-run` | `-d` - list dependencies which would be bumped, but don't update them

`--latest` | `-l` - bump dependencies to latest possible version (includes major changes)

`--legacy-peer-deps` - includes this option in the npm install under the hood

`--patch` | `-p` - only include patch version updates (experimental)

`--verbose` | `-v` - include all possible messages in console output (e.g. warnings from npm itself)

## Compiling
_Generate a release build_
```bash
cargo build --release
```

## Development
_Run locally against stub folder_
```bash
cargo build
cd ./npm_dir
../target/debug/npm-bumpall
```

_downgrade stub folder_
```bash
sh ./npm_dir/downgrade_deps.sh
```
