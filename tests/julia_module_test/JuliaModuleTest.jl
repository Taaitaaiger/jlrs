include("JuliaModule.jl")

using JlrsCore.Ledger
using Test

@testset "Freestanding functions" begin
    @test isnothing(JuliaModuleTest.takes_no_args_returns_nothing())
    @inferred JuliaModuleTest.takes_no_args_returns_nothing()

    @test JuliaModuleTest.takes_no_args_returns_usize() == 0
    @inferred JuliaModuleTest.takes_no_args_returns_usize()

    @test JuliaModuleTest.takes_usize_returns_usize(UInt(3)) == 4
    @inferred JuliaModuleTest.takes_usize_returns_usize(UInt(3))

    @test JuliaModuleTest.takes_ref_usize(UInt(3)) == 4
    @test JuliaModuleTest.takes_ref_module(Main) == 0
    @test JuliaModuleTest.takes_ref_any(Main) == 0
    @test JuliaModuleTest.takes_ref_number(1) == 0

    @test JuliaModuleTest.returns_jlrs_result(false) == 3
    @inferred JuliaModuleTest.returns_jlrs_result(false)
    @test_throws JlrsCore.JlrsError JuliaModuleTest.returns_jlrs_result(true)

    @test JuliaModuleTest.returns_ref_bool()
    @inferred JuliaModuleTest.returns_ref_bool()

    @test JuliaModuleTest.returns_typed_value()
    @inferred JuliaModuleTest.returns_typed_value()
end

@testset "Arrays" begin
    @test JuliaModuleTest.takes_array(Vector{UInt32}()) == 4
    @inferred JuliaModuleTest.takes_array(Vector{UInt32}())

    @test JuliaModuleTest.takes_array(Vector{AbstractChar}()) == sizeof(UInt)
    @inferred JuliaModuleTest.takes_array(Vector{AbstractChar}())

    @test JuliaModuleTest.takes_typed_array(Vector{UInt32}()) == 4
    @inferred JuliaModuleTest.takes_typed_array(Vector{UInt32}())

    @test JuliaModuleTest.takes_typed_ranked_array(Vector{UInt32}()) == 4
    @inferred JuliaModuleTest.takes_typed_ranked_array(Vector{UInt32}())

    @test JuliaModuleTest.takes_typed_value(UInt(3)) == 3
    @inferred JuliaModuleTest.takes_typed_value(UInt(3))

    @test size(JuliaModuleTest.returns_array(Int32)) == (2,2)
    @inferred Matrix{Int32} JuliaModuleTest.returns_array(Int32)
    @inferred Matrix{Int64} JuliaModuleTest.returns_array(Int64)

    @test size(JuliaModuleTest.returns_rank0_array(Int32)) == ()
    @inferred Array{Int32,0} JuliaModuleTest.returns_rank0_array(Int32)
    @inferred Array{Int64,0} JuliaModuleTest.returns_rank0_array(Int64)

    @test size(JuliaModuleTest.returns_rank1_array(Int32)) == (2,)
    @inferred Vector{Int32} JuliaModuleTest.returns_rank1_array(Int32)
    @inferred Vector{Int64} JuliaModuleTest.returns_rank1_array(Int64)

    @test size(JuliaModuleTest.returns_rank2_array(Int32)) == (2,2)
    @inferred Matrix{Int32} JuliaModuleTest.returns_rank2_array(Int32)
    @inferred Matrix{Int64} JuliaModuleTest.returns_rank2_array(Int64)

    @test size(JuliaModuleTest.returns_rank3_array(Int32)) == (2,2,2)
    @inferred Array{Int32,3} JuliaModuleTest.returns_rank3_array(Int32)
    @inferred Array{Int64,3} JuliaModuleTest.returns_rank3_array(Int64)

    @test size(JuliaModuleTest.returns_typed_array()) == (2,2)
    @inferred Matrix{Float32} JuliaModuleTest.returns_typed_array()

    @test size(JuliaModuleTest.returns_typed_rank2_array()) == (2,2)
    @inferred Matrix{Float32} JuliaModuleTest.returns_typed_rank2_array()
end

