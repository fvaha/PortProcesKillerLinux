#!/bin/bash
# PortKiller Portable Runner

# Create a temporary directory
TMP_DIR=$(mktemp -d /tmp/portkiller.XXXXXX)

# Find the start of the payload
ARCHIVE=$(awk '/^__PAYLOAD_BELOW__/ {print NR + 1; exit 0; }' "$0")

# Extract the payload
tail -n+$ARCHIVE "$0" | tar xz -C "$TMP_DIR"

# Run the application
"$TMP_DIR/portkiller" "$@"

# Cleanup after exit
rm -rf "$TMP_DIR"

exit 0

__PAYLOAD_BELOW__
