-- sandboxing (still tbd)
--_ENV = {
--    ipairs = ipairs,
--    next = next,
--    pairs = pairs,
--    pcall = pcall,
--    tonumber = tonumber,
--    tostring = tostring,
--    type = type,
--    unpack = unpack,
--    coroutine = { create = coroutine.create, resume = coroutine.resume,
--                  running = coroutine.running, status = coroutine.status,
--                  wrap = coroutine.wrap },
--    string = { byte = string.byte, char = string.char, find = string.find,
--               format = string.format, gmatch = string.gmatch, gsub = string.gsub,
--               len = string.len, lower = string.lower, match = string.match,
--               rep = string.rep, reverse = string.reverse, sub = string.sub,
--               upper = string.upper },
--    table = { insert = table.insert, maxn = table.maxn, remove = table.remove,
--              sort = table.sort },
--    math = { abs = math.abs, acos = math.acos, asin = math.asin,
--             atan = math.atan, atan2 = math.atan2, ceil = math.ceil, cos = math.cos,
--             cosh = math.cosh, deg = math.deg, exp = math.exp, floor = math.floor,
--             fmod = math.fmod, frexp = math.frexp, huge = math.huge,
--             ldexp = math.ldexp, log = math.log, log10 = math.log10, max = math.max,
--             min = math.min, modf = math.modf, pi = math.pi, pow = math.pow,
--             rad = math.rad, random = math.random, sin = math.sin, sinh = math.sinh,
--             sqrt = math.sqrt, tan = math.tan, tanh = math.tanh },
--    os = { clock = os.clock, difftime = os.difftime, time = os.time },
--}

-- __doukutsu_rs is an internal API used meant to be used solely by doukutsu-rs to implement higher-level,
-- documented APIs and is a subject to change. Do NOT use it or your scripts will break.

__doukutsu_rs_runtime_dont_touch = {}
doukutsu = {}

ModCS = {
    Flag = {},
    Game = {
        Act = nil,
    },
    Mod = {},
    NPC = {},
    Organya = {},
    Player = {},
    Rect = {},
    SkipFlag = {},
    Sound = {},
}

__doukutsu_rs_runtime_dont_touch._known_settings = {
    ["doukutsu-rs.intro.event_id"] = 0x1000,
    ["doukutsu-rs.intro.stage_id"] = 0x1001,
    ["doukutsu-rs.intro.pos"] = 0x1002,
    ["doukutsu-rs.new_game.event_id"] = 0x1003,
    ["doukutsu-rs.new_game.stage_id"] = 0x1004,
    ["doukutsu-rs.new_game.pos"] = 0x1005,
    ["doukutsu-rs.window.height"] = 0x1100,
    ["doukutsu-rs.window.width"] = 0x1101,
    ["doukutsu-rs.window.title"] = 0x1102,
    ["doukutsu-rs.font_scale"] = 0x2000,
    ["doukutsu-rs.tsc.encoding"] = 0x3000,
    ["doukutsu-rs.tsc.encrypted"] = 0x3001,
}

__doukutsu_rs_runtime_dont_touch._requires = {}

require = function(modname)
    if __doukutsu_rs_runtime_dont_touch._requires[modname] == nil then
        local mod = __doukutsu_rs:loadScript(modname)

        __doukutsu_rs_runtime_dont_touch._requires[modname] = { mod = mod, loaded = True }
    else
        return __doukutsu_rs_runtime_dont_touch._requires[modname].mod
    end
end

__doukutsu_rs_runtime_dont_touch._registered = {
    tick = {},
}

__doukutsu_rs_runtime_dont_touch._handlers = setmetatable({
    tick = function(scene)
        if type(ModCS.Game.Act) == 'function' then
            pcall(ModCS.Game.Act)
        end

        for _, h in pairs(__doukutsu_rs_runtime_dont_touch._registered.tick) do
            pcall(h, scene)
        end
    end,
}, {
    __index = function(self, event)
        error("Unknown event: " .. event)
    end,
})

