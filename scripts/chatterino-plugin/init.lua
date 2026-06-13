local PORT = "47549"

---@param chan c2.Channel
---@param path string
---@param cb fun(res: c2.HTTPResponse)
local function send_req(chan, path, cb)
	local req = c2.HTTPRequest.create(c2.HTTPMethod.Get, "http://localhost:" .. PORT .. path)
	req:on_success(function(result)
		cb(result)
	end)
	req:on_error(function(result)
		chan:add_system_message("Failed to send request: " .. result:error())
	end)
	req:set_timeout(1000)
	req:execute()
end

c2.register_command("/warn-rain", function(ctx)
	send_req(ctx.channel, "/warn-rain", function(res)
		ctx.channel:add_system_message("Warnings are now shown")
	end)
end)
c2.register_command("/unwarn-rain", function(ctx)
	send_req(ctx.channel, "/unwarn-rain", function(res)
		ctx.channel:add_system_message("Warnings are no longer shown")
	end)
end)
