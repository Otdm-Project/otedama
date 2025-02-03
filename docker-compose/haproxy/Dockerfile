FROM haproxy:2.8

# 必要なパッケージをインストール
USER root
RUN apt-get update && apt-get install -y iproute

# start.sh をコンテナ内にコピー
COPY start.sh /usr/local/etc/haproxy/start.sh
RUN chmod +x /usr/local/etc/haproxy/start.sh
# エントリポイントを変更
ENTRYPOINT ["/usr/local/etc/haproxy/start.sh"]