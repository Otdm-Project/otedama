構築ミス等で設定の初期化が必要になった場合には以下のコマンドを実行してください。
VPN設定削除

```
sudo rm /etc/wireguard/wg0.conf
sudo cp /etc/wireguard/wg0.conf.bk /etc/wireguard/wg0.conf

```
Proxy設定初期化
```
sudo rm /etc/haproxy/haproxy.cfg
sudo cp /etc/haproxy/haproxy.cfg.bk /etc/haproxy/haproxy.cfg
```
```
CREATE KEYSPACE IF NOT EXISTS customer_data WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};
SELECT * FROM system_schema.keyspaces;
USE customer_data;
CREATE TABLE IF NOT EXISTS customer_info (
    customer_id INT PRIMARY KEY,        -- 顧客ID（一意）
    client_public_key TEXT,             -- 顧客環境の公開鍵
    server_public_key TEXT,             -- サーバ群の公開鍵
    subdomain TEXT,                     -- 生成したサブドメイン
    vpn_ip_server TEXT,                 -- VPN用のIPアドレス（サーバ群用）
    vpn_ip_client TEXT,                 -- VPN用のIPアドレス（顧客環境用）
    created_at TIMESTAMP                -- データ作成日時
);
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
SELECT * FROM customer_data.customer_info ;
```
何も顧客データがない表が帰ってくればOK!