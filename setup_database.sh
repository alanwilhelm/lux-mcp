#!/bin/bash

# Database setup script for lux-mcp

DB_NAME="lux_mcp"
DB_USER="lux_user"
DB_PASS="lux_password"

echo "Setting up PostgreSQL database for lux-mcp..."

# Check if PostgreSQL is installed
if ! command -v psql &> /dev/null; then
    echo "PostgreSQL is not installed. Please install it first."
    echo "On macOS: brew install postgresql"
    echo "On Ubuntu: sudo apt-get install postgresql postgresql-contrib"
    exit 1
fi

# Create database and user
echo "Creating database and user..."

# For macOS/Homebrew, use current user. For Linux, use postgres user
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS with Homebrew PostgreSQL
    psql -d postgres << EOF
-- Create user if not exists
DO
\$do\$
BEGIN
   IF NOT EXISTS (
      SELECT FROM pg_catalog.pg_roles
      WHERE  rolname = '$DB_USER') THEN
      CREATE ROLE $DB_USER LOGIN PASSWORD '$DB_PASS';
   END IF;
END
\$do\$;

-- Create database if not exists
SELECT 'CREATE DATABASE $DB_NAME OWNER $DB_USER'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '$DB_NAME')\gexec

-- Grant all privileges
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;
EOF
else
    # Linux with standard PostgreSQL
    sudo -u postgres psql << EOF
-- Create user if not exists
DO
\$do\$
BEGIN
   IF NOT EXISTS (
      SELECT FROM pg_catalog.pg_roles
      WHERE  rolname = '$DB_USER') THEN
      CREATE ROLE $DB_USER LOGIN PASSWORD '$DB_PASS';
   END IF;
END
\$do\$;

-- Create database if not exists
SELECT 'CREATE DATABASE $DB_NAME OWNER $DB_USER'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '$DB_NAME')\gexec

-- Grant all privileges
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;
EOF
fi

echo "Database setup complete!"
echo ""
echo "Database connection string:"
echo "DATABASE_URL=postgres://$DB_USER:$DB_PASS@localhost/$DB_NAME"
echo ""
echo "Add this to your .env file or export it:"
echo "export DATABASE_URL=postgres://$DB_USER:$DB_PASS@localhost/$DB_NAME"