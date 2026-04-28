#!/bin/bash
set -e

SESSION="peer-pressure"
IMAGE="peer-pressure"
NETWORK="peer-pressure-net"

echo "Building container image..."
podman build -t "$IMAGE" .

# Clean up old containers/network
podman rm -f peer-a peer-b peer-c 2>/dev/null || true
podman network rm "$NETWORK" 2>/dev/null || true

# Create network with static subnet
podman network create --subnet 172.20.0.0/24 "$NETWORK" 2>/dev/null || true

# Kill existing tmux session if it exists
tmux kill-session -t "$SESSION" 2>/dev/null || true

# Ensure host data dirs exist
mkdir -p "$(pwd)/data/peer-a" "$(pwd)/data/peer-b" "$(pwd)/data/peer-c"

# Create tmux session — left pane runs peer-a
tmux new-session -d -s "$SESSION" \
  "podman run -it --rm --name peer-a --network $NETWORK --ip 172.20.0.101 -v $(pwd)/data/peer-a:/data $IMAGE --data-dir /data; echo 'peer-a exited'; read"

# Split vertically — middle pane runs peer-b
tmux split-window -h -t "$SESSION" \
  "podman run -it --rm --name peer-b --network $NETWORK --ip 172.20.0.102 -v $(pwd)/data/peer-b:/data $IMAGE --data-dir /data; echo 'peer-b exited'; read"

# Split again — right pane runs peer-c
tmux split-window -h -t "$SESSION" \
  "podman run -it --rm --name peer-c --network $NETWORK --ip 172.20.0.103 -v $(pwd)/data/peer-c:/data $IMAGE --data-dir /data; echo 'peer-c exited'; read"

# Even out the pane widths
tmux select-layout -t "$SESSION" even-horizontal

# Attach to the session
tmux attach-session -t "$SESSION"
