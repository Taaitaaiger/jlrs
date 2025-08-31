try
    using JlrsCore
catch e
    import Pkg; Pkg.add("JlrsCore")
    using JlrsCore
end

module JuliaModuleTest
using JlrsCore.Wrap

struct FourGenericsI{A, B, C, D}
    a::A
    b::B
    c::C
    d::D
end

mutable struct FourGenericsM{A, B, C, D}
    a::A
    b::B
    c::C
    d::D
end

@wrapmodule("./libjulia_module_test", :julia_module_tests_init_fn)

function __init__()
    @initjlrs
end
end