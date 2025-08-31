module JlrsStableTests
mutable struct WithAtomic
    @atomic a::UInt32
end

mutable struct WithLargeAtomic
    @atomic a::Tuple{UInt64, UInt64, UInt64, UInt64}
end

mutable struct WithOddlySizedAtomic
    @atomic a::Tuple{UInt32, UInt16}
end

mutable struct WithAtomicUnion
    @atomic a::Union{UInt32, UInt16}
end

mutable struct WithConst
    const i::Int
    j::Int
end
end
