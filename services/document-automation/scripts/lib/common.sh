#!/bin/bash

# Script to run load tests for Document Automation Service using k6

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'  # No Color

# Log functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Validate environment
validate_environment() {
    log_info "Validating environment..."
    
    # Check if docker is running
    if ! docker ps &> /dev/null; then
        log_error "Docker is not running"
        exit 1
    fi
    
    # Check if service is running
    if ! curl -s "http://localhost:8080/health" &> /dev/null; then
        log_error "Document Automation Service is not running"
        exit 1
    fi
    
    # Check database connection via db-manage script
    if [ -f "./db-manage.sh" ]; then
        ./db-manage.sh check &> /dev/null || {
            log_error "Database connection failed"
            exit 1
        }
    fi
    
    log_info "Environment validation successful"
}

# Add test validation function
validate_test_results() {
    local test_output="$1"
    local scenario="$2"
    
    log_info "Validating test results for scenario: $scenario"
    
    # Check for common error patterns
    if echo "$test_output" | grep -q "connection refused"; then
        log_error "Service connection failed during test"
        return 1
    fi
    
    if echo "$test_output" | grep -q "invalid token"; then
        log_warn "Authentication issues detected during test"
    fi
    
    # Validate scenario-specific metrics
    case "$scenario" in
        crud)
            if ! echo "$test_output" | grep -q "document_uploads"; then
                log_error "No document uploads recorded"
                return 1
            fi
            ;;
        workflow)
            if ! echo "$test_output" | grep -q "workflow_completions"; then
                log_error "No workflow completions recorded"
                return 1
            fi
            ;;
        security)
            if ! echo "$test_output" | grep -q "auth_successes"; then
                log_error "No successful authentication attempts recorded"
                return 1
            fi
            ;;
    esac
    
    return 0
}

# Check k6 installation
check_k6() {
    if ! command -v k6 &> /dev/null; then
        log_error "k6 is not installed"
        log_info "Install k6 using: docker pull grafana/k6"
        exit 1
    fi
}

# Create test file
create_test_file() {
    local scenario="$1"
    local output_type="$2"
    
    case "$scenario" in
        crud)
            create_crud_test
            ;;
        workflow)
            create_workflow_test
            ;;
        mixed)
            create_mixed_test
            ;;
        security)
            create_security_test
            ;;
        *)
            log_error "Unknown scenario: $scenario"
            exit 1
            ;;
    esac
    
    # Add output configuration if specified
    if [ -n "$output_type" ]; then
        add_output_config "$output_type"
    fi
    
    log_info "Created load test script: load-test.js"
}

# Create CRUD test scenario
create_crud_test() {
    cat > load-test.js << 'EOL'
import http from 'k6/http';
import { check, sleep } from 'k6';
import { randomString } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';
import { Counter, Rate, Trend } from 'k6/metrics';

// Custom metrics
const documentUploads = new Counter('document_uploads');
const documentDownloads = new Counter('document_downloads');
const uploadDuration = new Trend('upload_duration');
const downloadDuration = new Trend('download_duration');
const errorRate = new Rate('errors');

// Test configuration
export const options = {
    scenarios: {
        crud_test: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '1m', target: 10 },  // Ramp up
                { duration: '3m', target: 10 },  // Steady load
                { duration: '1m', target: 0 },   // Ramp down
            ],
        },
    },
    thresholds: {
        'http_req_duration': ['p(95)<500'],
        'http_req_failed': ['rate<0.01'],
        'upload_duration': ['p(95)<1000'],
        'download_duration': ['p(95)<500'],
        'errors': ['rate<0.01'],
    },
};

// Test setup
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || 'your-jwt-token';

// Helper function to create test document
function createTestDocument() {
    const content = `Test document ${randomString(8)}`;
    return {
        filename: `test-${randomString(8)}.txt`,
        content: content,
    };
}

