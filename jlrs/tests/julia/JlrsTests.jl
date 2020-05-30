module JlrsTests
mutable struct MutableStruct
    x
    y::UInt64
end

function inlinetuple()::Tuple{UInt32, UInt16, Int64}
    (1, 2, 3)
end
end