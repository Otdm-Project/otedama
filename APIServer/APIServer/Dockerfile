FROM api_baseimage:latest

# 作業ディレクトリを設定
WORKDIR /usr/local/bin

# ベースイメージからビルド済みバイナリをコピー
COPY --from=api_baseimage:latest /usr/local/bin/apiserver /usr/local/bin/apiserver

# 実行コマンド
CMD ["./apiserver"]
