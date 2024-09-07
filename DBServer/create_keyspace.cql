CREATE KEYSPACE IF NOT EXISTS customer_data WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};
SELECT * FROM system_schema.keyspaces;
