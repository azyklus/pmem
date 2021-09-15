#!/bin/zsh

ARGS=$(getopt -a --options t:h: --long "target:,help:" -- "$@")

eval set -- "$ARGS"

while true; do
  case "$1" in
    -t|--target)
      target="$2"
      cargo build --target $target\.json
      shift 2;;
    -h|--help)
      shift 2;;
    --)
      break;;
  esac
done
