#!/usr/bin/env bash
set -euo pipefail

# Evaluator-facing alias used by grant and review materials.
exec bash "$(dirname "$0")/check.sh" "$@"
