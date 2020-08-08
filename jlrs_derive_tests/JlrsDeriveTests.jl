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
    BitsWithTuples.BitsUInt8TupleInt32TupleInt16UInt16
])
"""
