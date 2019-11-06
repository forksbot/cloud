#!/bin/bash -e
# shellcheck source=cloud-auth/appenv
source "$1/appenv"
echo "$CRONMESSAGE"
echo "y" | gcloud scheduler jobs delete $APP
gcloud scheduler jobs create http "$APP" --time-zone "Etc/UTC" --description "$CRONMESSAGE" --schedule "$CRONEXPR" --uri "$CRONURI" --http-method GET "--oidc-service-account-email=${CRONSERVICEACCOUNT}"
