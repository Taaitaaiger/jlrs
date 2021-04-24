task = @async begin 
    condition = Base.AsyncCondition()
    output::Ref{UInt32} = C_NULL
    joinhandle = ccall((:multithreaded, "libccall_with_threads"), Ptr{Cvoid}, (Ref{UInt32}, Ptr{Cvoid}), output, condition.handle)
    wait(condition)
    ccall((:drop_handle, "libccall_with_threads"), Cvoid, (Ptr{Cvoid},), joinhandle)

    output[]
end

task2 = @async begin
    while !istaskdone(task)
        println("Still running")
        sleep(0.1)
    end

    @assert fetch(task) == 127 "Wrong result"
end

wait(task)
wait(task2)