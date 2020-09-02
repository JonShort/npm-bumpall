#!/bin/bash

outdated=$(npm outdated)

if [ "$outdated" = "" ]; then
  printf "All dependencies are up to date!\n"
  exit 0
fi

printf "Outdated dependencies\n\n${outdated}\n\n"

regex="^[a-z\/@\-]+ "
command="npm i"

while IFS= read -r line; do
  if [[ $line =~ $regex ]]
  then
    package=$BASH_REMATCH
    # trim off the trailing space
    package="${package// }"
    command="${command} ${package}@latest"
  fi
done <<< "$outdated"

printf "Running:\n$command\n\n"
$command
