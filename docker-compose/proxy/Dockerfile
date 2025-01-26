FROM debian:bookworm-slim

# 必要なツールとライブラリをインストール
RUN apt update \ 
    && apt install -y \
        libssl-dev \
        pkg-config \
        build-essential \
    && apt clean
# 作業ディレクトリを設定
WORKDIR /app

# Cargo.tomlをコピー 
# ソースコードをコピー
COPY ./src ./src

FROM rust:1.80
WORKDIR /app

# Cargo.tomlをコピー
COPY Cargo.toml ./

# ソースコードをコピー
COPY ./src ./src
# リリースビルド
RUN cargo build --release 

# すでにビルドは完了しているはずなのでバイナリを実行
CMD ["./target/release/proxyserver"]
