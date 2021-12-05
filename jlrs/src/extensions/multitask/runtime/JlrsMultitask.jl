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
    Base.Threads.@spawn begin
        try
            func(args...; kwargs...)
        finally
            if wakeptr != C_NULL
                ccall(wakerust[], Cvoid, (Ptr{Cvoid},), wakeptr)
            end
        end
    end
end

function asynccall(func::Function, args...; kwargs...)::Task
    @nospecialize func args kwargs
    Base.Threads.@spawn func(args...; kwargs...)
end

const inchannel = Channel{LocalTask}(1)
const outchannel = Channel{Task}(1)
Base.Threads.@spawn begin
    while true
        local_task = take!(inchannel)
        task = @async begin
            try
                local_task.func(local_task.args...; local_task.kwargs...)
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
    put!(inchannel, task)
    take!(outchannel)
end

function scheduleasynclocal(func::Function, args...; kwargs...)::Task
    @nospecialize func args kwargs
    task = LocalTask(func, args, kwargs, C_NULL)
    put!(inchannel, task)
    take!(outchannel)
end

function scheduleasync(func::Function, wakeptr::Ptr{Cvoid}, args...; kwargs...)::Task
    @nospecialize func wakeptr args kwargs
    @async begin
        try
            func(args...; kwargs...)
        finally
            if wakeptr != C_NULL
                ccall(wakerust[], Cvoid, (Ptr{Cvoid},), wakeptr)
            end
        end
    end
end

function scheduleasync(func::Function, args...; kwargs...)::Task
    @nospecialize func args kwargs
    @async func(args...; kwargs...)
end
end
