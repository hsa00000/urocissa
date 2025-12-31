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

# Create the entrypoint script using HEREDOC
# 使用 'EOF' (單引號) 確保變數是在 runtime 才展開，而不是 build time
RUN cat <<'EOF' > /entrypoint.sh
#!/bin/sh
set -e

# 定義顏色，讓 Log 好讀
CYAN='\033[0;36m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${CYAN}[ENTRYPOINT]${NC} $1"; }
log_err()  { echo -e "${RED}[ERROR]${NC}      $1"; }

# 專門用來顯示 ls -al 的函式，帶有標題與分隔線
list_directory() {
    local dir="$1"
    echo -e "${YELLOW}┌──────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${YELLOW}│ Listing: ${NC}${dir}"
    echo -e "${YELLOW}└──────────────────────────────────────────────────────────────┘${NC}"
    if [ -d "$dir" ]; then
        ls -al "$dir"
    else
        echo -e "${RED}(Directory not found)${NC}"
    fi
    echo "" # 空一行
}

# -----------------------------------------------------------------------------
# 1. 環境檢查
# -----------------------------------------------------------------------------
if [ -z "${UROCISSA_PATH}" ]; then
    log_err "UROCISSA_PATH is not set. Terminating."
    exit 1
fi

log_info "Target Root: ${UROCISSA_PATH}"

# -----------------------------------------------------------------------------
# 2. 準備檔案 (使用 cp -r 確保容器重啟時來源不會消失)
# -----------------------------------------------------------------------------
log_info "Syncing application files..."

mkdir -p "${UROCISSA_PATH}/gallery-backend"
mkdir -p "${UROCISSA_PATH}/gallery-frontend"

# 嘗試將檔案複製過去 (若已存在則覆蓋，確保版本最新)
if [ -d "/app/gallery-backend" ]; then
    cp -r /app/gallery-backend/. "${UROCISSA_PATH}/gallery-backend/"
else
    log_err "/app/gallery-backend missing!"
fi

if [ -d "/app/gallery-frontend" ]; then
    cp -r /app/gallery-frontend/. "${UROCISSA_PATH}/gallery-frontend/"
else
    log_err "/app/gallery-frontend missing!"
fi

# -----------------------------------------------------------------------------
# 3. 顯示詳細列表 (Debug 重點)
# -----------------------------------------------------------------------------
log_info "Verifying file permissions and ownership:"

list_directory "${UROCISSA_PATH}/gallery-backend"
list_directory "${UROCISSA_PATH}/gallery-frontend"

# -----------------------------------------------------------------------------
# 4. 啟動程式
# -----------------------------------------------------------------------------
cd "${UROCISSA_PATH}/gallery-backend"

log_info "Launching ./urocissa..."
echo -e "${GREEN}============================================================${NC}"
exec ./urocissa
EOF

RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]