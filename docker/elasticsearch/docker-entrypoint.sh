#!/bin/bash

set -e

if ! /usr/share/elasticsearch/bin/elasticsearch-plugin list | grep -q analysis-ik; then
    printf '%s\n' "Installing the chinese language analyzer plugin"
    if ! /usr/share/elasticsearch/bin/elasticsearch-plugin install -b https://get.infini.cloud/elasticsearch/analysis-ik/8.14.3; then
        printf '%s\n' "Failed to install the chinese language analyzer plugin"
        exit 1
    fi
fi

# Run the original entrypoint script
exec /bin/tini -- /usr/local/bin/docker-entrypoint.sh
