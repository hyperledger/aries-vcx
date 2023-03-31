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
npm install --save-exact @hyperledger/node-vcx-wrapper@${PUBLISH_VERSION} || exitWithErrMsg "Failed to install @hyperledger/node-vcx-wrapper@${PUBLISH_VERSION}"
npm install
npm version --no-git-tag-version $PUBLISH_VERSION
npm publish
