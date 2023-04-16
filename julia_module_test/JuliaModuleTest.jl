module JuliaModuleTestTests
try
    using JlrsCore
catch e
    import Pkg; Pkg.add(url="https://github.com/Taaitaaiger/JlrsCore.jl")
    using JlrsCore
end

using JlrsCore.Ledger
using Test

module JuliaModuleTest
using JlrsCore.Wrap

path = if Sys.islinux()
    "./libjulia_module_test.so"
elseif Sys.iswindows()
    "./libjulia_module_test.dll"
else
    "./libjulia_module_test.dylib"
end
@wrapmodule(path, :julia_module_tests_init_fn)

function __init__()
    @initjlrs
end
end

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

# using BenchmarkTools
#
# v = Vector{UInt32}()
# v2 = Vector{AbstractChar}()
# ty = Int32
#
# println("Benchmark freestanding_func_trivial")
# @btime JuliaModuleTest.freestanding_func_trivial()
# println("Benchmark freestanding_func_noargs")
# @btime JuliaModuleTest.freestanding_func_noargs()
# println("Benchmark freestanding_func_bitsarg")
# @btime JuliaModuleTest.freestanding_func_bitsarg(UInt(3))
# println("Benchmark freestanding_func_ref_bitsarg")
# @btime JuliaModuleTest.freestanding_func_ref_bitsarg(UInt(3))
# println("Benchmark freestanding_func_ref_mutarg")
# @btime JuliaModuleTest.freestanding_func_ref_mutarg(Main)
# println("Benchmark freestanding_func_ref_any")
# @btime JuliaModuleTest.freestanding_func_ref_any(Main)
# println("Benchmark freestanding_func_ref_abstract")
# @btime JuliaModuleTest.freestanding_func_ref_abstract(1)
# println("Benchmark freestanding_func_arrayarg")
# @btime JuliaModuleTest.freestanding_func_arrayarg(v)
# println("Benchmark freestanding_func_arrayarg")
# @btime JuliaModuleTest.freestanding_func_arrayarg(v2)
# println("Benchmark freestanding_func_typevaluearg")
# @btime JuliaModuleTest.freestanding_func_typevaluearg(UInt(3))
# println("Benchmark freestanding_func_ret_array")
# @btime JuliaModuleTest.freestanding_func_ret_array(ty)
# println("Benchmark freestanding_func_ret_rust_result")
# @btime JuliaModuleTest.freestanding_func_ret_rust_result(false)
end
