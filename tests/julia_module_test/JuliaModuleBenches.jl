include("JuliaModule.jl")

using BenchmarkTools

scrubgc() = (GC.enable(true); GC.gc(); GC.gc(); GC.gc(); GC.gc(); GC.enable(false))
const opaque_int = JuliaModuleTest.OpaqueInt(Int32(-1));
const arr = Int[1, 2, 3, 4];

function runbenches()
    println("Benchmark takes_no_args_returns_nothing")
    scrubgc()
    b = @benchmark JuliaModuleTest.takes_no_args_returns_nothing() gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark takes_no_args_returns_usize")
    scrubgc()
    b = @benchmark JuliaModuleTest.takes_no_args_returns_usize() gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark takes_usize_returns_usize")
    scrubgc()
    b = @benchmark JuliaModuleTest.takes_usize_returns_usize(UInt(3)) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark takes_ref_usize")
    scrubgc()
    b = @benchmark JuliaModuleTest.takes_ref_usize(UInt(3)) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark takes_ref_module")
    scrubgc()
    b = @benchmark JuliaModuleTest.takes_ref_module(Main) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark takes_ref_any")
    scrubgc()
    b = @benchmark JuliaModuleTest.takes_ref_any(Main) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark takes_ref_number")
    scrubgc()
    b = @benchmark JuliaModuleTest.takes_ref_number(1) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark takes_array")
    scrubgc()
    b = @benchmark JuliaModuleTest.takes_array(Vector{UInt32}()) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark assoc_func")
    scrubgc()
    b = @benchmark JuliaModuleTest.assoc_func() gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark takes_typed_value")
    scrubgc()
    b = @benchmark JuliaModuleTest.takes_typed_value(UInt(3)) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark returns_array")
    scrubgc()
    b = @benchmark JuliaModuleTest.returns_array(Int32) gctrial = false gcsample = false
    show(stdout, MIME"text/plain"(), b)
    print("\n\n")

    println("Benchmark returns_jlrs_result")
    scrubgc()
    b = @benchmark JuliaModuleTest.returns_jlrs_result(false) gctrial = false gcsample = false
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
end

runbenches()
