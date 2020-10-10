This example shows you how to start and use the asynchronous runtime, and how to implement a task that calls a Julia function asynchronously. 

In order to run this example, you must start Julia with more than one thread. If it is started with two threads, it will run two tasks sequentially. If it's started with at least three threads, these tasks will run in parallel. By default Julia uses only one thread, you can change that with the `JULIA_NUM_THREADS` environment variable:

`JULIA_NUM_THREADS=3 cargo run`

Because this example uses jlrs, the `JULIA_DIR` environment variable (and in the case of Windows, `CYGWIN_DIR`) must be set, and the library must be available on the library search path.
