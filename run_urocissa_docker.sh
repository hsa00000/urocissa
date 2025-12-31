#!/bin/bash

# ============================================================
# Function Definitions
# ============================================================

show_help() {
    cat <<EOF
Usage: ./run_urocissa_docker.sh [OPTIONS]

Description:
  This script runs the Urocissa Docker container from an already built image.
  It sets up environment variables, volumes, and port mappings based on the local configuration files.

Options:
  --help              Show this help message and exit.
  --debug             Enable debug mode to display additional information during execution.
  --dev               Use the 'dev' tag for the Docker image (hsa00000/urocissa:dev).
  --log-file <file>   Specify a log file for debug output. The file will be created if it does not exist,
                      or cleared if it already exists.

Examples:
  1. Run the container with default settings:
     ./run_urocissa_docker.sh

  2. Enable debug mode and specify a log file:
     ./run_urocissa_docker.sh --debug --log-file run.log

Notes:
  - Ensure that the Docker image 'urocissa' is already built by running ./build_urocissa_docker.sh beforehand.
  - Debug mode outputs information to the terminal unless a log file is specified.
  - The script will mount local directories and set UROCISSA_PATH based on the current directory structure.
EOF
}

debug_log() {
    local message="$1"
    if [[ "$DEBUG" == true ]]; then
        if [[ -n "$LOG_FILE" ]]; then
            echo "$message" >>"$LOG_FILE"
        else
            echo "$message"
        fi
    fi
}

ensure_config_file() {
    # This function checks if the target config file exists and adds it to the volume list.
    local target_file="$1"
    local volume_path="${2:-$target_file}"
    
    if [[ -f "$target_file" ]]; then
        if [[ -n "$volume_path" ]]; then
            PREDEFINED_VOLUMES+=("$target_file:$volume_path")
        fi
    else
        debug_log "$target_file not found. Skipping volume mount."
    fi
}

parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
        --help)
            show_help
            exit 0
            ;;
        --debug)
            DEBUG=true
            shift
            ;;
        --dev)
            DOCKER_TAG="dev"
            shift
            ;;
        --log-file)
            LOG_FILE="$2"
            >"$LOG_FILE"
            if [[ $? -ne 0 ]]; then
                echo "Error: Failed to initialize log file at $LOG_FILE"
                exit 1
            fi
            shift 2
            ;;
        *)
            echo "Error: Unknown option $1"
            exit 1
            ;;
        esac
    done
}

setup_environment() {
    SCRIPT_DIR=$(dirname "$(realpath "$0")")
    UROCISSA_PATH="$SCRIPT_DIR"

    debug_log "Script directory set to $SCRIPT_DIR"

    ENV_FILE="./gallery-backend/.env"

    # Initialize arrays
    PREDEFINED_VOLUMES=()
    DYNAMIC_VOLUMES=()

    # Ensure config files exist and mount them
    ensure_config_file "./gallery-backend/Rocket.toml" "${UROCISSA_PATH}/gallery-backend/Rocket.toml"
    ensure_config_file "./gallery-backend/config.json" "${UROCISSA_PATH}/gallery-backend/config.json"
    ensure_config_file "$ENV_FILE" "${UROCISSA_PATH}/gallery-backend/.env"

    # Ensure the upload folder exists
    UPLOAD_DIR="./gallery-backend/upload"
    if [ ! -d "$UPLOAD_DIR" ]; then
        mkdir -p "$UPLOAD_DIR"
        debug_log "Created upload directory at $UPLOAD_DIR"
    else
        debug_log "Upload directory already exists at $UPLOAD_DIR"
    fi

    # Convert CRLF to LF
    sed -i 's/\r$//' "$ENV_FILE"
}