// Main test scenario
export default function () {
    const headers = {
        'Authorization': `Bearer ${AUTH_TOKEN}`,
    };

    try {
        // Create document
        const doc = createTestDocument();
        const data = {
            file: http.file(doc.content, doc.filename, 'text/plain'),
        };
        
        const uploadStart = new Date();
        const uploadResponse = http.post(
            `${BASE_URL}/documents`,
            data,
            {
                headers: headers,
            }
        );
        uploadDuration.add(new Date() - uploadStart);
        
        check(uploadResponse, {
            'upload successful': (r) => r.status === 201,
            'has document id': (r) => r.json('id') !== undefined,
        }) || errorRate.add(1);

        if (uploadResponse.status === 201) {
            documentUploads.add(1);
            const docId = uploadResponse.json('id');

            // List documents
            const listResponse = http.get(
                `${BASE_URL}/documents`,
                { headers: headers }
            );
            
            check(listResponse, {
                'list successful': (r) => r.status === 200,
                'contains documents': (r) => r.json('documents').length > 0,
            }) || errorRate.add(1);

            // Get document
            const downloadStart = new Date();
            const getResponse = http.get(
                `${BASE_URL}/documents/${docId}`,
                { headers: headers }
            );
            downloadDuration.add(new Date() - downloadStart);
            
            check(getResponse, {
                'get successful': (r) => r.status === 200,
                'content matches': (r) => r.body.includes(doc.content),
            }) || errorRate.add(1);
            
            documentDownloads.add(1);

            // Delete document
            const deleteResponse = http.del(
                `${BASE_URL}/documents/${docId}`,
                null,
                { headers: headers }
            );
            
            check(deleteResponse, {
                'delete successful': (r) => r.status === 204,
            }) || errorRate.add(1);
        }
    } catch (e) {
        errorRate.add(1);
        console.error(e);
    }

    sleep(1);
}
EOL
}

# Create workflow test scenario
create_workflow_test() {
    cat > load-test.js << 'EOL'
import http from 'k6/http';
import { check, sleep } from 'k6';
import { randomString } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';
import { Counter, Rate, Trend } from 'k6/metrics';

// Custom metrics
const workflowStarts = new Counter('workflow_starts');
const workflowCompletions = new Counter('workflow_completions');
const classificationSuccesses = new Counter('classification_successes');
const workflowDuration = new Trend('workflow_duration');
const classificationDuration = new Trend('classification_duration');
const errorRate = new Rate('errors');

// Test configuration
export const options = {
    scenarios: {
        workflow_test: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '1m', target: 5 },   // Ramp up
                { duration: '5m', target: 5 },   // Steady load
                { duration: '1m', target: 0 },   // Ramp down
            ],
        },
    },
    thresholds: {
        'workflow_duration': ['p(95)<5000'],
        'classification_duration': ['p(95)<2000'],
        'http_req_failed': ['rate<0.01'],
        'errors': ['rate<0.01'],
    },
};

// Test setup
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || 'your-jwt-token';

// Helper function to create test document
function createTestDocument() {
    const classifications = ['legal', 'financial', 'medical', 'hr', 'general'];
    const securityLevels = ['public', 'internal', 'confidential', 'restricted'];
    const authors = ['john.doe', 'jane.smith', 'bob.wilson'];
    const departments = ['legal', 'finance', 'hr', 'operations'];
    
    const classification = classifications[Math.floor(Math.random() * classifications.length)];
    const securityLevel = securityLevels[Math.floor(Math.random() * securityLevels.length)];
    const author = authors[Math.floor(Math.random() * authors.length)];
    const department = departments[Math.floor(Math.random() * departments.length)];
    
    const content = `Test document ${randomString(8)}`;
    return {
        filename: `test-${randomString(8)}.txt`,
        content: content,
        metadata: {
            author: author,
            tags: ['test', 'workflow', department, classification],
            classification: classification,
            security_level: securityLevel,
            custom_fields: {
                priority: Math.floor(Math.random() * 3) + 1,
                department: department,
                project_code: `PRJ-${randomString(4)}`,
                review_required: Math.random() > 0.5,
            },
        },
    };
}

