DistriRAID
===================

DistriRAID is going to be a distributed blob storage based on erasure codes. While most storage solutions achieve durability via replication, DistriRAID will distribute data blocks along with their parities on a cluster of machines.

The code is written in Rust, in fact the primary goal of the project is me getting more familiar with the Rust programming language. Due to the lack of mature libraries, and the educational nature of the project I'll do lot of recreational programming and experiments in this project. These all doesn't necessarily mean the project never reaches production quality, and in the long term I'll aim to create a robust software.

Work in Progress
-------------
> **Note:** The code is in a very early stages, probably only compiles on my own machine.

The core module of the current codebase is **Jerasurs**. That's a Rust wrapper around the [Jerasure](http://lab.jerasure.org/jerasure/jerasure) C library. In the near future I'm planning to release that as a separate Rust crate in its own repository.

The other one is **diskio**. That's a module that uses a bare file or disk on a similar manner how dynamic memory is managed (malloc/free).
