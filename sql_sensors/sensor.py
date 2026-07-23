import enum
import mysql.connector
from mysql.connector import Error
from mysql.connector.pooling import CMySQLConnection
from option import Result, Option, Ok, Err
import socket
import threading

class Node(enum.Enum):
    node = 0
    node_id = 1
    node_name = 2

class Temperature(enum.Enum):
    temperature = 0
    register_id = 1
    node_id = 2
    register_date = 3
    val = 4

class Command(enum.StrEnum):
    REGISTER_NODE = enum.auto()
    REGISTER_TEMP = enum.auto()

class InsertNodeCommand(enum.Enum):
    NODE_NAME = 1

class InsertTempCommand(enum.Enum):
    NODE_ID = 1
    TEMP = 2

def create_server_connection(host: str, user: str, password: str) -> Result[CMySQLConnection, Error]:
    try:
        cursor = mysql.connector.connect(
            host = host,
            user = user,
            password = password
        )
        print("Connection to MySQL server successfully")
        return Ok(cursor)
    except Error as err:
        print(f"Error: {err}")
        return Err(err)

def create_database_connection(host: str, user: str, password: str, db: str) -> Result[CMySQLConnection, Error]:
    try:
        cursor = mysql.connector.connect(
            host = host,
            user = user,
            password = password,
            database = db
        )
        print(f"Connection to {db} database successfully")
        return Ok(cursor)
    except Error as err:
        print(f"Error: '{err}'")
        return Err(err)

def execute_query(connection: CMySQLConnection, query: str) -> bool:
    if (connection.is_connected()):
        cursor = connection.cursor()
        try:
            cursor.execute(query)
            connection.commit()
            print("Query successful")
            return True
        except Error as err:
            print(f"Error: '{err}'")
    return False

def create_database(connection: CMySQLConnection, name: str) -> bool:
    create_database_query = f"CREATE DATABASE {name}"
    return execute_query(connection, create_database_query)

def create_node_table(connection: CMySQLConnection) -> bool:
    create_node_table_query = f"""
        CREATE TABLE node (
            {Node.node_id.name} INT AUTO_INCREMENT PRIMARY KEY,
            {Node.node_name.name} VARCHAR(8) NOT NULL,
        );
    """
    return execute_query(connection, create_node_table_query)

def create_register_table(connection: CMySQLConnection) -> bool:
    create_register_table_query = f"""
        CREATE TABLE {Temperature.temperature.name} (
            {Temperature.register_id.name} INT AUTO_INCREMENT PRIMARY KEY,
            {Temperature.node_id.name} INT NOT NULL,
            {Temperature.register_date.name} DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
            {Temperature.val.name} FLOAT(2),
            FOREIGN KEY({Temperature.node_id.name}) REFERENCES {Node.node.name}({Node.node_id.name}) ON DELETE CASCADE
        );
    """
    return execute_query(connection, create_register_table_query)

def insert_node_entry(connection: CMySQLConnection, node_name: str):
    if (len(node_name) <= 8):
        insert_node_entry_query = f"""
            INSERT INTO {Node.node.name}({Node.node_name.name})
            VALUES ({node_name});
        """
        execute_query(connection, insert_node_entry_query)

def insert_temperature_entry(connection: CMySQLConnection, node_id: int, temp: float):
    insert_temperature_entry_query = f"""
        INSERT INTO {Temperature.temperature.name}({Temperature.node_id.name}, {Temperature.val.name})
        VALUES ({node_id}, {temp:.2f});
    """
    execute_query(connection, insert_temperature_entry_query)

def execute_client_request(connection: CMySQLConnection, request: str):
    command = request.split(",")
    if (len(command) <= 1):
        return
    if (command[0] == Command.REGISTER_NODE and len(command) == 2):
        node_name = command[InsertNodeCommand.NODE_NAME.value]
        insert_node_entry(connection, node_name)
        ## I also need to send the id from database to device so that
        ## device knows its id which will be used in insert_temperature_entry
        ## assuming that multiple device may have same name but different id
    elif (command[0] == Command.REGISTER_TEMP and len(command) == 3):
        try:
            node_id = int(command[InsertTempCommand.NODE_ID.value])
            temp = float(command[InsertTempCommand.TEMP.value])
            insert_temperature_entry(connection, node_id, temp)
        except ValueError:
            pass
    

def tcp_server(connection: CMySQLConnection):
    host = "192.168.1.101"
    port = 8080

    server = socket.create_server((host, port), reuse_port=False)
    while True:
        conn, addr = server.accept()
        print(f"Connection from {addr}")

        with conn:
            CHUNK_SIZE = 1024
            while True:
                data = conn.recv(CHUNK_SIZE)
                if not data:
                    break
                print(f"Received data: {data}")
                execute_client_request(connection, b"{data}".decode("utf-8"))

def handle_client(connection: CMySQLConnection, client_socket):
    CHUNK_SIZE = 1024
    request = client_socket.recv(CHUNK_SIZE)
    print(f"Received data: {request}")
    execute_client_request(connection, b"{request}".decode("utf-8"))
    client_socket.send("ACK".encode())
    client_socket.close()

def tcp_server_multiclient(connection: CMySQLConnection):
    host = "192.168.1.101"
    port = 8080
    num_connections = 5

    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.bind((host, port))
    server.listen(num_connections)

    print(f"Listening on {host}:{port}")
    while True:
        conn, addr = server.accept()
        print(f"Connection from {addr}")
        client_handler = threading.Thread(target=handle_client, args=(connection, conn,))
        client_handler.start()

connection = create_server_connection("localhost", "akai", "****************")

if (connection.is_ok):
    print("My connection: ", connection.unwrap())
