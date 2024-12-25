const express = require('express');
const cors = require('cors');
const bodyParser = require('body-parser');
const multer = require('multer');
const upload = multer({ dest: 'uploads/' });

const app = express();
const port = 8080;

app.use(cors());
app.use(bodyParser.json());

// Store state
let state = {
    bots: [],
    tenantLimits: {
        maxBots: 10
    },
    resourceConstraints: {
        gpu: true
    },
    serviceStatus: {
        inference: 'healthy'
    }
};

// Reset test data
app.post('/test/reset', (req, res) => {
    state = {
        bots: [],
        tenantLimits: {
            maxBots: 10
        },
        resourceConstraints: {
            gpu: true
        },
        serviceStatus: {
            inference: 'healthy'
        }
    };
    res.json({ success: true });
});

// Update tenant limits
app.put('/test/update-tenant-limits', (req, res) => {
    state.tenantLimits = { ...state.tenantLimits, ...req.body };
    res.json({ success: true });
});

// Simulate resource constraint
app.put('/test/simulate-resource-constraint', (req, res) => {
    const { resourceType, available } = req.body;
    state.resourceConstraints[resourceType] = available;
    res.json({ success: true });
});

// Simulate service disruption
app.put('/test/simulate-service-disruption', (req, res) => {
    const { serviceType, status } = req.body;
    state.serviceStatus[serviceType] = status;
    res.json({ success: true });
});

// Create bot
app.post('/api/bots', (req, res) => {
    if (state.bots.length >= state.tenantLimits.maxBots) {
        return res.status(400).json({
            success: false,
            error: 'Bot limit exceeded'
        });
    }

    const bot = {
        id: `bot${state.bots.length + 1}`,
        ...req.body,
        status: 'created'
    };
    state.bots.push(bot);

    res.json({
        success: true,
        bot
    });
});

// Train bot
app.post('/api/bots/:botId/train', upload.single('file'), (req, res) => {
    const bot = state.bots.find(b => b.id === req.params.botId);
    if (!bot) {
        return res.status(404).json({
            success: false,
            error: 'Bot not found'
        });
    }

    if (!req.file) {
        return res.status(400).json({
            success: false,
            error: 'No training data provided'
        });
    }

    // Simulate file validation
    if (req.file.originalname === 'invalid-data.jsonl') {
        return res.status(400).json({
            success: false,
            error: 'Invalid training data'
        });
    }

    bot.status = 'training';
    res.json({
        success: true,
        status: 'training'
    });
});

// Deploy bot
app.post('/api/bots/:botId/deploy', (req, res) => {
    const bot = state.bots.find(b => b.id === req.params.botId);
    if (!bot) {
        return res.status(404).json({
            success: false,
            error: 'Bot not found'
        });
    }

    if (!state.resourceConstraints.gpu) {
        return res.status(400).json({
            success: false,
            error: 'Insufficient resources'
        });
    }

    bot.status = 'deploying';
    res.json({
        success: true,
        status: 'deploying'
    });
});

// Inference
app.post('/api/bots/:botId/infer', (req, res) => {
    const bot = state.bots.find(b => b.id === req.params.botId);
    if (!bot) {
        return res.status(404).json({
            success: false,
            error: 'Bot not found'
        });
    }

    if (state.serviceStatus.inference !== 'healthy') {
        return res.status(503).json({
            success: false,
            error: 'Service unavailable'
        });
    }

    res.json({
        success: true,
        response: 'Mock response',
        responseTime: 100,
        tokenUsage: 50
    });
});

app.listen(port, () => {
    console.log(`Mock API server running at http://localhost:${port}`);
}); 