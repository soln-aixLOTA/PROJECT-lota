const { defineConfig } = require('cypress')

module.exports = defineConfig({
    e2e: {
        baseUrl: 'http://localhost:3000', // Adjust this based on your frontend service URL
        specPattern: 'tests/e2e/specs/**/*.cy.{js,jsx,ts,tsx}',
        supportFile: 'tests/e2e/support/e2e.js',
        fixturesFolder: 'tests/e2e/fixtures',
        screenshotsFolder: 'tests/e2e/screenshots',
        videosFolder: 'tests/e2e/videos',
        downloadsFolder: 'tests/e2e/downloads',
        viewportWidth: 1280,
        viewportHeight: 720,
        defaultCommandTimeout: 10000,
        requestTimeout: 10000,
        responseTimeout: 10000,
        pageLoadTimeout: 30000,
        video: true,
        screenshotOnRunFailure: true,
        env: {
            apiUrl: 'http://localhost:8080'
        },
        setupNodeEvents(on, config) {
            on('before:browser:launch', (browser = {}, launchOptions) => {
                if (browser.name === 'chrome' || browser.name === 'chromium') {
                    launchOptions.args.push('--remote-allow-origins=*')
                }
                return launchOptions
            })
            return config;
        },
        retries: {
            runMode: 2,
            openMode: 0
        },
        experimentalStudio: true,
        experimentalWebKitSupport: true,
        chromeWebSecurity: false,
        watchForFileChanges: false,
        experimentalMemoryManagement: true,
        numTestsKeptInMemory: 0
    },
}) 