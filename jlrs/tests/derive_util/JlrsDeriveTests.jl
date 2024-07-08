# using JlrsCore.Reflect

struct BitsIntChar
    a::Int
    b::Char
end

struct BitsCharBitsIntChar
    a::Char
    b::BitsIntChar
end

struct BitsUInt8TupleInt32Int64
    a::UInt8
    b::Tuple{Int32, Int64}
end

struct BitsUInt8TupleInt32TupleInt16UInt16
    a::UInt8
    b::Tuple{Int32, Tuple{Int16, UInt16}}
end

struct BitsIntBool
    a::Int
    b::Bool
end

struct BitsCharFloat32Float64
    a::Char
    b::Float32
    c::Float64
end

mutable struct MutF32
    a::Float32
end

mutable struct MutNested
    a::MutF32
end

struct Immut
    a::MutF32
end

mutable struct HasImmut
    a::Immut
end

struct DoubleImmut
    a::Immut
end

mutable struct HasGeneric{T}
    a::T
end

struct HasGenericImmut{T}
    a::HasGeneric{T}
end

mutable struct DoubleHasGeneric{T}
    a::HasGeneric{T}
end

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

struct SingleVariant
    a::Union{Int32}
end

struct DoubleVariant
    a::Union{Int16, Int32}
end

struct DoubleUVariant
    a::Union{UInt16, UInt32}
end

struct SizeAlignMismatch
    a::Union{Tuple{Int16, Int16, Int16}, Int32}
end

struct UnionInTuple
    a::Tuple{Union{Int16, Int32}}
end

struct WithArray
    a::Array{Float32,2}
end

struct WithCodeInstance
    a::Core.CodeInstance
end

struct WithDataType
    a::DataType
end

struct WithExpr
    a::Expr
end

struct WithString
    a::String
end

struct WithMethod
    a::Method
end

struct WithMethodInstance
    a::Core.MethodInstance
end

struct WithMethodTable
    a::Core.MethodTable
end

struct WithModule
    a::Module
end

struct WithSimpleVector
    a::Core.SimpleVector
end

struct WithSymbol
    a::Symbol
end

struct WithTask
    a::Task
end

struct WithTypeName
    a::Core.TypeName
end

struct WithTypeVar
    a::TypeVar
end

struct WithTypeMapEntry
    a::Core.TypeMapEntry
end

struct WithTypeMapLevel
    a::Core.TypeMapLevel
end

struct WithUnion
    a::Union
end

struct WithUnionAll
    a::UnionAll
end

struct WithGenericT{T}
    a::T
end

struct WithGenericTU{T,U}
    a::T
    b::U
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

struct NonBitsUnion
    a::Union{String,Real}
end

struct Empty end

struct TypedEmpty{T} end

struct HasElidedParam{T, U}
    a::T
end

@enum StandardEnum se_a=1 se_b=2 se_c=3

#reflect([
#    BitsCharBitsIntChar,
#    BitsCharFloat32Float64,
#    BitsIntBool,
#    BitsIntChar,
#    BitsTypeBool,
#    BitsTypeChar,
#    BitsTypeFloat32,
#    BitsTypeFloat64,
#    BitsTypeInt,
#    BitsTypeInt16,
#    BitsTypeInt32,
#    BitsTypeInt64,
#    BitsTypeInt8,
#    BitsTypeUInt,
#    BitsTypeUInt16,
#    BitsTypeUInt32,
#    BitsTypeUInt64,
#    BitsTypeUInt8,
#    BitsUInt8TupleInt32Int64,
#    BitsUInt8TupleInt32TupleInt16UInt16,
#    DoubleHasGeneric,
#    DoubleImmut,
#    DoubleVariant,
#    DoubleUVariant,
#    Empty,
#    HasGeneric,
#    HasGenericImmut,
#    HasImmut,
#    Immut,
#    MutF32,
#    MutNested,
#    NonBitsUnion,
#    SingleVariant,
#    SizeAlignMismatch,
#    TypedEmpty,
#    UnionInTuple,
#    WithArray,
#    WithCodeInstance,
#    WithDataType,
#    WithExpr,
#    WithGenericT,
#    WithGenericUnionAll,
#    WithMethod,
#    WithMethodInstance,
#    WithMethodTable,
#    WithModule,
#    WithNestedGenericT,
#    WithPropagatedLifetime,
#    WithPropagatedLifetimes,
#    WithSetGeneric,
#    WithSetGenericTuple,
#    WithSimpleVector,
#    WithString,
#    WithSymbol,
#    WithTask,
#    WithTypeMapEntry,
#    WithTypeMapLevel,
#    WithTypeName,
#    WithTypeVar,
#    WithUnion,
#    WithUnionAll,
#    WithValueType,
#])
