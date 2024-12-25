describe('Bot Management', () => {
    beforeEach(() => {
        cy.visit('http://localhost:3000');
    });

    describe('Bot Creation', () => {
        it('should successfully create a new bot', () => {
            cy.get('#botName').type('Test Bot');
            cy.get('#botModel').select('gpt-4');
            cy.get('#botDescription').type('A test bot');
            cy.get('#bot-creation').submit();
            cy.get('#bot-creation .success-message').should('be.visible').and('contain', 'Bot created successfully');
        });

        it('should show error when exceeding tenant bot limit', () => {
            cy.request({
                method: 'PUT',
                url: 'http://localhost:8080/test/update-tenant-limits',
                body: { maxBots: 1 }
            });

            cy.get('#botName').type('First Bot');
            cy.get('#botModel').select('gpt-4');
            cy.get('#bot-creation').submit();
            cy.get('#bot-creation .success-message').should('be.visible');

            cy.get('#botName').clear().type('Second Bot');
            cy.get('#bot-creation').submit();
            cy.get('#bot-creation .error-message').should('be.visible').and('contain', 'Bot limit exceeded');
        });
    });

    describe('Bot Training', () => {
        beforeEach(() => {
            cy.get('#botName').type('Training Test Bot');
            cy.get('#botModel').select('gpt-4');
            cy.get('#bot-creation').submit();
            cy.get('#bot-creation .success-message').should('be.visible');
        });

        it('should successfully initiate and monitor training', () => {
            cy.get('#training-data-upload').selectFile('cypress/fixtures/training-data.jsonl');
            cy.get('#training-form').submit();
            cy.get('#training-status').should('contain', 'Training in progress');
            cy.get('#training-progress').should('be.visible');
        });

        it('should handle training errors gracefully', () => {
            cy.get('#training-data-upload').selectFile('cypress/fixtures/invalid-data.jsonl');
            cy.get('#training-form').submit();
            cy.get('#training-form .error-message').should('be.visible').and('contain', 'Invalid training data');
        });
    });

    describe('Bot Deployment', () => {
        beforeEach(() => {
            cy.get('#botName').type('Deployment Test Bot');
            cy.get('#botModel').select('gpt-4');
            cy.get('#bot-creation').submit();
            cy.get('#bot-creation .success-message').should('be.visible');
        });

        it('should successfully deploy a trained bot', () => {
            cy.get('#deploymentEnv').select('production');
            cy.get('#minReplicas').type('2');
            cy.get('#maxReplicas').type('5');
            cy.get('#deployment-form').submit();
            cy.get('#deployment-status').should('contain', 'Deployment in progress');
            cy.get('#deployment-progress').should('be.visible');
        });

        it('should handle deployment failures gracefully', () => {
            cy.request({
                method: 'PUT',
                url: 'http://localhost:8080/test/simulate-resource-constraint',
                body: { resourceType: 'gpu', available: false }
            });

            cy.get('#deploymentEnv').select('production');
            cy.get('#deployment-form').submit();
            cy.get('#deployment-form .error-message').should('be.visible').and('contain', 'Insufficient resources');
        });
    });

    describe('Bot Inference', () => {
        beforeEach(() => {
            cy.get('#botName').type('Inference Test Bot');
            cy.get('#botModel').select('gpt-4');
            cy.get('#bot-creation').submit();
            cy.get('#bot-creation .success-message').should('be.visible');
        });

        it('should successfully make inference requests', () => {
            cy.get('#testInput').type('Hello, bot!');
            cy.get('#inference-form').submit();
            cy.get('#bot-response').should('be.visible');
            cy.get('#response-time').should('be.visible');
            cy.get('#token-usage').should('be.visible');
        });

        it('should handle inference errors gracefully', () => {
            cy.request({
                method: 'PUT',
                url: 'http://localhost:8080/test/simulate-service-disruption',
                body: { serviceType: 'inference', status: 'degraded' }
            });

            cy.get('#testInput').type('Hello, bot!');
            cy.get('#inference-form').submit();
            cy.get('#inference-form .error-message').should('be.visible').and('contain', 'Service unavailable');
        });
    });
}); 