// Main test scenario
export default function () {
    const headers = {
        'Authorization': `Bearer ${AUTH_TOKEN}`,
    };

    try {
        // Start workflow
        const doc = createTestDocument();
        const workflowStart = new Date();
        workflowStarts.add(1);
        
        // Upload document with metadata
        const data = {
            file: http.file(doc.content, doc.filename, 'text/plain'),
            metadata: JSON.stringify(doc.metadata),
        };
        
        const uploadResponse = http.post(
            `${BASE_URL}/documents`,
            data,
            { headers: headers }
        );
        
        check(uploadResponse, {
            'upload successful': (r) => r.status === 201,
            'has document id': (r) => r.json('id') !== undefined,
        }) || errorRate.add(1);

        if (uploadResponse.status === 201) {
            const docId = uploadResponse.json('id');
            
            // Poll for workflow completion and classification
            let attempts = 0;
            let completed = false;
            const classificationStart = new Date();
            
            while (attempts < 10 && !completed) {
                const statusResponse = http.get(
                    `${BASE_URL}/documents/${docId}`,
                    { headers: headers }
                );
                
                if (statusResponse.status === 200) {
                    const status = statusResponse.json('status');
                    const metadata = statusResponse.json('metadata');
                    
                    // Check if document is processed and classified
                    if (status === 'completed' && metadata && metadata.classification) {
                        completed = true;
                        workflowCompletions.add(1);
                        workflowDuration.add(new Date() - workflowStart);
                        classificationDuration.add(new Date() - classificationStart);
                        classificationSuccesses.add(1);
                        
                        // Verify metadata
                        check(statusResponse, {
                            'has valid classification': (r) => ['legal', 'financial', 'medical', 'hr', 'general'].includes(r.json('metadata.classification')),
                            'has security level': (r) => ['public', 'internal', 'confidential', 'restricted'].includes(r.json('metadata.security_level')),
                            'has tags': (r) => Array.isArray(r.json('metadata.tags')),
                            'has custom fields': (r) => typeof r.json('metadata.custom_fields') === 'object',
                        }) || errorRate.add(1);
                    } else if (status === 'failed') {
                        errorRate.add(1);
                        break;
                    }
                }
                
                attempts++;
                sleep(1);
            }
            
            if (!completed) {
                errorRate.add(1);
            }
        }
    } catch (e) {
        errorRate.add(1);
        console.error(e);
    }

    sleep(1);
}
EOL
}

# Update mixed test scenario
create_mixed_test() {
    cat > load-test.js << 'EOL'
import http from 'k6/http';
import { check, sleep } from 'k6';
import { randomString } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';
import { Counter, Rate, Trend } from 'k6/metrics';

// Custom metrics
const documentUploads = new Counter('document_uploads');
const documentDownloads = new Counter('document_downloads');
const workflowStarts = new Counter('workflow_starts');
const workflowCompletions = new Counter('workflow_completions');
const classificationSuccesses = new Counter('classification_successes');
const uploadDuration = new Trend('upload_duration');
const downloadDuration = new Trend('download_duration');
const workflowDuration = new Trend('workflow_duration');
const classificationDuration = new Trend('classification_duration');
const errorRate = new Rate('errors');

// Test configuration
export const options = {
    scenarios: {
        crud_operations: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '30s', target: 5 },
                { duration: '2m', target: 5 },
                { duration: '30s', target: 0 },
            ],
            exec: 'crudScenario',
        },
        workflows: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '1m', target: 3 },
                { duration: '3m', target: 3 },
                { duration: '1m', target: 0 },
            ],
            exec: 'workflowScenario',
        },
    },
    thresholds: {
        'http_req_duration': ['p(95)<500'],
        'upload_duration': ['p(95)<1000'],
        'download_duration': ['p(95)<500'],
        'workflow_duration': ['p(95)<5000'],
        'classification_duration': ['p(95)<2000'],
        'errors': ['rate<0.01'],
    },
};

// Test setup
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || 'your-jwt-token';

