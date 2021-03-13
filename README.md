# `turbocheck`

## What is this?

`turbocheck` is a command line tool for monitoring COVID-19 vaccination
appointments in the New York metropolitan area.

It uses data from https://turbovax.info to display terminal output with
appointment information (straight from TurboVax), including a Google Maps link
to the appointment site.

Here's an example of some terminal output when an appointment is found:

```
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax: ------------------------------ BEGIN -------------------------------
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax: 2021-03-13 14:27:34 -05:00 Manhattan: appointments available!
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax:
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax: Site: Harlem Hospital
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax:
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax: Area: Manhattan
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax: Sched: https://covid19.nychealthandhospitals.org/COVIDVaxEligibility
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax: Map: https://is.gd/wQ2JVQ
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax:
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax: Times: Mar 13 – 3:10PM, 3:20PM, 3:30PM + 1
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax:
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax: Appts Remaining: 4
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax: Last Updated: 2021-03-13 14:27:34 -05:00
Mar 13 14:27:35.006  INFO TurboxVaxClient::check_availability: turbocheck::turbovax: ------------------------------- END --------------------------------
```

It will also output a log message when a site has no more available appointments:

```
Mar 13 13:55:21.367  WARN TurboxVaxClient::check_availability: turbocheck::turbovax: 2021-03-13 13:54:50 -05:00 Brooklyn: Kings County Hospital appts no longer available
```

## What dependencies do I need?

### With `nix`

1. Run `nix-shell` in a clone of this repository

### Without `nix`

1. Install Rust (https://www.rust-lang.org/tools/install)
1. Install `pkg-config` using your favorite package manager
1. Install OpenSSL using your favorite package manager

## How do I use the application?

By default, `turbocheck` will log appointment details to your terminal, but it is
also capable of sending any number of phone numbers an SMS message for each
appointment that shows up.

### Search for appointments in all of NYC plus Long Island and some upstate areas

```
$ cargo run --release
```

### Search for appointments in Manhattan

```
$ cargo run --release -- --area manhattan
```

### Search for appointments in Queens whose site's name contains "hospital" or "Hospital"

```
$ cargo run --release -- --area queens --site-pattern="[hH]ospital"
```

### Send a text message with the appointment details using Twilio

```
$ cat <<EOF > /tmp/twilio.toml
sms_from = "a twilio phone number"
sms_to = [
  "a phone number",
  "another phone number"
]
account_sid = "<your twilio account sid>"
auth_token = "<your twilio auth token>"
EOF
$ cargo run --release -- --area=queens --site-pattern="[hH]ospital" --twilio-config=/tmp/twilio.toml
```
