# apiserver
# 事前にこれを実行しておく
cd ~/apiserver
# 自分の端末で実行する場合には cd ~/otedama/APIS    erver/APIServer/
```
# ベースイメージの構築
docker build --no-cache -f Dockerfile.base -t api_baseimage:latest .
# 本番イメージの構築
docker build --no-cache -f Dockerfile -t api:v1.0 .
```


# vpnserver
# 事前にこれを実行しておく
cd ~/vpnserver
# 自分の端末で実行する場合には cd ~/otedama/VPNServer
```
# ベースイメージの構築
docker build --no-cache -f Dockerfile.base -t vpn_baseimage:latest .
# 本番イメージの構築
docker build --no-cache -f Dockerfile -t vpn:v1.0 .
```

# proxyserver
# 事前にこれを実行しておく
cd ~/proxyserver
# 自分の端末で実行する場合には cd ~/otedama/VPNServer
```
# ベースイメージの構築
docker build --no-cache -f Dockerfile.base -t proxy_baseimage:latest .
# ↑1/20 廃止しました
# ↑1/27 廃止するとDockercomposeのエラーの時に毎回ビルドが入って遅いので再度作成
# 本番イメージの構築
docker build --no-cache -f Dockerfile -t proxy:v1.0 .
```

haproxy
docker build --no-cache -f Dockerfile.base -t proxy_baseimage:latest .
# ↑1/20 廃止しました
# ↑1/27 廃止するとDockercomposeのエラーの時に毎回ビルドが入って遅いので再度作成
# 本番イメージの構築
docker build --no-cache -f Dockerfile -t proxy:v1.0 .


実行中コンテナ確認コマンド（停止中も含むには -a ）
``` 
docker ps 
```
これらの停止+コンテナ削除コマンド
```
docker stop $(docker ps -a -q) && docker rm $(docker ps -aq)
```
Dockerイメージ削除コマンド
```
docker rmi $(docker images -q)
```

docker compose build --no-cache

docker compose up

Dockerfile更改要件に適用するよう変更