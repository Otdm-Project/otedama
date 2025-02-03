FROM api_baseimage:latest

# 作業ディレクトリを設定
WORKDIR /app

# バイナリをコピー
COPY ./target/release/apiserver ./apiserver

# 実行コマンド
CMD ["./apiserver"]
