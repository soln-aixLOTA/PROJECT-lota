#!/bin/bash

# Configuration
REGISTRY="${REGISTRY:-docker.io}" # Default to Docker Hub instead of NGC
IMAGES_FILE="container_images.txt"
LOG_FILE="container_updates.log"
BACKUP_DIR=".container_backups"
NGC_API_KEY="${NGC_API_KEY:-}" # NGC API key for authentication (optional)
HEALTH_CHECK_RETRIES="${HEALTH_CHECK_RETRIES:-5}"
HEALTH_CHECK_INTERVAL="${HEALTH_CHECK_INTERVAL:-5}"
HEALTH_CHECK_TIMEOUT="${HEALTH_CHECK_TIMEOUT:-30}"
ROLLING_UPDATE_DELAY="${ROLLING_UPDATE_DELAY:-10}"
SECURITY_SCAN="${SECURITY_SCAN:-true}"
REQUIRE_CONFIRMATION="${REQUIRE_CONFIRMATION:-true}"
COMPOSE_FILE="docker-compose.yml"

# NGC configuration guide
NGC_SETUP_GUIDE="
NGC Setup Instructions:
1. Visit https://ngc.nvidia.com and create/login to your account
2. Go to Setup > Get API Key
3. Generate a new API key if needed
4. Export the key: export NGC_API_KEY='your-api-key'
5. Verify you have access to required containers in NGC UI
"

# Log levels
declare -A LOG_LEVELS=([DEBUG]=0 [INFO]=1 [WARN]=2 [ERROR]=3)
CURRENT_LOG_LEVEL=${LOG_LEVEL:-INFO}

# Logging function with levels
log() {
    local level=$1
    local message=$2
    local timestamp=$(date +'%Y-%m-%d %H:%M:%S')

    if [[ ${LOG_LEVELS[$level]} -ge ${LOG_LEVELS[$CURRENT_LOG_LEVEL]} ]]; then
        echo "[$timestamp] [$level] $message" | tee -a "$LOG_FILE"
    fi
}

# Enhanced error handling function
handle_error() {
    local error_msg=$1
    local container_name=$2
    local old_image=$3
    local exit_code=$4
    local new_image=$5

    # Log error with exit code if available
    if [[ -n "$exit_code" ]]; then
        log "ERROR" "$error_msg (Exit Code: $exit_code)"
    else
        log "ERROR" "$error_msg"
    fi

    # Clean up failed image pull if specified
    if [[ -n "$new_image" ]]; then
        log "INFO" "Cleaning up failed image: $new_image"
        docker rmi "$new_image" >/dev/null 2>&1 || true
    fi

    # Attempt rollback if container info is provided
    if [[ -n "$container_name" && -n "$old_image" ]]; then
        log "INFO" "Attempting rollback for $container_name"
        # Stop and remove the failed container
        docker stop "$container_name" >/dev/null 2>&1 || true
        docker rm "$container_name" >/dev/null 2>&1 || true

        # Get the original container config
        local backup_file="$BACKUP_DIR/${container_name}_backup.json"
        if [[ -f "$backup_file" ]]; then
            # Extract run parameters from backup
            local run_args=$(jq -r '.[0].Config.Cmd[]?' "$backup_file" 2>/dev/null | tr '\n' ' ')
            local env_args=$(jq -r '.[0].Config.Env[]?' "$backup_file" 2>/dev/null | sed 's/^/-e /' | tr '\n' ' ')
            local volume_args=$(jq -r '.[0].HostConfig.Binds[]?' "$backup_file" 2>/dev/null | sed 's/^/-v /' | tr '\n' ' ')

            # Run container with original parameters
            if docker run -d --name "$container_name" "$env_args" "$volume_args" "$old_image" "$run_args"; then
                log "INFO" "Successfully rolled back $container_name to $old_image"
                return 0
            fi
        else
            # Fallback to simple container run if no backup
            if docker run -d --name "$container_name" "$old_image"; then
                log "INFO" "Successfully rolled back $container_name to $old_image (basic configuration)"
                return 0
            fi
        fi
        log "ERROR" "Failed to rollback $container_name"
        return 1
    fi
}

