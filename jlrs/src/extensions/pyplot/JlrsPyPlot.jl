module JlrsPyPlot
using PyCall
pygui(:gtk3)
using Plots
pyplot()

using Base.Threads: Event, threadid
import Base.wait

default(show=true, reuse=false)

struct PlotHandle
    id::Int
end

mutable struct ActivePlot
    plots::Vector{Plots.Plot{Plots.PyPlotBackend}}
    version::Int
    event::Event
end

const activeplots = Dict{PlotHandle, ActivePlot}()
global id = 0
const plotlock = ReentrantLock()

function jlrsplot(plotfn::Function, args...; kwargs...)::PlotHandle
    if threadid() != 1
        error("jlrsplot can only be called from the main Julia thread.")
    end

    global id
    plotid = id += 1
    handle = PlotHandle(plotid)
    
    onclose = function (obj)
        lock(() -> begin 
            plt = activeplots[handle]
            notify(plt.event)
        end, plotlock)
    end

    plt = plotfn(args...; kwargs...)
    plt.o.canvas.mpl_connect("close_event", onclose)
    
    closed = Event()
    ap = ActivePlot([plt], 1, closed)
    lock(() -> begin 
        activeplots[handle] = ap
    end, plotlock)

    handle    
end

function updateplot!(handle::PlotHandle, plotfn::Function, args...; kwargs...)::Int
    if threadid() != 1
        error("updateplot! can only be called from the main Julia thread.")
    end

    lock(() -> begin
        plot = get(activeplots, handle, nothing)
        if plot === nothing || plot.event.set
            error("Window was closed")
        end

        newplot = plotfn(plot.plots[plot.version], args...; kwargs...)
        if newplot !== nothing
            push!(plot.plots, newplot)
            plot.version = length(plot.plots)
        end

        plot.version
    end, plotlock)
end

function setversion(handle::PlotHandle, version::Int)
    lock(() -> begin
        plot = get(activeplots, handle, nothing)
        if plot === nothing || plot.event.set
            error("Window was closed")
        end

        n = length(plot.plots)
        if n < version
            error("Version is $version but only $n versions exist")
        end

        plot.version = version
    end, plotlock)

    nothing
end

function wait(handle::PlotHandle)::Nothing
    plot = lock(() -> get(activeplots, handle, nothing), plotlock)

    if plot !== nothing
        wait(plot.event)
        lock(() -> delete!(activeplots, handle), plotlock)
    end

    nothing
end
end
