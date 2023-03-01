-- Sets up the language server with the current directory
-- For neovim:
-- :source nvim.lua

local name = "zippyc"
local root_dir = vim.fn.getcwd()
local cmd = {vim.fn.resolve(root_dir .. "/target/debug/zc.exe"), "lsp"}

vim.api.nvim_create_autocmd({"FileType"}, {
    pattern = {"zippy"},
    callback = function()
        vim.lsp.start_client {
            name = name,
            root_dir = root_dir,
            cmd = cmd
        }
    end
})
