# Cloud-Deployment team

These notes are for the cloud deployment team.

### Building services

Stateless cloud functions, no matter if on the Google (*Cloud Run*) or Amazon (*Lambda*) infrastructure, are accepted 
as Docker compatible containers. Long runnong applications are also deployed as containers on Amazon ECS.

> A slim container (~ a few Megabytes) is preferred!

The binaries are compiled with [musl](https://www.musl-libc.org/) instead of libc to get a fully static, self-contained binary.
It is forbidden to introduce a dependency on any Rust Crate that links to a C library (like openSSL)!

Check with `ldd target/x86_64-unknown-linux-musl/debug/cloud-vault` for example.

You need the "x86_64-unknown-linux-musl" rust target on rust nightly to be installed.
Use `rustup toolchain nightly` to install rust nightly and `rustup target add x86_64-unknown-linux-musl` to add the target. 

### Deployment of stateless functions (GCloud)

Navigate to the root directory of the repository.

Call `./scripts/build-and-deploy.sh service-name` and replace "service-name" with one of the service directory names.
This builds the selected service in release mode and uploads a snapshot of the directory to Google Cloud Build.
The resulting container is deployed to G- Cloud Run.

* If the domain mapping got lost, restore it by calling `./scripts/gcloud-domain-map.sh service-name`.
* If the cron jobs are lost, restore those by calling `./scripts/gcloud-cron-setup.sh service-name`.
