# `turbocheck`

## What is this?

`turbocheck` is a command line tool for continuously monitoring COVID-19
vaccination appointments in the New York metropolitan area.

It uses data from https://turbovax.info to display terminal output with
appointment information (straight from TurboVax), including a Google Maps link
to the appointment site.

Here's an example of some terminal output when an appointment is found:

```
------------------------------ BEGIN -------------------------------
2021-03-13 14:27:34 -05:00 Manhattan: appointments available!

Site: Harlem Hospital

Area: Manhattan
Sched: https://covid19.nychealthandhospitals.org/COVIDVaxEligibility
Map: https://is.gd/wQ2JVQ

Times: Mar 13 – 3:10PM, 3:20PM, 3:30PM + 1

Appts Remaining: 4
Last Updated: 2021-03-13 14:27:34 -05:00
------------------------------- END --------------------------------
```

It will also output a log message when a site has no more available appointments:

```
2021-03-13 13:54:50 -05:00 Brooklyn: Kings County Hospital appts no longer available
```

## What dependencies do I need?

### With `nix`

1. Install `nix` (https://nixos.org/guides/install-nix.html).
1. Run `nix-shell` in a clone of this repository.

This will take a few minutes to complete.

### Without `nix`

1. Install Rust (https://www.rust-lang.org/tools/install).
1. Install `pkg-config` using your favorite package manager.
1. Install OpenSSL using your favorite package manager.

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

### Search for appointments in Queens whose site name contains "hospital" or "Hospital"

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

## Thank You

Thank you to Huge Ma for building TurboVax.
