# TCP Proxy

https://notes.alexxi.dev/Weekend-Projects/Building-a-secure-TCP-Forwarder

This secure TCP Proxy listens to traffic on one address and forwards it to another address and was designed to encrypt tcp traffic between a client and a server for secured communication. 

### Features

Proxy Mode: In this mode, the application forwards incoming network traffic from the specified listen address to the forward address. It acts as a typical network proxy server.

Client Mode: When in client mode, the application initiates a connection to the server (specified by the forward address), encrypts the traffic, and forwards the incoming traffic.

Server Mode: In server mode, the application listens for incoming connections on the specified address, decrypts the traffic, and forwards it to a target server.

AES-GCM Encryption: In Client Mode or Server Mode, the application expects an AES-GCM key to be set as an environment variable (AES_GCM_KEY) for secure encryption and decryption of network traffic.

### Usage

You can run the application with various command-line options to configure its behavior:


`-l` or `--listen`: Specify the address to listen on.

`-f` or `--forward`: Specify the address to forward traffic to.

`--client`: Run the application in client mode.

`--server`: Run the application in server mode.

`--proxy`: Run the application in proxy mode.





