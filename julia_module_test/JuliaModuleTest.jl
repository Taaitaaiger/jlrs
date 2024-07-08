include("JuliaModule.jl")

using JlrsCore.Ledger
using Test

@testset "Freestanding functions" begin
    @test isnothing(JuliaModuleTest.freestanding_func_trivial())
    @inferred JuliaModuleTest.freestanding_func_trivial()

    @test JuliaModuleTest.freestanding_func_noargs() == 0
    @inferred JuliaModuleTest.freestanding_func_noargs()

    @test JuliaModuleTest.freestanding_func_bitsarg(UInt(3)) == 4
    @inferred JuliaModuleTest.freestanding_func_bitsarg(UInt(3))

    @test JuliaModuleTest.freestanding_func_ref_bitsarg(UInt(3)) == 4
    @test JuliaModuleTest.freestanding_func_ref_mutarg(Main) == 0
    @test JuliaModuleTest.freestanding_func_ref_any(Main) == 0
    @test JuliaModuleTest.freestanding_func_ref_abstract(1) == 0

    @test JuliaModuleTest.freestanding_func_arrayarg(Vector{UInt32}()) == 4
    @inferred JuliaModuleTest.freestanding_func_arrayarg(Vector{UInt32}())

    @test JuliaModuleTest.freestanding_func_arrayarg(Vector{AbstractChar}()) == sizeof(UInt)
    @inferred JuliaModuleTest.freestanding_func_arrayarg(Vector{AbstractChar}())

    @test JuliaModuleTest.freestanding_func_typed_arrayarg(Vector{UInt32}()) == 4
    @inferred JuliaModuleTest.freestanding_func_typed_arrayarg(Vector{UInt32}())

    @test JuliaModuleTest.freestanding_func_typevaluearg(UInt(3)) == 3
    @inferred JuliaModuleTest.freestanding_func_typevaluearg(UInt(3))

    @test size(JuliaModuleTest.freestanding_func_ret_array(Int32)) == (2,2)
    @inferred Matrix{Int32} JuliaModuleTest.freestanding_func_ret_array(Int32)
    @inferred Matrix{Int64} JuliaModuleTest.freestanding_func_ret_array(Int64)

    @test size(JuliaModuleTest.freestanding_func_ret_ranked_array0(Int32)) == ()
    @inferred Array{Int32,0} JuliaModuleTest.freestanding_func_ret_ranked_array0(Int32)
    @inferred Array{Int64,0} JuliaModuleTest.freestanding_func_ret_ranked_array0(Int64)

    @test size(JuliaModuleTest.freestanding_func_ret_ranked_array1(Int32)) == (2,)
    @inferred Vector{Int32} JuliaModuleTest.freestanding_func_ret_ranked_array1(Int32)
    @inferred Vector{Int64} JuliaModuleTest.freestanding_func_ret_ranked_array1(Int64)

    @test size(JuliaModuleTest.freestanding_func_ret_ranked_array2(Int32)) == (2,2)
    @inferred Matrix{Int32} JuliaModuleTest.freestanding_func_ret_ranked_array2(Int32)
    @inferred Matrix{Int64} JuliaModuleTest.freestanding_func_ret_ranked_array2(Int64)

    @test size(JuliaModuleTest.freestanding_func_ret_ranked_array3(Int32)) == (2,2,2)
    @inferred Array{Int32,3} JuliaModuleTest.freestanding_func_ret_ranked_array3(Int32)
    @inferred Array{Int64,3} JuliaModuleTest.freestanding_func_ret_ranked_array3(Int64)

    @test size(JuliaModuleTest.freestanding_func_ret_typed_array()) == (2,2)
    @inferred Matrix{Float32} JuliaModuleTest.freestanding_func_ret_typed_array()
    @inferred Matrix{Float32} JuliaModuleTest.freestanding_func_ret_typed_array()

    @test JuliaModuleTest.freestanding_func_ret_rust_result(false) == 3
    @inferred JuliaModuleTest.freestanding_func_ret_rust_result(false)
    @test_throws JlrsCore.JlrsError JuliaModuleTest.freestanding_func_ret_rust_result(true)

    @test JuliaModuleTest.freestanding_func_ccall_ref_ret()
    @inferred JuliaModuleTest.freestanding_func_ccall_ref_ret()

    @test JuliaModuleTest.freestanding_func_typed_value_ret()
    @inferred JuliaModuleTest.freestanding_func_typed_value_ret()
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

    @test JuliaModuleTest.STATIC_CONST_U8 == 0x1
    @test !isconst(JuliaModuleTest, :STATIC_CONST_U8)

    @test JuliaModuleTest.STATIC_U8 == 0x2
    @test !isconst(JuliaModuleTest, :STATIC_U8)
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

    arr1 = Int[]
    arr2 = Int[1 1; 1 1]
    @test JuliaModuleTest.generic_arrays(arr1, arr2) == Int
    @test_throws MethodError JuliaModuleTest.generic_arrays(arr1, arr1)
    @test_throws MethodError JuliaModuleTest.generic_arrays(arr2, arr2)

    @test JuliaModuleTest.generic_ranked_arrays(arr1, arr2) == Int
    @test_throws MethodError JuliaModuleTest.generic_ranked_arrays(arr1, arr1)
    @test_throws MethodError JuliaModuleTest.generic_ranked_arrays(arr2, arr2)

    @test JuliaModuleTest.generic_ranked_array_ret_self(arr1) === arr1

    @test JuliaModuleTest.generic_typed_arrays(arr1, arr1) == Int
    @test JuliaModuleTest.generic_typed_arrays(arr1, arr2) == Int
    @test JuliaModuleTest.generic_typed_arrays(arr2, arr2) == Int

    arr3 = UInt[1 1; 1 1]
    @test_throws MethodError JuliaModuleTest.generic_typed_arrays(arr1, arr3)
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
    @test JuliaModuleTest.take_four_generics_m(v1) === v1
    @test_throws MethodError JuliaModuleTest.take_four_generics_m(v2)

    v3 = JuliaModuleTest.FourGenericsM{Int32, Int32, Int64, Int64}(1,2,3,4)
    @test JuliaModuleTest.take_four_generics_m_trailing1(v1) === v1
    @test JuliaModuleTest.take_four_generics_m_trailing1(v2) === v2
    @test_throws MethodError JuliaModuleTest.take_four_generics_m_trailing1(v3)

    v4 = JuliaModuleTest.FourGenericsM{Int32, Int64, Int32, Int64}(1,2,3,4)
    @test JuliaModuleTest.take_four_generics_m_trailing2(v1) === v1
    @test JuliaModuleTest.take_four_generics_m_trailing2(v2) === v2
    @test JuliaModuleTest.take_four_generics_m_trailing2(v3) === v3
    @test_throws MethodError JuliaModuleTest.take_four_generics_m_trailing2(v4)

    v5 = JuliaModuleTest.FourGenericsM{Int32, Int64, Int64, Int64}(1,2,3,4)
    @test JuliaModuleTest.take_four_generics_m_middle(v1) === v1
    @test JuliaModuleTest.take_four_generics_m_middle(v2) === v2
    @test_throws MethodError JuliaModuleTest.take_four_generics_m_middle(v3)
    @test JuliaModuleTest.take_four_generics_m_middle(v4) === v4
    @test_throws MethodError JuliaModuleTest.take_four_generics_m_middle(v5)

    v6 = JuliaModuleTest.FourGenericsM{Int64, Int32, Int32, Int32}(1,2,3,4)
    @test JuliaModuleTest.take_four_generics_m_start1(v1) === v1
    @test_throws MethodError JuliaModuleTest.take_four_generics_m_start1(v2)
    @test JuliaModuleTest.take_four_generics_m_start1(v6) === v6

    v7 = JuliaModuleTest.FourGenericsM{Int64, Int64, Int32, Int32}(1,2,3,4)
    @test JuliaModuleTest.take_four_generics_m_start2(v1) === v1
    @test_throws MethodError JuliaModuleTest.take_four_generics_m_start2(v2)
    @test JuliaModuleTest.take_four_generics_m_start2(v6) === v6
    @test JuliaModuleTest.take_four_generics_m_start2(v7) === v7
end