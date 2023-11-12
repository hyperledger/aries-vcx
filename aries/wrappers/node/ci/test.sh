#!/bin/bash
cd vcx/aries/wrappers/node/
npm i
npm run lint
npm run compile
npm test
npm run test-logging
