# マルチステージbuildを実装
FROM rust:latest AS builder
WORKDIR /app
COPY ./rust /app
RUN cargo build --release

FROM almalinux:9.5

# 必要なツールをインストール
# ヒアドキュメントを用いて可読性の向上(詳しくはhttps://zenn.dev/sigma_tom/articles/d7fe76cd063320)
# set -exはデバック用(詳しくはhttps://www.clear-code.com/blog/2012/10/11/tips-to-make-build-script-by-shell-script.html)
RUN <<EOF
set -ex

dnf install -y \
gcc \
openssl-devel \
iproute \
iptables \
pkgconfig \
python3-pip 

dnf -y install dnf-plugins-core && dnf config-manager --add-repo https://download.docker.com/linux/rhel/docker-ce.repo
dnf install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

dnf clean all 

# Pythonパッケージcqlshをインストール
pip install cqlsh

EOF

# 作業ディレクトリを設定
WORKDIR /app

# バイナリをコピー
COPY --from=builder /app/target/release/proxyserver ./proxyserver

# 必要なパッケージのインストール
RUN dnf install -y iproute && dnf clean all

# 実行コマンド
CMD ["./proxyserver"]

# HAProxyは自動起動なのでここにその関連は記述しない