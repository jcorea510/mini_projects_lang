"""
Run this once (or any time you need to re-verify the schema) before
starting the server:
    python setup_db.py
It connects without selecting a database, creates the database if
missing, then connects into it and creates the tables if missing.
Safe to re-run: everything uses IF NOT EXISTS.
"""
import mysql.connector
from mysql.connector import Error

import config
import db_schema


def main() -> None:
    try:
        admin_conn = mysql.connector.connect(
            host=config.DB_HOST,
            user=config.DB_USER,
            password=config.DB_PASSWORD,
        )
    except Error as err:
        print(f"[setup_db] could not connect to server: {err}")
        return

    if not db_schema.create_database(admin_conn, config.DB_NAME):
        print("[setup_db] failed to create database")
        admin_conn.close()
        return
    admin_conn.close()

    try:
        db_conn = mysql.connector.connect(
            host=config.DB_HOST,
            user=config.DB_USER,
            password=config.DB_PASSWORD,
            database=config.DB_NAME,
        )
    except Error as err:
        print(f"[setup_db] could not connect to database '{config.DB_NAME}': {err}")
        return

    ok = db_schema.create_node_table(db_conn) and db_schema.create_temperature_table(db_conn)
    db_conn.close()

    if ok:
        print(f"[setup_db] database '{config.DB_NAME}' and tables are ready")
    else:
        print("[setup_db] one or more tables failed to create, see errors above")


if __name__ == "__main__":
    main()
