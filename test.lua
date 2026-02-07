print("rining")
local id = vim.fn.jobstart({ "/Users/nicke/Code/semi_gui/target/debug/semi_gui" }, { rpc = true })
print(id)
-- wait 200 ms, then notify
vim.defer_fn(function()
	local r = vim.rpcnotify(id, "ai")
end, 1000) -- delay in milliseconds

-- local r = vim.rpcnotify(id, "test")
