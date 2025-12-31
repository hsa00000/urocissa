#!/bin/bash

# ============================================================
# Color & Logging Configuration
# ============================================================
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log_header() {
    echo -e "${BLUE}============================================================${NC}"
    echo -e "${BLUE}# $1${NC}"
    echo -e "${BLUE}============================================================${NC}"
}

log_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

log_debug() {
    if [[ "$DEBUG" == true ]]; then
        local msg="[DEBUG] $1"
        if [[ -n "$LOG_FILE" ]]; then
            echo "$msg" >>"$LOG_FILE"
        fi
        echo -e "${YELLOW}$msg${NC}"
    fi
}

# ============================================================
# Function Definitions
# ============================================================

show_help() {
    cat <<EOF
Usage: ./run_urocissa_docker.sh [OPTIONS]

Description:
  This script runs the Urocissa Docker container with enhanced logging.

Options:
  --help              Show this help message and exit.
  --debug             Enable debug mode (verbose output).
  --dev               Use the 'dev' tag (hsa00000/urocissa:dev).
  --log-file <file>   Specify a log file for debug output.

Examples:
  ./run_urocissa_docker.sh
  ./run_urocissa_docker.sh --debug --log-file run.log
EOF
}

ensure_config_file() {
    local target_file="$1"
    local volume_path="${2:-$target_file}"
    
    if [[ -f "$target_file" ]]; then
        if [[ -n "$volume_path" ]]; then
            PREDEFINED_VOLUMES+=("$target_file:$volume_path")
            log_success "Config found: $target_file -> Mounted"
        fi
    else
        log_warn "Config missing: $target_file -> Skipped"
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
            if ! : > "$LOG_FILE"; then
                log_error "Failed to initialize log file at $LOG_FILE"
            fi
            shift 2
            ;;
        *)
            log_error "Unknown option $1"
            ;;
        esac
    done
}

setup_environment() {
    log_header "Step 1: Environment Setup"
    
    SCRIPT_DIR=$(dirname "$(realpath "$0")")
    UROCISSA_PATH="$SCRIPT_DIR"
    log_info "Project Root: $UROCISSA_PATH"

    ENV_FILE="./gallery-backend/.env"

    # Initialize arrays
    PREDEFINED_VOLUMES=()
    DYNAMIC_VOLUMES=()

    # Ensure config files exist and mount them
    log_info "Checking configuration files..."
    ensure_config_file "./gallery-backend/Rocket.toml" "${UROCISSA_PATH}/gallery-backend/Rocket.toml"

    # Special handling for config.json: Create if missing
    if [[ ! -f "./gallery-backend/config.json" ]]; then
        echo "{}" > "./gallery-backend/config.json"
        log_success "Created empty config.json"
    fi
    ensure_config_file "./gallery-backend/config.json" "${UROCISSA_PATH}/gallery-backend/config.json"
    ensure_config_file "$ENV_FILE" "${UROCISSA_PATH}/gallery-backend/.env"

    # Ensure the upload folder exists
    UPLOAD_DIR="./gallery-backend/upload"
    if [ ! -d "$UPLOAD_DIR" ]; then
        mkdir -p "$UPLOAD_DIR"
        log_success "Created directory: $UPLOAD_DIR"
    else
        log_success "Directory exists: $UPLOAD_DIR"
    fi

    # Convert CRLF to LF
    if [[ -f "$ENV_FILE" ]]; then
        sed -i 's/\r$//' "$ENV_FILE"
        log_debug "Converted CRLF to LF for $ENV_FILE"
    fi
}

