# FoundryVTT Docker

**This docker container requires an active license.**

- [You can obtain a license from here](https://foundryvtt.com/purchase/)

## Installation Locally

### Running locally

```sh
docker run -rm  -p 4444:4444 \
  -e HOSTNAME="127.0.0.1" \
  -e SSL_PROXY="false" \
  -v ${PWD}/data:/foundrydata \
  mbround18/foundryvtt-docker:latest
```

## Post Installation (Docker)

1. Navigate to your URL [localhost:4444](http://localhost:4444/)
2. In another tab open up your Purchased Licenses page on [foundryvtt.com](https://foundryvtt.com/)
3. Now click the link icon to get a timed link.
4. Tab back over to [localhost:4444](http://localhost:4444/)
5. Paste the timed url into the input field.
6. Click the submit button on the page and watch the logs.
7. If all goes well, navigate to the base url `http://localhost:4444/` and you should be greeted with the FoundryVTT setup screen :)

