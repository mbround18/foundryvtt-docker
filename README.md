# FoundryVTT Docker

This is based on the FoundryVTT application, pleaase check them out over on Patreon! :) 
This docker container requires an active license to their beta. 

Please check them out here: [FoundryVTT Patreon](https://www.patreon.com/foundryvtt/posts)

## Installation 

### Running locally

```sh
docker run -rm  \
    -p 4444:4444 \
    -e HOSTNAME="127.0.0.1" \
    -e SSL_PROXY="false" \
    -e PATREON_LINK="NNN-XXXXXXXXXXXX/foundryvtt-N.N.N.zip"  \
    mbround18/foundryvtt:latest
```

Then navigate to http://127.0.0.1:4444

### Running locally with data persistance

```sh
docker run -rm  \
    -p 4444:4444 \
    -e HOSTNAME="127.0.0.1" \
    -e SSL_PROXY="false" \
    -e PATREON_LINK="NNN-XXXXXXXXXXXX/foundryvtt-N.N.N.zip"  \
    -v ${PWD}/data:/foundrydata \ 
    mbround18/foundryvtt:latest
```

Then navigate to http://127.0.0.1:4444


### Running in Kubernetes

TBD