__doukutsu_rs_runtime_dont_touch._registeredNPCHooks = {}

__doukutsu_rs_runtime_dont_touch._tryNPCHook = function(npc_id, npc_type)
    local hook = __doukutsu_rs_runtime_dont_touch._registeredNPCHooks[npc_type]
    if hook ~= nil then
        local npc = __doukutsu_rs_runtime_dont_touch._getNPCRef(npc_id)
        if npc ~= nil then
            local status, err = pcall(hook, npc)

            if not status then
                print("error in npc handler:" .. err)
            end
        end

        return true
    end

    return false
end

__doukutsu_rs_runtime_dont_touch._initializeScript = function(script)
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

__doukutsu_rs_runtime_dont_touch._createPlayerRef = function(player_id)
    local player_ref = { id = player_id }

    function player_ref.damage(self, value)
        __doukutsu_rs:playerCommand(rawget(self, "id"), 0x200, value)
    end

    setmetatable(player_ref, {
        __index = function(self, property)
            if property == "x" then
                return __doukutsu_rs:playerCommand(rawget(self, "id"), 0x10)
            elseif property == "y" then
                return __doukutsu_rs:playerCommand(rawget(self, "id"), 0x11)
            elseif property == "velX" then
                return __doukutsu_rs:playerCommand(rawget(self, "id"), 0x12)
            elseif property == "velY" then
                return __doukutsu_rs:playerCommand(rawget(self, "id"), 0x13)
            else
                return nil
            end
        end,
        __newindex = function(self, property, val)
            if property == "x" then
                __doukutsu_rs:playerCommand(rawget(self, "id"), 0x110, val)
            elseif property == "y" then
                __doukutsu_rs:playerCommand(rawget(self, "id"), 0x111, val)
            elseif property == "velX" then
                __doukutsu_rs:playerCommand(rawget(self, "id"), 0x112, val)
            elseif property == "velY" then
                __doukutsu_rs:playerCommand(rawget(self, "id"), 0x113, val)
            end

            return nil
        end,
    })

    return player_ref
end

__doukutsu_rs_runtime_dont_touch._npcRefs = {}

__doukutsu_rs_runtime_dont_touch._getNPCRef = function(npc_id)
    if __doukutsu_rs_runtime_dont_touch._npcRefs[npc_id] == nil then
        local npc_ref = __doukutsu_rs_runtime_dont_touch._createNPCRef(npc_id)
        if npc_ref == nil then
            return nil
        end

        __doukutsu_rs_runtime_dont_touch._npcRefs[npc_id] = npc_ref

        return npc_ref
    end

    return __doukutsu_rs_runtime_dont_touch._npcRefs[npc_id]
end

