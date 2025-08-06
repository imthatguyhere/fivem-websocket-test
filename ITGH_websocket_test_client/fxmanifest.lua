fx_version 'cerulean'
games { 'gta5' }
lua54 'yes'

author 'Imthatguyhere (ITGH | Tyler)'
description 'FiveM Client WebSocket Test'
version '1.0.0'

client_scripts({
	"client/client_main.lua",
	"shared/shared_config.lua",
})

server_scripts({
	"server/server_main.lua",
	"shared/shared_config.lua",
})

files({
	"nui/index.html",
	"nui/js/*.js",
})

ui_page("nui/index.html")