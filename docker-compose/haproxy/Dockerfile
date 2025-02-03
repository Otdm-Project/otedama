FROM haproxy:2.8

# 必要なパッケージをインストール
USER root
RUN apt-get update && apt-get install -y iproute

# start.sh をコンテナ内にコピー
COPY start.sh /usr/local/etc/haproxy/start.sh
RUN chmod +x /usr/local/etc/haproxy/start.sh
RUN ip route add 100.64.0.0/10 via 10.0.10.20
# エントリポイントを変更
ENTRYPOINT ["/usr/local/etc/haproxy/start.sh"]