@testset "Generic arrays" begin
    arr1 = Int[]
    arr2 = Int[1 1; 1 1]
    arr3 = Float32[1 1; 1 1]
    @test JuliaModuleTest.takes_generic_typed_ranked_arrays_ctor(arr1, arr2) == Int
    @test_throws MethodError JuliaModuleTest.takes_generic_typed_ranked_arrays_ctor(arr1, arr1)
    @test_throws MethodError JuliaModuleTest.takes_generic_typed_ranked_arrays_ctor(arr2, arr2)

    @test JuliaModuleTest.takes_generic_typed_ranked_arrays(arr1, arr2) == Int
    @test_throws MethodError JuliaModuleTest.takes_generic_typed_ranked_arrays(arr1, arr1)
    @test_throws MethodError JuliaModuleTest.takes_generic_typed_ranked_arrays(arr2, arr2)

    @test JuliaModuleTest.takes_generic_typed_arrays_ctor(arr1, arr2) == Int
    @test JuliaModuleTest.takes_generic_typed_arrays_ctor(arr1, arr1) == Int
    @test JuliaModuleTest.takes_generic_typed_arrays_ctor(arr2, arr2) == Int

    @test JuliaModuleTest.takes_generic_typed_arrays_ctor(arr1, arr2) == Int
    @test JuliaModuleTest.takes_generic_typed_arrays_ctor(arr1, arr1) == Int
    @test JuliaModuleTest.takes_generic_typed_arrays_ctor(arr2, arr2) == Int

    @test JuliaModuleTest.takes_and_returns_generic_typed_ranked_array(arr1) === arr1

    @test JuliaModuleTest.takes_restricted_generic_typed_arrays(arr1) === Int
    @test JuliaModuleTest.takes_restricted_generic_typed_arrays(arr2) === Int
    @test_throws MethodError JuliaModuleTest.takes_restricted_generic_typed_arrays(arr3)

    @test JuliaModuleTest.takes_generic_typed_arrays(arr1, arr1) == Int
    @test JuliaModuleTest.takes_generic_typed_arrays(arr1, arr2) == Int
    @test JuliaModuleTest.takes_generic_typed_arrays(arr2, arr2) == Int

    arr3 = UInt[1 1; 1 1]
    @test_throws MethodError JuliaModuleTest.takes_generic_typed_arrays(arr1, arr3)
end

@testset "OpaqueInt" begin
    opaque_int = JuliaModuleTest.OpaqueInt(Int32(-1))

    @test JlrsCore.Ledger.try_borrow_shared(opaque_int)
    @test_throws JlrsCore.BorrowError isnothing(JuliaModuleTest.increment!(opaque_int))
    @test JlrsCore.Ledger.unborrow_shared(opaque_int)

    @test JlrsCore.Ledger.try_borrow_exclusive(opaque_int)
    @test_throws JlrsCore.BorrowError JuliaModuleTest.get_cloned(opaque_int)
    @test JlrsCore.Ledger.unborrow_exclusive(opaque_int)

    @test JuliaModuleTest.get_cloned(opaque_int) == Int32(-1)
    @inferred JuliaModuleTest.get_cloned(opaque_int)

    @test isnothing(JuliaModuleTest.increment!(opaque_int))
    @inferred JuliaModuleTest.increment!(opaque_int)

    @test JuliaModuleTest.unbox_opaque(opaque_int) == Int32(1)
    @inferred JuliaModuleTest.unbox_opaque(opaque_int)
end

@testset "ForeignThing" begin
    foreign_thing = JuliaModuleTest.ForeignThing(Int32(-1))
    Base.GC.gc()
    Base.GC.gc()

    @test JuliaModuleTest.extract_inner(foreign_thing) == Int32(-1)

    @test isnothing(JuliaModuleTest.set_inner!(foreign_thing, UInt32(1)))

    @test JuliaModuleTest.extract_inner(foreign_thing) == UInt32(1)
    Base.GC.gc()
    Base.GC.gc()
    @test JuliaModuleTest.extract_inner(foreign_thing) == UInt32(1)
end

@testset "Associated function" begin
    @test JuliaModuleTest.assoc_func() == 1
    @inferred JuliaModuleTest.assoc_func()
end

@testset "Constants and globals" begin
    @test JuliaModuleTest.CONST_U8 == 0x1
    @test isconst(JuliaModuleTest, :CONST_U8)

    @test JuliaModuleTest.CONST_STATIC_U8 == 0x2
    @test isconst(JuliaModuleTest, :CONST_STATIC_U8)
end

@testset "POpaque" begin
    @test JuliaModuleTest.POpaque isa UnionAll

    p32 = JuliaModuleTest.POpaque(Float32(1.0))
    @test JuliaModuleTest.popaque_get(p32) === Float32(1.0)

    p64 = JuliaModuleTest.POpaque(Float64(1.0))
    @test JuliaModuleTest.popaque_get(p64) === Float64(1.0)

    p64 = JuliaModuleTest.POpaque64(Float64(1.0))
    @test JuliaModuleTest.popaque_get(p64) === Float64(1.0)
end

@testset "has_generic" begin
    @test JuliaModuleTest.has_generic(Float32(1.0)) == Float32(1.0)
    @test JuliaModuleTest.has_generic(Float64(1.0)) == Float64(1.0)
end

@testset "generic_from_env" begin
    arr = Float32[]
    @test JuliaModuleTest.takes_generics_from_env(arr, Float32(1.0)) === nothing

    arr = Float64[]
    @test JuliaModuleTest.takes_generics_from_env(arr, Float64(1.0)) === nothing

    arr = Int[]
    @test_throws MethodError JuliaModuleTest.takes_generics_from_env(arr, 1)
