# imvr spec

The main idea is that there is one main server where a bunch of clients 
can connect and remotly control one or many windows.

## Binary

The binary is split into two modes. The server and a clients.

The server is complicated thing. The client is the simple one in that just 
passes information to the running server.

All programs start by parsing information passed from the command line.

There are flags that can do things and then there are any number of file 
or dir paths.

If there is a non-zero amount of valid paths passed on the command line 
then it assumes the role of client. The binary will only every become a 
server if it is told to.

As a client:
 - A client will start by checking if there is a running server. If there is 
 not it will start one by (idk bro). It will then send its commands to the 
 server. If it was told to (it does by default) it will stick on the cli 
 allowing you to control the window from there. All windows can be 
 controlled through commands within them.

As a server:
 - The server will daemonize and then wait for incomming connections over tcp.
 as connections come in it will respond to them.


## Architeture

### Client
```
+--------------+--------------------+
|              |                    |
|  Parse Args  | Pass Request Along |
|              |  (tcp stream)      |
+--------------+--------------------+----------------
               |
               | Listen on Cli ...............
               |  (tokio read + write from stdin)
               +-------------------------------------
```

### Server
```
+-------+-------+------------------------+
| Parse | Wait  | Render Thread          |
|  Args |  ...  |  (parse window events) |
|       |       |  (run user callbacks)  |
+-------+-+-----+------------------------+
          | Listen on Tcp ...............
          |  (tokio read)
          +-------------------------------------
          | Listen on Stdin ...............
          |  (tokio read)
          +----+---------------------------------
               | Listen on Window ...............
               |  ()
               +-------------------------------------
```


## Event Structure
```
+---------------------+---------------------------------+
| Main Render Thread  |          Tokio Runtime          |
+---------------------+---------------------------------+
|                     | Socket        Stdin       Argv  |
|                     |   |             |           |   |
|    WinitEvent       |   +------------Msg----------+   |
|        |            |                 |               |
|   WindowEvent <-- WindowMsg <---------+               |
|        |            |                 |               |
|      Event          |             TerminalMsg         |
|        |            |                 |               |
|     Request ---> (output) -------> Response           |
|                     |                                 |
+---------------------+---------------------------------+
```

```rust
pub type WindowEvent = winit::event::Event<WindowMsg>;

```

