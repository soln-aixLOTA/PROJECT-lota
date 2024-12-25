// Import commands.js using ES2015 syntax:
import 'cypress-file-upload';
import './commands';

// Prevent uncaught exception from failing tests
Cypress.on('uncaught:exception', (err, runnable) => {
    // returning false here prevents Cypress from failing the test
    return false
})

// Add custom commands for common operations
Cypress.Commands.add('login', (email, password) => {
    cy.request({
        method: 'POST',
        url: `${Cypress.env('apiUrl')}/auth/login`,
        body: { email, password }
    }).then((response) => {
        window.localStorage.setItem('token', response.body.token)
    })
    cy.visit('/')
})

Cypress.Commands.add('createTenant', (tenantName, tenantConfig = {}) => {
    cy.request({
        method: 'POST',
        url: `${Cypress.env('apiUrl')}/api/tenants`,
        headers: {
            Authorization: `Bearer ${window.localStorage.getItem('token')}`
        },
        body: {
            name: tenantName,
            ...tenantConfig
        }
    })
})

Cypress.Commands.add('createBot', (botName, botConfig = {}) => {
    cy.request({
        method: 'POST',
        url: `${Cypress.env('apiUrl')}/api/bots`,
        headers: {
            Authorization: `Bearer ${window.localStorage.getItem('token')}`
        },
        body: {
            name: botName,
            ...botConfig
        }
    })
})

Cypress.Commands.add('resetTestData', () => {
    cy.request('POST', `${Cypress.env('apiUrl')}/test/reset`)
})

Cypress.Commands.add('seedTestData', (type, data) => {
    cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-${type}`, data)
})

Cypress.Commands.add('simulateNetworkCondition', (condition) => {
    cy.request('POST', `${Cypress.env('apiUrl')}/test/simulate-network-condition`, { condition })
})

Cypress.Commands.add('simulateResourceConstraint', (resourceType, available) => {
    cy.request('POST', `${Cypress.env('apiUrl')}/test/simulate-resource-constraint`, { resourceType, available })
})

Cypress.Commands.add('simulateServiceDisruption', (serviceType, status) => {
    cy.request('POST', `${Cypress.env('apiUrl')}/test/simulate-service-disruption`, { serviceType, status })
}) 