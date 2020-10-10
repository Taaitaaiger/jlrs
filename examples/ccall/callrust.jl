x = ccall((:add, "libccall"), Int32, (Int32, Int32), 1, 2)
println(1, " + ", 2, " = ", x)

y = [1.0 2.0 3.0; 4.0 5.0 6.0; 7.0 8.0 9.0]
println("Before increment: ", y)
ccall((:incr_array, "libccall"), Cvoid, (Array{Float64},), y)
println("After increment: ", y)
