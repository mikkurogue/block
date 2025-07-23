#!/bin/bash
set -e

clickhouse client -n <<-EOSQL
    CREATE DATABASE IF NOT EXISTS default;
    USE default;
    CREATE TABLE IF NOT EXISTS sales (
        sale_id UUID,
        product String,
        category String,
        quantity Int32,
        price Float64,
        sale_date Date,
        organization_id String
    ) ENGINE = MergeTree() ORDER BY sale_date;

    INSERT INTO sales (sale_id, product, category, quantity, price, sale_date, organization_id) VALUES
    (generateUUIDv4(), 'Laptop', 'Electronics', 1, 1200.00, '2025-07-20', 'org_a'),
    (generateUUIDv4(), 'Keyboard', 'Electronics', 2, 75.50, '2025-07-21', 'org_a'),
    (generateUUIDv4(), 'Mouse', 'Electronics', 3, 25.00, '2025-07-21', 'org_b'),
    (generateUUIDv4(), 'T-Shirt', 'Apparel', 5, 20.00, '2025-07-22', 'org_a'),
    (generateUUIDv4(), 'Jeans', 'Apparel', 2, 80.00, '2025-07-23', 'org_b'),
    (generateUUIDv4(), 'Monitor', 'Electronics', 1, 300.00, '2025-07-20', 'org_c'),
    (generateUUIDv4(), 'Desk Chair', 'Furniture', 1, 150.00, '2025-07-21', 'org_a'),
    (generateUUIDv4(), 'Webcam', 'Electronics', 1, 50.00, '2025-07-22', 'org_c'),
    (generateUUIDv4(), 'Headphones', 'Electronics', 2, 100.00, '2025-07-23', 'org_b'),
    (generateUUIDv4(), 'Notebook', 'Stationery', 10, 5.00, '2025-07-24', 'org_a');
EOSQL
