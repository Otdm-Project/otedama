#!/bin/sh
# IPv4フォワードを有効化
# sysctl -w net.ipv4.ip_forward=1

# WireGuardを起動
wg-quick up /etc/wireguard/wg0.conf

# VPNサーバを起動
exec /usr/local/bin/vpnserver
