// Authentication commands
Cypress.Commands.add('loginByApi', (email, password) => {
    cy.request('POST', `${Cypress.env('apiUrl')}/auth/login`, {
        email,
        password
    }).then((response) => {
        window.localStorage.setItem('token', response.body.token)
    })
})

// Data seeding commands
Cypress.Commands.add('seedTestData', (type, data) => {
    cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-${type}`, data)
})

// Cleanup commands
Cypress.Commands.add('cleanup', () => {
    cy.request('POST', `${Cypress.env('apiUrl')}/test/reset`)
})

// UI interaction commands
Cypress.Commands.add('waitForLoader', () => {
    cy.get('[data-cy=loading-indicator]').should('not.exist')
})

Cypress.Commands.add('confirmDialog', () => {
    cy.get('[data-cy=confirm-button]').click()
})

// File upload commands
Cypress.Commands.add('uploadFile', (selector, fileName) => {
    cy.fixture(fileName).then(fileContent => {
        cy.get(selector).attachFile({
            fileContent,
            fileName,
            mimeType: 'application/json'
        })
    })
})

// Verification commands
Cypress.Commands.add('verifyToast', (message) => {
    cy.get('[data-cy=toast]')
        .should('be.visible')
        .and('contain', message)
})

Cypress.Commands.add('verifyErrorMessage', (message) => {
    cy.get('[data-cy=error-message]')
        .should('be.visible')
        .and('contain', message)
})

// Performance measurement commands
Cypress.Commands.add('measurePageLoad', (path) => {
    cy.window().then(win => {
        const start = win.performance.now()
        cy.visit(path)
        cy.window().then(() => {
            const end = win.performance.now()
            cy.log(`Page load time for ${path}: ${end - start}ms`)
        })
    })
})

// API testing commands
Cypress.Commands.add('verifyApiResponse', (method, url, expectedStatus = 200) => {
    cy.request({
        method,
        url: `${Cypress.env('apiUrl')}${url}`,
        failOnStatusCode: false
    }).then(response => {
        expect(response.status).to.equal(expectedStatus)
    })
})

// Resource monitoring commands
Cypress.Commands.add('checkResourceUsage', () => {
    cy.window().then(win => {
        if (win.performance.memory) {
            const memoryUsage = win.performance.memory.usedJSHeapSize / (1024 * 1024)
            cy.log(`Memory usage: ${memoryUsage.toFixed(2)}MB`)
        }
    })
})

// Utility function to generate random data
Cypress.Commands.add('generateRandomData', () => {
    const timestamp = Date.now()
    return {
        email: `test-${timestamp}@example.com`,
        password: 'TestPass123!',
        firstName: 'Test',
        lastName: 'User',
        tenantName: `Test Tenant ${timestamp}`,
        botName: `Test Bot ${timestamp}`
    }
})

// Wait for API request to complete
Cypress.Commands.add('waitForApi', (method, url) => {
    cy.intercept(method, url).as('apiRequest')
    cy.wait('@apiRequest')
})

// Check if element exists
Cypress.Commands.add('elementExists', (selector) => {
    cy.get('body').then($body => {
        return $body.find(selector).length > 0
    })
})

// Clear local storage and cookies
Cypress.Commands.add('clearAuthData', () => {
    cy.clearLocalStorage()
    cy.clearCookies()
})

// Wait for loading state to complete
Cypress.Commands.add('waitForLoading', () => {
    cy.get('[data-cy=loading-indicator]').should('not.exist')
})

// Check for error message
Cypress.Commands.add('hasErrorMessage', (message) => {
    cy.get('[data-cy=error-message]')
        .should('be.visible')
        .and('contain', message)
})

// Check for success message
Cypress.Commands.add('hasSuccessMessage', (message) => {
    cy.get('[data-cy=success-message]')
        .should('be.visible')
        .and('contain', message)
})

// Fill form fields
Cypress.Commands.add('fillFormFields', (fields) => {
    Object.entries(fields).forEach(([field, value]) => {
        cy.get(`[data-cy=${field}-input]`).type(value)
    })
})

// Submit form
Cypress.Commands.add('submitForm', (submitButtonSelector = '[data-cy=submit-button]') => {
    cy.get(submitButtonSelector).click()
})

// Check if URL contains path
Cypress.Commands.add('urlContains', (path) => {
    cy.url().should('include', path)
})

// Check if element is disabled
Cypress.Commands.add('isDisabled', (selector) => {
    cy.get(selector).should('be.disabled')
})

// Check if element is enabled
Cypress.Commands.add('isEnabled', (selector) => {
    cy.get(selector).should('not.be.disabled')
})

// Check if element is visible
Cypress.Commands.add('isVisible', (selector) => {
    cy.get(selector).should('be.visible')
})

// Check if element is hidden
Cypress.Commands.add('isHidden', (selector) => {
    cy.get(selector).should('not.be.visible')
})

// Check if element has class
Cypress.Commands.add('hasClass', (selector, className) => {
    cy.get(selector).should('have.class', className)
})

// Check if element has attribute
Cypress.Commands.add('hasAttribute', (selector, attribute, value) => {
    cy.get(selector).should('have.attr', attribute, value)
})

// Check if element has text
Cypress.Commands.add('hasText', (selector, text) => {
    cy.get(selector).should('contain', text)
})

// Check if element has value
Cypress.Commands.add('hasValue', (selector, value) => {
    cy.get(selector).should('have.value', value)
})

// Check if element exists in DOM
Cypress.Commands.add('exists', (selector) => {
    cy.get(selector).should('exist')
})

// Check if element does not exist in DOM
Cypress.Commands.add('notExists', (selector) => {
    cy.get(selector).should('not.exist')
})

// Wait for animation to complete
Cypress.Commands.add('waitForAnimation', () => {
    cy.wait(500) // Adjust the wait time based on your animation duration
})

// Force click on element
Cypress.Commands.add('forceClick', (selector) => {
    cy.get(selector).click({ force: true })
})

// Check if page has title
Cypress.Commands.add('hasTitle', (title) => {
    cy.title().should('include', title)
})

// Check if page has meta description
Cypress.Commands.add('hasMetaDescription', (description) => {
    cy.get('meta[name="description"]').should('have.attr', 'content', description)
})

// Check if element is focused
Cypress.Commands.add('isFocused', (selector) => {
    cy.get(selector).should('have.focus')
})

// Check if element has style
Cypress.Commands.add('hasStyle', (selector, property, value) => {
    cy.get(selector).should('have.css', property, value)
})

// Check if element has computed style
Cypress.Commands.add('hasComputedStyle', (selector, property, value) => {
    cy.get(selector).then($el => {
        expect(window.getComputedStyle($el[0])[property]).to.equal(value)
    })
})

// Check if element is in viewport
Cypress.Commands.add('isInViewport', (selector) => {
    cy.get(selector).then($el => {
        const rect = $el[0].getBoundingClientRect()
        expect(rect.top).to.be.greaterThan(0)
        expect(rect.bottom).to.be.lessThan(Cypress.config('viewportHeight'))
        expect(rect.left).to.be.greaterThan(0)
        expect(rect.right).to.be.lessThan(Cypress.config('viewportWidth'))
    })
})

// Check if element is not in viewport
Cypress.Commands.add('isNotInViewport', (selector) => {
    cy.get(selector).then($el => {
        const rect = $el[0].getBoundingClientRect()
        expect(rect.bottom).to.be.lessThan(0).or.greaterThan(Cypress.config('viewportHeight'))
        expect(rect.right).to.be.lessThan(0).or.greaterThan(Cypress.config('viewportWidth'))
    })
})

// Check if element has focus
Cypress.Commands.add('hasFocus', (selector) => {
    cy.get(selector).should('have.focus')
})

// Check if element does not have focus
Cypress.Commands.add('hasNoFocus', (selector) => {
    cy.get(selector).should('not.have.focus')
})

// Check if element has specific HTML
Cypress.Commands.add('hasHtml', (selector, html) => {
    cy.get(selector).should('have.html', html)
})

// Check if element contains HTML
Cypress.Commands.add('containsHtml', (selector, html) => {
    cy.get(selector).invoke('html').should('contain', html)
})

// Check if element has specific prop
Cypress.Commands.add('hasProp', (selector, prop, value) => {
    cy.get(selector).invoke('prop', prop).should('equal', value)
})

// Check if element has specific data attribute
Cypress.Commands.add('hasData', (selector, attr, value) => {
    cy.get(selector).invoke('data', attr).should('equal', value)
})

// Custom login command
Cypress.Commands.add('login', (email, password) => {
    cy.visit('/login')
    cy.get('[data-cy=email-input]').type(email)
    cy.get('[data-cy=password-input]').type(password)
    cy.get('[data-cy=login-button]').click()
})

// Overwrite scrollIntoView command
Cypress.Commands.overwrite('scrollIntoView', (originalFn, subject, options) => {
    return originalFn(subject, {
        ...options,
        behavior: 'instant',
        block: 'center',
        inline: 'center'
    })
})

// Custom command to reset application state
Cypress.Commands.add('resetAppState', () => {
    cy.request('POST', `${Cypress.env('apiUrl')}/test/reset`)
})

// Custom command to seed test data
Cypress.Commands.add('seedTestData', (endpoint, data) => {
    cy.request('POST', `${Cypress.env('apiUrl')}/test/seed-${endpoint}`, data)
})

// Custom command to check error message
Cypress.Commands.add('checkErrorMessage', (message) => {
    cy.get('[data-cy=error-message]')
        .should('be.visible')
        .and('contain', message)
})

// Custom command to check success message
Cypress.Commands.add('checkSuccessMessage', (message) => {
    cy.get('[data-cy=success-message]')
        .should('be.visible')
        .and('contain', message)
}) 