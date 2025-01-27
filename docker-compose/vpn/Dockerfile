FROM vpn_baseimage:latest 

# 作業ディレクトリを設定
WORKDIR /app

# WireGuard設定ファイルをコピー
#COPY /home/ubuntu/otedama/docker-compose/vpn/wg0.conf /etc/wireguard/wg0.conf

# 起動スクリプトをコピー
COPY ./start.sh ./start.sh

# スクリプトに実行権限を付与
RUN chown root:root ./start.sh && chmod +x ./start.sh

# エントリーポイントをスクリプトに設定
CMD ["./start.sh"]
