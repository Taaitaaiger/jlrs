module Jlrs
using Base.StackTraces

struct TracedException
    exc
    stacktrace::StackTrace
end

const wakerust = Ref{Ptr{Cvoid}}(C_NULL)
const droparray = Ref{Ptr{Cvoid}}(C_NULL)

const condition = Base.AsyncCondition()

function awaitcondition()
    wait(condition)
end

function runasync(func::Function, wakeptr::Ptr{Cvoid}, args...; kwargs...)::Any
    @nospecialize func, wakeptr, args, kwargs
    try
        func(args...; kwargs...)
    finally
        ccall(wakerust[], Cvoid, (Ptr{Cvoid},), wakeptr)
    end
end

function asynccall(func::Function, wakeptr::Ptr{Cvoid}, args...; kwargs...)::Task
    @nospecialize func, wakeptr, args, kwargs
    @assert wakerust[] != C_NULL "wakerust is null"
    Base.Threads.@spawn runasync(func, wakeptr, args...; kwargs...)
end

function localasynccall(func::Function, wakeptr::Ptr{Cvoid}, args...; kwargs...)::Task
    @nospecialize func, wakeptr, args, kwargs
    @assert wakerust[] != C_NULL "wakerust is null"
    @async runasync(func, wakeptr, args...; kwargs...)
end

function tracingcall(@nospecialize(func::Function))::Function
    function (args...; kwargs...)
        @nospecialize args, kwargs

        try
            func(args...; kwargs...)
        catch
            for (exc, bt) in Base.catch_stack()
                showerror(stderr, exc, bt)
                println(stderr)
            end

            rethrow()
        end
    end
end

function attachstacktrace(@nospecialize(func::Function))::Function
    function (args...; kwargs...)
        @nospecialize args, kwargs

        try
            func(args...; kwargs...)
        catch exc
            st::StackTrace = stacktrace(catch_backtrace(), true)
            rethrow(TracedException(exc, st))
        end
    end
end

function finalizearray(@nospecialize(a::Array))
    @assert droparray[] != C_NULL "droparray is null"
    ccall(droparray[], Cvoid, (Array,), a)
end

function valuestring(@nospecialize(value))::String
    io = IOBuffer()
    show(io, "text/plain", value)
    String(take!(io))
end

function errorstring(@nospecialize(value))::String
    io = IOBuffer()
    showerror(io, value)
    String(take!(io))
end
end
