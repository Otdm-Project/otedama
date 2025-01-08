# =======================
# ビルドステージ
# =======================
FROM rust:1.80 AS builder

# 必要なパッケージをインストール
RUN apt-get update && apt-get install -y \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 作業ディレクトリを設定
WORKDIR /app

# ソースコードをコピー
COPY . .

# リリースビルド
RUN cargo build --release

# =======================
# ランタイムステージ
# =======================
FROM almalinux:9


# 作業ディレクトリを設定
WORKDIR /app

# ビルドされたバイナリをコピー
COPY --from=builder /app/target/release/proxyserver /usr/local/bin/proxyserver

# HAProxy設定ファイルをコピー
COPY src/haproxy.cfg /home/tomoya/github/otdmproject/otedama/ProxyServer/src/haproxy.cfg
### COPY haproxy.cfg /usr/local/etc/haproxy/haproxy.cfg

# ポートを公開
EXPOSE 8100 443

# ProxyServerとHAProxyを同時に起動
CMD ["sh", "-c", "haproxy -f /usr/local/etc/haproxy/haproxy.cfg & ./proxyserver"]
