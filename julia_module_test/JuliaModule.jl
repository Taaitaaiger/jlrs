try
    using JlrsCore
catch e
    import Pkg; Pkg.add("JlrsCore")
    using JlrsCore
end

module JuliaModuleTest
using JlrsCore.Wrap

@wrapmodule("./libjulia_module_test", :julia_module_tests_init_fn)

function __init__()
    @initjlrs
end
end