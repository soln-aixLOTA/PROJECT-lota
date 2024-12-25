describe('Scaling and Load Balancing', () => {
    beforeEach(() => {
        // Reset application state and seed test data
        cy.request('POST', `${Cypress.env('apiUrl')}/test/reset`)
        cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-scaling-test-data`)
        cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-user`, {
            email: 'scaling-test@example.com',
            password: 'TestPass123!',
            verified: true,
            role: 'admin'
        })
        cy.login('scaling-test@example.com', 'TestPass123!')
    })

    describe('Horizontal Pod Scaling', () => {
        it('should automatically scale up under load', () => {
            // Deploy a test bot
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-deployed-bot`, {
                name: 'Scaling Test Bot',
                minReplicas: 1,
                maxReplicas: 5
            })

            // Generate load
            cy.request('POST', `${Cypress.env('apiUrl')}/test/generate-load`, {
                targetBot: 'Scaling Test Bot',
                requestsPerSecond: 100,
                duration: 60 // seconds
            })

            // Monitor scaling behavior
            cy.request(`${Cypress.env('apiUrl')}/test/get-scaling-metrics`).then(response => {
                const initialPodCount = response.body.podCount

                // Wait for autoscaling to kick in
                cy.wait(30000)

                cy.request(`${Cypress.env('apiUrl')}/test/get-scaling-metrics`).then(response => {
                    const scaledPodCount = response.body.podCount
                    expect(scaledPodCount).to.be.greaterThan(initialPodCount)
                })
            })

            // Verify performance during scaling
            cy.request(`${Cypress.env('apiUrl')}/test/get-performance-metrics`).then(response => {
                expect(response.body.averageResponseTime).to.be.lessThan(1000) // 1 second
                expect(response.body.successRate).to.be.greaterThan(0.99) // 99%
            })
        })

        it('should scale down when load decreases', () => {
            // Deploy a test bot with multiple replicas
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-deployed-bot`, {
                name: 'Scale Down Test Bot',
                minReplicas: 1,
                maxReplicas: 5,
                initialReplicas: 5
            })

            // Wait for initial deployment
            cy.wait(10000)

            // Verify initial pod count
            cy.request(`${Cypress.env('apiUrl')}/test/get-scaling-metrics`).then(response => {
                expect(response.body.podCount).to.equal(5)
            })

            // Wait for scale down (low traffic)
            cy.wait(300000) // 5 minutes

            // Verify pod count has decreased
            cy.request(`${Cypress.env('apiUrl')}/test/get-scaling-metrics`).then(response => {
                expect(response.body.podCount).to.be.lessThan(5)
            })
        })
    })

    describe('Load Balancing', () => {
        it('should distribute load evenly across pods', () => {
            // Deploy test bot with fixed number of replicas
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-deployed-bot`, {
                name: 'Load Balance Test Bot',
                replicas: 3
            })

            // Generate distributed load
            cy.request('POST', `${Cypress.env('apiUrl')}/test/generate-load`, {
                targetBot: 'Load Balance Test Bot',
                requestsPerSecond: 100,
                duration: 60
            })

            // Verify load distribution
            cy.request(`${Cypress.env('apiUrl')}/test/get-pod-metrics`).then(response => {
                const podMetrics = response.body.pods
                const requestCounts = podMetrics.map(pod => pod.requestCount)

                // Calculate standard deviation of request counts
                const average = requestCounts.reduce((a, b) => a + b) / requestCounts.length
                const variance = requestCounts.reduce((a, b) => a + Math.pow(b - average, 2), 0) / requestCounts.length
                const stdDev = Math.sqrt(variance)

                // Verify even distribution (standard deviation should be less than 10% of average)
                expect(stdDev).to.be.lessThan(average * 0.1)
            })
        })

        it('should handle pod failures gracefully', () => {
            // Deploy test bot with multiple replicas
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-deployed-bot`, {
                name: 'Failover Test Bot',
                replicas: 3
            })

            // Start continuous load
            cy.request('POST', `${Cypress.env('apiUrl')}/test/generate-load`, {
                targetBot: 'Failover Test Bot',
                requestsPerSecond: 50,
                duration: 120
            })

            // Simulate pod failure
            cy.request('POST', `${Cypress.env('apiUrl')}/test/simulate-pod-failure`, {
                bot: 'Failover Test Bot',
                podIndex: 1
            })

            // Verify no request failures during failover
            cy.request(`${Cypress.env('apiUrl')}/test/get-performance-metrics`).then(response => {
                expect(response.body.successRate).to.be.greaterThan(0.99)
            })

            // Verify traffic redistribution
            cy.request(`${Cypress.env('apiUrl')}/test/get-pod-metrics`).then(response => {
                const activePods = response.body.pods.filter(pod => pod.status === 'active')
                expect(activePods.length).to.equal(2)

                // Verify remaining pods handle the load
                activePods.forEach(pod => {
                    expect(pod.requestCount).to.be.greaterThan(0)
                })
            })
        })
    })

    describe('Resource Utilization', () => {
        it('should efficiently utilize resources under varying load', () => {
            // Deploy test bot
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-deployed-bot`, {
                name: 'Resource Test Bot',
                minReplicas: 1,
                maxReplicas: 5
            })

            // Generate varying load patterns
            const loadPatterns = [
                { rps: 10, duration: 60 },
                { rps: 50, duration: 60 },
                { rps: 100, duration: 60 },
                { rps: 20, duration: 60 }
            ]

            loadPatterns.forEach(pattern => {
                cy.request('POST', `${Cypress.env('apiUrl')}/test/generate-load`, {
                    targetBot: 'Resource Test Bot',
                    requestsPerSecond: pattern.rps,
                    duration: pattern.duration
                })

                cy.wait(pattern.duration * 1000)

                // Verify resource utilization
                cy.request(`${Cypress.env('apiUrl')}/test/get-resource-metrics`).then(response => {
                    const metrics = response.body

                    // CPU utilization should be within efficient range (40-80%)
                    expect(metrics.cpuUtilization).to.be.within(40, 80)

                    // Memory usage should not exceed 80%
                    expect(metrics.memoryUtilization).to.be.lessThan(80)

                    // Response times should remain consistent
                    expect(metrics.averageResponseTime).to.be.lessThan(1000)
                })
            })
        })
    })

    describe('GPU Scaling', () => {
        it('should scale GPU resources based on demand', () => {
            // Deploy GPU-enabled bot
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-deployed-bot`, {
                name: 'GPU Test Bot',
                gpuEnabled: true,
                minGpus: 1,
                maxGpus: 4
            })

            // Generate GPU-intensive load
            cy.request('POST', `${Cypress.env('apiUrl')}/test/generate-gpu-load`, {
                targetBot: 'GPU Test Bot',
                intensity: 'high',
                duration: 120
            })

            // Monitor GPU allocation
            cy.request(`${Cypress.env('apiUrl')}/test/get-gpu-metrics`).then(response => {
                const initialGpuCount = response.body.allocatedGpus

                // Wait for GPU scaling
                cy.wait(60000)

                cy.request(`${Cypress.env('apiUrl')}/test/get-gpu-metrics`).then(response => {
                    const scaledGpuCount = response.body.allocatedGpus
                    expect(scaledGpuCount).to.be.greaterThan(initialGpuCount)

                    // Verify GPU utilization
                    expect(response.body.gpuUtilization).to.be.within(60, 95)
                })
            })
        })
    })

    describe('Multi-Region Deployment', () => {
        it('should handle traffic routing across regions', () => {
            // Deploy bot across multiple regions
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-multi-region-bot`, {
                name: 'Multi-Region Bot',
                regions: ['us-east', 'us-west', 'eu-west']
            })

            // Generate geographically distributed load
            const regions = ['us-east', 'us-west', 'eu-west']
            regions.forEach(region => {
                cy.request('POST', `${Cypress.env('apiUrl')}/test/generate-regional-load`, {
                    targetBot: 'Multi-Region Bot',
                    region: region,
                    requestsPerSecond: 50,
                    duration: 60
                })
            })

            // Verify regional traffic distribution
            cy.request(`${Cypress.env('apiUrl')}/test/get-regional-metrics`).then(response => {
                const metrics = response.body

                // Verify each region handles its local traffic
                regions.forEach(region => {
                    expect(metrics[region].requestCount).to.be.greaterThan(0)
                    expect(metrics[region].averageLatency).to.be.lessThan(100) // 100ms
                })
            })
        })
    })
}) 