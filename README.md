# `turbocheck`

## What is this?

`turbocheck` is a command line tool for monitoring COVID-19 vaccination appointments.

It uses data from https://turbovax.info to display terminal output with
appointment information (straight from TurboVax), including a Google Maps link
to the appointment site.

It will also output a log message a site has no more available appointments.

## What dependencies do I need?

### With `nix`

1. Run `nix-shell` in a clone of this repository

### Without `nix`

1. Install Rust (https://www.rust-lang.org/tools/install)
1. Install `pkg-config` using your favorite package manager
1. Install openssl using your favorite package manager

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
