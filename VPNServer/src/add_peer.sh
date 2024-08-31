#!/bin/bash

WG_CONFIG_FILE="/etc/wireguard/wg0.conf"

echo "" >> $WG_CONFIG_FILE
echo "[Peer]" >> $WG_CONFIG_FILE
echo "PublicKey = $1" >> $WG_CONFIG_FILE
echo "AllowedIPs = $2/32" >> $WG_CONFIG_FILE

echo "Peer information added: PublicKey = $1, AllowedIPs = $2/32"

