module SingleFieldBits
struct BitsTypeBool
    a::Bool
end

struct BitsTypeChar
    a::Char
end

struct BitsTypeUInt8
    a::UInt8
end

struct BitsTypeUInt16
    a::UInt16
end

struct BitsTypeUInt32
    a::UInt32
end

struct BitsTypeUInt64
    a::UInt64
end

struct BitsTypeUInt
    a::UInt
end

struct BitsTypeInt8
    a::Int8
end

struct BitsTypeInt16
    a::Int16
end

struct BitsTypeInt32
    a::Int32
end

struct BitsTypeInt64
    a::Int64
end

struct BitsTypeInt
    a::Int
end

struct BitsTypeFloat32
    a::Float32
end

struct BitsTypeFloat64
    a::Float64
end
end

module MultiFieldBits
struct BitsIntBool
    a::Int
    b::Bool
end

struct BitsCharFloat32Float64
    a::Char
    b::Float32
    c::Float64
end
end

module BitsWithCustom
struct BitsIntChar
    a::Int
    b::Char
end

struct BitsCharBitsIntChar
    a::Char
    b::BitsIntChar
end
end

module BitsWithTuples
struct BitsUInt8TupleInt32Int64
    a::UInt8
    b::Tuple{Int32, Int64}
end

struct BitsUInt8TupleInt32TupleInt16UInt16
    a::UInt8
    b::Tuple{Int32, Tuple{Int16, UInt16}}
end
end

module WithBitsUnion
struct SingleVariant
    a::Int8
    b::Union{Int32}
    c::Int8
end

struct DoubleVariant
    a::Int8
    b::Union{Int16, Int32}
    c::Int8
end

struct SizeAlignMismatch
    a::Int8
    b::Union{Tuple{Int16, Int16, Int16}, Int32}
    c::Int8
end

struct UnionInTuple
    a::Int8
    b::Tuple{Union{Int16, Int32}}
    c::Int8
end
end

module WithNonBitsUnion
struct NonBitsUnion
    a::Union{String,Real}
end
end

module WithStrings
struct WithString
    a::String
end
end

module WithGeneric
struct WithGenericT{T}
    a::T
end

struct WithNestedGenericT{T}
    a::WithGenericT{T}
end

struct WithSetGeneric
    a::WithGenericT{Int64}
end

struct WithValueType{N}
    a::Int64
end

withvaluetype(a::Int64) = WithValueType{2}(a)

struct WithGenericUnionAll
    a::WithGenericT
end

struct WithSetGenericTuple
    a::Tuple{WithGenericT{Int64}}
end

struct WithPropagatedLifetime
    a::WithGenericT{Module}
end

struct WithPropagatedLifetimes
    a::WithGenericT{Tuple{Int32, WithGenericT{Array{Int32, 2}}}}
end
end

"""
JlrsReflect.reflect([
    SingleFieldBits.BitsTypeBool,
    SingleFieldBits.BitsTypeChar,
    SingleFieldBits.BitsTypeUInt8,
    SingleFieldBits.BitsTypeUInt16,
    SingleFieldBits.BitsTypeUInt32,
    SingleFieldBits.BitsTypeUInt64,
    SingleFieldBits.BitsTypeUInt,
    SingleFieldBits.BitsTypeInt8,
    SingleFieldBits.BitsTypeInt16,
    SingleFieldBits.BitsTypeInt32,
    SingleFieldBits.BitsTypeInt64,
    SingleFieldBits.BitsTypeInt,
    SingleFieldBits.BitsTypeFloat32,
    SingleFieldBits.BitsTypeFloat64,
    MultiFieldBits.BitsIntBool,
    MultiFieldBits.BitsCharFloat32Float64,
    BitsWithCustom.BitsIntChar,
    BitsWithCustom.BitsCharBitsIntChar,
    BitsWithTuples.BitsUInt8TupleInt32Int64,
    BitsWithTuples.BitsUInt8TupleInt32TupleInt16UInt16,
    WithBitsUnion.SingleVariant,
    WithBitsUnion.DoubleVariant,
    WithBitsUnion.SizeAlignMismatch,
    WithBitsUnion.UnionInTuple,
    WithNonBitsUnion.NonBitsUnion,
    WithGeneric.WithGenericT,
    WithGeneric.WithNestedGenericT,
    WithGeneric.WithSetGeneric,
    WithGeneric.WithValueType,
    WithGeneric.WithGenericUnionAll,
    WithGeneric.WithSetGenericTuple,
    WithGeneric.WithPropagatedLifetime,
    WithGeneric.WithPropagatedLifetimes,
])
"""