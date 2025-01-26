FROM vpn_baseimage:latest 

# 作業ディレクトリを設定
WORKDIR /home/ubuntu/otedama/docker-compose/vpn/app

# WireGuard設定ファイルをコピー
#COPY /home/ubuntu/otedama/docker-compose/vpn/wg0.conf /etc/wireguard/wg0.conf

# 起動スクリプトをコピー
#COPY /home/ubuntu/otedama/docker-compose/vpn/start.sh ./start.sh

# スクリプトに実行権限を付与
# RUN chmod +x ./start.sh

# エントリーポイントをスクリプトに設定
CMD ["./start.sh"]
