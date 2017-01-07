# Echelon0 - first echelon of logs analysis

[![Build Status](https://travis-ci.org/Ostrovski/echelon0.svg?branch=master)](https://travis-ci.org/Ostrovski/echelon0)

Like [Logstash](https://www.elastic.co/products/logstash) but written in Rust. With an intention to become a drop-in replacement.

Under construction. Coming soon...

## Tests

    cargo tests

## Generate docs

    cargo rustdoc --  --no-defaults              \
                      --passes strip-hidden      \
                      --passes collapse-docs     \
                      --passes unindent-comments \
                      --passes strip-priv-imports