__doukutsu_rs_runtime_dont_touch._createNPCRef = function(npc_id)
    local npc_ref = { id = npc_id }

    function npc_ref.closestPlayer(self)
        return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x200)
    end

    function npc_ref.random(self, min, max)
        return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x201, min, max)
    end

    function npc_ref.hitLeftWall(self)
        local flags = __doukutsu_rs:npcCommand(rawget(self, "id"), 0x0f)
        return (flags & 1) ~= 0
    end

    function npc_ref.hitCeiling(self)
        local flags = __doukutsu_rs:npcCommand(rawget(self, "id"), 0x0f)
        return (flags & 2) ~= 0
    end

    function npc_ref.hitRightWall(self)
        local flags = __doukutsu_rs:npcCommand(rawget(self, "id"), 0x0f)
        return (flags & 4) ~= 0
    end

    function npc_ref.hitFloor(self)
        local flags = __doukutsu_rs:npcCommand(rawget(self, "id"), 0x0f)
        return (flags & 8) ~= 0
    end

    function npc_ref.parentNPC(self)
        local id = __doukutsu_rs:npcCommand(rawget(self, "id"), 0x1c)
        return __doukutsu_rs_runtime_dont_touch._getNPCRef(id)
    end

    function npc_ref.getAnimRect(self)
        local l, t, r, b = __doukutsu_rs:npcCommand(rawget(self, "id"), 0x202)
        return { l, t, r, b }
    end

    function npc_ref.setAnimRect(self, l, t, r, b)
        if type(l) == "number" then
            __doukutsu_rs:npcCommand(rawget(self, "id"), 0x203, l, t, r, b)
        elseif type(l) == "table" then
            __doukutsu_rs:npcCommand(rawget(self, "id"), 0x203, l[1], l[2], l[3], l[4])
        else
            error("Invalid parameters supplied.")
        end
    end

    setmetatable(npc_ref, {
        __index = function(self, property)
            if property == "x" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x10)
            elseif property == "y" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x11)
            elseif property == "velX" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x12)
            elseif property == "velY" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x13)
            elseif property == "velX2" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x14)
            elseif property == "velY2" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x15)
            elseif property == "actionNum" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x16)
            elseif property == "animNum" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x17)
            elseif property == "actionCounter" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x18)
            elseif property == "actionCounter2" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x19)
            elseif property == "actionCounter3" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x1a)
            elseif property == "animCounter" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x1b)
            elseif property == "parentId" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x1c)
            elseif property == "npcType" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x1d)
            elseif property == "life" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x1e)
            elseif property == "flagNum" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x1f)
            elseif property == "eventNum" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x20)
            elseif property == "direction" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x21)
            elseif property == "rawDirection" then
                return __doukutsu_rs:npcCommand(rawget(self, "id"), 0x22)
            else
                return nil
            end
        end,
        __newindex = function(self, property, val)
            if property == "x" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x110, val)
            elseif property == "y" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x111, val)
            elseif property == "velX" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x112, val)
            elseif property == "velY" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x113, val)
            elseif property == "velX2" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x114, val)
            elseif property == "velY2" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x115, val)
            elseif property == "actionNum" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x116, val)
            elseif property == "animNum" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x117, val)
            elseif property == "actionCounter" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x118, val)
            elseif property == "actionCounter2" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x119, val)
            elseif property == "actionCounter3" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x11a, val)
            elseif property == "animCounter" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x11b, val)
            elseif property == "parentId" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x11c, val)
            elseif property == "npcType" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x11d, val)
            elseif property == "life" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x11e, val)
            elseif property == "flagNum" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x11f, val)
            elseif property == "eventNum" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x120, val)
            elseif property == "direction" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x121, val)
            elseif property == "rawDirection" then
                __doukutsu_rs:npcCommand(rawget(self, "id"), 0x121, val) -- <- not a typo
            end

            return nil
        end,
    })

    return npc_ref
end

__doukutsu_rs_runtime_dont_touch._playerRef0 = __doukutsu_rs_runtime_dont_touch._createPlayerRef(0)
__doukutsu_rs_runtime_dont_touch._playerRef1 = __doukutsu_rs_runtime_dont_touch._createPlayerRef(1)

doukutsu.rs = {}
setmetatable(doukutsu.rs, {
    __index = function(self, property)
        if property == "lightingMode" then
            return __doukutsu_rs:stageCommand(0x01)
        elseif property == "lightingEnabled" then
            return __doukutsu_rs:stageCommand(0x02)
        else
            return nil
        end
    end,
    __newindex = function(self, property, val)
        if property == "lightingMode" then
            __doukutsu_rs:stageCommand(0x101, val)
        end

        return nil
    end,
})

setmetatable(doukutsu, {
    __index = function(self, property) 
    	if property == "currentStage" then
            local v = __doukutsu_rs:stageCommand(0x03)
            if v == nil then
            	v = -1
            end
            
            return v
    	end
    end,
})

doukutsu.player = __doukutsu_rs_runtime_dont_touch._playerRef0

function doukutsu.playSfx(id)
    __doukutsu_rs:playSfx(id)
end

function doukutsu.playSfxLoop(id)
    __doukutsu_rs:playSfxLoop(id)
end

function doukutsu.playSong(id)
    __doukutsu_rs:playSong(id)
end

