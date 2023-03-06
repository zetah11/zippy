-- Sets up the language server with the current directory
-- For neovim:
-- :source nvim.lua

local init_zippyc = function()
    local name = "zippyc"
    local exec = vim.fs.find({'zc.exe', 'zc'}, {
        path = vim.fn.resolve(vim.fn.getcwd() .. '/../target/debug')
    })

    if #exec == 0 then
        print "no zc exec found. has it been built?"
        return
    end

    exec = exec[1]

    local root_dir = vim.fn.getcwd()
    local cmd = {exec, "lsp"}
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
end

init_zippyc()