prepare_volumes() {
    # 初始化路徑陣列
    local RAW_PATHS_LIST=()
    local FOUND_IN_CONFIG=false
    local CONFIG_FILE="./gallery-backend/config.json"
    
    # ============================================================
    # 1. 嘗試從 config.json 讀取 syncPaths
    # ============================================================
    if [[ -f "$CONFIG_FILE" ]]; then
        # 將檔案內容壓縮成一行以處理多行 JSON，並刪除 carriage return (\r)
        local FLAT_JSON
        FLAT_JSON=$(cat "$CONFIG_FILE" | tr -d '\n' | tr -d '\r')

        # 使用 sed 匹配 "syncPaths": [ ... ] 內部的內容
        # 解釋：
        #   s/.*"syncPaths"[[:space:]]*:[[:space:]]*\[//  -> 刪除 "syncPaths": [ 之前的所有內容
        #   s/\].*//                                      -> 刪除 ] 之後的所有內容 (閉合陣列)
        #   注意：這假設 syncPaths 後面的 ] 是該陣列的結尾，對於簡單結構有效
        if echo "$FLAT_JSON" | grep -q '"syncPaths"'; then
             local JSON_CONTENT
             JSON_CONTENT=$(echo "$FLAT_JSON" | sed -e 's/.*"syncPaths"[[:space:]]*:[[:space:]]*\[//' -e 's/\].*//')
             
             # 如果內容不為空
             if [[ -n "$JSON_CONTENT" ]]; then
                debug_log "Found syncPaths in config.json: $JSON_CONTENT"
                FOUND_IN_CONFIG=true
                
                # 以逗號分隔處理陣列元素
                IFS=',' read -ra JSON_ARR <<< "$JSON_CONTENT"
                for item in "${JSON_ARR[@]}"; do
                    # 1. 移除雙引號
                    # 2. 移除前後空白 (xargs)
                    # 3. 將 Windows 反斜線 (\\ 或 \) 轉換為 Forward Slash (/)
                    local cleaned_path
                    cleaned_path=$(echo "$item" | tr -d '"' | xargs | sed 's|\\\\|/|g' | sed 's|\\|/|g')
                    
                    if [[ -n "$cleaned_path" ]]; then
                        RAW_PATHS_LIST+=("$cleaned_path")
                    fi
                done
             fi
        fi
    fi

    # ============================================================
    # 2. 如果 config.json 沒找到，回退讀取 .env
    # ============================================================
    if [[ "$FOUND_IN_CONFIG" == false ]]; then
        SYNC_PATH=$(grep -E '^SYNC_PATH\s*=\s*' ./gallery-backend/.env | sed 's/^SYNC_PATH\s*=\s*//')
        if [[ -n "$SYNC_PATH" ]]; then
            # 移除引號
            SYNC_PATH="${SYNC_PATH%\"}"
            SYNC_PATH="${SYNC_PATH#\"}"
            debug_log "Using SYNC_PATH from .env: $SYNC_PATH"

            IFS=',' read -ra PATHS <<<"$SYNC_PATH"
            for path in "${PATHS[@]}"; do
                RAW_PATHS_LIST+=("$(echo "$path" | xargs)")
            done
        else
            debug_log "Warning: No sync paths found in config.json or .env."
        fi
    fi

    # ============================================================
    # 3. 統一處理路徑並掛載 (realpath 與 存在性檢查)
    # ============================================================
    for path in "${RAW_PATHS_LIST[@]}"; do
        # 處理相對路徑與絕對路徑
        local abs_path
        if [[ "$path" = /* ]]; then
            abs_path=$(realpath -m "$path")
        else
            # 這裡假設相對路徑是相對於 gallery-backend 資料夾 (根據原本 .env 的邏輯推測)
            # 但原本代碼中 .env 是相對於 .env 檔案的位置。
            # 這裡統一相對於腳本執行目錄的 ./gallery-backend/.. 或者維持原有的 .env 邏輯
            # 為了保險，我們使用相對於 config/env 檔案所在的目錄 (gallery-backend)
            abs_path=$(realpath -m "./gallery-backend/$path")
        fi

        debug_log "Processing volume mount: $path -> $abs_path"

        # Panic if the path does not exist
        if [[ ! -e "$abs_path" ]]; then
            echo "Error: Path '$abs_path' (derived from '$path') does not exist. Aborting."
            exit 1
        fi

        DYNAMIC_VOLUMES+=("$abs_path:$abs_path")
    done

    # ============================================================
    # 4. 加入預定義 Volume
    # ============================================================
    PREDEFINED_VOLUMES+=( "./gallery-backend/db:${UROCISSA_PATH}/gallery-backend/db" )
    PREDEFINED_VOLUMES+=( "./gallery-backend/object:${UROCISSA_PATH}/gallery-backend/object" )

    debug_log "Predefined volumes: ${PREDEFINED_VOLUMES[*]}"
    debug_log "Dynamic volumes: ${DYNAMIC_VOLUMES[*]}"
}

run_container() {
    # Extract port from Rocket.toml
    ROCKET_PORT=$(grep -E '^port\s*=\s*' ./gallery-backend/Rocket.toml | sed -E 's/^port\s*=\s*"?([0-9]+)"?/\1/' | tr -d '[:space:]')
    ROCKET_PORT=${ROCKET_PORT:-4000}
    debug_log "Using port: $ROCKET_PORT"

    # Generate Docker Run command
    DOCKER_RUN_COMMAND="docker run -it --rm"
    for vol in "${PREDEFINED_VOLUMES[@]}"; do
        DOCKER_RUN_COMMAND+=" -v $vol"
    done
    for vol in "${DYNAMIC_VOLUMES[@]}"; do
        DOCKER_RUN_COMMAND+=" -v $vol"
    done

    DOCKER_RUN_COMMAND+=" -e UROCISSA_PATH=${UROCISSA_PATH}"
    DOCKER_RUN_COMMAND+=" -p ${ROCKET_PORT}:${ROCKET_PORT} hsa00000/urocissa:${DOCKER_TAG}"

    debug_log "Generated Docker Run command: $DOCKER_RUN_COMMAND"
    eval "$DOCKER_RUN_COMMAND"

    if [[ $? -ne 0 ]]; then
        echo "Error: Docker Run command failed to execute"
        exit 1
    else
        debug_log "Docker container has been successfully started"
    fi
}

main() {
    DEBUG=false
    LOG_FILE=""
    DOCKER_TAG="latest"

    parse_arguments "$@"
    setup_environment
    prepare_volumes
    run_container
}

main "$@"
