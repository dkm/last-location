The main repository is [on sourcehut](https://git.sr.ht/~dkm/last-location). Github is only a mirror.

# last-location
## What it is
- A testing ground for experimenting with Rust and some web-related crates (mainly [diesel](https://diesel.rs), [Rocket](https://rocket.rs) and [Axum](https://github.com/tokio-rs/axum/)).
- Expect lot of code churn.
- A small and simple tool to keep track of the last position reported by a client (usually a smartphone, using [GPS Logger](https://gpslogger.app/)). My main use cases being flying using a paraglider or riding a MTB in nearby mountains.
- Yet another software maintained by a single hobbyist with a [bus factor = 1](https://en.wikipedia.org/wiki/Bus_factor)
- As is the case with many projects, the issue is "scoping". I'll try to keep the feature set small, no fancy things.
## What it is NOT
- Yet another live tracking solution (ok, maybe a little)
- Yet another activity sharing ("strava")
- Something anyone should rely on
- A battle tested software
- A carefully designed solution
## TLDR
- No user registration/email/personnal information needed
- The user gives an easy and stable URL to one or more relatives
- The user enables the tracking (e.g. on their phone) and go out
- The tracking device sends regular POST requests with location information (+ other data) to an instance of the `last-position` server.
- The server records the last position
- In case of emergency, any relative with the provided URL can query the server for the last known position

After playing around, this has also evolved into an encrypted tracking system:
- The user gives a stable URL to one or more relatives
- The user enables the tracking (e.g. on their phone) and go out
- The tracking device sends regular POST requests with location information (+ other data) as an encrypted payload to an instance of the `last-position` server.
- The server records the encrypted data, without the possibility to decrypt it.
- In case of emergency, any relative with the provided URL can query the server for the last known position. The URL contains the decryption key.

## Why not existing tracking solution
- Most are proprietary (e.g. [SportsTrackLive](https://www.sportstracklive.com), [xcontest](https://xcontest.org))
- May require dedicated login (can't ask relative to create an account)
- May not provide stable URL
- May not fit the multi-activity use case (e.g. xcontest is for paragliding)
- May require proprietary hardware (e.g. [Syride](https://www.syride.com/))
- Usually come with many features with security not being central (nice 3D tracking, social features, log book, etc).
- Existing solution come and go
- Closest solution would be [FFVL](https://data.ffvl.fr/api/?help=tracker), but not convinced by its current implementation (for the very small part that is being made public).

# Tracked information
- latitude/longitude: for obvious reasons;
- altitude: same;
- speed/direction: is user moving or not?
- accuracy: precision as reported by the device. Having to search around a clean GPS fix ~3m is not the same thing as searching within a 50m radius;
- loc_provider: how was the location acquired (GSM, GPS, something else);
- battery: in case of low battery, the device may change its reporting strategy and even suspend any tracking to save battery;

# How does the encrypted tracking work?

The current prototype works using a symmetric key encryption algorithm
(AES-GCM). The secret key is shared ONLY between the tracking client and the
people being granted access to the tracking data. In particular, the server
doesn't have access to the secret and can't decrypt the data.

The common workflow is:
- the client app creates a secret key
- the client app encrypts location data and sends it to the server
- the client crafts an URL with the secret key in the fragment identifier (part that is NOT sent to the server) 

The server still has access to some informations:
- the server stores a server-side timestamp along with every encrypted data
- clients' IP addresses can be seen in server logs (it's not tracked/stored by `last-position`):
  - for the tracking client
  - for the web client accessing the tracking information
 
# Possible abuse
There are many ways this service can be abused. Some examples:
- DoS by flooding the service with new log requests
- DoS by flooding the service with locations
- using the encrypted data as a storage

## Flood mitigation
The service is intended to be used behind a reverse proxy (e.g. nginx, cady,
apache). Here's a possible nginx config snippet that enforce rate limit on the
API (1 request per second, with possible bursts of 5 reqs):

``` nginx
limit_req_zone $binary_remote_addr zone=last_rate_zone:10m rate=1r/s;
limit_req_status 429;

location / {
    root /path/to/last-position/static;
}

location ~ ^/(s|api)/ {
    limit_req zone=last_rate_zone burst=5 nodelay;
    proxy_pass http://127.0.0.1:3000;
    proxy_redirect    default;
    proxy_set_header  Host $host;
    proxy_set_header  X-Real-IP $remote_addr;
    proxy_set_header  X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header  X-Forwarded-Host $server_name;
    proxy_set_header  X-Forwarded-Proto $scheme;
}
```

## Service as a Storage mitigation

The rate limit (see above) and a strict 400 bytes size limit on the payload
should refrain any abuse. The limit can probably be lowered as the average
payload size is closer to 200 bytes.

# Roadmap
- ✅ prototype a MVP: existing Android client connects to an instance, provides location:
  - a modified [GPS Logger](https://git.sr.ht/~dkm/gpslogger) android app is available (the code is, haven't setup any apk build yet)
- ✅ write unit and integration tests
- ✅ some CI 
  - [![builds.sr.ht status](https://builds.sr.ht/~dkm.svg?search=last-location)](https://builds.sr.ht/~dkm?search=last-location)
- write documentation (including API)
- fuzz the API

# License
All the software is distributed under the terms of the [AGPLv3 or later](https://spdx.org/licenses/AGPL-3.0-or-later.html)
The TLDR for this license:
- you must provide the source code if you distribute a modified version of the software
- you must provide the source code if you use the software on a network server

