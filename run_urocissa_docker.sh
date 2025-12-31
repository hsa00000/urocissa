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
    # 初始化變數
    local RAW_PATHS_LIST=()
    local CONFIG_SOURCE="None" # 用來記錄來源是 config.json 還是 .env
    local CONFIG_FILE="./gallery-backend/config.json"
    
    # ============================================================
    # 1. 嘗試從 config.json 讀取 syncPaths
    # ============================================================
    if [[ -f "$CONFIG_FILE" ]]; then
        # 壓縮 JSON 為一行並移除 CR 符號
        local FLAT_JSON
        FLAT_JSON=$(cat "$CONFIG_FILE" | tr -d '\n' | tr -d '\r')

        # 檢查是否有 syncPaths 欄位
        if echo "$FLAT_JSON" | grep -q '"syncPaths"'; then
             local JSON_CONTENT
             # 提取 [ ... ] 內的內容
             JSON_CONTENT=$(echo "$FLAT_JSON" | sed -e 's/.*"syncPaths"[[:space:]]*:[[:space:]]*\[//' -e 's/\].*//')
             
             # 如果內容不為空
             if [[ -n "$JSON_CONTENT" ]]; then
                CONFIG_SOURCE="config.json"
                
                # 以逗號分隔處理陣列元素
                IFS=',' read -ra JSON_ARR <<< "$JSON_CONTENT"
                for item in "${JSON_ARR[@]}"; do
                    # 1. 移除雙引號
                    # 2. 移除前後空白
                    # 3. 將 Windows 雙反斜線 (\\) 或單反斜線 (\) 轉為 Linux 斜線 (/)
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
    # 2. 如果 config.json 沒找到有效內容，回退讀取 .env
    # ============================================================
    if [[ "$CONFIG_SOURCE" == "None" ]]; then
        SYNC_PATH=$(grep -E '^SYNC_PATH\s*=\s*' ./gallery-backend/.env | sed 's/^SYNC_PATH\s*=\s*//')
        if [[ -n "$SYNC_PATH" ]]; then
            CONFIG_SOURCE=".env"
            
            # 移除字串前後的引號
            SYNC_PATH="${SYNC_PATH%\"}"
            SYNC_PATH="${SYNC_PATH#\"}"

            IFS=',' read -ra PATHS <<<"$SYNC_PATH"
            for path in "${PATHS[@]}"; do
                # 簡單清理空白
                local cleaned_path
                cleaned_path=$(echo "$path" | xargs)
                if [[ -n "$cleaned_path" ]]; then
                    RAW_PATHS_LIST+=("$cleaned_path")
                fi
            done
        fi
    fi

    # ============================================================
    # 3. [新增] 輸出 Log 資訊 (顯示來源與讀取到的路徑)
    # ============================================================
    echo "============================================================"
    echo " [Volume Configuration Info]"
    echo " Source Type : $CONFIG_SOURCE"
    if [ ${#RAW_PATHS_LIST[@]} -eq 0 ]; then
         echo " Paths Found : (None)"
    else
         echo " Paths Found :"
         for raw_p in "${RAW_PATHS_LIST[@]}"; do
             echo "   - $raw_p"
         done
    fi
    echo "============================================================"

    # ============================================================
    # 4. 統一處理路徑並掛載 (realpath 與 存在性檢查)
    # ============================================================
    for path in "${RAW_PATHS_LIST[@]}"; do
        local abs_path
        
        if [[ "$path" = /* ]]; then
            # 如果已經是絕對路徑 (Linux 格式)
            abs_path=$(realpath -m "$path")
        else
            # 相對路徑：預設相對於 gallery-backend 資料夾
            abs_path=$(realpath -m "./gallery-backend/$path")
        fi

        debug_log "Processing volume mount: $path -> $abs_path"

        # 檢查路徑是否存在，不存在則報錯並退出
        if [[ ! -e "$abs_path" ]]; then
            echo "Error: Path '$abs_path' (derived from '$path') does not exist."
            echo "       Check your $CONFIG_SOURCE configuration."
            exit 1
        fi

        DYNAMIC_VOLUMES+=("$abs_path:$abs_path")
    done

    # ============================================================
    # 5. 加入預定義 Volume
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
