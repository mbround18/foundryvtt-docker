<img width="500" src="https://repository-images.githubusercontent.com/261890725/ef8c0180-be60-11eb-987b-2e45ff426696" />

<!-- Rebuild, commenting for release due to CI issue, remove me later -->

# FoundryVTT Docker

**This docker container requires an active license.**

- [You can obtain a license from here](https://foundryvtt.com/purchase/)

While there are many docker containers which can serve up FoundryVTT, this container was created with simplicity in mind. 
There are no additional credentials you need to supply, web driver, or web automation required. The installation process is simplified 
by presenting you with an easy-to-use web interface to install the application by using a timed url provided by Foundry. 

## Installation Locally

### Running locally

```sh
docker run --rm -it \
  -p 4444:4444 \
  -e HOSTNAME="127.0.0.1" \
  -e SSL_PROXY="false" \
  -v ${PWD}/foundry/data:/foundrydata \
  -v ${PWD}/foundry/app:/foundryvtt \
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

