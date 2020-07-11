module JlrsDerive
function derive(datatype::DataType)
    println(datatype)
    # println("Size: ", (datatype))
    println("N fields: ", fieldcount(datatype))
    println("Field names: ", fieldnames(datatype))
    println("Field types: ", fieldtypes(datatype))

    for i in 1:fieldcount(datatype)
        println("Field offset ", i, ": ", fieldoffset(datatype, i))
        println("Field type type ", i, ": ", typeof(fieldtype(datatype, i)))
    end
end

struct MyTypeAA
    a::Bool
    b::Int64
end

struct MyTypeAB
    a::MyTypeAA
    b
end

struct MyTypeAC
    a::MyTypeAA
    b::MyTypeAB
end

struct MyTypeBA
    a::Array
    b::Symbol
    c::Union{MyTypeAB, Int64}
end

function toposort(data::Dict{T,Set{T}}) where T
    data = copy(data)

    for item in setdiff(reduce(∪, values(data)), keys(data))
        data[item] = Set{T}()
    end

    rst = Vector{T}()
    while true
        ordered = Set(item for (item, dep) in data if isempty(dep))
        if isempty(ordered) break end
        append!(rst, ordered)
        data = Dict{T,Set{T}}(item => setdiff(dep, ordered) for (item, dep) in data if item ∉ ordered)
    end

    return rst
end

function dependencies!(out::Dict, type::Union{UnionAll, Union})
    println(type)
end

function dependencies!(out::Dict, type::DataType)
    if haskey(out, type) || type == Any
        return
    end

    out[type] = Set([t for t in fieldtypes(type) if t isa DataType])
    for t in fieldtypes(type)
        dependencies!(out, t)
    end
end

function dependencies(types::Vector{DataType})
    out = Dict{DataType, Set{DataType}}()
    for t in types
        dependencies!(out, t)
    end

    println(toposort(out))
end

function bindings(types::Vector{DataType})
    dependencies(types)
end

x = [MyTypeAA; MyTypeAB; MyTypeAC; MyTypeBA; StackTraces.StackFrame]
bindings(x)

end
