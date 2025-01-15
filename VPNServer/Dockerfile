FROM vpn_baseimage:latest 

# 作業ディレクトリを設定
WORKDIR /app

# 事前に構築されたバイナリをコンテナにコピー
COPY vpnserver /usr/local/bin/vpnserver

# WireGuard設定ファイルをコピー
COPY ./src/wg0.conf /etc/wireguard/wg0.conf

# エントリーポイントを直接CMDで指定
CMD sysctl -w net.ipv4.ip_forward=1 && \
    wg-quick up /etc/wireguard/wg0.conf && \
    ./vpnserver
