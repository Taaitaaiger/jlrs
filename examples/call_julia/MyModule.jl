module MyModule
function complexfunc(a::Int, b::Int)::Float64
    @time c(a, b)
end

function c(a::Int, b::Int)::Float64
    x = rand(Float64, (a, a))
    for i in 1:b
        x += rand(Float64, (a, a))
    end

    z::Float64 = 0.0
    for j in 1:a
        z += x[j, j]
    end

    z
end
end