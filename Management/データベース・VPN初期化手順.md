構築ミス等でDBとVPN設定の初期化が必要担った場合には以下のコマンドを実行してください。
VPN設定削除

```
sudo rm /etc/wireguard/wg0.conf
```

DB初期化
```
USE customer_data;
DROP TABLE customer_data.customer_info ;
CREATE TABLE IF NOT EXISTS customer_info (
    customer_id INT PRIMARY KEY,
    client_public_key TEXT,
    server_public_key TEXT,
    subdomain TEXT,
    vpn_ip_server TEXT,
    vpn_ip_client TEXT,
    created_at TIMESTAMP
);
```
