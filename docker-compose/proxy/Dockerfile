FROM proxy_baseimage:latest

# 作業ディレクトリを設定
WORKDIR /app

# バイナリをコピー
COPY ./target/release/proxyserver ./proxyserver

# 必要なパッケージのインストール
RUN dnf install -y iproute && dnf clean all

# 実行コマンド
CMD ["./proxyserver"]

# HAProxyは自動起動なのでここにその関連は記述しない