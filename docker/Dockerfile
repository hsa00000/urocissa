FROM rust:bookworm AS builder

ARG BUILD_TYPE=release
ENV BUILD_TYPE=${BUILD_TYPE}

WORKDIR /app/gallery-backend

COPY ./gallery-backend/Cargo.lock /app/gallery-backend/Cargo.lock
COPY ./gallery-backend/Cargo.toml /app/gallery-backend/Cargo.toml
COPY ./gallery-backend/src /app/gallery-backend/src

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Build the backend binary based on the build type
RUN if [ "${BUILD_TYPE}" = "release" ]; then \
    cargo build --release --bin urocissa; \
    elif [ "${BUILD_TYPE}" = "debug" ]; then \
    cargo build --bin urocissa; \
    else \
    cargo build --profile "${BUILD_TYPE}" --bin urocissa; \
    fi

RUN cp /app/gallery-backend/target/${BUILD_TYPE}/urocissa /app/gallery-backend/urocissa

######################
# Frontend builder stage
######################
FROM node:lts AS frontend-builder
WORKDIR /app/gallery-frontend
COPY ./gallery-frontend /app/gallery-frontend
RUN npm ci && npm run build

######################
# Runtime stage
######################
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/gallery-backend

# Copy backend binary
COPY --from=builder /app/gallery-backend/urocissa /app/gallery-backend/urocissa

# Copy frontend assets
COPY --from=frontend-builder /app/gallery-frontend/dist /app/gallery-frontend/dist
COPY --from=frontend-builder /app/gallery-frontend/public /app/gallery-frontend/public

# Add an entrypoint script that will:
# 1. Check if UROCISSA_PATH is set
# 2. Move /app/* to ${UROCISSA_PATH}/* if set
# 3. Run the urocissa binary
WORKDIR /app

# Create the entrypoint script
RUN echo '#!/bin/sh' > /entrypoint.sh && \
    echo 'set -e' >> /entrypoint.sh && \
    echo 'if [ -z "${UROCISSA_PATH}" ]; then' >> /entrypoint.sh && \
    echo '    echo "Error: UROCISSA_PATH is not set. Terminating."' >> /entrypoint.sh && \
    echo '    exit 1' >> /entrypoint.sh && \
    echo 'else' >> /entrypoint.sh && \
    echo '    mkdir -p "${UROCISSA_PATH}/gallery-backend"' >> /entrypoint.sh && \
    echo '    mkdir -p "${UROCISSA_PATH}/gallery-frontend"' >> /entrypoint.sh && \
    echo '    mv /app/gallery-backend/* "${UROCISSA_PATH}/gallery-backend"' >> /entrypoint.sh && \
    echo '    mv /app/gallery-frontend/* "${UROCISSA_PATH}/gallery-frontend"' >> /entrypoint.sh && \
    echo '    echo "Listing ${UROCISSA_PATH}/gallery-backend:"' >> /entrypoint.sh && \
    echo '    ls -al "${UROCISSA_PATH}/gallery-backend"' >> /entrypoint.sh && \
    echo '    echo "Listing ${UROCISSA_PATH}/gallery-frontend:"' >> /entrypoint.sh && \
    echo '    ls -al "${UROCISSA_PATH}/gallery-frontend"' >> /entrypoint.sh && \
    echo '    cd "${UROCISSA_PATH}/gallery-backend"' >> /entrypoint.sh && \
    echo 'fi' >> /entrypoint.sh && \
    echo 'echo "Attempting to run ./urocissa"' >> /entrypoint.sh && \
    echo 'exec ./urocissa' >> /entrypoint.sh && \
    chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]