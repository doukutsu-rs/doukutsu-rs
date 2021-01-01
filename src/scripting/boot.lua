doukutsu = {}

doukutsu._registered = {
    tick = {},
}

doukutsu._handlers = setmetatable({
    tick = function(scene)
        for _, h in pairs(doukutsu._registered.tick) do
            pcall(h, scene)
        end
    end,
}, {
    __index = function(self, event)
        error("Unknown event: " .. event)
    end,
})

doukutsu._initialize_script = function(script)
    -- for compatibility with Lua 5.2+, copy-pasted from Lua mailing list
    -- http://lua-users.org/lists/lua-l/2010-06/msg00313.html
    local _setfenv = setfenv or function(f, t)
        f = (type(f) == 'function' and f or debug.getinfo(f + 1, 'f').func)
        local name
        local up = 0
        repeat
            up = up + 1
            name = debug.getupvalue(f, up)
        until name == '_ENV' or name == nil
        if name then

            debug.upvaluejoin(f, up, function()
                return name
            end, 1)
            debug.setupvalue(f, up, t)
        end
    end

    global_copy = {}
    for k, v in pairs(_G) do
        global_copy[k] = v
    end

    _setfenv(script, global_copy)
    script()
end

doukutsu.play_sfx = function(id)
    __doukutsu:play_sfx(id)
end

doukutsu.play_song = function(id)
    __doukutsu:play_song(id)
end

doukutsu.on = function(event, handler)
    assert(type(event) == "string", "event type must be a string.")
    assert(type(handler) == "function", "event handler must be a function.")

    if doukutsu._registered[event] == nil then
        error("Unknown event: " .. event)
    end

    table.insert(doukutsu._registered[event], handler)

    return handler
end

doukutsu.remove_handler = function(event, handler)
    assert(type(event) == "string", "event type must be a string.")
    assert(type(handler) == "function", "event handler must be a function.")

    if doukutsu._registered[event] == nil then
        error("Unknown event: " .. event)
    end

    local index = -1
    for i, h in pairs(doukutsu._registered[event]) do
        if handler == h then
            index = i
            break
        end
    end

    if index ~= -1 then
        table.remove(doukutsu._registered[event], index)
    end

    return handler
end
