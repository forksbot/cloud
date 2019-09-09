#!/bin/sh
gcloud --project=openhabx source repos clone vault secrets_temp && rm -rf secrets && mv secrets_temp secrets || true