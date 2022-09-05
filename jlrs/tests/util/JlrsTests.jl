module JlrsTests
mutable struct MutableStruct
    x
    y::UInt64
end

struct ParameterStruct{T}
    a::T
end

struct ValueTypeStruct{T}
end

valuedispatch(::ValueTypeStruct{1})::Int = 3
valuedispatch(::ValueTypeStruct{2})::Float64 = 3.0

function inlinetuple()::Tuple{UInt32, UInt16, Int64}
    (1, 2, 3)
end

function symbol()::Symbol
    :x
end

function base()::Module
    Base
end

function datatype()::DataType
    Bool
end

function callrust(ptr::Ptr{Cvoid})::Bool
    ccall(ptr, Bool, ())
end

function callrustwitharr(ptr::Ptr{Cvoid}, arr::Array{Float64, 1})::Bool
    ccall(ptr, Bool, (Array{Float64, 1},), arr)
end

function callrustwithasynccond(func::Ptr{Cvoid}, destroyhandle::Ptr{Cvoid})::UInt32
    condition = Base.AsyncCondition()
    output::Ref{UInt32} = C_NULL
    joinhandle = ccall(func, Ptr{Cvoid}, (Ref{UInt32}, Ptr{Cvoid}), output, condition.handle)
    wait(condition)
    ccall(destroyhandle, Cvoid, (Ptr{Cvoid},), joinhandle)

    output[]
end

function vecofmodules()::Vector{Module}
    [Base; Core; Main]
end

function anothervecofmodules()::Vector{Module}
    [Base; Core; Main]
end

function funcwithkw(a::Int; b::Int=1)
    a + b
end

function funcwithkw(a::Int, rest...; b::Int=1)
    a + sum(rest) + b
end

function funcwithkw(; b::Int=1, c::Int=2)
    b + c
end

function funcwithabstractkw(a::Float32; b::Real=1.0f0)
    a + b
end

function throws_exception(args...; kwargs...)
    throw("This should happen")
end

struct ModuleOrNothing
    a::Union{Module, Nothing}
end

has_nothing = ModuleOrNothing(nothing)
has_module = ModuleOrNothing(Base)

struct NoUnionsBits
    a::Int16
    b::Int32
end

struct NoUnionsBitsPtr
    a::Int16
    b::DataType
end

struct BitsBitsUnion
    a::Int16
    b::Union{Int16, Int32}
end

struct PtrBitsUnion
    a::DataType
    b::Union{Int16, Int32}
end

struct PtrNonBitsUnion
    a::DataType
    b::Union{Int16, Int32, DataType}
end

struct HasArray
    a::Array{Float64, 2}
end

struct UaArray
    a::Array
end

struct WithEmpty{T}
    a::UInt
    b::T
    c::UInt
    WithEmpty{T}() where T = new()
end

struct WithAbstract
    a::Real
end

struct HasConstructors
    a::DataType
    b::Union{Int16, Int32}
    HasConstructors(i::Int16) = new(Int16, i)
    HasConstructors(i::Int32) = new(Int32, i)
    HasConstructors(i::Bool) = new(Bool, i ? one(Int32) : zero(Int32))
end

HasConstructors() = HasConstructors(false)
end
