module JlrsMultitask

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


scheduleasynclocal(func::Function, wakeptr::Ptr{Cvoid}, args...; kwargs...)::Task = interactivecall(func, wakeptr, args...; kwargs...)
scheduleasynclocal(func::Function, args...; kwargs...)::Task = interactivecall(func, args...; kwargs...)
scheduleasync(func::Function, wakeptr::Ptr{Cvoid}, args...; kwargs...)::Task = asynccall(func, wakeptr, args...; kwargs...)
scheduleasync(func::Function, args...; kwargs...)::Task = asynccall(func, args...; kwargs...)

end

