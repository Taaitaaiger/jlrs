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

    @test JuliaModuleTest.freestanding_func_typevaluearg(UInt(3)) == 3
    @inferred JuliaModuleTest.freestanding_func_typevaluearg(UInt(3))

    @test size(JuliaModuleTest.freestanding_func_ret_array(Int32)) == (2,2)
    @inferred Matrix{Int32} JuliaModuleTest.freestanding_func_ret_array(Int32)
    @inferred Matrix{Int64} JuliaModuleTest.freestanding_func_ret_array(Int64)

    @test JuliaModuleTest.freestanding_func_ret_rust_result(false) == 3
    @inferred JuliaModuleTest.freestanding_func_ret_rust_result(false)
    @test_throws JlrsCore.JlrsError JuliaModuleTest.freestanding_func_ret_rust_result(true)
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

@testset "Async callback" begin
    arr = Int[1, 2, 3, 4]
    @test JuliaModuleTest.async_callback(arr) == 10
    @inferred JuliaModuleTest.async_callback(arr)

    @test Ledger.try_borrow_exclusive(arr)
    @test_throws JlrsCore.JlrsError JuliaModuleTest.async_callback(arr)
    @test Ledger.unborrow_exclusive(arr)

    @test_throws JlrsCore.JlrsError JuliaModuleTest.async_callback_init_err()
    @test_throws JlrsCore.JlrsError JuliaModuleTest.async_callback_callback_err()
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
end

@testset "has_generic" begin
    @test JuliaModuleTest.has_generic(Float32(1.0)) == Float32(1.0)
    @test JuliaModuleTest.has_generic(Float64(1.0)) == Float64(1.0)
end

@testset "generic_callback" begin
    @test JuliaModuleTest.generic_async_callback(Float32(1.0)) == Float32(1.0)
    @test JuliaModuleTest.generic_async_callback(Float64(1.0)) == Float64(1.0)
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
