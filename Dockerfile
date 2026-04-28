# Build stage
FROM rust:latest AS builder

WORKDIR /app

# Cache dependencies - copy only manifests first
COPY Cargo.toml Cargo.lock ./
COPY 01_domain/Cargo.toml 01_domain/Cargo.toml
COPY 02_application/Cargo.toml 02_application/Cargo.toml
COPY 03_infrastructure/Cargo.toml 03_infrastructure/Cargo.toml
COPY 04_presentation/Cargo.toml 04_presentation/Cargo.toml

# Create dummy src files so cargo can resolve the workspace
RUN mkdir -p src 01_domain/src 02_application/src 03_infrastructure/src 04_presentation/src && \
    echo "fn main() {}" > src/main.rs && \
    touch 01_domain/src/lib.rs && \
    touch 02_application/src/lib.rs && \
    touch 03_infrastructure/src/lib.rs && \
    touch 04_presentation/src/lib.rs

# Build dependencies only (this layer is cached until Cargo.toml changes)
RUN cargo build 2>/dev/null || true

# Now copy actual source and rebuild
COPY . .

# Touch source files to invalidate the dummy builds
RUN touch src/main.rs 01_domain/src/lib.rs 02_application/src/lib.rs \
    03_infrastructure/src/lib.rs 04_presentation/src/lib.rs

RUN cargo build

# Runtime stage
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/debug/peer-pressure /usr/local/bin/peer-pressure

ENTRYPOINT ["peer-pressure"]
CMD ["--port", "9000", "--bind", "0.0.0.0"]
