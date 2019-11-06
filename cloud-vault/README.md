# Cloud Vault

[![Build Status](https://github.com/openhab-nodes/cloud-ci-cd/workflows/Integration/badge.svg)](https://github.com/openhab-nodes/cloud-ci-cd/actions)
[![](https://img.shields.io/badge/license-MIT-blue.svg)](http://opensource.org/licenses/MIT)

This service, served under vault.openhabx.com, generates access tokens for the GCloud service account.

It also provides access to all secrets that are necessary to setup and run openhabx.com
and distributes new deployment keys periodically to the  CI/CD service (Travis CI).

Endpoints:
* `/{secrets_file}?code={token}`: Returns the requested secrets file.
  This is one of "travis-token.txt", "github-access.json", "google-ci-key.json", "docker-access.json", "docker-token.txt", "jwtRS256.key"
  Returns 401 if the token is incorrect.
* `/nenew`: Renews all access tokens via the Travis CI API. Must be called by a cron job periodically.
  This endpoint is only accessible via the GCloud Cron service, ie it requires an oauth OIC token of the GCloud travis-ci service account.
* `/jwtRS256.key.pub`: The public key part of the jwt token signing pair.

## How CI/CD service deployment works

All OHX core and addon services are bundled as software containers
and are deployed to the openhabx organisation of the docker.io registry.

Travis CI needs login access to the registry to deploy newly build versions.
It will ask this service for a docker access token, which it will only receive
if it can authenticate.

That way the normal `docker login` or `podman login` can be used:

```bash
CRED=`curl -H "Authorization: Bearer $DEPLOY_ACCESS_TOKEN" vault.openhabx.com/get/docker-token.txt` && \
 echo $CRED | jq -r '.Secret' | docker login -u $(echo $CRED | jq -r '.Username') --password-stdin docker.io && \
 unset CRED
```

You can also use the GCloud Cloud Build registry (https://gcr.io), which speaks the docker registry API.
See https://cloud.google.com/container-registry/docs/advanced-authentication for more information.

Authentication happens via an access token (`DEPLOY_ACCESS_TOKEN` environment variable),
that is set and renewed by this service via the [Travis CI API](https://developer.travis-ci.com/resource/env_var#Env%20var)
periodically every 6 hours.

For this to happen you need to enter all Travis CI enabled repositories in the `repositories.json` file.

A token is a [Time-Based One-Time Password](https://tools.ietf.org/html/rfc6238) with a period of 6 hours.
The seed key for this service is generated on deployment.
If a token leaked, it is therefore sufficient to rebuild and redeploy this service.
