#!/usr/bin/env bash
# Runs on the VM. Idempotent: safe to re-run.
#
# Required env (passed in by the CD workflow):
#   IMAGE_TAG      git sha of the image to deploy
#   GHCR_USER      github actor for ghcr.io login
#   GHCR_TOKEN     short-lived GITHUB_TOKEN with read:packages
set -euo pipefail

APP_DIR="$HOME/dearday"

if ! command -v docker >/dev/null 2>&1; then
    echo "[deploy] installing docker"
    curl -fsSL https://get.docker.com | sh
    systemctl enable --now docker
fi

mkdir -p "$APP_DIR"
cd "$APP_DIR"

echo "$GHCR_TOKEN" | docker login ghcr.io -u "$GHCR_USER" --password-stdin

export IMAGE_TAG
docker compose pull
docker compose up -d --remove-orphans

# Caddyfile is a mounted file, not part of the compose container spec, so
# `up -d` won't recreate caddy when the file changes. Live-reload instead.
if docker compose ps --status running --services | grep -q '^caddy$'; then
    docker compose exec -T caddy caddy reload \
        --config /etc/caddy/Caddyfile --adapter caddyfile
fi

docker image prune -f
