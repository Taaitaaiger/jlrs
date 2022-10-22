module JlrsMultitask
struct LocalTask
    func::Function
    args::Tuple
    kwargs
    wakeptr::Ptr{Cvoid}
end

const wakerust = Ref{Ptr{Cvoid}}(C_NULL)

function asynccall(func::Function, wakeptr::Ptr{Cvoid}, args...; kwargs...)::Task
    @nospecialize func wakeptr args kwargs
    Base.Threads.@spawn :default begin
        try
            Base.invokelatest(func, args...; kwargs...)
        finally
            if wakeptr != C_NULL
                ccall(wakerust[], Cvoid, (Ptr{Cvoid},), wakeptr)
            end
        end
    end
end

function asynccall(func::Function, args...; kwargs...)::Task
    @nospecialize func args kwargs
    Base.Threads.@spawn :default Base.invokelatest(func, args...; kwargs...)
end

function interactivecall(func::Function, wakeptr::Ptr{Cvoid}, args...; kwargs...)::Task
    @nospecialize func wakeptr args kwargs
    Base.Threads.@spawn :interactive begin
        try
            Base.invokelatest(func, args...; kwargs...)
        finally
            if wakeptr != C_NULL
                ccall(wakerust[], Cvoid, (Ptr{Cvoid},), wakeptr)
            end
        end
    end
end

function interactivecall(func::Function, args...; kwargs...)::Task
    @nospecialize func args kwargs
    Base.Threads.@spawn :interactive Base.invokelatest(func, args...; kwargs...)
end

const inchannel = Channel{LocalTask}(1)
const outchannel = Channel{Task}(1)
Base.Threads.@spawn :default begin
    while true
        local_task = take!(inchannel)

        task = @async begin
            try
                Base.invokelatest(local_task.func, local_task.args...; local_task.kwargs...)
            finally
                if local_task.wakeptr != C_NULL
                    ccall(wakerust[], Cvoid, (Ptr{Cvoid},), local_task.wakeptr)
                end
            end
        end
        put!(outchannel, task)
    end
end

function scheduleasynclocal(func::Function, wakeptr::Ptr{Cvoid}, args...; kwargs...)::Task
    @nospecialize func wakeptr args kwargs
    task = LocalTask(func, args, kwargs, wakeptr)
    print("Put")
    put!(inchannel, task)
    print("Task")
    take!(outchannel)
end

function scheduleasynclocal(func::Function, args...; kwargs...)::Task
    @nospecialize func args kwargs
    task = LocalTask(func, args, kwargs, C_NULL)
    print("Put")
    put!(inchannel, task)
    print("Task")
    take!(outchannel)
end

function scheduleasync(func::Function, wakeptr::Ptr{Cvoid}, args...; kwargs...)::Task
    @nospecialize func wakeptr args kwargs
    @async begin
        try
            Base.invokelatest(func, args...; kwargs...)
        finally
            if wakeptr != C_NULL
                ccall(wakerust[], Cvoid, (Ptr{Cvoid},), wakeptr)
            end
        end
    end
end

function scheduleasync(func::Function, args...; kwargs...)::Task
    @nospecialize func args kwargs
    @async Base.invokelatest(func, args...; kwargs...)
end

# function borrowthread(func::Ptr{Cvoid})::Task
#     Base.Threads.@spawn :default ccall(func, Cvoid, ())
# end
end

