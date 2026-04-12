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
test_library_path = get(ENV, "JULIA_MODULE_TEST_LIB_DIR", ".$separator")
@wrapmodule("$(test_library_path)libjulia_module_test", :julia_module_tests_init_fn)

function __init__()
    @initjlrs
end
end # module JuliaModuleTest
