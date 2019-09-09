#/bin/sh
source appenv
gcloud beta run domain-mappings create --service $APP --domain $DOMAIN --platform managed --region us-central1