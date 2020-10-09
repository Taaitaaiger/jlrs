module Jlrs
using Base.StackTraces

struct TracedException
    exc
    stacktrace::StackTrace
end

const wakerust = Ref{Ptr{Cvoid}}(C_NULL)

function runasync(func::Function, wakeptr::Ptr{Cvoid}, args...)::Any
    try
        func(args...)
    finally
        ccall(wakerust[], Cvoid, (Ptr{Cvoid},), wakeptr)
    end
end

function asynccall(func::Function, wakeptr::Ptr{Cvoid}, args...)::Task
    @assert wakerust[] != C_NULL "wakerust is null"
    Base.Threads.@spawn runasync(func, wakeptr, args...)
end

function tracingcall(func::Function)::Function
    function wrapper(args...)
        try
            func(args...)
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
    function wrapper(args...)
        try
            func(args...)
        catch exc
            st::StackTrace = stacktrace(catch_backtrace(), true)
            rethrow(TracedException(exc, st))
        end
    end

    wrapper
end
end