// Helper functions
function createTestDocument(type = 'simple') {
    const classifications = ['legal', 'financial', 'medical', 'hr', 'general'];
    const securityLevels = ['public', 'internal', 'confidential', 'restricted'];
    const authors = ['john.doe', 'jane.smith', 'bob.wilson'];
    const departments = ['legal', 'finance', 'hr', 'operations'];
    
    const classification = classifications[Math.floor(Math.random() * classifications.length)];
    const securityLevel = securityLevels[Math.floor(Math.random() * securityLevels.length)];
    const author = authors[Math.floor(Math.random() * authors.length)];
    const department = departments[Math.floor(Math.random() * departments.length)];
    
    const content = `Test document ${randomString(8)}`;
    const metadata = type === 'workflow' ? {
        author: author,
        tags: ['test', 'workflow', department, classification],
        classification: classification,
        security_level: securityLevel,
        custom_fields: {
            priority: Math.floor(Math.random() * 3) + 1,
            department: department,
            project_code: `PRJ-${randomString(4)}`,
            review_required: Math.random() > 0.5,
        },
    } : {};
    
    return {
        filename: `test-${randomString(8)}.txt`,
        content: content,
        metadata: metadata,
    };
}

// CRUD scenario
export function crudScenario() {
    const headers = {
        'Authorization': `Bearer ${AUTH_TOKEN}`,
    };

    try {
        // Create and upload document
        const doc = createTestDocument();
        const uploadStart = new Date();
        const uploadResponse = http.post(
            `${BASE_URL}/documents`,
            {
                file: http.file(doc.content, doc.filename, 'text/plain'),
            },
            { headers: headers }
        );
        uploadDuration.add(new Date() - uploadStart);
        
        check(uploadResponse, {
            'upload successful': (r) => r.status === 201,
            'has document id': (r) => r.json('id') !== undefined,
        }) || errorRate.add(1);
        
        if (uploadResponse.status === 201) {
            documentUploads.add(1);
            const docId = uploadResponse.json('id');
            
            // Download and verify
            const downloadStart = new Date();
            const getResponse = http.get(
                `${BASE_URL}/documents/${docId}`,
                { headers: headers }
            );
            downloadDuration.add(new Date() - downloadStart);
            
            check(getResponse, {
                'download successful': (r) => r.status === 200,
                'content matches': (r) => r.body.includes(doc.content),
            }) || errorRate.add(1);
            
            documentDownloads.add(1);

            // Cleanup
            const deleteResponse = http.del(
                `${BASE_URL}/documents/${docId}`,
                null,
                { headers: headers }
            );
            
            check(deleteResponse, {
                'delete successful': (r) => r.status === 204,
            }) || errorRate.add(1);
        }
    } catch (e) {
        errorRate.add(1);
        console.error(e);
    }
    
    sleep(1);
}

// Workflow scenario
export function workflowScenario() {
    const headers = {
        'Authorization': `Bearer ${AUTH_TOKEN}`,
    };

    try {
        // Start workflow
        const doc = createTestDocument('workflow');
        const workflowStart = new Date();
        workflowStarts.add(1);
        
        const uploadResponse = http.post(
            `${BASE_URL}/documents`,
            {
                file: http.file(doc.content, doc.filename, 'text/plain'),
                metadata: JSON.stringify(doc.metadata),
            },
            { headers: headers }
        );
        
        check(uploadResponse, {
            'upload successful': (r) => r.status === 201,
            'has document id': (r) => r.json('id') !== undefined,
        }) || errorRate.add(1);
        
        if (uploadResponse.status === 201) {
            const docId = uploadResponse.json('id');
            const classificationStart = new Date();
            
            // Poll for completion
            let attempts = 0;
            let completed = false;
            while (attempts < 10 && !completed) {
                const statusResponse = http.get(
                    `${BASE_URL}/documents/${docId}`,
                    { headers: headers }
                );
                
                if (statusResponse.status === 200) {
                    const status = statusResponse.json('status');
                    const metadata = statusResponse.json('metadata');
                    
                    if (status === 'completed' && metadata && metadata.classification) {
                        completed = true;
                        workflowCompletions.add(1);
                        workflowDuration.add(new Date() - workflowStart);
                        classificationDuration.add(new Date() - classificationStart);
                        classificationSuccesses.add(1);
                        
                        // Verify metadata
                        check(statusResponse, {
                            'has valid classification': (r) => ['legal', 'financial', 'medical', 'hr', 'general'].includes(r.json('metadata.classification')),
                            'has security level': (r) => ['public', 'internal', 'confidential', 'restricted'].includes(r.json('metadata.security_level')),
                            'has tags': (r) => Array.isArray(r.json('metadata.tags')),
                            'has custom fields': (r) => typeof r.json('metadata.custom_fields') === 'object',
                        }) || errorRate.add(1);
                    } else if (status === 'failed') {
                        errorRate.add(1);
                        break;
                    }
                }
                
                attempts++;
                sleep(1);
            }
            
            if (!completed) {
                errorRate.add(1);
            }

            // Cleanup
            http.del(
                `${BASE_URL}/documents/${docId}`,
                null,
                { headers: headers }
            );
        }
    } catch (e) {
        errorRate.add(1);
        console.error(e);
    }
    
    sleep(1);
}
EOL
}

