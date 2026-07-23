from math import log
import socket, random
from machine import ADC, Pin
import time
import network

# Thermistor NTC
RT0 = 5000.0
B = 3975.0
R = 3000.0
T0_KELVIN = 25 + 273.15

# Server connection
WIFI_PASSWORD = "****"
WIFI_SSID = "****"

HOST = "localhost"
PORT = 8080
BUF_SIZE = 1024
READING_INTERVAL_S = 60 * 5


DEVICE_ID_FILE = "device_id.txt"

REGISTER_NODE = "REGISTER_NODE,{0}\n"
REGISTER_TEMP = "REGISTER_TEMP,{0},{1:.2f}\n"

adc = ADC(Pin(28))
CONVERSION_FACTOR = 3.3 / 65535

def gen_random_name(max_len=16):
    alphabet = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ' + "0123456789"
    length = random.randint(6, max_len)
    return "".join(alphabet[random.randint(0, len(alphabet) - 1)] for _ in range(length))

def load_device_id():
    try:
        with open(DEVICE_ID_FILE, "r") as f:
            return int(f.read().strip())
    except (OSError, ValueError):
        return None

def save_device_id(device_id):
    with open(DEVICE_ID_FILE, "w") as f:
        f.write(str(device_id))

def recv_line(soc):
    """Read until a newline, since the server frames responses with \\n
    and a single recv() call isn't guaranteed to return the whole line."""
    buffer = b""
    while b"\n" not in buffer:
        chunk = soc.recv(BUF_SIZE)
        if not chunk:
            raise OSError("connection closed by server")
        buffer += chunk
    line, _, _ = buffer.partition(b"\n")
    return line.decode("utf-8")

def register_device(soc):
    name = gen_random_name()
    print(f"Registering as '{name}'")
    soc.send(REGISTER_NODE.format(name).encode())
 
    response = recv_line(soc)
    parts = response.split(",")
    if len(parts) == 2 and parts[0] == "NODE_ID":
        device_id = int(parts[1])
        save_device_id(device_id)
        print(f"Registered with id {device_id}")
        return device_id
 
    raise OSError(f"registration failed: {response}")

def read_temperature_c():
    raw_value = adc.read_u16()
    voltage = raw_value * CONVERSION_FACTOR
 
    vr = 3.3 - voltage
    rt = voltage / (vr / R)
    lnrt = log(rt / RT0)
    tx = 1 / ((lnrt / B) + (1 / T0_KELVIN))
    return tx - 273.15

def main():
    # Connect to Wi
    wlan = network.WLAN(network.STA_IF)
    wlan.active(True)
    wlan.connect(WIFI_SSID, WIFI_PASSWORD)

    # Wait for connection
    while not wlan.isconnected():
        time.sleep(1)
    print('Connected to WiFi. IP:', wlan.ifconfig()[0])

    device_id = load_device_id()
 
    print("Connecting to server")
    soc = socket.socket()
    soc.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    soc.connect((HOST, PORT))

    try:
        if device_id is None:
            device_id = register_device(soc)
        else:
            print(f"Using stored device id {device_id}")
 
        while True:
            temperature_c = read_temperature_c()
            print(f"Temperature: {temperature_c:.2f} C")
 
            soc.send(REGISTER_TEMP.format(device_id, temperature_c).encode())
            response = recv_line(soc)
            if response != "ACK":
                print(f"Server rejected reading: {response}")
 
            time.sleep(READING_INTERVAL_S)
    except OSError as err:
        print(f"Socket error: {err}")
    finally:
        soc.close()
        print("Socket closed")
        print("----------Bye----------")
 
 
if __name__ == "__main__":
    main()
 
