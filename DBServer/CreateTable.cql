USE my_keyspace;

CREATE TABLE vpn_info (
    id int PRIMARY KEY,
    customer_public_key text,
    server_public_key text,
    customer_vpn_address text,
    server_vpn_address text,
    proxy_subdomain text
);
