FROM proxy_baseimage:latest

WORKDIR /app
COPY ./target/release/proxyserver ./proxyserver
RUN dnf install -y iproute && dnf clean all
CMD ["./proxyserver"]

# HAProxyは自動起動なのでここにその関連は記述しない