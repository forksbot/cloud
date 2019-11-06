# Cloud Addon Registry

Addon Registry backend, used by the Addon Registry CLI and website.
This service interacts with Github to alter the registry file.

Endpoints:

* GET `/update_stats` To be called periodically.
  Accumulates and transfers voting and download stats from Firestore to the registry file.
* PUT `/addon/<addonid>` Adds / Edits an Addon. Will succeed if the received json is valid and the given version
  is equal or greater than the former version.
* DELETE `/addon/<addonid>` Sets maintenance status to Unmaintained, so that this Addon does not appear
  in Registry Addon listings anymore. The Addon is not really removed to not break existing installations.
  A warning will be issued to users who have installed this Addon and are connected via the Cloud Connector. 
