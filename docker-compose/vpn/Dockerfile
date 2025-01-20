FROM vpn_baseimage:latest 

# 作業ディレクトリを設定
WORKDIR /app

# 事前に構築されたバイナリをコンテナにコピー
COPY --from=vpn_baseimage:latest /usr/local/bin/vpnserver /usr/local/bin/vpnserver

# WireGuard設定ファイルをコピー
COPY ./src/wg0.conf /etc/wireguard/wg0.conf

# 起動スクリプトをコピー
COPY ./start.sh ./start.sh

# スクリプトに実行権限を付与
RUN chmod +x ./start.sh

# エントリーポイントをスクリプトに設定
CMD ["./start.sh"]
