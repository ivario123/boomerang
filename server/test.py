
# echo-server.py

import socket

HOST = "127.0.0.1"  # Standard loopback interface address (localhost)
PORT = 2047  # Port to listen on (non-privileged ports are > 1023)

s = socket.socket(socket.AF_INET,socket.SOCK_STREAM)
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
    s.connect((HOST, PORT))
    my_bytes = bytearray()
    my_bytes.append(3)

    s.send(my_bytes)
    data = s.recv(1024)
    print(f"{data}")
    s.send(data);

