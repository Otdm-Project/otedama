FROM vpn_baseimage:latest 

# 作業ディレクトリを設定
WORKDIR /app

# WireGuard設定ファイルをコピー
#COPY /home/ubuntu/otedama/docker-compose/vpn/wg0.conf /etc/wireguard/wg0.conf
# /etc/wireguardディレクトリを作成し、適切な権限を設定
RUN mkdir -p /etc/wireguard && \
    chmod 700 /etc/wireguard && \
    chown root:root /etc/wireguard

# 起動スクリプトをコピー
COPY ./start.sh ./start.sh


# WireGuardの設定ファイルを移動
COPY ./wg0.conf /etc/wireguard/wg0.conf
# WireGuardの秘密鍵を生成
RUN wg genkey | tee /etc/wireguard/privatekey | sed -i "s#PRIVKEY#$(cat /etc/wireguard/privatekey | tr -d '\n')#" /etc/wireguard/wg0.conf

# VPNServerのバイナリをコピー(start.shで使用)
COPY ./target/release/vpnserver ./vpnserver


# スクリプトに実行権限を付与
RUN chown root:root ./start.sh && chmod +x ./start.sh

# エントリーポイントをスクリプトに設定
CMD ["./start.sh"]