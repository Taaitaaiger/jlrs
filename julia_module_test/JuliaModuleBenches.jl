include("JuliaModule.jl")

using BenchmarkTools

println("Benchmark freestanding_func_trivial")
@btime JuliaModuleTest.freestanding_func_trivial()

println("Benchmark freestanding_func_noargs")
@btime JuliaModuleTest.freestanding_func_noargs()

println("Benchmark freestanding_func_bitsarg")
@btime JuliaModuleTest.freestanding_func_bitsarg(UInt(3))

println("Benchmark freestanding_func_ref_bitsarg")
@btime JuliaModuleTest.freestanding_func_ref_bitsarg(UInt(3))

println("Benchmark freestanding_func_ref_mutarg")
@btime JuliaModuleTest.freestanding_func_ref_mutarg(Main)

println("Benchmark freestanding_func_ref_any")
@btime JuliaModuleTest.freestanding_func_ref_any(Main)

println("Benchmark freestanding_func_ref_abstract")
@btime JuliaModuleTest.freestanding_func_ref_abstract(1)

println("Benchmark freestanding_func_arrayarg")
@btime JuliaModuleTest.freestanding_func_arrayarg(Vector{UInt32}())

println("Benchmark assoc_func")
@btime JuliaModuleTest.assoc_func()

println("Benchmark freestanding_func_typevaluearg")
@btime JuliaModuleTest.freestanding_func_typevaluearg(UInt(3))

println("Benchmark freestanding_func_ret_array")
@btime JuliaModuleTest.freestanding_func_ret_array(Int32)

println("Benchmark freestanding_func_ret_rust_result")
@btime JuliaModuleTest.freestanding_func_ret_rust_result(false)

println("Benchmark POpaqueInt")
@btime JuliaModuleTest.OpaqueInt(Int32(-1))

const opaque_int = JuliaModuleTest.OpaqueInt(Int32(-1))
println("Benchmark unbox_opaque")
@btime JuliaModuleTest.unbox_opaque(opaque_int)

println("Benchmark unbox_opaque_untracked")
@btime JuliaModuleTest.unbox_opaque_untracked(opaque_int)

println("Benchmark increment!")
@btime JuliaModuleTest.increment!(opaque_int)

println("Benchmark increment_unchecked!")
@btime JuliaModuleTest.increment_unchecked!(opaque_int)

println("Benchmark increment_unchecked_nogc!")
@btime JuliaModuleTest.increment_unchecked_nogc!(opaque_int)

println("Benchmark has_generic{Float32}")
@btime JuliaModuleTest.has_generic(Float32(1.0))

println("Benchmark has_generic{Float64}")
@btime JuliaModuleTest.has_generic(Float64(1.0))

println("Benchmark POpaqueTwo")
@btime JuliaModuleTest.POpaqueTwo(Float32(1.0), Float32(2.0))

println("Benchmark ForeignThing")
@btime JuliaModuleTest.ForeignThing(Int32(-1))

const arr = Int[1, 2, 3, 4]
println("Benchmark async_callback")
@btime JuliaModuleTest.async_callback(arr)

println("Benchmark generic_async_callback")
@btime JuliaModuleTest.generic_async_callback(Float32(1.0))
