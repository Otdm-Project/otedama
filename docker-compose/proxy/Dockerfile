FROM proxy_baseimage:latest

# 作業ディレクトリを設定
WORKDIR /app

# バイナリをコピー
COPY ./target/release/proxyserver ./proxyserver

# プロキシ用ルート設定
RUN ip route add 100.64.0.0/10 via 100.64.0.1 dev eth0

# 実行コマンド
CMD ["./proxyserver"]

# HAProxyは自動起動なのでここにその関連は記述しない