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

dnf install -y epel-release dnf-plugins-core
dnf config-manager --set-enabled crb
dnf upgrade -y

dnf install -y \
wireguard-tools \
gcc \
clang \
openssl-devel \
libstdc++-static \
make \
python3-pip \
iproute \
iptables \ 

dnf clean all 

# Pythonパッケージcqlshをインストール
pip install --no-cache-dir cqlsh 

EOF

# 作業ディレクトリを設定
WORKDIR /app

# WireGuard設定ファイルをコピー
#COPY /home/ubuntu/otedama/docker-compose/vpn/wg0.conf /etc/wireguard/wg0.conf
# /etc/wireguardディレクトリを作成し、適切な権限を設定
RUN <<EOF
set -ex

mkdir -p /etc/wireguard 
chmod 700 /etc/wireguard 
chown root:root /etc/wireguard

EOF

# 起動スクリプトをコピー
COPY ./start.sh ./start.sh


# WireGuardの設定ファイルを移動
COPY ./wg0.conf /etc/wireguard/wg0.conf
# WireGuardの秘密鍵を生成
RUN wg genkey | tee ./privatekey | sed -i "s#PRIVKEY#$(cat ./privatekey | tr -d '\n')#" ./wg0.conf

# VPNServerのバイナリをコピー(start.shで使用)
COPY --from=builder /app/target/release/vpnserver ./vpnserver


# スクリプトに実行権限を付与
RUN chown root:root ./start.sh && chmod +x ./start.sh

# エントリーポイントをスクリプトに設定
CMD ["./start.sh"]