const express = require('express');
const app = express();
app.use(express.json());

let registeredExtensions = {};

app.get('/', (req, res) => {
    res.status(200).send("ok");
})

// Emulate the /extension/register endpoint
app.post('/2020-01-01/extension/register', (req, res) => {
    console.log("register");
    const extensionName = req.headers['lambda-extension-name'];
    const acceptFeature = req.headers['lambda-extension-accept-feature'];

    if (!extensionName) {
        return res.status(400).json({ error: 'Missing Lambda-Extension-Name header' });
    }

    const extensionId = Math.random().toString(36).substring(2, 15);
    registeredExtensions[extensionId] = {
        name: extensionName,
        events: req.body.events || [],
        features: acceptFeature ? acceptFeature.split(',') : []
    };

    res.setHeader('Lambda-Extension-Identifier', extensionId);
    res.status(200).json({
        functionName: 'mockFunction',
        functionVersion: '$LATEST',
        handler: 'index.handler'
    });
});

// Emulate the /extension/event/next endpoint
app.get('/2020-01-01/extension/event/next', (req, res) => {
    console.log("Waiting for 10 seconds...");

    setTimeout(() => {
        console.log("10 seconds passed. Returning shutdown event.");
        res.status(200).json({
            eventType: 'SHUTDOWN',
            shutdownReason: 'spindown',
            deadlineMs: Date.now() + 1000 // Adjusting the deadline for 1 second from now
        });
    }, 10000); // Wait for 10 seconds
});

// Emulate the /extension/init/error endpoint
app.post('/2020-01-01/extension/init/error', (req, res) => {
    console.log("initerr");
    console.error('Extension initialization error:', req.body.errorMessage);
    res.status(200).json({ message: "Initialization error logged" });
});

// Emulate the /extension/exit/error endpoint
app.post('/2020-01-01/extension/exit/error', (req, res) => {
    console.log("exiterr");
    console.error('Extension exit error:', req.body.errorMessage);
    res.status(200).json({ message: "Exit error logged" });
});

const PORT = process.env.PORT || 3099;
app.listen(PORT, () => console.log(`Mock Lambda environment running on port ${PORT}`));
