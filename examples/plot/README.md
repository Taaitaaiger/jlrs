This example shows you how to call PyPlot from Rust.

In order to run this example, you must start Julia with more than one thread. If it is started with two threads, it will run two tasks sequentially. If it's started with at least three threads, these tasks will run in parallel. By default Julia uses only one thread, you can change that with the `JULIA_NUM_THREADS` environment variable:

`JULIA_NUM_THREADS=3 cargo run`

Additionally, the `PyPlot`, `PyCall` and `Plots` packages must be available in Julia. 
