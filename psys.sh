#!/bin/zsh

ARGS=$(getopt -a --options b:c:h:t: --long "build:,cmd:,help:,test:" -- "$@")

USAGE="\n\rPSYS BUILD SCRIPT\n\r
\n\r
("$0" [-h] [-b <target>] [-t <target>])\n\r
\n\r
\r./psys.sh -b or --build | target  | > Builds the program/library\n
\r./psys.sh -c or --cmd   | command | > Runs the specified comnmand\n
\r./psys.sh -t or --test  | target  | > Runs tests on the program/library\n
\r./psys.sh -h or --help              > Displays this help message\n
\n\r
"

eval set -- "$ARGS"

while true; do
  case "$1" in
    -b|--build)
      target="$2"
      cargo build --target="./$target.json"
      exit
      ;;
    -h|--help)
      echo $USAGE
      exit
      ;;
    -c|--cmd)
      case "$2" in
        "help")
          echo $USAGE
          exit
          ;;
        "build")
          target="$4"
          cargo build --target="./$target.json"
          exit
          ;;
        "test")
          target="$4"
          cargo test --target="./$target.json"
          exit
          ;;
        --)
          break
          ;;
      esac
      exit
      ;;
    -t|--test)
      target="$2"
      cargo test --target="./$target.json"
      exit
      ;;
    --)
      break;;
  esac
done
