#!/bin/bash

errcho(){ >&2 echo ">> ERROR: $@"; }

function exitWithErrMsg() {
  errcho "$1"
  exit 1
}

if [ -z "$NPMJS_TOKEN" ]; then
    exitWithErrMsg  "NPMJS_TOKEN environment variable not set."
fi
if [ -z "$PUBLISH_VERSION" ]; then
    exitWithErrMsg  "PUBLISH_VERSION environment variable not set."
fi

cd "$(dirname "$0")" || exit
echo '//registry.npmjs.org/:_authToken=${NPMJS_TOKEN}' > .npmrc
npm install
npm run compile
npm version $PUBLISH_VERSION
npm publish