end

@testset "POpaqueTwo" begin
    @test JuliaModuleTest.POpaqueTwo isa UnionAll

    f32f32 = JuliaModuleTest.POpaqueTwo(Float32(1.0), Float32(2.0))
    @test typeof(f32f32) === JuliaModuleTest.POpaqueTwo{Float32, Float32}
    @test JuliaModuleTest.get_v1(f32f32) === Float32(1.0)
    @test JuliaModuleTest.get_v2(f32f32) === Float32(2.0)

    f32i32 = JuliaModuleTest.POpaqueTwo(Float32(1.0), Int32(2))
    @test typeof(f32i32) === JuliaModuleTest.POpaqueTwo{Float32, Int32}
    @test JuliaModuleTest.get_v1(f32i32) === Float32(1.0)
    @test JuliaModuleTest.get_v2(f32i32) === Int32(2)

    f64f64 = JuliaModuleTest.POpaqueTwo(Float64(1.0), Float64(2.0))
    @test typeof(f64f64) === JuliaModuleTest.POpaqueTwo{Float64, Float64}
    @test JuliaModuleTest.get_v1(f64f64) === Float64(1.0)
    @test JuliaModuleTest.get_v2(f64f64) === Float64(2.0)

    f64i32 = JuliaModuleTest.POpaqueTwo(Float64(1.0), Int32(2))
    @test typeof(f64i32) === JuliaModuleTest.POpaqueTwo{Float64, Int32}
    @test JuliaModuleTest.get_v1(f64i32) === Float64(1.0)
    @test JuliaModuleTest.get_v2(f64i32) === Int32(2)
end

@testset "has_two_generics" begin
    @test JuliaModuleTest.has_two_generics(Float32(1.0), Float32(2.0)) === Float32(1.0)
    @test JuliaModuleTest.has_two_generics(Float32(1.0), Int32(2)) === Float32(1.0)
    @test JuliaModuleTest.has_two_generics(Float64(1.0), Float64(2.0)) === Float64(1.0)
    @test JuliaModuleTest.has_two_generics(Float64(1.0), Int32(2)) === Float64(1.0)
end

@testset "Four generics M" begin
    v1 = JuliaModuleTest.FourGenericsM{Int32, Int32, Int32, Int32}(1,2,3,4)
    v2 = JuliaModuleTest.FourGenericsM{Int32, Int32, Int32, Int64}(1,2,3,4)
    @test JuliaModuleTest.takes_four_generics_m(v1) === v1
    @test_throws MethodError JuliaModuleTest.takes_four_generics_m(v2)

    v3 = JuliaModuleTest.FourGenericsM{Int32, Int32, Int64, Int64}(1,2,3,4)
    @test JuliaModuleTest.takes_four_generics_m_trailing1(v1) === v1
    @test JuliaModuleTest.takes_four_generics_m_trailing1(v2) === v2
    @test_throws MethodError JuliaModuleTest.takes_four_generics_m_trailing1(v3)

    v4 = JuliaModuleTest.FourGenericsM{Int32, Int64, Int32, Int64}(1,2,3,4)
    @test JuliaModuleTest.takes_four_generics_m_trailing2(v1) === v1
    @test JuliaModuleTest.takes_four_generics_m_trailing2(v2) === v2
    @test JuliaModuleTest.takes_four_generics_m_trailing2(v3) === v3
    @test_throws MethodError JuliaModuleTest.takes_four_generics_m_trailing2(v4)

    v5 = JuliaModuleTest.FourGenericsM{Int32, Int64, Int64, Int64}(1,2,3,4)
    @test JuliaModuleTest.takes_four_generics_m_middle(v1) === v1
    @test JuliaModuleTest.takes_four_generics_m_middle(v2) === v2
    @test_throws MethodError JuliaModuleTest.takes_four_generics_m_middle(v3)
    @test JuliaModuleTest.takes_four_generics_m_middle(v4) === v4
    @test_throws MethodError JuliaModuleTest.takes_four_generics_m_middle(v5)

    v6 = JuliaModuleTest.FourGenericsM{Int64, Int32, Int32, Int32}(1,2,3,4)
    @test JuliaModuleTest.takes_four_generics_m_start1(v1) === v1
    @test_throws MethodError JuliaModuleTest.takes_four_generics_m_start1(v2)
    @test JuliaModuleTest.takes_four_generics_m_start1(v6) === v6

    v7 = JuliaModuleTest.FourGenericsM{Int64, Int64, Int32, Int32}(1,2,3,4)
    @test JuliaModuleTest.takes_four_generics_m_start2(v1) === v1
    @test_throws MethodError JuliaModuleTest.takes_four_generics_m_start2(v2)
    @test JuliaModuleTest.takes_four_generics_m_start2(v6) === v6
    @test JuliaModuleTest.takes_four_generics_m_start2(v7) === v7
end