describe('User Management', () => {
    beforeEach(() => {
        // Reset application state before each test
        cy.request('POST', `${Cypress.env('apiUrl')}/test/reset`)
        cy.request('POST', `${Cypress.env('apiUrl')}/api/users`, {
            email: 'test@example.com',
            password: 'password123'
        });
        cy.visit('http://localhost:3000/login');
    })

    describe('User Registration', () => {
        it('should successfully register a new user', () => {
            const testUser = {
                email: `test-${Date.now()}@example.com`,
                password: 'TestPassword123!',
                firstName: 'Test',
                lastName: 'User'
            }

            cy.visit('/register')
            cy.get('[data-cy=email-input]').type(testUser.email)
            cy.get('[data-cy=password-input]').type(testUser.password)
            cy.get('[data-cy=first-name-input]').type(testUser.firstName)
            cy.get('[data-cy=last-name-input]').type(testUser.lastName)
            cy.get('[data-cy=register-button]').click()

            // Verify successful registration
            cy.url().should('include', '/verify-email')
            cy.contains('Please verify your email').should('be.visible')
        })

        it('should show appropriate error for existing email', () => {
            const existingUser = {
                email: 'existing@example.com',
                password: 'TestPassword123!'
            }

            // First register the user
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-user`, existingUser)

            // Try to register again with same email
            cy.visit('/register')
            cy.get('[data-cy=email-input]').type(existingUser.email)
            cy.get('[data-cy=password-input]').type(existingUser.password)
            cy.get('[data-cy=register-button]').click()

            cy.get('[data-cy=error-message]')
                .should('be.visible')
                .and('contain', 'Email already exists')
        })
    })

    describe('User Login', () => {
        beforeEach(() => {
            // Seed a test user before each login test
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-user`, {
                email: 'test@example.com',
                password: 'TestPassword123!',
                verified: true
            })
        })

        it('should successfully login with valid credentials', () => {
            cy.login('test@example.com', 'TestPassword123!')
            cy.url().should('include', '/dashboard')
            cy.get('[data-cy=user-menu]').should('be.visible')
        })

        it('should show error with invalid credentials', () => {
            cy.login('test@example.com', 'WrongPassword123!')
            cy.get('[data-cy=error-message]')
                .should('be.visible')
                .and('contain', 'Invalid credentials')
        })
    })

    describe('Profile Management', () => {
        beforeEach(() => {
            // Seed and login as test user
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-user`, {
                email: 'test@example.com',
                password: 'TestPassword123!',
                verified: true
            })
            cy.login('test@example.com', 'TestPassword123!')
        })

        it('should successfully update user profile', () => {
            const updates = {
                firstName: 'Updated',
                lastName: 'Name',
                phoneNumber: '+1234567890'
            }

            cy.visit('/profile')
            cy.get('[data-cy=first-name-input]').clear().type(updates.firstName)
            cy.get('[data-cy=last-name-input]').clear().type(updates.lastName)
            cy.get('[data-cy=phone-input]').clear().type(updates.phoneNumber)
            cy.get('[data-cy=save-profile-button]').click()

            // Verify success message
            cy.get('[data-cy=success-message]')
                .should('be.visible')
                .and('contain', 'Profile updated successfully')

            // Verify updates persisted
            cy.reload()
            cy.get('[data-cy=first-name-input]').should('have.value', updates.firstName)
            cy.get('[data-cy=last-name-input]').should('have.value', updates.lastName)
            cy.get('[data-cy=phone-input]').should('have.value', updates.phoneNumber)
        })
    })
}) 