# Create security test scenario
create_security_test() {
    cat > load-test.js << 'EOL'
import http from 'k6/http';
import { check, sleep } from 'k6';
import { randomString } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';
import { Counter, Rate, Trend } from 'k6/metrics';

// Custom metrics
const unauthorizedAttempts = new Counter('unauthorized_attempts');
const invalidTokenAttempts = new Counter('invalid_token_attempts');
const authSuccesses = new Counter('auth_successes');
const authLatency = new Trend('auth_latency');
const errorRate = new Rate('errors');

// Test configuration
export const options = {
    scenarios: {
        security_test: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '30s', target: 5 },
                { duration: '2m', target: 5 },
                { duration: '30s', target: 0 },
            ],
        },
    },
    thresholds: {
        'auth_latency': ['p(95)<200'],
        'errors': ['rate<0.01'],
    },
};

// Test setup
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || 'your-jwt-token';

// Security test scenarios
export default function () {
    try {
        // Test 1: No authentication
        const noAuthStart = new Date();
        const noAuthResponse = http.get(`${BASE_URL}/documents`);
        authLatency.add(new Date() - noAuthStart);
        
        check(noAuthResponse, {
            'unauthorized request blocked': (r) => r.status === 401,
        }) || errorRate.add(1);
        
        unauthorizedAttempts.add(1);

        // Test 2: Invalid token
        const invalidTokenStart = new Date();
        const invalidTokenResponse = http.get(
            `${BASE_URL}/documents`,
            {
                headers: {
                    'Authorization': 'Bearer invalid.token.here',
                },
            }
        );
        authLatency.add(new Date() - invalidTokenStart);
        
        check(invalidTokenResponse, {
            'invalid token rejected': (r) => r.status === 401,
        }) || errorRate.add(1);
        
        invalidTokenAttempts.add(1);

        // Test 3: Valid token
        const validTokenStart = new Date();
        const validTokenResponse = http.get(
            `${BASE_URL}/documents`,
            {
                headers: {
                    'Authorization': `Bearer ${AUTH_TOKEN}`,
                },
            }
        );
        authLatency.add(new Date() - validTokenStart);
        
        check(validTokenResponse, {
            'valid token accepted': (r) => r.status === 200,
        }) || errorRate.add(1);
        
        authSuccesses.add(1);

        // Test 4: Document access control
        const doc = {
            filename: `test-${randomString(8)}.txt`,
            content: `Test document ${randomString(8)}`,
            metadata: {
                security_level: 'restricted',
                classification: 'legal',
                tags: ['confidential'],
            },
        };

        // Upload with valid token
        const uploadResponse = http.post(
            `${BASE_URL}/documents`,
            {
                file: http.file(doc.content, doc.filename, 'text/plain'),
                metadata: JSON.stringify(doc.metadata),
            },
            {
                headers: {
                    'Authorization': `Bearer ${AUTH_TOKEN}`,
                },
            }
        );

        if (uploadResponse.status === 201) {
            const docId = uploadResponse.json('id');

            // Try to access with no token
            const noAuthAccessResponse = http.get(
                `${BASE_URL}/documents/${docId}`
            );
            
            check(noAuthAccessResponse, {
                'restricted document protected': (r) => r.status === 401,
            }) || errorRate.add(1);

            // Try to access with invalid token
            const invalidTokenAccessResponse = http.get(
                `${BASE_URL}/documents/${docId}`,
                {
                    headers: {
                        'Authorization': 'Bearer invalid.token.here',
                    },
                }
            );
            
            check(invalidTokenAccessResponse, {
                'restricted document token verified': (r) => r.status === 401,
            }) || errorRate.add(1);

            // Cleanup
            http.del(
                `${BASE_URL}/documents/${docId}`,
                null,
                {
                    headers: {
                        'Authorization': `Bearer ${AUTH_TOKEN}`,
                    },
                }
            );
        }
    } catch (e) {
        errorRate.add(1);
        console.error(e);
    }

    sleep(1);
}
EOL
}

