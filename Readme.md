# Reactive Notebooks 

Instead of treating every cell separately, this projects finds a [topological sorting](https://en.wikipedia.org/wiki/Topological_sorting) of the cells and evaluates them in that order. If you change a value in one cell, every cell that depends on it will automatically change.

## Demo


https://user-images.githubusercontent.com/12029285/222263333-17708f92-5c23-4f65-8fd7-4292096b5a85.mp4


## How does it work

When a cell is evaluated, the code is parsed and a directed acyclic graph (DAG) is build. The nodes of this graph are the cell uuids and an edge between cell `a` and cell `b` is inserted if `a` uses a variable from cell `b`. Afterwards we build an topological order of the cell dependencies, split the code of each cell up into smaller "statements" of different types (Definitions, Exec, Eval) and send via [ØMQ](https://zeromq.org/) to a python mini kernel. This kernel is responsible to eval/exec the code and sends it back. Then the response is streamed via Websockets to the client.

## Getting started

First you need to install the Python dependencies [dill](https://pypi.org/project/dill/) via `pip install dill` and [pyzmq](https://zeromq.org/languages/python/) via `pip install pyzmq`. Then you can run the project via cargo
 
```
cargo run --release
```

Afterwards cd into the client directory, install all the dependencies there and start the dev client

```
cd client
npm i
npm run dev
```

## Status

This Repository is only a Prototype and should not used in production. There are major parts missing like e.g. inline plotting and handing of complex data structures (np tensors etc.).

## Inspiration

Heavily inspired by [Observable](https://observablehq.com/) and [Pluto](https://github.com/fonsp/Pluto.jl)