module MixedBagMod
struct ImmutableUnionStruct
    bits_union::Union{Int32, Int64}
    normal_union::Union{Nothing, Module}
end

mutable struct MutableUnionStruct
    bits_union::Union{Int32, Int64}
    normal_union::Union{Nothing, Module}
end

struct ModuleWrapper
    a::Module
end

mutable struct MutableNormalFields
    mutable_unions::MutableUnionStruct
    immutable_unions::ImmutableUnionStruct
    number::Number
end

struct ImmutableNormalFields
    mutable_unions::MutableUnionStruct
    immutable_unions::ImmutableUnionStruct
    number::Number
end

mutable struct MutableF32
    a::Float32
end

struct Tuples
    empty::Tuple{}
    single::Tuple{Int32}
    double::Tuple{Int32, Int64}
    abstract::Tuple{Int32, Number}
end

struct HasPointer
    u16::UInt16
    mut_f32::MutableF32
end

struct Arrays
    u8vec::Vector{UInt8}
    unionvec::Vector{Union{UInt8, UInt16}}
    wrappervec::Vector{Module}
    ptrvec::Vector{MutableF32}
    inlinedptrvec::Vector{HasPointer}
    u8array::Array{UInt8, 2}
    inlinedptrarray::Array{HasPointer, 2}
end

mutable struct MixedBag
    mutabl::MutableNormalFields
    immutabl::ImmutableNormalFields
    tuples::Tuples
    nonexistent::MutableF32
    arrays::Arrays
    MixedBag(mutabl::MutableNormalFields, immutabl::ImmutableNormalFields, tuples::Tuples, arrays::Arrays) = (x = new(); x.mutabl = mutabl; x.immutabl = immutabl; x.tuples = tuples; x.arrays = arrays; x)
end

const unionvec = Vector{Union{UInt8, UInt16}}()
push!(unionvec, UInt8(1), UInt16(2), UInt8(3))

const arrays = Arrays(
    [UInt8(1); UInt8(2); UInt8(3)],
    unionvec,
    [Main; Base; Core],
    [MutableF32(1.0); MutableF32(2.0); MutableF32(3.0)],
    [HasPointer(UInt16(1), MutableF32(2.0)); HasPointer(UInt16(3), MutableF32(4.0)); HasPointer(UInt16(5), MutableF32(6.0))],
    [UInt8(1) UInt8(2); UInt8(3) UInt8(4)],
    [HasPointer(UInt16(1), MutableF32(2.0)) HasPointer(UInt16(3), MutableF32(4.0)); HasPointer(UInt16(5), MutableF32(6.0)) HasPointer(UInt16(7), MutableF32(8.0))]
)

const mixedbag = MixedBag(
    MutableNormalFields(
        MutableUnionStruct(
            Int32(3),
            nothing
        ),
        ImmutableUnionStruct(
            Int64(7),
            Main
        ),
        Float64(3.0)
    ),
    ImmutableNormalFields(
        MutableUnionStruct(
            Int32(-3),
            Main
        ),
        ImmutableUnionStruct(
            Int64(-7),
            nothing
        ),
        Int16(-3)
    ),
    Tuples((), (Int32(1),), (Int32(2), Int64(-4)), (Int32(1), Float64(4.0))),
    arrays
)
end
