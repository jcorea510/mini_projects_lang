"""
Tables:
- node: node_id is auto-increment and unique. node_name is NOT required to be unique, since 
multiple devices may share a human-readable name.

- temperature: register_id and register_date auto-populate on insert.
node_id is a foreign key back to node, ON DELETE CASCADE so removing
a device cleans up its readings too.
"""

import enum
from mysql.connector import Error
from mysql.connector.pooling import PooledMySQLConnection


class NodeColumn(enum.StrEnum):
    NODE_ID = "node_id"
    NODE_NAME = "node_name"


class TemperatureColumn(enum.StrEnum):
    REGISTER_ID = "register_id"
    NODE_ID = "node_id"
    REGISTER_DATE = "register_date"
    VAL = "val"


NODE_TABLE = "node"
TEMPERATURE_TABLE = "temperature"

_CREATE_NODE_TABLE = f"""
    CREATE TABLE IF NOT EXISTS {NODE_TABLE} (
        {NodeColumn.NODE_ID} INT AUTO_INCREMENT PRIMARY KEY,
        {NodeColumn.NODE_NAME} VARCHAR(32) NOT NULL
    );
"""

_CREATE_TEMPERATURE_TABLE = f"""
    CREATE TABLE IF NOT EXISTS {TEMPERATURE_TABLE} (
        {TemperatureColumn.REGISTER_ID} INT AUTO_INCREMENT PRIMARY KEY,
        {TemperatureColumn.NODE_ID} INT NOT NULL,
        {TemperatureColumn.REGISTER_DATE} DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
        {TemperatureColumn.VAL} DECIMAL(5,2) NOT NULL,
        FOREIGN KEY ({TemperatureColumn.NODE_ID})
            REFERENCES {NODE_TABLE}({NodeColumn.NODE_ID})
            ON DELETE CASCADE
    );
"""


def create_database(connection: PooledMySQLConnection, name: str) -> bool:
    """Create the database if it doesn't exist. Run once, via a connection
    that is NOT already scoped to a specific database."""
    return _execute(connection, f"CREATE DATABASE IF NOT EXISTS {name};")


def create_node_table(connection: PooledMySQLConnection) -> bool:
    return _execute(connection, _CREATE_NODE_TABLE)


def create_temperature_table(connection: PooledMySQLConnection) -> bool:
    return _execute(connection, _CREATE_TEMPERATURE_TABLE)


def _execute(connection: PooledMySQLConnection, query: str) -> bool:
    cursor = connection.cursor()
    try:
        cursor.execute(query)
        connection.commit()
        return True
    except Error as err:
        print(f"[db_schema] Error: {err}")
        return False
    finally:
        cursor.close()
