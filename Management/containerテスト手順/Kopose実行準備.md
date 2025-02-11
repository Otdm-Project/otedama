Komposeをインストールする
```
curl -L https://github.com/kubernetes/kompose/releases/download/v1.35.0/kompose-linux-amd64 -o kompose
chmod +x kompose
sudo mv ./kompose /usr/local/bin/kompose
```

Komposeによるyamlファイルへの変換を行う
```
kompose -f compose.yaml -f compose.yaml convert
```

[ソース](https://kompose.io/installation/)


これを見ながら必要設定の変換を行う。
```
https://kompose.io/conversion/
```

本システムで変換する場所は以下のとおりである。






-----------------------------------------------------------------------


# APIコンテナ
docker tag api:v1.0 maritosnet/api:v1.0
docker push maritosnet/api:v1.0

# VPNコンテナ
docker tag vpn:v1.0 maritosnet/vpn:v1.0
docker push maritosnet/vpn:v1.0

# Proxyコンテナ
docker tag proxy:v1.0 maritosnet/proxy:v1.0
docker push maritosnet/proxy:v1.0
