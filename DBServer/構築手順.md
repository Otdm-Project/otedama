# 構築手順
## DBServerの初期設定・構築コマンド
```
sudo dnf update -y
mkdir temp
cd temp
curl -O https://download.java.net/java/ga/jdk11/openjdk-11_linux-x64_bin.tar.gz
sudo tar -zxvf openjdk-11_linux-x64_bin.tar.gz
sudo mv ./jdk-11 /opt/java-11-openjdk
echo "export JAVA_HOME=/opt/java-11-openjdk" | sudo tee -a /etc/profile.d/jdk.sh
echo "export PATH=\$JAVA_HOME/bin:\$PATH" | sudo tee -a /etc/profile.d/jdk.sh
source /etc/profile.d/jdk.sh
sudo yum install python3 -y 
```
```
sudo vi /etc/yum.repos.d/cassandra.repo
```
にて以下を記述
```
[cassandra]
name=Apache Cassandra
baseurl=https://redhat.cassandra.apache.org/41x/
gpgcheck=1
repo_gpgcheck=1
gpgkey=https://downloads.apache.org/cassandra/KEYS
```
cassandraをインストール
```
sudo yum update -y 
sudo yum install cassandra -y 
sudo reboot 
```
再起動するので待機

CQLSHをインストール
```
sudo service cassandra start
sudo dnf install python3-pip -y
pip install cqlsh
````
キースペース作成
```
cqlsh

CREATE KEYSPACE my_keyspace WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};
```
テーブル作成
```
USE my_keyspace;

CREATE TABLE vpn_info (
    id int PRIMARY KEY,
    customer_public_key text,
    server_public_key text,
    customer_vpn_address text,
    server_vpn_address text,
    proxy_subdomain text
);
```
CQLSHから抜ける
```
exit
```

外部からの接続アドレス設定
```
sudo vi /etc/cassandra/conf/cassandra.yaml
# 変更前
# listen_address: localhost
# rpc_address: localhost
seed_provider:
  - class_name: org.apache.cassandra.locator.SimpleSeedProvider
    parameters:
         - seeds: "127.0.0.1"  


# 変更後
listen_address: <DBServerのプライベートIPアドレス>
rpc_address: 0.0.0.0
broadcast_rpc_address: <DBServerのグローバルIPアドレス>
seed_provider:
  - class_name: org.apache.cassandra.locator.SimpleSeedProvider
    parameters:
    seeds: "<DBServerのプライベートIPアドレス>"  
```
Cassandraを再起動
```
sudo systemctl restart cassandra
```

使用メモリの領域を拡大
```
vi /etc/sysctl.conf
vm.max_map_count=1048575
sudo sysctl -p
```
