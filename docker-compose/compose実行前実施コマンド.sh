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
docker build --no-cache -f Dockerfile.base -t vpn_proxyimage:latest .
# 本番イメージの構築
docker build --no-cache -f Dockerfile -t proxy:v1.0 .
```