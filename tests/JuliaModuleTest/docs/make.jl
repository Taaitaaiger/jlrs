push!(LOAD_PATH,"../src/")

using Documenter, JuliaModuleTest
makedocs(
    sitename="JuliaModuleTest",
    modules = [JuliaModuleTest]
)