# Initial infrastructure setup

This generator tool in this directory will create an X.509 certificate, public key and private key file as well as a JWKS file.
The private key allows to create access tokens in the name of OHX and should never be shared!

OHX cloud services run on GClouds Cloud Run for stateless functions and Amazon EC2 Tiny for the 24/7 message broker service
and a tiny Redis instance. 

There are a few secrets, and external services, that need to be setup first.

Start the tool with `cargo run --bin create-secrets` in the root directory of the repository.

### GCloud Cloud Run Service Account

If not done yet, you need to authenticate the GCloud cli tool via `gcloud auth login` and change
to the correct project, for example via `gcloud config set project openhabx`.

The `gcloud-setup.sh` script sets up a GCloud service account, used for CI/CD deployments. 

The resulting `travisci-deployer@openhabx.iam.gserviceaccount.com.key`
file from the `gcloud_setup.sh` script must be moved to the `secrets` directory.

The service account is used to upload build artifacts or sources to the GCloud to be deployed to the Google Cloud Run service.

### Firebase project

Create a firebase project and download the credentials file to
"secrets/openhabx-device@openhabx.iam.gserviceaccount.com.key".

Firebase Auth is used as user database.

### Redis Database

Redis is used to store temporary tokens during the OAuth device-flow procedure.
Create a file "redis.txt" in the "secrets" directory with the redis URL including user and password parts:

```
redis://user:3467463427858986634234@redis-181.c78.eu-west-1-2.ec2.cloud.redislabs.com:18381
```

### Github token

The addon registry uses a Github repository as persistent storage.
Create a Github token file "github-access.json" in "secrets" with content similar to this:

```json
{
    "user":"openhab-nodes-bot",
    "email":"bot@openhabx.com",
    "password":"the password"
}
```

The referenced account requires access to the Github repository that hosts the addon repo files.

### Docker Access

Docker.io is used for OHX Addon containers. Create a "docker-access.json" file in "secrets":

```json
{
    "Username":"the_username",
    "Secret":"the_password"
}
```

The Addon CLI will request this file via the vault service to upload containers to the registry.

### Travis CI Token

You must also login to Travis CI, to compile in a Travis CI token.

* Install the travis CLI (`gem install travis`).
* Execute `travis login` and login in with a Github account that has access to the "openhab-nodes" organisation.
* Execute `echo "$(travis token)" > secrets/travis-token.txt`
