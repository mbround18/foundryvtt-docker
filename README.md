<img width="500" src="https://repository-images.githubusercontent.com/261890725/ef8c0180-be60-11eb-987b-2e45ff426696" />

# FoundryVTT Docker

<div align="center">
  <a href="https://hub.docker.com/r/mbround18/foundryvtt-docker"><img src="https://img.shields.io/docker/pulls/mbround18/foundryvtt-docker?style=for-the-badge" alt="Docker Pulls"></a>
  <a href="https://github.com/mbround18/foundryvtt-docker/blob/main/LICENSE.md"><img src="https://img.shields.io/github/license/mbround18/foundryvtt-docker?style=for-the-badge" alt="License"></a>
  <a href="https://github.com/mbround18/foundryvtt-docker/stargazers"><img src="https://img.shields.io/github/stars/mbround18/foundryvtt-docker?style=for-the-badge" alt="GitHub Stars"></a>
</div>

## Overview

**⚠️ This docker container requires an active Foundry VTT license. [Purchase one here](https://foundryvtt.com/purchase/).**

A streamlined Docker container for Foundry Virtual Tabletop with an Actix-powered web uploader. This container was designed with simplicity in mind - no credentials to supply, no web driver configurations, and no web automation required.

### Key Features

- 🚀 **Simple Installation** - Easy-to-use web interface for installation
- 🔒 **Secure** - No credentials stored in the container
- 🔄 **Persistent Storage** - Mount volumes for data and application
- 🌐 **Flexible Networking** - Configurable hostname and SSL options

## Quick Start

### Running with Docker

```sh
docker run --rm -it \
  -p 4444:4444 \
  -e HOSTNAME="127.0.0.1" \
  -e SSL_PROXY="false" \
  -v ${PWD}/foundry/data:/foundrydata \
  -v ${PWD}/foundry/app:/foundryvtt \
  mbround18/foundryvtt-docker:latest
```

### Running with Docker Compose

Create a `docker-compose.yml` file:

```yaml
version: "3"
services:
  foundry:
    image: mbround18/foundryvtt-docker:latest
    ports:
      - "4444:4444"
    environment:
      - HOSTNAME=127.0.0.1
      - SSL_PROXY=false
    volumes:
      - ./foundry/data:/foundrydata
      - ./foundry/app:/foundryvtt
    restart: unless-stopped
```

Then run:

```sh
docker-compose up -d
```

## Installation Process

1. Launch the container using one of the methods above
2. Navigate to your installation URL: [http://localhost:4444](http://localhost:4444/)
3. In another tab, open your Purchased Licenses page on [foundryvtt.com](https://foundryvtt.com/)
4. Click the link icon to generate a timed download link
5. Return to [http://localhost:4444](http://localhost:4444/) and paste the timed URL
6. Click the submit button and monitor the logs
7. When complete, navigate to [http://localhost:4444/](http://localhost:4444/) to access the Foundry VTT setup screen

## Environment Variables

| Variable              | Description                             | Default   |
| --------------------- | --------------------------------------- | --------- |
| `HOSTNAME`            | The hostname for the server             | `0.0.0.0` |
| `SSL_PROXY`           | Whether SSL is being handled by a proxy | `false`   |
| `APPLICATION_PORT`    | The port the application runs on        | `4444`    |
| `ADMIN_KEY`           | Admin password for Foundry              | _(empty)_ |
| `MINIFY_STATIC_FILES` | Whether to minify static files          | `true`    |

## Volumes

| Path           | Description                            |
| -------------- | -------------------------------------- |
| `/foundrydata` | Foundry user data, worlds, and modules |
| `/foundryvtt`  | Foundry application files              |

## Troubleshooting

### Common Issues

- **Port already in use**: Change the port mapping in your docker run command (e.g., `-p 8080:4444`)
- **Permissions errors**: Ensure your mounted volumes have the correct permissions
- **Download failures**: Verify your Foundry license and that the timed URL is still valid

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## License

This project is licensed under the [BSD 3-Clause License](LICENSE.md).
