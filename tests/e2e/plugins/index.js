/// <reference types="cypress" />

/**
 * @type {Cypress.PluginConfig}
 */
module.exports = (on, config) => {
    // Browser launch configuration
    on('before:browser:launch', (browser = {}, launchOptions) => {
        if (browser.name === 'chrome' || browser.name === 'chromium') {
            launchOptions.args.push('--remote-allow-origins=*')
        }
        return launchOptions
    })

    // Add file preprocessor for handling test data files
    on('task', {
        log(message) {
            console.log(message)
            return null
        },
        table(message) {
            console.table(message)
            return null
        }
    })

    return config
} 