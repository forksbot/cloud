# Cloud Subscriptions

Subscriptions and payments backend.
This service is used by the OHX Website to bill users. Although open source, the running infrastructure
on Google and Amazon servers incur costs that need to be paid.

This service interacts with the Braintree API for credit card and Paypal payments.

Endpoints:
* `/check_payments` To be called periodically. Checks braintree and IBAN payment status and updates user accounts.
* `/client_token` Creates a braintree client token for the UI. Creates a braintree customer first if necessary.
* `/confirm` Confirms a braintree payment.
  This will either vault the payment method and charge it once or charge it directly; update the user account
