"""
Central configuration for the sensor server.

All values can be overridden with environment variables so credentials
"""
import os

# Database
DB_HOST = os.environ.get("DB_HOST", "localhost")
DB_USER = os.environ.get("DB_USER", "***")
DB_PASSWORD = os.environ.get("DB_PASSWORD", "***")
DB_NAME = os.environ.get("DB_NAME", "sensors")

# Connection pool: reused across client threads instead of sharing one
# connection object (a single mysql-connector connection is NOT thread-safe).
DB_POOL_NAME = "sensor_pool"
DB_POOL_SIZE = int(os.environ.get("DB_POOL_SIZE", "5"))

# TCP server
TCP_HOST = os.environ.get("TCP_HOST", "0.0.0.0")
TCP_PORT = int(os.environ.get("TCP_PORT", "8080"))
MAX_QUEUED_CONNECTIONS = int(os.environ.get("MAX_QUEUED_CONNECTIONS", "5"))
