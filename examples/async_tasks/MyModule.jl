module MyModule
function complexfunc(a::Int, b::Int)::Float64
    x = rand(Float64, (a, a))
    for _ in 1:b
        # This is extremely inefficient, if you look at `htop` while
        # `async-tasks` is running you'll notice that three threads are doing
        # a lot of work: two threads are working on the task, the third is the
        # garbage collector.
        x += rand(Float64, (a, a))
    end

    z::Float64 = 0.0
    for j in 1:a
        z += x[j, j]
    end

    z
end
end
