fx_version 'cerulean'
games { 'gta5' }
lua54 'yes'

author 'Imthatguyhere (ITGH | Tyler)'
description 'FiveM Client WebSocket Test'
version '1.0.0'

client_scripts({
	"client/client_main.lua",
})

server_scripts({
	"server/server_main.lua",
})

files({
	"nui/build/index.html",
	"nui/build/assets/*.js",
	"nui/build/assets/*.css",
})

ui_page("nui/build/index.html")