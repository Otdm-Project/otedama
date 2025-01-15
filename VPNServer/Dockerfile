FROM vpn_baseimage:latest 

WORKDIR /usr/local/bin

# Copy built binary from the builder stage
COPY --from=vpn_baseimage:latest /usr/local/bin/vpnserver /usr/local/bin/vpnserver

# Set the default command to execute the binary
CMD ["./vpnserver"]
