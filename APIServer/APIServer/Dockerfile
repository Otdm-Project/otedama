<<<<<<< HEAD
FROM almalinux:9

# 必要なパッケージのインストール
RUN dnf install -y \
    libstdc++ \
    # 例として cqlsh（Cassandra クライアント）などが必要であればここで追加インストール
    # dnf install -y python3-cassandra
    && dnf clean all

# 作業ディレクトリ
=======
FROM api_baseimage:latest

# 作業ディレクトリを設定
>>>>>>> otdm-container
WORKDIR /usr/local/bin

# ベースイメージからビルド済みバイナリをコピー
COPY --from=api_baseimage:latest /usr/local/bin/apiserver /usr/local/bin/apiserver

<<<<<<< HEAD
# ログ出力レベルの設定など
ENV RUST_LOG=info

# コンテナ起動時に実行されるコマンド
CMD ["./APIServer"]
=======
# 実行コマンド
CMD ["./apiserver"]
>>>>>>> otdm-container