prepare_volumes() {
    log_header "Step 2: Volume & Path Parsing"

    local RAW_PATHS_LIST=()
    local CONFIG_SOURCE="None" 
    local CONFIG_FILE="./gallery-backend/config.json"
    
    # ------------------------------------------------------------
    # 1. Try to read syncPaths from config.json
    # ------------------------------------------------------------
    if [[ -f "$CONFIG_FILE" ]]; then
        local FLAT_JSON
        FLAT_JSON=$(tr -d '\n\r' < "$CONFIG_FILE")

        if echo "$FLAT_JSON" | grep -q '"syncPaths"'; then
             local JSON_CONTENT
             JSON_CONTENT=$(echo "$FLAT_JSON" | sed -e 's/.*"syncPaths"[[:space:]]*:[[:space:]]*\[//' -e 's/\].*//')
             
             if [[ -n "$JSON_CONTENT" ]]; then
                CONFIG_SOURCE="config.json"
                IFS=',' read -ra JSON_ARR <<< "$JSON_CONTENT"
                for item in "${JSON_ARR[@]}"; do
                    # Cleanup: remove quotes, whitespace, and normalize Windows slashes
                    local cleaned_path
                    cleaned_path=$(echo "$item" | tr -d '"' | xargs | sed 's|\\\\|/|g' | sed 's|\\|/|g')
                    
                    if [[ -n "$cleaned_path" ]]; then
                        RAW_PATHS_LIST+=("$cleaned_path")
                    fi
                done
             fi
        fi
    fi

    # ------------------------------------------------------------
    # 2. Fallback to reading .env
    # ------------------------------------------------------------
    if [[ "$CONFIG_SOURCE" == "None" ]]; then
        if [[ -f "./gallery-backend/.env" ]]; then
            SYNC_PATH=$(grep -E '^SYNC_PATH\s*=\s*' ./gallery-backend/.env | sed 's/^SYNC_PATH\s*=\s*//')
            if [[ -n "$SYNC_PATH" ]]; then
                CONFIG_SOURCE=".env"
                SYNC_PATH="${SYNC_PATH%\"}"
                SYNC_PATH="${SYNC_PATH#\"}"

                IFS=',' read -ra PATHS <<<"$SYNC_PATH"
                for path in "${PATHS[@]}"; do
                    local cleaned_path
                    cleaned_path=$(echo "$path" | xargs)
                    if [[ -n "$cleaned_path" ]]; then
                        RAW_PATHS_LIST+=("$cleaned_path")
                    fi
                done
            fi
        fi
    fi

    # ------------------------------------------------------------
    # 3. Output path detection results (Visual Log)
    # ------------------------------------------------------------
    echo "------------------------------------------------------------"
    echo -e " ${CYAN}Sync Path Source :${NC} $CONFIG_SOURCE"
    if [ ${#RAW_PATHS_LIST[@]} -eq 0 ]; then
         echo -e " ${YELLOW}Paths Found      :${NC} (None)"
    else
         echo -e " ${GREEN}Paths Found      :${NC}"
         for raw_p in "${RAW_PATHS_LIST[@]}"; do
             echo "   - $raw_p"
         done
    fi
    echo "------------------------------------------------------------"

    # ------------------------------------------------------------
    # 4. Validate and convert to absolute path
    # ------------------------------------------------------------
    for path in "${RAW_PATHS_LIST[@]}"; do
        local abs_path
        
        if [[ "$path" = /* ]]; then
            abs_path=$(realpath -m "$path")
        else
            abs_path=$(realpath -m "./gallery-backend/$path")
        fi

        log_debug "Resolving: $path -> $abs_path"

        if [[ ! -e "$abs_path" ]]; then
            echo ""
            log_error "Path not found!\n    Source: $CONFIG_SOURCE\n    Raw:    $path\n    Abs:    $abs_path"
        fi

        DYNAMIC_VOLUMES+=("$abs_path:$abs_path")
    done

    # ------------------------------------------------------------
    # 5. Add predefined Volumes
    # ------------------------------------------------------------
    PREDEFINED_VOLUMES+=( "./gallery-backend/db:${UROCISSA_PATH}/gallery-backend/db" )
    PREDEFINED_VOLUMES+=( "./gallery-backend/object:${UROCISSA_PATH}/gallery-backend/object" )
    
    log_success "Volume preparation complete."
}

run_container() {
    log_header "Step 3: Docker Execution"

    # Extract port
    if [[ -f "./gallery-backend/Rocket.toml" ]]; then
        ROCKET_PORT=$(grep -E '^port\s*=\s*' ./gallery-backend/Rocket.toml | sed -E 's/^port\s*=\s*"?([0-9]+)"?/\1/' | tr -d '[:space:]')
    fi
    ROCKET_PORT=${ROCKET_PORT:-5673}
    log_info "Service Port: $ROCKET_PORT"
    log_info "Image Tag   : $DOCKER_TAG"

    # Generate Command using Arrays (Safe for spaces)
    local cmd=(docker run -it --rm)
    
    for vol in "${PREDEFINED_VOLUMES[@]}"; do
        cmd+=(-v "$vol")
    done
    for vol in "${DYNAMIC_VOLUMES[@]}"; do
        cmd+=(-v "$vol")
    done

    cmd+=(-e "UROCISSA_PATH=${UROCISSA_PATH}")
    cmd+=(-p "${ROCKET_PORT}:${ROCKET_PORT}")
    cmd+=( "hsa00000/urocissa:${DOCKER_TAG}" )

    # Pretty print the command for debugging
    echo ""
    echo -e "${YELLOW}┌──────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${YELLOW}│ Final Docker Command (Copy below if you need to debug)       │${NC}"
    echo -e "${YELLOW}└──────────────────────────────────────────────────────────────┘${NC}"
    # Print escaped command for easy copy-pasting
    printf "%q " "${cmd[@]}"
    echo ""
    echo ""

    log_info "Starting container..."
    "${cmd[@]}"
    local exit_code=$?

    if [[ $exit_code -ne 0 ]]; then
        log_error "Docker container exited with error code $exit_code"
    else
        log_header "Execution Finished Successfully"
    fi
}

main() {
    DEBUG=false
    LOG_FILE=""
    DOCKER_TAG="latest"

    # Catch Ctrl+C
    trap 'echo -e "\n${RED}[ABORT] Script interrupted by user.${NC}"; exit 1' INT

    parse_arguments "$@"
    
    setup_environment
    prepare_volumes
    run_container
}

main "$@"