# Verify prerequisites
verify_prerequisites() {
    # Check if docker is installed
    if ! command -v docker &>/dev/null; then
        log "ERROR" "Docker is not installed"
        exit 1
    fi

    # Check if docker-compose is installed if a compose file exists
    if [ -f "$COMPOSE_FILE" ] && ! command -v docker-compose &>/dev/null; then
        log "ERROR" "Docker Compose is not installed, but $COMPOSE_FILE exists."
        exit 1
    fi

    # Check NVIDIA runtime and Jetson environment
    verify_nvidia_environment

    # Check if images file exists and is not empty
    if [ ! -f "$IMAGES_FILE" ]; then
        log "ERROR" "Images file ($IMAGES_FILE) not found"
        exit 1
    fi

    if [ ! -s "$IMAGES_FILE" ]; then
        log "ERROR" "Images file ($IMAGES_FILE) is empty"
        exit 1
    fi

    # Create backup directory if it doesn't exist
    mkdir -p "$BACKUP_DIR"
}

# Verify NVIDIA environment and Jetson-specific requirements
verify_nvidia_environment() {
    # Check for Jetson environment
    if [ -f "/etc/nv_tegra_release" ]; then
        log "INFO" "Detected Jetson environment"

        # Check L4T version
        local l4t_version=$(head -n 1 /etc/nv_tegra_release | cut -d' ' -f2)
        log "INFO" "L4T Version: $l4t_version"

        # Check JetPack version if available
        if [ -f "/etc/jetpack_version" ]; then
            local jetpack_version=$(cat /etc/jetpack_version)
            log "INFO" "JetPack Version: $jetpack_version"
        fi

        # Verify NVIDIA container runtime
        if ! docker info 2>/dev/null | grep -q "Runtimes:.*nvidia"; then
            log "ERROR" "NVIDIA runtime not found. This is required for Jetson containers."
            log "ERROR" "Please install nvidia-docker2 package and restart the Docker daemon."
            exit 1
        fi

        # Check if nvidia-container-cli is available
        if ! command -v nvidia-container-cli &>/dev/null; then
            log "ERROR" "nvidia-container-cli not found. This is required for GPU support."
            log "ERROR" "Please install nvidia-container-toolkit package."
            exit 1
        fi

        # Test NVIDIA runtime with a simple CUDA container
        log "INFO" "Testing NVIDIA runtime..."
        if ! docker run --rm --runtime=nvidia --network=none nvcr.io/nvidia/cuda:12.3.1-base-ubuntu22.04 nvidia-smi &>/dev/null; then
            log "ERROR" "NVIDIA runtime test failed. Please check your container toolkit installation."
            exit 1
        fi
    else
        # Not a Jetson device, but still check for NVIDIA runtime
        if ! docker info 2>/dev/null | grep -q "Runtimes:.*nvidia"; then
            log "WARN" "NVIDIA runtime not found. GPU support may not be available"
        fi
    fi
}

# NGC authentication function
authenticate_ngc() {
    # Only authenticate with NGC if we have NGC containers
    if grep -q "nvcr.io" "$IMAGES_FILE"; then
        if [[ -z "$NGC_API_KEY" ]]; then
            log "WARN" "NGC API key not provided. NGC containers will not be accessible."
            log "INFO" "Proceeding with Docker Hub containers only."
            return 0
        fi

        log "INFO" "Logging into NGC registry..."
        if ! docker login nvcr.io -u "\$oauthtoken" -p "$NGC_API_KEY"; then
            log "WARN" "NGC login failed. NGC containers will not be accessible."
            log "INFO" "Proceeding with Docker Hub containers only."
            return 0
        fi
    fi
    return 0
}

# Backup current container state
backup_container() {
    local container=$1
    local backup_file="$BACKUP_DIR/${container}_backup.json"

    docker inspect "$container" >"$backup_file" 2>/dev/null
    if [ $? -eq 0 ]; then
        log "DEBUG" "Backed up container $container configuration to $backup_file"
        return 0
    else
        log "WARN" "Failed to backup container $container"
        return 1
    fi
}

# Check if running in swarm mode
is_swarm_mode() {
    docker info 2>/dev/null | grep -q "Swarm: active"
}

# Check if image exists and is newer
is_image_update_needed() {
    local new_image=$1
    local current_image=$2

    # Get image IDs
    local new_id=$(docker image inspect "$new_image" --format '{{.Id}}' 2>/dev/null)
    local current_id=$(docker image inspect "$current_image" --format '{{.Id}}' 2>/dev/null)

    # Compare IDs
    [[ "$new_id" != "$current_id" ]]
}

