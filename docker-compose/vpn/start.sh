#!/bin/sh
# IPv4フォワードを有効化
# sysctl -w net.ipv4.ip_forward=1

wg genkey | tee /etc/wireguard/privatekey > /dev/null && sed -i "s#PRIVKEY#$(cat /etc/wireguard/privatekey)#" /etc/wireguard/wg0.conf

# WireGuardを起動
wg-quick up /etc/wireguard/wg0.conf

# VPNサーバを起動
exec ./vpnserver
