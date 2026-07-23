"""
Entry point. Run `python setup_db.py` once beforehand to make sure the
database and tables exist, then python main.py
"""

import db_pool
import tcp_server
from db_schema import NODE_TABLE, TEMPERATURE_TABLE, NodeColumn, TemperatureColumn
import db_queries
import argparse
import subprocess

def main() -> None:
    db_pool.init_pool()
    tcp_server.run()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="A mysql manager and tcp for reading sensor data from Raspberry pi pico W")
    parser.add_argument("--notcp",
                        default=False,
                        action="store_true",
                        help="Avoid running tcp connection")
    parser.add_argument("--first-run",
                        default=False,
                        action="store_true",
                        help="Runs setup_db.py for setting project")
    parser.add_argument("--print-table",
                        choices=[NODE_TABLE, TEMPERATURE_TABLE],
                        help="Print a table of choice in your terminal")
    
    args = parser.parse_args()
    if (args.first_run):
        print("Attempt to setup databse")
        result = subprocess.run(['python', 'setup_db.py'])
        if result.returncode == 0:
            print("Script finished successfully")
        else:
            print(f"Script failed with exit code {result.returncode}")

    if (args.print_table is not None):
        db_pool.init_pool()
        if (args.print_table == NODE_TABLE):
            print(f"Print table: {NODE_TABLE}")
            results = db_queries.get_table(NODE_TABLE)
            columnns = [NodeColumn.NODE_ID,
                        NodeColumn.NODE_NAME]
            db_queries.print_table(results, columnns)

        elif (args.print_table == TEMPERATURE_TABLE):
            print(f"Print table: {TEMPERATURE_TABLE}")
            results = db_queries.get_table(TEMPERATURE_TABLE)
            columnns = [TemperatureColumn.REGISTER_ID,
                        TemperatureColumn.NODE_ID,
                        TemperatureColumn.REGISTER_DATE,
                        TemperatureColumn.VAL]
            db_queries.print_table(results, columnns)
        else:
            print("Incorrect: Table doesn't exist.")

    if (not args.notcp):
        print("Attempt to run tcp server")
        main()
    else:
        print("Runned without tcp server")
