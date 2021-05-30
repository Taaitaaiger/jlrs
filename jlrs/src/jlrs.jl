module Jlrs
using Base.StackTraces

struct TracedException
    exc
    stacktrace::StackTrace
end

const wakerust = Ref{Ptr{Cvoid}}(C_NULL)
const droparray = Ref{Ptr{Cvoid}}(C_NULL)

function runasync(func::Function, wakeptr::Ptr{Cvoid}, args...; kwargs...)::Any
    try
        func(args...; kwargs...)
    finally
        ccall(wakerust[], Cvoid, (Ptr{Cvoid},), wakeptr)
    end
end

function asynccall(func::Function, wakeptr::Ptr{Cvoid}, args...; kwargs...)::Task
    @assert wakerust[] != C_NULL "wakerust is null"
    Base.Threads.@spawn runasync(func, wakeptr, args...; kwargs...)
end

function tracingcall(func::Function)::Function
    function wrapper(args...; kwargs...)
        try
            func(args...; kwargs...)
        catch
            for s in stacktrace(catch_backtrace(), true)
                println(stderr, s)
            end

            rethrow()
        end
    end

    wrapper
end

function attachstacktrace(func::Function)::Function
    function wrapper(args...; kwargs...)
        try
            func(args...; kwargs...)
        catch exc
            st::StackTrace = stacktrace(catch_backtrace(), true)
            rethrow(TracedException(exc, st))
        end
    end

    wrapper
end

function clean(a::Array)
    @assert droparray[] != C_NULL "droparray is null"
    ccall(droparray[], Cvoid, (Array,), a)
end
end
