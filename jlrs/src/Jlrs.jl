module Jlrs

const color = Ref{Bool}(false)

function valuestring(@nospecialize(value::Any))::String
    io = IOBuffer()
    show(io, "text/plain", value)
    String(take!(io))
end

function errorstring(@nospecialize(value::Any))::String
    io = IOBuffer()
    showerror(IOContext(io, :color => color[], :compact => true), value)
    String(take!(io))
end
end
