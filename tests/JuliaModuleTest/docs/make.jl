push!(LOAD_PATH,"../src/")

using Documenter, JuliaModuleTest, Documenter.Remotes
makedocs(
    sitename="JuliaModuleTest",
    modules = [JuliaModuleTest],
    repo = Remotes.GitHub("Taaitaaiger", "jlrs"),
    remotes = nothing
)