#!/bin/bash
# shellcheck source=cloud-auth/appenv
source "$1/appenv"
gcloud beta run domain-mappings create --service "$APP" --domain "$DOMAIN" --platform managed --region us-central1