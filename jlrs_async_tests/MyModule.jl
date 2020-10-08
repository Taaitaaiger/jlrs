module MyModule
function complexfunc(dims::Int, iters::Int)::Float64
    x::Array{Float64, 2} = ones(Float64, (dims, dims))
    for i in 1:iters
        x .+= 1.0
    end

    z::Float64 = 0.0
    for j in 1:dims
        z += x[j, j]
    end

    z
end
end