# Advanced health check function
check_container_health() {
    local container_name=$1
    local retry_count=0
    local delay=$HEALTH_CHECK_INTERVAL

    while [ $retry_count -lt "$HEALTH_CHECK_RETRIES" ]; do
        # Check if container is running
        if ! docker ps --filter "name=$container_name" --format '{{.Status}}' | grep -q "Up"; then
            # Give container a moment to start up
            sleep 5
            if ! docker ps --filter "name=$container_name" --format '{{.Status}}' | grep -q "Up"; then
                log "WARN" "Container $container_name is not running"
                # Check container logs for startup issues
                docker logs "$container_name" 2>&1 | tail -n 20 >>"$LOG_FILE"
                return 1
            fi
        fi

        # Try to get health check command from container labels
        local health_cmd=$(docker inspect --format '{{index .Config.Labels "health.check.command"}}' "$container_name")
        if [[ -n "$health_cmd" ]]; then
            if eval "$health_cmd"; then
                log "INFO" "Health check passed for $container_name using custom command"
                return 0
            fi
        else
            # Default health check - check if container responds
            if docker inspect --format '{{.State.Health.Status}}' "$container_name" 2>/dev/null | grep -q "healthy"; then
                log "INFO" "Health check passed for $container_name"
                return 0
            fi
        fi

        log "WARN" "Health check attempt $((retry_count + 1)) failed for $container_name"
        sleep "$delay"
        delay=$((delay * 2)) # Exponential backoff
        retry_count=$((retry_count + 1))
    done

    log "ERROR" "Health check failed after $HEALTH_CHECK_RETRIES attempts for $container_name"
    return 1
}

# Security scan function
scan_image() {
    local image=$1
    local scan_failed=0

    if [[ "$SECURITY_SCAN" == "true" ]]; then
        log "INFO" "Scanning image $image for vulnerabilities"

        # Check if running as root
        if docker inspect "$image" --format '{{.Config.User}}' | grep -q "^$\|^0\|^root$"; then
            log "WARN" "Container $image is configured to run as root. Consider using a non-root user."
        fi

        # Check for exposed ports
        local exposed_ports=$(docker inspect "$image" --format '{{.Config.ExposedPorts}}')
        if [[ -n "$exposed_ports" ]]; then
            log "INFO" "Container exposes ports: $exposed_ports"
        fi

        # Primary vulnerability scan with Trivy
        if command -v trivy &>/dev/null; then
            log "INFO" "Running Trivy vulnerability scan..."

            # Create a temporary file for the scan report
            local scan_report=$(mktemp)

            if ! trivy image --quiet --severity HIGH,CRITICAL --format json "$image" >"$scan_report" 2>/dev/null; then
                scan_failed=1
            fi

            # Parse and report vulnerabilities
            if [ -s "$scan_report" ]; then
                local crit_vulns=$(jq -r '[.Results[].Vulnerabilities[] | select(.Severity=="CRITICAL")] | length' "$scan_report")
                local high_vulns=$(jq -r '[.Results[].Vulnerabilities[] | select(.Severity=="HIGH")] | length' "$scan_report")

                if [[ $crit_vulns -gt 0 || $high_vulns -gt 0 ]]; then
                    log "WARN" "Security vulnerabilities found in $image:"
                    log "WARN" "- Critical vulnerabilities: $crit_vulns"
                    log "WARN" "- High vulnerabilities: $high_vulns"
                    log "WARN" "Please review the full scan report and consider updating the image"
                    scan_failed=1
                fi
            fi

            # Cleanup
            rm -f "$scan_report"
        else
            # Fallback to Docker Scout if available
            if docker scout --version &>/dev/null; then
                log "INFO" "Trivy not found, using Docker Scout for vulnerability scanning"
                if ! docker scout quickview "$image" 2>/dev/null; then
                    log "WARN" "Docker Scout scan detected vulnerabilities in $image"
                    scan_failed=1
                fi
            else
                log "WARN" "Neither Trivy nor Docker Scout found. Please install a vulnerability scanner"
                log "WARN" "Recommended: Install Trivy (https://github.com/aquasecurity/trivy#installation)"
            fi
        fi

        # Check for sensitive environment variables
        local env_vars=$(docker inspect "$image" --format '{{.Config.Env}}')
        if echo "$env_vars" | grep -iE '(password|secret|key|token|credential)' &>/dev/null; then
            log "WARN" "Possible sensitive environment variables detected in image"
            scan_failed=1
        fi
    fi

    return $scan_failed
}

