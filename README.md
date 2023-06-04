# VPNr-try

This is a first try to make a VPN service with rust. This project is just to study how socket works with rust.

## Protocol

|          | size    | msg id  | flags  | addr    | port    | msg           |
|----------|---------|---------|--------|---------|---------|---------------|
| size     | 1 bytes | 2 bytes | 1 byte | 4 bytes | 2 bytes | max 500 bytes |
| bytes    | 0       | 00      | 0      | 0000    | 00      | 0...498...0   |

**Protocol description**

- **size**: Message size, starting in the flag and ending in the last message character.
- **msg id**: Message identification for the client.
- **flags**: 8 bits flags to pass extra information.
- **addr**: 4 bytes defining ipv4 destination address.
- **port**: 2 bytes defining the destination port.
- **msg**: The message with not more than 512 byte length.

## ToDo

- Start client
    - Start lissening in port to get all the traffic connections
- Start communication between server and client
    - Start server
    - Connect client to server
    - Send all received trafic in the client to the server, using the protocol
- Handle iptables to redirect all trafic to the client app.

