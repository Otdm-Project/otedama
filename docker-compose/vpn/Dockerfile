FROM vpn_baseimage:latest 

# 作業ディレクトリを設定
WORKDIR /app

RUN mkdir -p /etc/wireguard && \
    chmod 700 /etc/wireguard && \
    chown root:root /etc/wireguard

COPY ./start.sh ./start.sh
COPY ./wg0.conf /etc/wireguard/wg0.conf
COPY ./privatekey /etc/wireguard/privatekey
COPY ./target/release/vpnserver ./vpnserver

RUN chown root:root ./start.sh && chmod +x ./start.sh

CMD ["./start.sh"]