# Rolling update for swarm services
update_swarm_service() {
    local service_name=$1
    local new_image=$2

    log "INFO" "Updating swarm service $service_name to image $new_image"
    if ! docker service update \
        --image "$new_image" \
        --update-parallelism 1 \
        --update-delay "${ROLLING_UPDATE_DELAY}s" \
        --update-failure-action rollback \
        "$service_name"; then
        log "ERROR" "Failed to update service $service_name"
        return 1
    fi

    # Wait for service to stabilize
    if ! docker service ls --filter "name=$service_name" --format '{{.Replicas}}' | grep -q "[0-9]*/[0-9]*"; then
        log "ERROR" "Service $service_name failed to stabilize"
        return 1
    fi

    return 0
}

# Manual rolling update for standalone containers
update_standalone_container() {
    local container_name=$1
    local new_image=$2
    local old_image=$3
    local container_config=$4

    # Start new container with temporary name
    local temp_name="${container_name}_new"
    log "INFO" "Starting new container $temp_name"

    if ! eval "docker run -d --name $temp_name $container_config $new_image"; then
        log "ERROR" "Failed to start new container $temp_name"
        return 1
    fi

    # Check health of new container
    if ! check_container_health "$temp_name"; then
        log "ERROR" "New container $temp_name failed health check"
        docker rm -f "$temp_name" 2>/dev/null
        return 1
    fi

    # Stop and remove old container
    log "INFO" "Stopping old container $container_name"
    docker stop "$container_name"
    docker rm "$container_name"

    # Rename new container to original name
    if ! docker rename "$temp_name" "$container_name"; then
        log "ERROR" "Failed to rename container $temp_name to $container_name"
        return 1
    fi

    return 0
}

# Get container version information
get_container_version() {
    local image=$1
    local version_info

    # Try to get version from image labels
    version_info=$(docker inspect "$image" --format '{{index .Config.Labels "com.nvidia.version"}}' 2>/dev/null)
    if [[ -z "$version_info" ]]; then
        # Fall back to image digest
        version_info=$(docker image inspect "$image" --format '{{.Id}}' 2>/dev/null)
    fi
    echo "$version_info"
}

# Compare versions
is_newer_version() {
    local new_image=$1
    local old_image=$2

    local new_version
    local old_version

    new_version=$(get_container_version "$new_image")
    old_version=$(get_container_version "$old_image")

    [[ "$new_version" != "$old_version" ]]
}

# Get update summary for an image
get_update_summary() {
    local image=$1
    local current_digest=""
    local new_digest=""
    local current_created=""
    local new_created=""
    local summary=""

    # Add docker.io prefix if no registry specified
    if [[ ! "$image" =~ ^[^/]+\.[^/]+/ ]]; then
        image="docker.io/$image"
    fi

    # Get current image info if it exists
    if docker image inspect "$image" &>/dev/null; then
        current_digest=$(docker image inspect "$image" --format '{{.Id}}')
        current_created=$(docker image inspect "$image" --format '{{.Created}}')
    fi

    # Pull image to get new info
    if docker pull "$image" &>/dev/null; then
        new_digest=$(docker image inspect "$image" --format '{{.Id}}')
        new_created=$(docker image inspect "$image" --format '{{.Created}}')
    fi

    # Compare and create summary
    if [[ -n "$current_digest" && -n "$new_digest" ]]; then
        if [[ "$current_digest" != "$new_digest" ]]; then
            summary="Update available: $(date -d "$current_created" "+%Y-%m-%d") → $(date -d "$new_created" "+%Y-%m-%d")"
        else
            summary="No update needed (latest version)"
        fi
    else
        summary="New image to be pulled"
    fi

    echo "$summary"
}

