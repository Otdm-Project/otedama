FROM proxy_baseimage:latest

# 作業ディレクトリを設定
WORKDIR /app

# バイナリをコピー
COPY ./target/release/proxyserver ./proxyserver

# 必要なパッケージのインストール
RUN dnf install -y iproute && dnf clean all

# エントリポイントスクリプトを追加
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# エントリポイントを設定
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

# 実行コマンド
CMD ["./proxyserver"]

# HAProxyは自動起動なのでここにその関連は記述しない