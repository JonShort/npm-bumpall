## Bash

### Usage
Executing the bash script will:
1. Execute `npm outdated` in the current dir
1. Collect all the outdated deps
1. Execute `npm i {dep1}@latest {dep2}@latest`, updating all deps to latest

_Executing the bash script_
```bash
sh ./bumpall.sh
```

### Development
1. `cd` into the `npm_dir` folder
1. Run the bumpall script with `sh ../bumpall.sh`
1. Run `sh ./downgrade_deps.sh` to revert deps to original state
