# roli

A low level wrapper for the Rolimons.com REST API.

# Notes

* This crate is a low level wrapper due to the fact that allowed requests to the API are limited. To maintain flexibiliy while also using the API endpoints responsibly, the user is expected to maintain their own caching.

* This library is currently *unstable* until the release of v1.0.0, APIs may be subject to change across minor releases without warning.

# API Coverage Checklist
- [x] Items API
- [x] Deals API
- [ ] Trade Ad API

(more to be added, issues for API coverage requests welcome)

# Contributing
There is still a lot of work to be done! Pull requests are welcome!

Note that all public non-util functions are required to make exactly one API call.

# License
MIT License
