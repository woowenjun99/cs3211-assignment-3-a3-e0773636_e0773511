## Outline of TCP server

The TCP server is built on top of the `tokio` asynchronous runtime, and it is assumed that it is capable of efficiently handling IO-bound and CPU-bound tasks. It is designed for TCP communication, ensuring reliable, ordered and fault-tolerant delivery of data bytes. 

Incoming tasks is processed based on its type, which are either CPU-intensive or IO-intensive tasks.

## Implementation details

The server listens for incoming TCP connections using a `TcpListener`, and upon accepting a new connection, it immediately spawns a new `tokio` task to handle the connection, effectively allowing the server to handle multiple connections concurrently without blocking.

A `BufReader` is also wrapped around the TCP stream to efficiently read incoming data line by line.

Depending on the content of the data, which is parsed in `get_task_value`, different tasks are executed.
For CPU-intensive tasks, `task::spawn_blocking` is used, which offloads the task to a thread where blocking is acceptable, while for IO-intensive tasks, an asynchronous task execution method is used.

## Processing Requests from Multiple Clients

The server can handle multiple clients concurrently because of the`tokio::spawn` function, which is used to create a new asynchronous task for each incoming connection. This design leverages Tokio's multi-threaded executor to run multiple tasks across available CPU cores. Since each connection is handled in its own task, the server efficiently manages multiple connections without one blocking another, enabling high concurrency and scalability.

## Task parallelism

The server is able to run tasks in parallel, inherently due to the its design and the use of the `Tokio` runtime. Since each connection has its own dedicated thread, the server achieves concurrency by allowing multiple connections to be handled at once. For operations that are CPU-bound and inherently more resource-intensive, the server utilizes Tokio's `spawn_blocking` function. This function offloads the CPU-intensive tasks to a dedicated thread pool that is optimized for blocking operations. The separation of CPU-bound tasks into a distinct thread pool ensures that these operations can proceed in parallel with other asynchronous tasks.

