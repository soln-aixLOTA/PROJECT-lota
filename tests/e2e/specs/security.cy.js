describe('Security Testing', () => {
    beforeEach(() => {
        // Reset application state and seed test data
        cy.request('POST', `${Cypress.env('apiUrl')}/test/reset`)
        cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-security-test-data`)
    })

    describe('Authentication Security', () => {
        it('should prevent brute force attacks', () => {
            const maxAttempts = 5
            const loginAttempts = Array(maxAttempts + 1).fill().map(() =>
                cy.request({
                    method: 'POST',
                    url: `${Cypress.env('apiUrl')}/auth/login`,
                    failOnStatusCode: false,
                    body: {
                        email: 'test@example.com',
                        password: 'wrongpassword'
                    }
                })
            )

            // Verify account lockout after max attempts
            cy.wrap(Promise.all(loginAttempts)).then(responses => {
                const lastResponse = responses[responses.length - 1]
                expect(lastResponse.status).to.equal(429)
                expect(lastResponse.body).to.contain({ message: 'Account temporarily locked' })
            })
        })

        it('should enforce password complexity requirements', () => {
            const weakPasswords = [
                'short',
                'nouppercaseornumbers',
                'NoSpecialChars123',
                'Only!Special@Chars'
            ]

            weakPasswords.forEach(password => {
                cy.request({
                    method: 'POST',
                    url: `${Cypress.env('apiUrl')}/auth/register`,
                    failOnStatusCode: false,
                    body: {
                        email: 'test@example.com',
                        password: password
                    }
                }).then(response => {
                    expect(response.status).to.equal(400)
                    expect(response.body).to.contain({ message: 'Password does not meet complexity requirements' })
                })
            })
        })

        it('should properly handle session expiration', () => {
            // Login and get token
            cy.request('POST', `${Cypress.env('apiUrl')}/auth/login`, {
                email: 'test@example.com',
                password: 'ValidPass123!'
            }).then(response => {
                const token = response.body.token

                // Simulate token expiration
                cy.request('POST', `${Cypress.env('apiUrl')}/test/expire-token`, { token })

                // Attempt to use expired token
                cy.request({
                    url: `${Cypress.env('apiUrl')}/api/protected-resource`,
                    failOnStatusCode: false,
                    headers: { Authorization: `Bearer ${token}` }
                }).then(response => {
                    expect(response.status).to.equal(401)
                    expect(response.body).to.contain({ message: 'Token expired' })
                })
            })
        })
    })

    describe('Authorization Security', () => {
        beforeEach(() => {
            // Seed users with different roles
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-users`, {
                users: [
                    { email: 'admin@example.com', role: 'admin', password: 'AdminPass123!' },
                    { email: 'user@example.com', role: 'user', password: 'UserPass123!' }
                ]
            })
        })

        it('should enforce role-based access control', () => {
            // Test admin access
            cy.request('POST', `${Cypress.env('apiUrl')}/auth/login`, {
                email: 'admin@example.com',
                password: 'AdminPass123!'
            }).then(response => {
                const adminToken = response.body.token

                cy.request({
                    url: `${Cypress.env('apiUrl')}/api/admin/users`,
                    headers: { Authorization: `Bearer ${adminToken}` }
                }).then(response => {
                    expect(response.status).to.equal(200)
                })
            })

            // Test regular user access
            cy.request('POST', `${Cypress.env('apiUrl')}/auth/login`, {
                email: 'user@example.com',
                password: 'UserPass123!'
            }).then(response => {
                const userToken = response.body.token

                cy.request({
                    url: `${Cypress.env('apiUrl')}/api/admin/users`,
                    failOnStatusCode: false,
                    headers: { Authorization: `Bearer ${userToken}` }
                }).then(response => {
                    expect(response.status).to.equal(403)
                })
            })
        })

        it('should prevent privilege escalation', () => {
            cy.request('POST', `${Cypress.env('apiUrl')}/auth/login`, {
                email: 'user@example.com',
                password: 'UserPass123!'
            }).then(response => {
                const userToken = response.body.token

                cy.request({
                    method: 'PUT',
                    url: `${Cypress.env('apiUrl')}/api/users/self`,
                    failOnStatusCode: false,
                    headers: { Authorization: `Bearer ${userToken}` },
                    body: {
                        role: 'admin'
                    }
                }).then(response => {
                    expect(response.status).to.equal(403)
                })
            })
        })
    })

    describe('Data Security', () => {
        it('should prevent SQL injection attacks', () => {
            const sqlInjectionAttempts = [
                "' OR '1'='1",
                "; DROP TABLE users;--",
                "' UNION SELECT * FROM users--"
            ]

            sqlInjectionAttempts.forEach(attempt => {
                cy.request({
                    url: `${Cypress.env('apiUrl')}/api/users/search?q=${attempt}`,
                    failOnStatusCode: false
                }).then(response => {
                    expect(response.status).to.be.oneOf([400, 403])
                })
            })
        })

        it('should prevent XSS attacks', () => {
            const xssPayload = '<script>alert("XSS")</script>'

            cy.request({
                method: 'POST',
                url: `${Cypress.env('apiUrl')}/api/comments`,
                failOnStatusCode: false,
                body: {
                    content: xssPayload
                }
            }).then(response => {
                expect(response.status).to.equal(400)

                // Verify the content is escaped when retrieved
                cy.request(`${Cypress.env('apiUrl')}/api/comments`).then(response => {
                    expect(response.body).to.not.contain(xssPayload)
                })
            })
        })

        it('should enforce proper CORS policies', () => {
            cy.request({
                url: `${Cypress.env('apiUrl')}/api/public-resource`,
                headers: {
                    Origin: 'https://malicious-site.com'
                },
                failOnStatusCode: false
            }).then(response => {
                expect(response.status).to.equal(403)
            })
        })
    })

    describe('API Security', () => {
        it('should rate limit API requests', () => {
            const requests = Array(100).fill().map(() =>
                cy.request({
                    url: `${Cypress.env('apiUrl')}/api/public-endpoint`,
                    failOnStatusCode: false
                })
            )

            cy.wrap(Promise.all(requests)).then(responses => {
                const rateLimitedResponses = responses.filter(r => r.status === 429)
                expect(rateLimitedResponses.length).to.be.greaterThan(0)
            })
        })

        it('should validate request payloads', () => {
            const invalidPayloads = [
                { email: 'invalid-email' },
                { email: 'test@example.com', password: '' },
                { email: 'test@example.com', password: 'short' }
            ]

            invalidPayloads.forEach(payload => {
                cy.request({
                    method: 'POST',
                    url: `${Cypress.env('apiUrl')}/auth/register`,
                    failOnStatusCode: false,
                    body: payload
                }).then(response => {
                    expect(response.status).to.equal(400)
                    expect(response.body).to.have.property('errors')
                })
            })
        })
    })

    describe('File Upload Security', () => {
        it('should prevent malicious file uploads', () => {
            const maliciousFiles = [
                { name: 'malicious.exe', type: 'application/x-msdownload' },
                { name: 'script.php', type: 'application/x-php' },
                { name: 'large.txt', size: 100 * 1024 * 1024 } // 100MB
            ]

            maliciousFiles.forEach(file => {
                cy.request({
                    method: 'POST',
                    url: `${Cypress.env('apiUrl')}/api/upload`,
                    failOnStatusCode: false,
                    body: file
                }).then(response => {
                    expect(response.status).to.equal(400)
                })
            })
        })
    })
}) 