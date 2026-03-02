#!/bin/bash

# Configuration
COMPOSE_FILE="docker-compose.yml"

# Cleanup function to run on Ctrl+C (SIGINT) or script exit
cleanup() {
    echo -e "\n\n[!] Stopping PixelWar services..."
    docker compose -f $COMPOSE_FILE stop
    exit 0
}

# Trap signals
trap cleanup SIGINT SIGTERM

echo " Starting PixelWar Distributed Stack..."

# 1. Check for .env
if [ ! -f .env ]; then
    echo "[?] No .env found, creating from example..."
    cp .env.example .env
fi

# 2. Build and Start in the background
# --remove-orphans ensures old deleted services don't stick around
docker compose -f $COMPOSE_FILE up --build -d --remove-orphans

if [ $? -ne 0 ]; then
    echo "[!] Docker failed to start. Check your Docker Desktop status."
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  PixelWar is running!"
echo ""
echo "  API Gateway:         http://localhost:3000"
echo "  Jaeger UI:           http://localhost:16686"
echo "  Prometheus:          http://localhost:9090"
echo "  Grafana:             http://localhost:3100  (admin/admin)"
echo "  NATS monitor:        http://localhost:8222"
echo "  Postgres (pgAdmin):  http://localhost:5050  (admin@postgres / password)"
echo "  Redis Insight:       http://localhost:5540  (admin / password)"
echo ""
echo "  Press Ctrl-C to stop everything."
echo "════════════════════════════════════════════════════════════"
echo ""

# Wait for any child to exit (or Ctrl-C)
docker compose -f $COMPOSE_FILE logs -f