function doukutsu.players()
    return { __doukutsu_rs_runtime_dont_touch._playerRef0, __doukutsu_rs_runtime_dont_touch._playerRef1 }
end

function doukutsu.setSetting(key, value)
    assert(type(key) == "string", "key must be a string.")

    local id = __doukutsu_rs_runtime_dont_touch._known_settings[key]
    if id ~= nil then
        __doukutsu_rs:setEngineConstant(id, value)
    end
end

function doukutsu.setNPCHandler(npc_type, handler)
    assert(type(npc_type) == "number", "npc type must be an integer.")

    __doukutsu_rs_runtime_dont_touch._registeredNPCHooks[npc_type] = handler
end

function doukutsu.on(event, handler)
    assert(type(event) == "string", "event type must be a string.")
    assert(type(handler) == "function", "event handler must be a function.")

    if __doukutsu_rs_runtime_dont_touch._registered[event] == nil then
        error("Unknown event: " .. event)
    end

    table.insert(__doukutsu_rs_runtime_dont_touch._registered[event], handler)

    return handler
end

function doukutsu.removeHandler(event, handler)
    assert(type(event) == "string", "event type must be a string.")
    assert(type(handler) == "function", "event handler must be a function.")

    if __doukutsu_rs_runtime_dont_touch._registered[event] == nil then
        error("Unknown event: " .. event)
    end

    local index = -1
    for i, h in pairs(__doukutsu_rs_runtime_dont_touch._registered[event]) do
        if handler == h then
            index = i
            break
        end
    end

    if index ~= -1 then
        table.remove(__doukutsu_rs_runtime_dont_touch._registered[event], index)
    end

    return handler
end

ModCS.Color = { r = 0, g = 0, b = 0 }
function ModCS.Color:_new(o)
    o = o or {}
    setmetatable(o, self)
    self.__index = self
    self.r = 0
    self.g = 0
    self.b = 0
    return o
end

ModCS.Color.Create = function(color)
    return ModCS.Color:_new(nil)
end

function ModCS.Color:Set(r, g, b)
    self.r = tonumber(r) or 0
    self.g = tonumber(g) or 0
    self.b = tonumber(b) or 0
end

function ModCS.Color:Box()
    -- stub
end

function ModCS.Mod.SetName(name)
    -- stub
end

function ModCS.Mod.SetAuthor(name)
    -- stub
end

function ModCS.Mod.SetVersion(v1, v2, v3, v4)
    -- stub
end

function ModCS.Mod.SetOpening(stage_id, event_id, ticks)
    __doukutsu_rs:setEngineConstant(0x1000, event_id)
    __doukutsu_rs:setEngineConstant(0x1001, stage_id)
    -- todo ticks
end

function ModCS.Mod.SetStart(stage_id, pos_x, pos_y, event_id)
    __doukutsu_rs:setEngineConstant(0x1003, event_id)
    __doukutsu_rs:setEngineConstant(0x1004, stage_id)
    __doukutsu_rs:setEngineConstant(0x1005, pos_x, pos_y)
end

function ModCS.Flag.Set(id)
    __doukutsu_rs:setFlag(id, True)
end

function ModCS.Flag.Unset(id)
    __doukutsu_rs:setFlag(id, False)
end

function ModCS.Flag.Get(id)
    return __doukutsu_rs:getFlag(id) or False
end

function ModCS.SkipFlag.Set(id)
    __doukutsu_rs:setSkipFlag(id, True)
end

function ModCS.SkipFlag.Unset(id)
    __doukutsu_rs:setSkipFlag(id, False)
end

function ModCS.SkipFlag.Get(id)
    return __doukutsu_rs:getSkipFlag(id) or False
end

function ModCS.Organya.Play(id)
    __doukutsu_rs:playSong(id)
end

function ModCS.Sound.Play(id, loop)
    if loop then
        __doukutsu_rs:playSfxLoop(id)
    else
        __doukutsu_rs:playSfx(id)
    end
end

function ModCS.Player.AddMaxLife(life)
    -- stub
end
