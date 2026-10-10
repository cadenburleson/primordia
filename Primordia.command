#!/bin/zsh
set -e
cd "${0:A:h}"
if ! command -v node >/dev/null 2>&1; then
  print "Primordia needs Node.js 22.12 or newer. Install Node, then open this launcher again."
  read "?Press Return to close."
  exit 1
fi
if [[ ! -d node_modules ]]; then npm install; fi
if curl -fsS http://127.0.0.1:5194/ >/dev/null 2>&1; then
  open http://127.0.0.1:5194/
  exit 0
fi
(sleep 2; open http://127.0.0.1:5194/) &
npm run dev
