
include("JuliaModule.jl")

using JlrsCore.Ledger
using Test
using Random

@testset "Parallel Agent Environment" begin
    agent = JuliaModuleTest.Agent() do env
        s = String(env)
        r = ""
        # Force trigger GC
        GC.gc()
        for (i,ch) in enumerate(s)
            # Allocate a bunch of memory
            z = rand(500, 500)
            if i == string(sum(z))
                sleep(0.01)
            end
            r = string(r, ch)
        end
        JuliaModuleTest.Action(string(string(r), s))
    end
    n = 200
    playground = JuliaModuleTest.Playground()
    Threads.@threads for _ in 1:5
        result = JuliaModuleTest.play(agent, n, playground)
    end
end
