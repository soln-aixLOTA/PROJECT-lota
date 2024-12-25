describe('Tenant Management', () => {
    beforeEach(() => {
        // Reset application state and seed test user
        cy.request('POST', `${Cypress.env('apiUrl')}/test/reset`)
        cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-user`, {
            email: 'admin@example.com',
            password: 'AdminPass123!',
            verified: true,
            role: 'admin'
        })
        cy.login('admin@example.com', 'AdminPass123!')
    })

    describe('Tenant Creation', () => {
        it('should successfully create a new tenant', () => {
            const tenantName = `Test Tenant ${Date.now()}`
            const tenantConfig = {
                maxBots: 5,
                maxUsers: 10,
                storageLimit: '100GB'
            }

            cy.visit('/tenants/create')
            cy.get('[data-cy=tenant-name-input]').type(tenantName)
            cy.get('[data-cy=max-bots-input]').type(tenantConfig.maxBots)
            cy.get('[data-cy=max-users-input]').type(tenantConfig.maxUsers)
            cy.get('[data-cy=storage-limit-input]').type(tenantConfig.storageLimit)
            cy.get('[data-cy=create-tenant-button]').click()

            // Verify tenant creation
            cy.url().should('include', '/tenants')
            cy.get('[data-cy=success-message]')
                .should('be.visible')
                .and('contain', 'Tenant created successfully')

            // Verify tenant appears in list
            cy.get('[data-cy=tenant-list]')
                .should('contain', tenantName)
        })

        it('should show error for duplicate tenant name', () => {
            const tenantName = 'Duplicate Tenant'

            // First create a tenant
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-tenant`, {
                name: tenantName
            })

            // Try to create tenant with same name
            cy.visit('/tenants/create')
            cy.get('[data-cy=tenant-name-input]').type(tenantName)
            cy.get('[data-cy=create-tenant-button]').click()

            cy.get('[data-cy=error-message]')
                .should('be.visible')
                .and('contain', 'Tenant name already exists')
        })
    })

    describe('Tenant Management', () => {
        beforeEach(() => {
            // Seed a test tenant
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-tenant`, {
                name: 'Test Tenant',
                maxBots: 5,
                maxUsers: 10,
                storageLimit: '100GB'
            })
        })

        it('should successfully update tenant settings', () => {
            const updates = {
                maxBots: 10,
                maxUsers: 20,
                storageLimit: '200GB'
            }

            cy.visit('/tenants')
            cy.contains('Test Tenant').click()
            cy.get('[data-cy=edit-tenant-button]').click()

            cy.get('[data-cy=max-bots-input]').clear().type(updates.maxBots)
            cy.get('[data-cy=max-users-input]').clear().type(updates.maxUsers)
            cy.get('[data-cy=storage-limit-input]').clear().type(updates.storageLimit)
            cy.get('[data-cy=save-tenant-button]').click()

            // Verify success message
            cy.get('[data-cy=success-message]')
                .should('be.visible')
                .and('contain', 'Tenant updated successfully')

            // Verify updates persisted
            cy.reload()
            cy.get('[data-cy=max-bots-input]').should('have.value', updates.maxBots)
            cy.get('[data-cy=max-users-input]').should('have.value', updates.maxUsers)
            cy.get('[data-cy=storage-limit-input]').should('have.value', updates.storageLimit)
        })

        it('should successfully delete a tenant', () => {
            cy.visit('/tenants')
            cy.contains('Test Tenant').click()
            cy.get('[data-cy=delete-tenant-button]').click()

            // Confirm deletion
            cy.get('[data-cy=confirm-delete-button]').click()

            // Verify success message
            cy.get('[data-cy=success-message]')
                .should('be.visible')
                .and('contain', 'Tenant deleted successfully')

            // Verify tenant no longer appears in list
            cy.get('[data-cy=tenant-list]')
                .should('not.contain', 'Test Tenant')
        })
    })

    describe('Tenant Isolation', () => {
        beforeEach(() => {
            // Seed two test tenants
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-tenant`, {
                name: 'Tenant A',
                users: [{
                    email: 'user-a@example.com',
                    password: 'UserPass123!',
                    role: 'tenant_admin'
                }]
            })
            cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-tenant`, {
                name: 'Tenant B',
                users: [{
                    email: 'user-b@example.com',
                    password: 'UserPass123!',
                    role: 'tenant_admin'
                }]
            })
        })

        it('should ensure tenant data isolation', () => {
            // Login as Tenant A user
            cy.login('user-a@example.com', 'UserPass123!')

            // Create a bot in Tenant A
            cy.createBot('Tenant A Bot')

            // Verify bot is visible
            cy.get('[data-cy=bot-list]')
                .should('contain', 'Tenant A Bot')

            // Login as Tenant B user
            cy.login('user-b@example.com', 'UserPass123!')

            // Verify Tenant A's bot is not visible
            cy.get('[data-cy=bot-list]')
                .should('not.contain', 'Tenant A Bot')
        })
    })
}) 