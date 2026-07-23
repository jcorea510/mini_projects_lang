import socket
import threading

import config
import protocol

CHUNK_SIZE = 1024


def _handle_client(conn: socket.socket, addr) -> None:
    print(f"[tcp_server] connection from {addr}")
    buffer = b""
    try:
        with conn:
            while True:
                data = conn.recv(CHUNK_SIZE)
                if not data:
                    break  # client closed the connection
                buffer += data

                while b"\n" in buffer:
                    line, buffer = buffer.split(b"\n", 1)
                    line = line.strip()
                    if not line:
                        continue
                    _process_line(conn, line, addr)
    except (ConnectionResetError, BrokenPipeError) as err:
        print(f"[tcp_server] {addr} disconnected abruptly: {err}")
    finally:
        print(f"[tcp_server] connection from {addr} closed")


def _process_line(conn: socket.socket, line: bytes, addr) -> None:
    try:
        message = line.decode("utf-8")
    except UnicodeDecodeError:
        print(f"[tcp_server] {addr} sent undecodable bytes, ignoring")
        return

    print(f"[tcp_server] {addr} -> {message}")
    response = protocol.handle_message(message)
    print(f"[tcp_server] {addr} <- {response}")
    try:
        conn.sendall((response + "\n").encode("utf-8"))
    except OSError as err:
        print(f"[tcp_server] failed to reply to {addr}: {err}")


def run() -> None:
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind((config.TCP_HOST, config.TCP_PORT))
    server.listen(config.MAX_QUEUED_CONNECTIONS)
    print(f"[tcp_server] listening on {config.TCP_HOST}:{config.TCP_PORT}")

    try:
        while True:
            conn, addr = server.accept()
            threading.Thread(target=_handle_client, args=(conn, addr), daemon=True).start()
    except KeyboardInterrupt:
        print("[tcp_server] shutting down")
    finally:
        server.close()
