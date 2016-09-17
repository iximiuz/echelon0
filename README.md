# Echelon0 - first echelon of logs analysis

[![Build Status](https://travis-ci.org/Ostrovski/echelon0.svg?branch=master)](https://travis-ci.org/Ostrovski/echelon0)

Under construction. Coming soon...

## Examples

    # Parse nginx access_log (log_format combined)
    RUST_LOG=monstrio=info cargo run '/([\d\.]+) - (.+) \[(.+)\] "(.+) ([^?]+)\??(.*) HTTP.+" (\d{3}) (\d+) "(.+)" "(.+)"/ remote_addr,remote_user,time_local:dt[%d/%b/%Y:%H:%M:%S %z],method,path,query,status:uint,body_bytes_sent:uint,referrer,user_agent' ./../../../*.log


## Tests

    cargo tests
