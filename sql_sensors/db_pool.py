"""
Connection handling.

Coded with help of Claude: My original code used single mysql.connector
acroos threaths

The original code shared ONE mysql.connector connection across every
client thread. mysql-connector-python connections are not thread-safe for
concurrent use, so two devices talking to the server at the same time
could corrupt each other's queries. A connection pool hands each thread
its own connection for the duration of a request, then returns it.

https://havus.medium.com/understanding-connection-pooling-for-mysql-28be6c9e2dc0
"""
from contextlib import contextmanager
from mysql.connector import Error
from mysql.connector.pooling import MySQLConnectionPool, PooledMySQLConnection

import config

_pool: MySQLConnectionPool | None = None


def init_pool() -> None:
    """Call once at startup, after the database + tables already exist."""
    global _pool
    _pool = MySQLConnectionPool(
        pool_name = config.DB_POOL_NAME,
        pool_size = config.DB_POOL_SIZE,
        host = config.DB_HOST,
        user = config.DB_USER,
        password = config.DB_PASSWORD,
        database = config.DB_NAME,
    )
    print(f"[db_pool] Pool '{config.DB_POOL_NAME}' ready ({config.DB_POOL_SIZE} connections)")


@contextmanager
def get_connection() -> PooledMySQLConnection:
    """Borrow a connection from the pool and always return it, even on error.

    Usage:
        with get_connection() as conn:
            cursor = conn.cursor()
            ...
    """
    if _pool is None:
        raise RuntimeError("Connection pool not initialized — call init_pool() first")

    conn = _pool.get_connection()
    try:
        yield conn
    finally:
        conn.close()  # returns the connection to the pool, doesn't actually close it


def get_admin_connection():
    """A one-off single connection with no database selected.
    Only used for initial setup (CREATE DATABASE)."""
    import mysql.connector
    return mysql.connector.connect(
        host=config.DB_HOST,
        user=config.DB_USER,
        password=config.DB_PASSWORD,
    )
