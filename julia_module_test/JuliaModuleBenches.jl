include("JuliaModule.jl")

using BenchmarkTools

scrubgc() = (GC.enable(true); GC.gc(); GC.gc(); GC.gc(); GC.gc(); GC.enable(false))
const opaque_int = JuliaModuleTest.OpaqueInt(Int32(-1));
const arr = Int[1, 2, 3, 4];

function runbenches()
    println("Benchmark freestanding_func_trivial")
    scrubgc()
    b = @benchmark JuliaModuleTest.freestanding_func_trivial() gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark freestanding_func_noargs")
    scrubgc()
    b = @benchmark JuliaModuleTest.freestanding_func_noargs() gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark freestanding_func_bitsarg")
    scrubgc()
    b = @benchmark JuliaModuleTest.freestanding_func_bitsarg(UInt(3)) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark freestanding_func_ref_bitsarg")
    scrubgc()
    b = @benchmark JuliaModuleTest.freestanding_func_ref_bitsarg(UInt(3)) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark freestanding_func_ref_mutarg")
    scrubgc()
    b = @benchmark JuliaModuleTest.freestanding_func_ref_mutarg(Main) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark freestanding_func_ref_any")
    scrubgc()
    b = @benchmark JuliaModuleTest.freestanding_func_ref_any(Main) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark freestanding_func_ref_abstract")
    scrubgc()
    b = @benchmark JuliaModuleTest.freestanding_func_ref_abstract(1) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark freestanding_func_arrayarg")
    scrubgc()
    b = @benchmark JuliaModuleTest.freestanding_func_arrayarg(Vector{UInt32}()) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark assoc_func")
    scrubgc()
    b = @benchmark JuliaModuleTest.assoc_func() gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark freestanding_func_typevaluearg")
    scrubgc()
    b = @benchmark JuliaModuleTest.freestanding_func_typevaluearg(UInt(3)) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark freestanding_func_ret_array")
    scrubgc()
    b = @benchmark JuliaModuleTest.freestanding_func_ret_array(Int32) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark freestanding_func_ret_rust_result")
    scrubgc()
    b = @benchmark JuliaModuleTest.freestanding_func_ret_rust_result(false) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark POpaqueInt")
    scrubgc()
    b = @benchmark JuliaModuleTest.OpaqueInt(Int32(-1)) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark POpaque64")
    scrubgc()
    b = @benchmark JuliaModuleTest.POpaque64(Float64(1.0)) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark unbox_opaque")
    scrubgc()
    b = @benchmark JuliaModuleTest.unbox_opaque(opaque_int) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark unbox_opaque_untracked")
    scrubgc()
    b = @benchmark JuliaModuleTest.unbox_opaque_untracked(opaque_int) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark increment!")
    scrubgc()
    b = @benchmark JuliaModuleTest.increment!(opaque_int) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark increment_unchecked!")
    scrubgc()
    b = @benchmark JuliaModuleTest.increment_unchecked!(opaque_int) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark increment_unchecked_nogc!")
    scrubgc()
    b = @benchmark JuliaModuleTest.increment_unchecked_nogc!(opaque_int) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark has_generic{Float32}")
    scrubgc()
    b = @benchmark JuliaModuleTest.has_generic(Float32(1.0)) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark has_generic{Float64}")
    scrubgc()
    b = @benchmark JuliaModuleTest.has_generic(Float64(1.0)) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark POpaqueTwo")
    scrubgc()
    b = @benchmark JuliaModuleTest.POpaqueTwo(Float32(1.0), Float32(2.0)) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark ForeignThing")
    scrubgc()
    b = @benchmark JuliaModuleTest.ForeignThing(Int32(-1)) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark async_callback")
    scrubgc()
    b = @benchmark JuliaModuleTest.async_callback(arr) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark generic_async_callback")
    scrubgc()
    b = @benchmark JuliaModuleTest.generic_async_callback(Float32(1.0)) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")
end

runbenches()
