# マルチステージbuildを実装
FROM rust:latest AS builder
WORKDIR /app
COPY ./rust /app
RUN cargo build --release

FROM almalinux:9.5

# 必要なツールをインストールし、Pythonパッケージcqlshをインストール
# ヒアドキュメントを用いて可読性の向上(詳しくはhttps://zenn.dev/sigma_tom/articles/d7fe76cd063320)
# set -exはデバック用(詳しくはhttps://www.clear-code.com/blog/2012/10/11/tips-to-make-build-script-by-shell-script.html)
RUN <<EOF
set -ex

dnf update -y

dnf install -y \
gcc \
openssl-devel \
pkgconfig \
python3-pip 

dnf clean all 

# Pythonパッケージcqlshをインストール
pip install cqlsh

EOF

# 作業ディレクトリを設定
WORKDIR /app 

# バイナリをコピー
COPY --from=builder /app/target/release/apiserver ./apiserver
# 実行コマンド
CMD ["./apiserver"]
