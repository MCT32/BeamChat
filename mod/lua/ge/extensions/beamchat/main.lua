local M = {}

local host, port = "127.0.0.1", 5555
local socket = require('socket')

function M.onExtensionLoaded()
    log("I", "beamchat", "BeamChat GELua extension loaded")

    -- Create TCP stream
    local tcp, err = assert(socket.tcp())
    if not tcp then
        log("E", "beamchat", "socket.tcp failed: " .. tostring(err))
        return
    end

    tcp:settimeout(3)

    -- Connect stream to local program
    local ok, connectErr = tcp:connect(host, port)
    if not ok then
        log("E", "beamchat", "connect failed: " .. tostring(connectErr))
        tcp:close()
        return
    end

    tcp:send("hello\n")
    log("I", "beamchat", "Sent hello")

    local s, status, partial = tcp:receive()
    log("I", "beamchat", "Received " .. s or partial)
    
    tcp:close()
end

return M