"""
Read/write helpers for the node and temperature tables.

Both inserts use parameterized queries (%s placeholders).

mysql-connector handles the quoting/escaping correctly when you pass params separately.
"""
from typing import Optional
from mysql.connector import Error
from tabulate import tabulate
import pandas as pd

from db_pool import get_connection
from db_schema import NODE_TABLE, TEMPERATURE_TABLE, NodeColumn, TemperatureColumn


def insert_node(node_name: str) -> Optional[int]:
    """Register a new device. Returns the new node_id, or None on failure.
    Device names are capped at 32 chars (matches the column) but are not
    required to be unique — several devices can share a name."""
    node_name = node_name.strip()[:32]
    if not node_name:
        return None

    query = f"""
        INSERT INTO {NODE_TABLE} ({NodeColumn.NODE_NAME})
        VALUES (%s);
    """
    with get_connection() as conn:
        cursor = conn.cursor()
        try:
            cursor.execute(query, (node_name,))
            conn.commit()
            return cursor.lastrowid
        except Error as err:
            print(f"[db_queries] insert_node error: {err}")
            return None
        finally:
            cursor.close()


def insert_temperature(node_id: int, temp: float) -> bool:
    """Log a reading for an existing node_id. Returns False (and logs)
    if node_id doesn't exist, since the FOREIGN KEY constraint will
    reject it rather than silently accepting bad data."""
    query = f"""
        INSERT INTO {TEMPERATURE_TABLE} ({TemperatureColumn.NODE_ID}, {TemperatureColumn.VAL})
        VALUES (%s, %s);
    """
    with get_connection() as conn:
        cursor = conn.cursor()
        try:
            cursor.execute(query, (node_id, round(temp, 2)))
            conn.commit()
            return True
        except Error as err:
            print(f"[db_queries] insert_temperature error: {err}")
            return False
        finally:
            cursor.close()


def node_exists(node_id: int) -> bool:
    query = f"SELECT 1 FROM {NODE_TABLE} WHERE {NodeColumn.NODE_ID} = %s;"
    with get_connection() as conn:
        cursor = conn.cursor()
        try:
            cursor.execute(query, (node_id,))
            return cursor.fetchone() is not None
        except Error as err:
            print(f"[db_queries] node_exists error: {err}")
            return False
        finally:
            cursor.close()

def get_table(table: str):
    query = f"""
        SELECT * 
        FROM {table};
    """
    result = None
    with get_connection() as conn:
        cursor = conn.cursor()
        try:
            cursor.execute(query)
            result = cursor.fetchall()
            return result
        except Error as err:
            print(f"[db_queries] get_{table}_table error: {err}")
            return result
        finally:
            cursor.close()
    return result

def print_table(results, columns):
    if (results is None):
        print("There is no data")
    else:
        from_db = []
        for result in results:
            result = list(result)
            from_db.append(result)
        df = pd.DataFrame(from_db, columns=columns)
        print(tabulate(df, headers='keys', tablefmt='psql'))

