-- Sets up the language server with the current directory
-- For neovim:
-- :source nvim.lua

local name = "zippyc"
local root_dir = vim.fn.getcwd()
local cmd = {vim.fn.resolve(root_dir .. "/../target/debug/zc.exe"), "lsp"}
local cmd_env = { ["RUST_BACKTRACE"] = 1 }

vim.api.nvim_create_autocmd({"BufRead", "BufNewFile"}, {
    pattern = {"*.z"},
    command = "set filetype=zippy"
})

vim.api.nvim_create_autocmd({"FileType"}, {
    pattern = {"zippy"},
    callback = function()
        vim.lsp.start {
            name = name,
            root_dir = root_dir,
            cmd = cmd,
            cmd_env = cmd_env,
        }
    end
})
