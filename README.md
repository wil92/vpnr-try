# VPNr-try

This is a first try to make a VPN service with rust. This project is just to study how socket works with rust.

## Protocol

|           | size    | msg id  | flags  | addr    | port    | msg           |
|-----------|---------|---------|--------|---------|---------|---------------|
| size      | 1 bytes | 2 bytes | 1 byte | 4 bytes | 2 bytes | max 500 bytes |
| bytes     | 0       | 00      | 0      | 0000    | 00      | 0...498...0   |
| start pos | 0       | 1       | 3      | 4       | 8       | 10            |

**Protocol description**

- **size**: Message size, starting in the flag and ending in the last message character.
- **msg id**: Message identification for the client.
- **flags**: 8 bits flags to pass extra information.
- **addr**: 4 bytes defining ipv4 destination address.
- **port**: 2 bytes defining the destination port.
- **msg**: The message with not more than 512 byte length.

## iptables commands

```
# redirect all traffic to the application
sudo iptables -t nat -A OUTPUT -j REDIRECT -p tcp --to-port 4333 -m owner ! --uid-owner root

# redirect google traffic to the application
sudo iptables -t nat -A OUTPUT -p tcp -d google.com --dport 80 -j REDIRECT --to-port 4333 -m owner ! --uid-owner root 
sudo iptables -t nat -A OUTPUT -p tcp -d google.com --dport 443 -j REDIRECT --to-port 4333 -m owner ! --uid-owner root 

# list iptables rules created
sudo iptables -t nat -L --line-number

# remove a particular iptable rule
sudo iptables -t nat -D OUTPUT <line-num>

# clear iptables
sudo iptables -t nat -F
```
# ToDo

- [x] Start client
    - [x] Start lissening in port to get all the traffic connections
- [x] Start communication between server and client
    - [x] Start server
    - [x] Connect client to server
    - [x] Send all received trafic in the client to the server, using the protocol
    - [x] Get addr and port from redirection information
    - [x] Connect server to the destination addr/port and send response to the client
- [ ] Handle iptables to redirect all trafic to the client app.
- [ ] Handle server disconnection from the client (try to connect againg to the server).
- [ ] Massive refactorization :P

