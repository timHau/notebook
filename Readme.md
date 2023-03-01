# Reactive Notebooks 

Instead of treating every cell separately, this projects finds a [topological sorting](https://en.wikipedia.org/wiki/Topological_sorting) of the cells and evaluates them in that order. If you change a value in one cell, every cell that depends on it will automatically change.

## Demo

## How does it work

When a cell is evaluated, the code is parsed and a directed acyclic graph (DAG) is build. The nodes of this graph are the cell uuids and an edge between cell `a` and cell `b` is inserted if `a` uses a variable from cell `b`. Afterwards we build an topological order of the cell dependencies, split the code of each cell up into smaller "statements" of different types (Definitions, Exec, Eval) and send via [ØMQ](https://zeromq.org/) to a python mini kernel. This kernel is responsible to eval/exec the code and sends it back. Then the response is streamed via Websockets to the client.

## Status

This Repository is only a Prototype and should not used in production. There are major parts missing like e.g. inline plotting and handing of complex data structures (np tensors etc.).