#!/bin/bash -e
source appenv
PROJECT_ID="$(gcloud config get-value project -q)"

secs_to_mins_and_secs() {
    if [[ -z ${1} || ${1} -lt 60 ]] ;then
        min=0 ; secs="${1}"
    else
        time_mins=$(echo "scale=2; ${1}/60" | bc)
        min=$(echo ${time_mins} | cut -d'.' -f1)
        secs="0.$(echo ${time_mins} | cut -d'.' -f2)"
        secs=$(echo ${secs}*60|bc|awk '{print int($1+0.5)}')
    fi
    echo "Time Elapsed : ${min} minutes and ${secs} seconds."
}

# Print the elapsed time
start=$(date +%s)
function finish {
  secs_to_mins_and_secs "$(($(date +%s) - start))"
}
trap finish EXIT

if [ ! -f ./target/upx ]; then
  mkdir -p target && wget https://github.com/upx/upx/releases/download/v3.95/upx-3.95-amd64_linux.tar.xz
  tar xf upx-3.95-amd64_linux.tar.xz && cp ./upx-3.95-amd64_linux/upx target/upx && chmod +x upx && rm -rf upx-3.95-amd64_linux*
fi

if [ ! -f ./secrets/access_scopes.json ]; then
  gcloud "--project=${PROJECT_ID}" source repos clone vault secrets_temp && rm -rf secrets && mv secrets_temp secrets
fi

# Build and copy to ./ohx-app
cargo build --release --target x86_64-unknown-linux-musl
cp "target/x86_64-unknown-linux-musl/release/${APP}" ./ohx-app

# Strip and compress binary
strip ohx-app && ./target/upx ohx-app

# Submit binary to cloud build
gcloud config set builds/use_kaniko False
gcloud builds submit --timeout=2m --tag "gcr.io/${PROJECT_ID}/${APP}"
rm ohx-app
# Deploy new image and delete all but the :latest container images
gcloud beta run deploy --image "gcr.io/${PROJECT_ID}/${APP}" --platform managed --region us-central1 --allow-unauthenticated "${APP}" --memory 128Mi --timeout 60
gcloud container images list-tags "gcr.io/${PROJECT_ID}/${APP}" --filter='-tags:*' --format='get(digest)' --limit=unlimited | \
    awk -v "PROJECT_ID=$PROJECT_ID" -v "APP=$APP" '{print "gcr.io/" PROJECT_ID "/" APP "@" $1}' | xargs gcloud container images delete --quiet

