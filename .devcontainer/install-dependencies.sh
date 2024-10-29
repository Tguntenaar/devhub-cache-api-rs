#!/bin/bash
sudo apt update
sudo apt install -y postgresql

cargo install sqlx-cli

# Start PostgreSQL service
sudo service postgresql start

# Switch to postgres user to set up roles and database
sudo su - postgres -c "
    psql -c \"CREATE ROLE devhub WITH LOGIN PASSWORD 'caching_api';\"
    psql -c \"ALTER ROLE devhub CREATEDB;\"
    psql -c \"GRANT ALL PRIVILEGES ON DATABASE devhub_cache TO devhub;\"
"

# Export database URL for SQLx
echo "export DATABASE_URL=postgres://devhub:caching_api@127.0.0.1:5432/devhub_cache" >> ~/.bashrc
source ~/.bashrc

# Create the database using SQLx
sqlx database create

# Run migrations
sqlx migrate run
