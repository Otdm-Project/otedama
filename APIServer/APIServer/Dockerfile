FROM almalinux:9

# 必要なパッケージのインストール
RUN dnf install -y \
    libstdc++ \
    # 例として cqlsh（Cassandra クライアント）などが必要であればここで追加インストール
    # dnf install -y python3-cassandra
    && dnf clean all

# 作業ディレクトリ
WORKDIR /usr/local/bin

# ビルドされたバイナリをコピー
COPY --from=builder /app/target/release/apiserver .

# ポートの公開
EXPOSE 8080

# ログ出力レベルの設定など
ENV RUST_LOG=info

# セキュリティ向上のためユーザー作成
RUN useradd -m apiserver
USER apiserver

# コンテナ起動時に実行されるコマンド
CMD ["./APIServer"]
