#!/bin/sh

# ルート設定
# ip route add 100.64.0.0/10 via 10.0.10.20

# HAProxy の起動
exec haproxy -f /usr/local/etc/haproxy/haproxy.cfg
