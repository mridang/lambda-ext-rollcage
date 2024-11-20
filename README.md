# Rollcage

Rollcage is an extension for AWS Lambda that monitors the telemetry
sent by AWS, looks for crashes and reports them to Sentry.

This means that you receive crash reports even when your lambda
function is not operational.

### Usage

To use the extension, simply reference the publicly available layer:

```
arn:aws:lambda:us-east-1:188628773952:layer:apx:14
```

More information on how layers work and their usage can be found at
https://docs.aws.amazon.com/lambda/latest/dg/chapter-layers.html

The layer requires two environment variables to be set:

* `SENTRY_DSN`: Indicates the endpoint (Data Source Name) for your project e.g `https://a0ae9328b5566cb61@o4506874198163456.ingest.us.sentry.io/4507144488391424`
* `NODE_ENV`: Indicates the name of the current environment in which the application is running e.g. `production`

## Table of Contents

- [Architecture](#architecture)
- [Developing](#developing)
- [Contributing](#contributing)
- [License](#license)

## Architecture

This is a very lightweight Rust-based service that runs as an extension for AWS
Lambda.

These extensions run as a companion to the function code.
https://docs.aws.amazon.com/lambda/latest/dg/telemetry-api.html

> The Telemetry API enables your extensions to receive telemetry data 
> directly from Lambda. During function initialization and invocation, 
> Lambda automatically captures telemetry, including logs, platform metrics, 
> and platform traces. The Telemetry API enables extensions to access this 
> telemetry data directly from Lambda in near real time.

Since extensions are expected to be lightweight, Rust was chosen. Since the 
extension is expected to run on any runtime and not specifically Amazon Linux 2,
the extension target [MUSL](https://musl.libc.org/) to avoid GLIBC compatibility errors. 
To further increase the portability, the extension does not require OpenSSL and
instead uses [RustTLS](https://github.com/rustls/rustls).

When a Lambda function invocation fails, the Telemetry API sends the log message
to the extension. The extension filters out messages that contain a mention
of the `Runtime.HandlerNotFound` error and if found, reports it to Sentry. 
To keep the extension footprint small, the Sentry SDK was omitted and instead the
crashes are reported using the [simple reporting endpoints used by SDK developers](https://develop.sentry.dev/sdk/overview/).

## Developing

The app is built with Rust using the latest stable Rust version.

After checking out the repository, run `cargo fetch` to install all
the required dependencies.

### Configure the local environment

Configure the `.env` file with the necessary information. This file should be set up
with example values for a template, but you'll need to replace them with actual data
relevant to your application. Here’s a detailed breakdown of each variable:

- `SENTRY_DSN`: Directs errors and performance data to Sentry for monitoring, aiding in
  quick identification and resolution of issues.

> [!NOTE]
> It is fine to add sensitive information to this file as this file only
> serves as a template and Git has been configured to not track any
> changes this file using `git update-index --assume-unchanged .env`

---

### Configure the GitHub environment

To ensure the smooth operation of GitHub Actions within this project, it's
essential to configure certain environment variables and secrets. These settings
are crucial for various deployment tasks and integrating with external services
like AWS.

You need to set the following environment variables in the GitHub repository
settings:

- `AWS_REGION`: The AWS region where your services are deployed, e.g., `us-east-1`.

These variables are used by GitHub Actions workflows to configure the deployment
environment correctly.

Additionally, you must configure the following secrets in your GitHub repository.
These secrets are sensitive and provide access to external services essential for
deployments and monitoring:

- `AWS_ACCESS_KEY_ID`: Your AWS access key ID, used by Serverless for deployments.
- `AWS_SECRET_ACCESS_KEY`: Your AWS secret access key, used by Serverless for deployments.

Please treat these secrets with the utmost care and never expose them publicly.

> [!IMPORTANT]
> Deployments will not work correctly if these environment variables and secrets
> are not configured properly. Ensure that you've entered the correct values
> corresponding to your AWS and Sentry accounts to avoid any deployment issues.

---

### Linting the code

Lint the code using `devbox run lint`. This command runs clippy and
lints all the files. 

### Formatting the code

Reformat the code using `devbox run format`. This runs cargofmt and
reformats all the code.

> [!NOTE]
> GitHub Actions has been configured to automatically reformat all the
> code on every commit and commit the changes back to the branch.

### Deploying the extension

The application is automatically deployed when a push is made to the
default branch. You can manually trigger the deployment workflow if
you need to deploy the latest changes.

It is not recommended to deploy from your local machine but if needed,
it can be deployed using `devbox run deploy`.

> [!IMPORTANT]
> You'll need to ensure that you have the AWS credentials configured. Read the
> guide on how to configure the variables https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-envvars.html

Once deployed, the ARN of the newly published layer is printed.

### Running tests

Run the test suite using `devbox run test`.

The project has been configured to automatically collect coverage from tests,
and these can be found in the `target` directory.

### Running the extension

There isn't a way to run this locally at the time of writing. Lambda extensions
connect to a local API which hasn't been emulated.

## Contributing

If you have suggestions for how this extension could be improved, or
want to report a bug, open an issue - we'd love all and any
contributions.

## License

Apache License 2.0 © 2024 Mridang Agarwalla
