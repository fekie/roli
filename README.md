# roli

A low level api wrapper for Rolimons.com.

# Notes

* This crate is a low level wrapper due to the fact that allowed requests to the website are limited. To maintain flexibiliy while also using the endpoints responsibly, the user is expected to maintain their own caching.

* This library is currently *unstable* while under version v0.0.x, APIs may be subject to change across minor releases without warning.

# API Coverage Checklist
- [x] Items API
- [x] Deals API
- [ ] Trade Ad API

# Contributing
Pull requests are welcome!

Note that all public non-util functions are required to make exactly one https request to Rolimons.com.

# License
MIT License
