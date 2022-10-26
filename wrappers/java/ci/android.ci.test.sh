#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"

echo "android.ci.test.sh >> calling android.prepare.sh"
source ${SCRIPT_DIR}/android.prepare.sh
echo "android.ci.test.sh >> android.test.sh armv7"
source ${SCRIPT_DIR}/android.test.sh armv7

echo "android.ci.test.sh >> finished"