module JlrsTests
mutable struct MutableStruct
    x
    y::UInt64
end

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

function callrust(ptr::Ptr)::Bool
    ccall(ptr, Bool, ())
end
end