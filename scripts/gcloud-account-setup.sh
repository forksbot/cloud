#!/bin/bash
PROJECT_ID="$(gcloud config get-value project -q)"
SVCACCT_NAME="travisci-deployer"
gcloud iam service-accounts create "${SVCACCT_NAME?}"
SVCACCT_EMAIL="$(gcloud iam service-accounts list \
  --filter="name:${SVCACCT_NAME?}@"  \
  --format=value\(email\))"
gcloud iam service-accounts keys create "travisci-deployer@openhabx.iam.gserviceaccount.com.key" \
   --iam-account="${SVCACCT_EMAIL?}"
# Storage Admin: Used for pushing docker images to Google Container Registry (GCR).
gcloud projects add-iam-policy-binding "${PROJECT_ID?}" \
   --member="serviceAccount:${SVCACCT_EMAIL?}" \
   --role="roles/storage.admin"
# Cloud Run Admin: Used for deploying services to Cloud Run.
gcloud projects add-iam-policy-binding "${PROJECT_ID?}" \
   --member="serviceAccount:${SVCACCT_EMAIL?}" \
   --role="roles/run.admin"
# IAM Service Account user: Required by Cloud Run to be able to "act as" the runtime identity of the Cloud Run application
gcloud projects add-iam-policy-binding "${PROJECT_ID?}" \
   --member="serviceAccount:${SVCACCT_EMAIL?}" \
   --role="roles/iam.serviceAccountUser"
echo "${SVCACCT_EMAIL?}"
