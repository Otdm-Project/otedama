FROM haproxy:2.8

# 必要なファイルを追加
COPY haproxy.cfg /usr/local/etc/haproxy/haproxy.cfg
COPY --from=proxy_baseimage:latest /usr/local/bin/proxyserver /usr/local/bin/proxyserver

# 両方のプロセスを起動するコマンドを指定
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/bin/sh", "-c", "proxyserver & haproxy -f /usr/local/etc/haproxy/haproxy.cfg"]

