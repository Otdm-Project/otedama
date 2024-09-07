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

DESCRIBE keyspaces;
