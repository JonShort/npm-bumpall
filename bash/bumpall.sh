#!/bin/bash

if [ -p /dev/stdin ]; then
  regex="^[a-z@\-]+ "
  output="npm i"

  count=0
  while read line
  do
    if [[ $line =~ $regex ]]
    then
      package=$BASH_REMATCH
      # trim off the trailing space
      package="${package// }"
      output="${output} ${package}@latest"
    fi
  done

  echo "${output}"
else
  echo "Pipe the results of npm outdated to this script"
fi
