#!/bin/bash

# Source common functions
source ./lib/common.sh

# Default values from environment if set
URL="${SERVICE_URL:-http://localhost:8080}"
TOKEN=""
SCENARIO="crud"
OUTPUT=""
TEST_MODE="${TEST_MODE:-false}"
VUS="${TEST_VUS:-10}"
DURATION="${TEST_DURATION:-30s}"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --url)
            URL="$2"
            shift 2
            ;;
        --token)
            TOKEN="$2"
            shift 2
            ;;
        --scenario)
            SCENARIO="$2"
            shift 2
            ;;
        --output)
            OUTPUT="$2"
            shift 2
            ;;
        --test)
            TEST_MODE="true"
            shift
            ;;
        --vus)
            VUS="$2"
            shift 2
            ;;
        --duration)
            DURATION="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [options]"
            echo
            echo "Options:"
            echo "  --url URL         Base URL of the service (default: $URL)"
            echo "  --token TOKEN     JWT token for authentication"
            echo "  --scenario TYPE   Test scenario type: crud|workflow|mixed|security (default: crud)"
            echo "  --output TYPE     Output type: prometheus|influxdb (optional)"
            echo "  --test            Run in test mode (skips service checks)"
            echo "  --vus N           Number of virtual users (default: $VUS)"
            echo "  --duration D      Test duration (default: $DURATION)"
            echo "  -h, --help        Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate token
if [ -z "$TOKEN" ]; then
    log_error "No authentication token provided"
    echo "Usage: $0 [options]"
    echo
    echo "Options:"
    echo "  --url URL         Base URL of the service (default: $URL)"
    echo "  --token TOKEN     JWT token for authentication"
    echo "  --scenario TYPE   Test scenario type: crud|workflow|mixed|security (default: crud)"
    echo "  --output TYPE     Output type: prometheus|influxdb (optional)"
    echo "  --test            Run in test mode (skips service checks)"
    echo "  --vus N           Number of virtual users (default: $VUS)"
    echo "  --duration D      Test duration (default: $DURATION)"
    echo "  -h, --help        Show this help message"
    exit 1
fi

log_info "Using token: $TOKEN"

# Validate environment
if [ "$TEST_MODE" != "true" ]; then
    validate_environment
fi

# Check k6
if [ "$TEST_MODE" != "true" ]; then
    check_k6
fi

# Create k6 script
if [ "$TEST_MODE" != "true" ]; then
    cat > test.js << EOL
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
    vus: ${VUS},
    duration: '${DURATION}',
};

const BASE_URL = '${URL}';
const TOKEN = '${TOKEN}';

export default function() {
    const headers = {
        'Authorization': 'Bearer ' + TOKEN,
        'Content-Type': 'application/json',
    };

    switch('${SCENARIO}') {
        case 'crud':
            // CRUD operations
            const createRes = http.post(BASE_URL + '/documents', JSON.stringify({
                name: 'test.pdf',
                content: 'base64content'
            }), { headers });
            check(createRes, { 'create status is 201': (r) => r.status === 201 });

            const listRes = http.get(BASE_URL + '/documents', { headers });
            check(listRes, { 'list status is 200': (r) => r.status === 200 });

            if (createRes.json('id')) {
                const getRes = http.get(BASE_URL + '/documents/' + createRes.json('id'), { headers });
                check(getRes, { 'get status is 200': (r) => r.status === 200 });

                const deleteRes = http.del(BASE_URL + '/documents/' + createRes.json('id'), null, { headers });
                check(deleteRes, { 'delete status is 204': (r) => r.status === 204 });
            }
            break;

        case 'workflow':
            // Workflow operations
            const startRes = http.post(BASE_URL + '/workflows', JSON.stringify({
                name: 'test-workflow',
                steps: ['ocr', 'classify']
            }), { headers });
            check(startRes, { 'start status is 201': (r) => r.status === 201 });

            if (startRes.json('id')) {
                const statusRes = http.get(BASE_URL + '/workflows/' + startRes.json('id'), { headers });
                check(statusRes, { 'status check is 200': (r) => r.status === 200 });
            }
            break;

        case 'mixed':
            // Mixed operations
            if (Math.random() < 0.5) {
                const docRes = http.post(BASE_URL + '/documents', JSON.stringify({
                    name: 'test.pdf',
                    content: 'base64content'
                }), { headers });
                check(docRes, { 'document create is 201': (r) => r.status === 201 });
            } else {
                const wfRes = http.post(BASE_URL + '/workflows', JSON.stringify({
                    name: 'test-workflow',
                    steps: ['ocr', 'classify']
                }), { headers });
                check(wfRes, { 'workflow start is 201': (r) => r.status === 201 });
            }
            break;

        case 'security':
            // Security operations
            const validRes = http.get(BASE_URL + '/documents', { headers });
            check(validRes, { 'valid token is 200': (r) => r.status === 200 });

            const invalidRes = http.get(BASE_URL + '/documents', {
                headers: { 'Authorization': 'Bearer invalid' }
            });
            check(invalidRes, { 'invalid token is 401': (r) => r.status === 401 });
            break;
    }

    sleep(1);
}
EOL

    # Run test
    log_info "Running $SCENARIO scenario..."
    if [ -n "$OUTPUT" ]; then
        docker run --rm -v $(pwd):/scripts grafana/k6 run --out "$OUTPUT" /scripts/test.js
    else
        docker run --rm -v $(pwd):/scripts grafana/k6 run /scripts/test.js
    fi

    # Validate results
    validate_test_results "$output" "$SCENARIO"
else
    log_info "Running in test mode, skipping actual test execution"
fi 