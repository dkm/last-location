The main repository is [on sourcehut](https://git.sr.ht/~dkm/last-location). Github is only a mirror.

# last-location
## What it is
- A testing ground for experimenting with Rust and some web-related crates (mainly [diesel](https://diesel.rs), [Rocket](https://rocket.rs) and [Axum](https://github.com/tokio-rs/axum/)).
- Expect lot of code churn.
- A small and simple tool to keep track of the last position reported by a client (usually a smartphone, using [GPS Logger](https://gpslogger.app/)). My main use cases being flying using a paraglider or riding a MTB in nearby mountains.
- Yet another software maintained by a single hobbyist with a [bus factor = 1](https://en.wikipedia.org/wiki/Bus_factor)
- As is the case with many projects, the issue is "scoping". I'll try to keep the feature set small, no fancy things.
## What it is NOT
- Yet another live tracking solution
- Yet another activity sharing ("strava")
- Something anyone should rely on.
- A battle tested software
- A carefully designed solution
## TLDR
- The user gives an easy and stable URL to one or more relatives
- The user enables the tracking (e.g. on their phone) and go out
- The tracking device sends regular POST requests with location information (+ other data) to an instance of the `last-position` server.
- The server records the last position
- In case of emergency, any relative with the provided URL can query the server for the last known position
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
# Roadmap
- prototype a MVP: existing Android client connects to an instance, provides location
- write unit and integration tests
- some CI 
  - [![builds.sr.ht status](https://builds.sr.ht/~dkm.svg?search=last-location)](https://builds.sr.ht/~dkm?search=last-location)
- write documentation (including API)
- fuzz the API
# License
All the software is distributed under the terms of the [GPLv3 or later](https://spdx.org/licenses/GPL-3.0-or-later.html)
