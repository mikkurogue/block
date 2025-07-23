#!/bin/bash
set -e

clickhouse client -n <<-EOSQL
    CREATE DATABASE IF NOT EXISTS default;
    CREATE TABLE IF NOT EXISTS default.sales (
        sale_id UUID,
        product String,
        category String,
        quantity Int32,
        price Float64,
        sale_date Date
    ) ENGINE = MergeTree() ORDER BY sale_date;

    INSERT INTO default.sales (sale_id, product, category, quantity, price, sale_date) VALUES
    (generateUUIDv4(), 'Laptop', 'Electronics', 1, 1200.00, '2025-07-20'),
    (generateUUIDv4(), 'Keyboard', 'Electronics', 2, 75.50, '2025-07-21'),
    (generateUUIDv4(), 'Mouse', 'Electronics', 3, 25.00, '2025-07-21'),
    (generateUUIDv4(), 'T-Shirt', 'Apparel', 5, 20.00, '2025-07-22'),
    (generateUUIDv4(), 'Jeans', 'Apparel', 2, 80.00, '2025-07-23');
EOSQL
