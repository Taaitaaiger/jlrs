using JuliaModuleTest
using Documenter

DocMeta.setdocmeta!(JuliaModuleTest, :DocTestSetup, :(using JuliaModuleTest); recursive=true)

makedocs(;
    modules=[JuliaModuleTest],
    authors="Thomas van Doornmalen <thomas.vandoornmalen@gmail.com> and contributors",
    sitename="JuliaModuleTest.jl",
    format=Documenter.HTML(;
        canonical="https://Taaitaaiger.github.io/JuliaModuleTest.jl",
        edit_link="master",
        assets=String[],
    ),
    pages=[
        "Home" => "index.md",
    ],
)

# deploydocs(;
#     repo="github.com/Taaitaaiger/jlrs",
#     devbranch="master",
# )
