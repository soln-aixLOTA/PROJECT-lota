describe('Performance and Load Testing', () => {
    beforeEach(() => {
        // Reset application state and seed test data
        cy.request('POST', `${Cypress.env('apiUrl')}/test/reset`)
        cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-performance-data`)
        cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-user`, {
            email: 'perf-test@example.com',
            password: 'TestPass123!',
            verified: true,
            role: 'admin'
        })
        cy.login('perf-test@example.com', 'TestPass123!')
    })

    describe('API Response Times', () => {
        it('should meet response time SLAs for critical endpoints', () => {
            const endpoints = [
                { path: '/api/users', sla: 500 },
                { path: '/api/tenants', sla: 500 },
                { path: '/api/bots', sla: 1000 },
                { path: '/api/inference', sla: 2000 }
            ]

            endpoints.forEach(endpoint => {
                cy.request({
                    url: `${Cypress.env('apiUrl')}${endpoint.path}`,
                    time: true
                }).then(response => {
                    expect(response.duration).to.be.lessThan(endpoint.sla,
                        `${endpoint.path} response time ${response.duration}ms exceeded SLA ${endpoint.sla}ms`)
                })
            })
        })

        it('should handle concurrent API requests efficiently', () => {
            const concurrentRequests = 10
            const requests = Array(concurrentRequests).fill().map(() =>
                cy.request({
                    url: `${Cypress.env('apiUrl')}/api/bots`,
                    time: true
                })
            )

            // Execute requests concurrently and verify response times
            cy.wrap(Promise.all(requests)).then(responses => {
                responses.forEach(response => {
                    expect(response.duration).to.be.lessThan(2000,
                        `Concurrent request response time ${response.duration}ms exceeded threshold`)
                })
            })
        })
    })

    describe('UI Performance', () => {
        it('should load and render pages within performance targets', () => {
            const pages = [
                { path: '/dashboard', target: 2000 },
                { path: '/bots', target: 2000 },
                { path: '/tenants', target: 1500 },
                { path: '/users', target: 1500 }
            ]

            pages.forEach(page => {
                // Start performance measurement
                cy.window().then(win => {
                    const perfData = win.performance

                    cy.visit(page.path)

                    // Wait for page to fully load
                    cy.window().then(() => {
                        const timing = perfData.timing
                        const pageLoadTime = timing.loadEventEnd - timing.navigationStart

                        expect(pageLoadTime).to.be.lessThan(page.target,
                            `${page.path} load time ${pageLoadTime}ms exceeded target ${page.target}ms`)
                    })
                })
            })
        })

        it('should maintain UI responsiveness under load', () => {
            // Load test data
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-load-test-data`, {
                botsCount: 100,
                usersCount: 1000,
                tenantsCount: 50
            })

            // Test pagination and filtering with large datasets
            cy.visit('/bots')

            // Measure time to filter and sort
            cy.window().then(win => {
                const start = win.performance.now()

                cy.get('[data-cy=filter-input]').type('test')
                cy.get('[data-cy=sort-select]').select('name')

                cy.get('[data-cy=bot-list]').should('be.visible').then(() => {
                    const end = win.performance.now()
                    const filterSortTime = end - start

                    expect(filterSortTime).to.be.lessThan(1000,
                        `Filter and sort operation took ${filterSortTime}ms, exceeding 1000ms target`)
                })
            })

            // Test infinite scroll performance
            cy.window().then(win => {
                const start = win.performance.now()

                // Scroll to bottom multiple times
                for (let i = 0; i < 5; i++) {
                    cy.get('[data-cy=bot-list]').scrollTo('bottom')
                    cy.wait(500) // Wait for data to load
                }

                cy.get('[data-cy=bot-list]').should('be.visible').then(() => {
                    const end = win.performance.now()
                    const scrollTime = (end - start) / 5 // Average time per scroll

                    expect(scrollTime).to.be.lessThan(500,
                        `Infinite scroll operation took ${scrollTime}ms on average, exceeding 500ms target`)
                })
            })
        })
    })

    describe('Resource Usage', () => {
        it('should maintain acceptable memory usage', () => {
            cy.window().then(win => {
                if (win.performance.memory) {
                    const memoryUsage = win.performance.memory.usedJSHeapSize / (1024 * 1024)
                    expect(memoryUsage).to.be.lessThan(100,
                        `Memory usage ${memoryUsage}MB exceeded 100MB threshold`)
                }
            })
        })

        it('should optimize network requests', () => {
            cy.visit('/dashboard')

            cy.window().then(win => {
                const resourceEntries = win.performance
                    .getEntriesByType('resource')
                    .filter(entry => !entry.name.includes('cypress'))

                // Check total payload size
                const totalSize = resourceEntries.reduce((sum, entry) =>
                    sum + (entry.encodedBodySize || 0), 0) / (1024 * 1024)

                expect(totalSize).to.be.lessThan(5,
                    `Total resource size ${totalSize}MB exceeded 5MB threshold`)

                // Check number of requests
                expect(resourceEntries.length).to.be.lessThan(50,
                    `Number of requests ${resourceEntries.length} exceeded 50 threshold`)
            })
        })
    })

    describe('Error Handling Performance', () => {
        it('should handle errors efficiently under load', () => {
            // Generate multiple error conditions simultaneously
            const errorRequests = 10
            const requests = Array(errorRequests).fill().map(() =>
                cy.request({
                    url: `${Cypress.env('apiUrl')}/api/error-test`,
                    failOnStatusCode: false,
                    time: true
                })
            )

            // Verify error handling performance
            cy.wrap(Promise.all(requests)).then(responses => {
                responses.forEach(response => {
                    expect(response.duration).to.be.lessThan(1000,
                        `Error handling time ${response.duration}ms exceeded 1000ms threshold`)
                })
            })
        })
    })

    describe('Database Performance', () => {
        it('should maintain database query performance under load', () => {
            // Seed large dataset
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-db-performance-data`)

            const queries = [
                { name: 'Complex Join Query', endpoint: '/api/test/complex-query', threshold: 2000 },
                { name: 'Aggregation Query', endpoint: '/api/test/aggregation', threshold: 3000 },
                { name: 'Search Query', endpoint: '/api/test/search', threshold: 1500 }
            ]

            queries.forEach(query => {
                cy.request({
                    url: `${Cypress.env('apiUrl')}${query.endpoint}`,
                    time: true
                }).then(response => {
                    expect(response.duration).to.be.lessThan(query.threshold,
                        `${query.name} took ${response.duration}ms, exceeding ${query.threshold}ms threshold`)
                })
            })
        })
    })
}) 