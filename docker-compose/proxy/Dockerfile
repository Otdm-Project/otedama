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
COPY ./Cargo.toml ./Cargo.toml

# ソースコードをコピー
COPY ./src ./src
# リリースビルド
RUN cargo build --release 

RUN cp target/release/proxyserver /usr/local/bin/proxyserver
# すでにビルドは完了しているはずなのでバイナリを実行
CMD ["/usr/local/bin/proxyserver"]