# Add output configuration
add_output_config() {
    local output_type="$1"
    local config=""
    
    case "$output_type" in
        prometheus)
            config="export const output = {
    prometheus: {
        url: 'http://localhost:9090/metrics',
    },
};"
            ;;
        influxdb)
            config="export const output = {
    influxdb: {
        url: 'http://localhost:8086/api/v2/write',
        organization: 'docautomation',
        bucket: 'k6',
        token: 'your-token',
    },
};"
            ;;
        *)
            log_error "Unknown output type: $output_type"
            exit 1
            ;;
    esac
    
    # Add output configuration to test file
    sed -i "1i\\$config\\n" load-test.js
}

# Run load test
run_test() {
    local base_url="$1"
    local auth_token="$2"
    local scenario="$3"
    local output_type="$4"
    
    log_info "Starting load test..."
    log_info "Base URL: $base_url"
    log_info "Scenario: $scenario"
    
    # Build k6 command
    local k6_cmd="k6 run"
    
    # Add output if specified
    if [ -n "$output_type" ]; then
        case "$output_type" in
            prometheus)
                k6_cmd="$k6_cmd --out prometheus"
                ;;
            influxdb)
                k6_cmd="$k6_cmd --out influxdb"
                ;;
        esac
    fi
    
    # Add environment variables
    k6_cmd="$k6_cmd -e BASE_URL=$base_url -e AUTH_TOKEN=$auth_token load-test.js"
    
    # Run test and capture output
    local test_output
    if ! test_output=$(eval "$k6_cmd" 2>&1); then
        log_error "Load test failed"
        echo "$test_output"
        return 1
    fi
    
    # Validate test results
    if ! validate_test_results "$test_output" "$scenario"; then
        log_error "Test validation failed"
        return 1
    fi
    
    log_info "Load test completed successfully"
    echo "$test_output"
    return 0
}

# Show usage
show_usage() {
    echo "Usage: $0 [options]"
    echo
    echo "Options:"
    echo "  --url URL         Base URL of the service (default: http://localhost:8080)"
    echo "  --token TOKEN     JWT token for authentication"
    echo "  --scenario TYPE   Test scenario type: crud|workflow|mixed|security (default: crud)"
    echo "  --output TYPE     Output type: prometheus|influxdb (optional)"
    echo "  -h, --help        Show this help message"
    echo
    exit 0
}

# Main function
main() {
    local base_url="http://localhost:8080"
    local auth_token=""
    local scenario="crud"
    local output_type=""
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --url)
                base_url="$2"
                shift 2
                ;;
            --token)
                auth_token="$2"
                shift 2
                ;;
            --scenario)
                scenario="$2"
                shift 2
                ;;
            --output)
                output_type="$2"
                shift 2
                ;;
            -h|--help)
                show_usage
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                ;;
        esac
    done
    
    # Check required arguments
    if [ -z "$auth_token" ]; then
        if [ -f ".env.dev" ]; then
            source .env.dev
            auth_token="$AUTH_TOKEN"
        else
            log_error "No authentication token provided"
            show_usage
        fi
    fi
    
    # Validate environment before running tests
    validate_environment
    
    # Run tests
    check_k6
    create_test_file "$scenario" "$output_type"
    if ! run_test "$base_url" "$auth_token" "$scenario" "$output_type"; then
        log_error "Test execution failed"
        exit 1
    fi
}

# Run main function with all arguments
main "$@" 