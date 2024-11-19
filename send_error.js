fetch('https://o4506874198163456.ingest.us.sentry.io/api/4507145788391424/envelope/', {
    method: 'POST',
    headers: {
        'X-Sentry-Auth': 'Sentry sentry_version=7, sentry_client=custom-client/1.0, sentry_key=a0ae9168ae01c272f11c328b55aacb61',
        'Content-Type': 'application/json'
    },
    body: [
        {
            "event_id": [...Array(32)].map(() => Math.floor(Math.random() * 16).toString(16)).join(''),
            "sent_at": new Date().toISOString(),
            "sdk": {"name": "sentry.javascript.node", "version": "8.38.0"},
            "trace": {"environment": "production", "public_key": "a0ae9168ae01c272f11c328b55aacb61"}
        },
        {"type": "event"},
        {
            "exception": {
                "values": [{
                    "type": "Crash",
                    "value": "Sameole crash repio",
                    "stacktrace": {},
                    "mechanism": {
                        "type": "generic",
                        description: "Holy shit",
                        "handled": false,
                        "synthetic": true,
                        data: {
                            strace: "Yabbana dbabdbsad doooooo"
                        }
                    }
                }]
            },
            "platform": "node",
            "server_name": "Mridangs-MacBook-Pro.local",
            "environment": "production"
        }
    ]
        .map(event => JSON.stringify(event))  // Use map to stringify each object
        .join('\n')
})
    .then(response => response.json())
    .then(data => console.log('Response:', data))
    .catch(error => console.error('Error:', error));
