#ifndef LUA_EXPORTS_H 
#define LUA_EXPORTS_H

#include "freetype/freetype.h"
#include <lua/lua.h>
#include <lua/lauxlib.h>
#include <lua/lualib.h>

int draw_text(int startX, int startY, char* string, FT_Face typeFace, int* sprites, int* num_sprites, uint24_RGB* color, uint24_RGB* bgcol, int newline_offset);

/* int l_draw_text(lua_State* L); */
/* int l_setsize(lua_State* L); */
/* int l_readrotary(lua_State* L); */
/* int l_getgpio(lua_State* L); */
/* int l_wait(lua_State* L); */
/* int l_center_sprite_group_x(lua_State* L); */
/* int l_draw_sprites(lua_State* L); */
/* int l_delete_sprites(lua_State* L); */
/* int luaopen_lpb(lua_State* L); */

lua_State* lua_init();

#endif
