"""
Wire protocol for messages coming from the Pico W devices.

Plain-text, comma-separated, one command per line:
    REGISTER_NODE,<name>
    REGISTER_TEMP,<node_id>,<temperature>

Responses sent back to the device:
    NODE_ID,<id>      -- reply to REGISTER_NODE, so the device can store
                          its assigned id and use it for later readings
    ACK               -- reply to a successful REGISTER_TEMP
    ERR,<reason>       -- anything that failed
"""
import enum

import db_queries


class Command(enum.StrEnum):
    REGISTER_NODE = "REGISTER_NODE"
    REGISTER_TEMP = "REGISTER_TEMP"


def handle_message(raw: str) -> str:
    """Parse one line of client input and return the response string to send back."""
    parts = [p.strip() for p in raw.strip().split(",")]
    if not parts or not parts[0]:
        return "ERR,empty command"

    try:
        command = Command(parts[0])
    except ValueError:
        return f"ERR,unknown command '{parts[0]}'"

    if command is Command.REGISTER_NODE:
        return _handle_register_node(parts)
    if command is Command.REGISTER_TEMP:
        return _handle_register_temp(parts)

    return f"ERR,unhandled command '{parts[0]}'"  # unreachable, kept for safety


def _handle_register_node(parts: list[str]) -> str:
    if len(parts) != 2:
        return "ERR,REGISTER_NODE expects 1 argument: name"

    node_name = parts[1]
    node_id = db_queries.insert_node(node_name)
    if node_id is None:
        return "ERR,could not register node"
    return f"NODE_ID,{node_id}"


def _handle_register_temp(parts: list[str]) -> str:
    if len(parts) != 3:
        return "ERR,REGISTER_TEMP expects 2 arguments: node_id, temperature"

    try:
        node_id = int(parts[1])
        temp = float(parts[2])
    except ValueError:
        return "ERR,node_id must be int and temperature must be a number"

    if not db_queries.insert_temperature(node_id, temp):
        return "ERR,could not save reading (is node_id registered?)"
    return "ACK"
