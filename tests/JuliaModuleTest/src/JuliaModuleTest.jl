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

separator = Sys.iswindows() ? '\\' : '/'
test_library_dir = get(ENV, "JULIA_MODULE_TEST_LIB_DIR", ".")
test_library_name = Sys.iswindows() ? "julia_module_test" : "libjulia_module_test"
@wrapmodule("$(test_library_dir)$(separator)$(test_library_name)", :julia_module_tests_init_fn)

function __init__()
    @initjlrs
end
end # module JuliaModuleTest
