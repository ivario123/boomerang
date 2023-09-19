
# echo-server.py

import socket

HOST = "127.0.0.1"  # Standard loopback interface address (localhost)
PORT = 2047  # Port to listen on (non-privileged ports are > 1023)

s = socket.socket(socket.AF_INET,socket.SOCK_STREAM)
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
    s.connect((HOST, PORT))
    my_bytes = bytearray()
    my_bytes.append(6)
    other_bytes = bytearray()
    other_bytes.append(1)
    other_bytes.append(2)
    other_bytes.append(0)
    for i in range(0,10):
        s.send(other_bytes)
        import time
        time.sleep(1)
    s.send(other_bytes)
    s.send(other_bytes)
    while 1:

        data = s.recv(1024)
        s.send(my_bytes)
        print(f"{data}")

