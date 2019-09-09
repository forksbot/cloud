#/bin/sh
source appenv
echo "Setup cron job for 3am"
echo "y" | gcloud scheduler jobs delete $APP
gcloud scheduler jobs create http $APP --schedule "0 3 * * *" --uri "$CRONURI" --http-method GET --oidc-service-account-email=${CRONSERVICEACCOUNT}