# User confirmation function
confirm_updates() {
    if [[ "$REQUIRE_CONFIRMATION" != "true" ]]; then
        return 0
    fi

    local total_updates=0
    local update_list=""

    log "INFO" "Checking for updates..."

    # Process each image and build update summary
    while IFS= read -r line || [ -n "$line" ]; do
        # Skip empty lines and comments
        [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue

        local image="$line"
        local summary=$(get_update_summary "$image")

        if [[ "$summary" != "No update needed (latest version)" ]]; then
            total_updates=$((total_updates + 1))
            update_list+="  - $image: $summary\n"
        fi
    done <"$IMAGES_FILE"

    # If no updates needed, exit early
    if [[ $total_updates -eq 0 ]]; then
        log "INFO" "No updates required for any containers"
        return 1
    fi

    # Display update summary
    echo -e "\nContainer Update Summary:"
    echo -e "------------------------"
    echo -e "$update_list"
    echo -e "Total containers to update: $total_updates\n"

    # Check if docker-compose file exists
    if [ -f "$COMPOSE_FILE" ]; then
        echo -e "\nDocker Compose managed services will be updated using 'docker-compose up'."
    fi

    # Show additional information for Jetson environment
    if [ -f "/etc/nv_tegra_release" ]; then
        echo -e "Jetson Environment Details:"
        echo -e "-------------------------"
        echo -e "L4T Version: $(head -n 1 /etc/nv_tegra_release | cut -d' ' -f2)"
        [ -f "/etc/jetpack_version" ] && echo -e "JetPack Version: $(cat /etc/jetpack_version)"
        echo -e "NVIDIA Runtime: $(docker info 2>/dev/null | grep "Runtimes.*nvidia" || echo "Not detected")\n"
    fi

    # Ask for confirmation
    read -p "Do you want to proceed with these updates? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log "INFO" "Update cancelled by user"
        return 1
    fi

    return 0
}

# Main update function
update_containers() {
    local is_jetson=false
    [ -f "/etc/nv_tegra_release" ] && is_jetson=true
    local updating_compose_managed=false

    # Verify prerequisites and environment
    verify_prerequisites

    # Authenticate with NGC if needed
    authenticate_ngc

    # Get confirmation before proceeding
    if ! confirm_updates; then
        return 1
    fi

    # Track successful and failed updates
    local successful_updates=()
    local failed_updates=()

    # Process each image
    while IFS= read -r line || [ -n "$line" ]; do
        # Skip empty lines and comments
        [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue

        local image="$line"
        local container_name=$(echo "$image" | awk -F/ '{print $NF}' | awk -F: '{print $1}')
        local update_success=true

        log "INFO" "Processing container: $container_name ($image)"

        # Check if the container is managed by docker-compose
        if [ -f "$COMPOSE_FILE" ] && docker-compose -f "$COMPOSE_FILE" ps -q "$container_name" > /dev/null 2>&1; then
            updating_compose_managed=true
            log "INFO" "Container $container_name is managed by Docker Compose. Updating using docker-compose."
            log "INFO" "Pulling image: $image"
            if ! docker-compose -f "$COMPOSE_FILE" pull "$container_name"; then
                log "ERROR" "Failed to pull image $image for service $container_name"
                failed_updates+=("$image")
                continue
            fi

            # Scan image for vulnerabilities
            if ! scan_image "$image"; then
                log "WARN" "Security scan failed for $image"
                if [[ "$SECURITY_SCAN" == "true" ]]; then
                    read -p "Continue despite security warnings? (y/N) " -n 1 -r
                    echo
                    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                        failed_updates+=("$image")
                        continue
                    fi
                fi
            fi

            log "INFO" "Updating service: $container_name"
            if ! docker-compose -f "$COMPOSE_FILE" up -d --no-deps --force-recreate "$container_name"; then
                log "ERROR" "Failed to update service $container_name using docker-compose"
                update_success=false
            fi

            # Health check for docker-compose managed services
            if ! docker-compose -f "$COMPOSE_FILE" ps -q "$container_name" | xargs docker inspect --format='{{.State.Health.Status}}' 2>/dev/null | grep -q "healthy"; then
                log "ERROR" "Health check failed for Docker Compose service: $container_name"
                update_success=false
            fi

        else
            # Handle standalone containers
            # Backup existing container if it exists
            if docker ps -a --format '{{.Names}}' | grep -q "^${container_name}$"; then
                backup_container "$container_name" || log "WARN" "Failed to backup container $container_name"
            fi

            # Pull the new image
            log "INFO" "Pulling image: $image"
            if ! docker pull "$image"; then
                log "ERROR" "Failed to pull image: $image"
                failed_updates+=("$image")
                continue
            fi

            # Scan image for vulnerabilities
            if ! scan_image "$image"; then
                log "WARN" "Security scan failed for $image"
                if [[ "$SECURITY_SCAN" == "true" ]]; then
                    read -p "Continue despite security warnings? (y/N) " -n 1 -r
                    echo
                    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                        failed_updates+=("$image")
                        continue
                    fi
                fi
            fi

            # Stop and remove existing container if it exists
            if docker ps -a --format '{{.Names}}' | grep -q "^${container_name}$"; then
                log "INFO" "Stopping existing container: $container_name"
                docker stop "$container_name" || log "WARN" "Failed to stop container $container_name"
                docker rm "$container_name" || log "WARN" "Failed to remove container $container_name"
            fi

            # Determine if we're in swarm mode
            if is_swarm_mode; then
                if ! update_swarm_service "$container_name" "$image"; then
                    update_success=false
                fi
            else
                # Get existing container configuration if available
                local container_config=""
                if [ -f "$BACKUP_DIR/${container_name}_backup.json" ]; then
                    container_config=$(jq -r '.[0].Config' "$BACKUP_DIR/${container_name}_backup.json")
                fi

                # Prepare run command with appropriate runtime
                local run_cmd="docker run -d"
                if $is_jetson; then
                    run_cmd+=" --runtime=nvidia"
                    # Add Jetson-specific device mounts
                    run_cmd+=" --device=/dev/nvhost-ctrl"
                    run_cmd+=" --device=/dev/nvhost-ctrl-gpu"
                    run_cmd+=" --device=/dev/nvhost-prof-gpu"
                    run_cmd+=" --device=/dev/nvmap"
                    run_cmd+=" --device=/dev/nvhost-gpu"
                    run_cmd+=" --device=/dev/nvhost-as-gpu"
                elif docker info 2>/dev/null | grep -q "Runtimes:.*nvidia"; then
                    run_cmd+=" --runtime=nvidia"
                fi

                # Add container configuration if available
                if [ -n "$container_config" ]; then
                    # Extract and add volume mounts
                    local volumes=$(echo "$container_config" | jq -r '.Volumes | keys[]' 2>/dev/null)
                    for volume in $volumes; do
                        run_cmd+=" -v $volume:$volume"
                    done

                    # Extract and add environment variables
                    local envs=$(echo "$container_config" | jq -r '.Env[]' 2>/dev/null)
                    for env in $envs; do
                        run_cmd+=" -e $env"
                    done

                    # Extract and add exposed ports
                    local ports=$(echo "$container_config" | jq -r '.ExposedPorts | keys[]' 2>/dev/null)
                    for port in $ports; do
                        run_cmd+=" -p $port:$port"
                    done
                fi

                # Start the new container
                run_cmd+=" --name $container_name $image"
                log "INFO" "Starting new container: $container_name"
                if ! eval "$run_cmd"; then
                    log "ERROR" "Failed to start container: $container_name"
                    update_success=false
                fi

                # Verify container health
                if ! check_container_health "$container_name"; then
                    log "ERROR" "Container health check failed: $container_name"
                    update_success=false
                fi
            fi
        fi

        if $update_success; then
            successful_updates+=("$image")
            log "INFO" "Successfully updated container: $container_name"
        else
            failed_updates+=("$image")
            log "ERROR" "Failed to update container: $container_name"

            # Attempt rollback if backup exists and not a docker-compose managed service
            if [ -f "$BACKUP_DIR/${container_name}_backup.json" ] && ! updating_compose_managed; then
                log "INFO" "Attempting rollback for $container_name"
                if docker run --name "$container_name" $(jq -r '.[0].Config.Image' "$BACKUP_DIR/${container_name}_backup.json"); then
                    log "INFO" "Successfully rolled back $container_name"
                else
                    log "ERROR" "Failed to rollback $container_name"
                fi
            fi
        fi
        updating_compose_managed=false # Reset flag for the next container
    done <"$IMAGES_FILE"

    # Print summary
    echo -e "\nUpdate Summary:"
    echo -e "---------------"
    if [ ${#successful_updates[@]} -gt 0 ]; then
        echo -e "\nSuccessfully updated:"
        printf '%s\n' "${successful_updates[@]}"
    fi
    if [ ${#failed_updates[@]} -gt 0 ]; then
        echo -e "\nFailed updates:"
        printf '%s\n' "${failed_updates[@]}"
        return 1
    fi

    return 0
}

# Main execution
main() {
    # Initialize log file
    echo "Container Update Log - $(date)" >"$LOG_FILE"

    # Set up trap for cleanup
    trap 'log "ERROR" "Script interrupted"; exit 1' INT TERM

    # Run the update process
    if update_containers; then
        log "INFO" "Container update process completed successfully"
        exit 0
    else
        log "ERROR" "Container update process completed with errors"
        exit 1
    fi
}

main "$@"
