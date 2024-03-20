#include "rotenc.h"
#include "oam.h"
#include <rom/ets_sys.h>
#include "board_config.h"
#include "lua_exports.h"
#include "utf8.h"
#include "pwm_fade.h"
#include "button.h"
#include "pwm_output.h"
#include "system_status.h"
#include "socket.h"

#define FT_ERR_HANDLE(code, loc) error = code; if(error) ets_printf("Error occured at %s! Error: %d\n", loc, (int) error);

extern SPRITE_24_H** OAM_SPRITE_TABLE;
extern SPRITE_BITMAP* bitmap_cache[SPRITE_LIMIT];
extern int text_cache[SPRITE_LIMIT];
extern int text_size_cache[SPRITE_LIMIT];
extern uint8_t text_cache_size;
extern uint24_RGB* background_color;
extern uint64_t advance_x_cache[SPRITE_LIMIT];
extern uint16_t y_loc_cache[SPRITE_LIMIT];
extern uint16_t width_cache[SPRITE_LIMIT];
extern uint16_t height_cache[SPRITE_LIMIT];
extern uint24_RGB fg_cache[SPRITE_LIMIT];
extern uint24_RGB bg_cache[SPRITE_LIMIT];
extern uint24_RGB* foreground_color;

extern FT_Face typeFace;
extern rotary_encoder_info_t* infop;
extern spi_device_handle_t spi;
extern pwm_fade_info_t pfade_channels[8];
extern QueueHandle_t* button_events;

int draw_text(int startX, int startY, char* string, FT_Face typeFace, int* sprites, int* num_sprites, uint24_RGB* color, uint24_RGB* bgcol, int newline_offset) {
    FT_Vector offset;
    FT_GlyphSlot slot;

	slot = typeFace->glyph;
	offset.x = startX << 6;
	offset.y = startY << 6;

    int i = 0;
    char* reader_head = string; // so that there is no modification
    int err;
    int curchar;
    uint8_t alphaR, alphaG, alphaB;
    uint24_RGB* bg;
    uint64_t advance_x;
    int yloc = startY;
    uint16_t width;
    uint16_t height;
    FT_Int bmp_top;
    if (bgcol == NULL)
        bg = background_color;
    else
        bg = bgcol;
    SPRITE_BITMAP* bmp;
    if(num_sprites != NULL)
        *num_sprites = 0;
    while (*reader_head != 0) {
        curchar = decode_code_point(&reader_head);

        for(int x=0;x<text_cache_size;++x) {
            if(text_cache[x] == curchar && text_size_cache[x] == typeFace->size->metrics.height && coloreq(&fg_cache[x], color) && coloreq(&bg_cache[x], bg)) {
                bmp = bitmap_cache[x];
                bmp_top = 240 - y_loc_cache[x] - yloc;
                advance_x = advance_x_cache[x];
                width = width_cache[x];
                height = height_cache[x];

                if(((offset.x >> 6) + width) > 320 && newline_offset > 0) {
                    yloc -= newline_offset;
                    offset.y = yloc << 6;
                    offset.x = startX << 6;
                    bmp_top += newline_offset;
                }
                goto skip_bitmap_assignment;
            }
        }

		FT_Set_Transform(typeFace, NULL, &offset);
		err = FT_Load_Char(typeFace, curchar, FT_LOAD_RENDER | FT_LOAD_TARGET_LCD_V);
        if(err)
            return err;

		uint24_RGB* spriteBuf = (uint24_RGB*) malloc(slot->bitmap.rows * slot->bitmap.width);
        bmp = (SPRITE_BITMAP*) malloc(sizeof(SPRITE_BITMAP));
        bmp->refcount = 0;
        bmp->c = spriteBuf;
		int sz = slot->bitmap.rows*slot->bitmap.width / 3;
		for(int p=0;p<sz;p++) {
            alphaB = slot->bitmap.buffer[((p*3/slot->bitmap.rows))+((p*3)%slot->bitmap.rows)*slot->bitmap.width];
            alphaG = slot->bitmap.buffer[((p*3/slot->bitmap.rows))+((p*3+1)%slot->bitmap.rows)*slot->bitmap.width];
            alphaR = slot->bitmap.buffer[((p*3/slot->bitmap.rows))+((p*3+2)%slot->bitmap.rows)*slot->bitmap.width];
			spriteBuf[p].pixelB = ((255-alphaB) * bg->pixelB + alphaB * color->pixelB) / 255;
			spriteBuf[p].pixelG = ((255-alphaG) * bg->pixelG + alphaG * color->pixelG) / 255;
			spriteBuf[p].pixelR = ((255-alphaR) * bg->pixelR + alphaR * color->pixelR) / 255;
		}

		bmp_top = 240 - slot->bitmap_top;
        advance_x = slot->advance.x;
        width = slot->bitmap.width;
        height = slot->bitmap.rows/3;
        if(((offset.x >> 6) + width) > 320 && newline_offset > 0) {
            yloc -= newline_offset;
            offset.y = yloc << 6;
            offset.x = startX << 6;
            bmp_top += newline_offset;
        }
        if(text_cache_size < SPRITE_LIMIT) {
            text_cache[text_cache_size] = curchar;
            memcpy(&fg_cache[text_cache_size], color, sizeof(uint24_RGB));
            memcpy(&bg_cache[text_cache_size], bg, sizeof(uint24_RGB));
            bitmap_cache[text_cache_size] = bmp;
            text_size_cache[text_cache_size] = typeFace->size->metrics.height;
            advance_x_cache[text_cache_size] = advance_x;
            y_loc_cache[text_cache_size] = 240 - yloc - bmp_top;
            width_cache[text_cache_size] = width;
            height_cache[text_cache_size] = height;
            text_cache_size++;
        }

skip_bitmap_assignment:
		// int inx = init_sprite(bmp, slot->bitmap_left, bmp_top, slot->bitmap.width, slot->bitmap.rows/3, false, false, true);
		int inx = init_sprite(bmp, offset.x >> 6, bmp_top, width, height, false, false, true);

        if (sprites && curchar != ' ') {
            sprites[i++] = inx;
            if(num_sprites != NULL)
                (*num_sprites)++;
        }

		offset.x += advance_x;
		offset.y += slot->advance.y;
	}

    return 0;
}

static int l_draw_text(lua_State* L) {
    // int startX, int startY, char* string, FT_Face typeFace, int* sprites, uint24_RGB* color, uint24_RGB* bgcol
    int x = luaL_checkinteger(L, 1);
    int y = luaL_checkinteger(L, 2);
    size_t len;
    const char* str = luaL_checklstring(L, 3, NULL);
    len = utf8strlen(str);
    luaL_checktype(L, 4, LUA_TTABLE);
    luaL_checktype(L, 5, LUA_TTABLE);
    uint24_RGB fgcol, bgcol;
    // int* sprites = malloc(len);
    int sprites[len];

    lua_pushinteger(L, 1);
    lua_gettable(L, 4);
    fgcol.pixelR = luaL_checkinteger(L, -1);
    lua_pushinteger(L, 1);
    lua_gettable(L, 5);
    bgcol.pixelR = luaL_checkinteger(L, -1);

    lua_pushinteger(L, 2);
    lua_gettable(L, 4);
    fgcol.pixelG = luaL_checkinteger(L, -1);
    lua_pushinteger(L, 2);
    lua_gettable(L, 5);
    bgcol.pixelG = luaL_checkinteger(L, -1);

    lua_pushinteger(L, 3);
    lua_gettable(L, 4);
    fgcol.pixelB = luaL_checkinteger(L, -1);
    lua_pushinteger(L, 3);
    lua_gettable(L, 5);
    bgcol.pixelB = luaL_checkinteger(L, -1);
    lua_pop(L, 6);

    draw_text(x, y, str, typeFace, sprites, NULL, &fgcol, &bgcol, 0);
    lua_newtable(L);
    for(int i=1;i<=len;++i) {
        lua_pushnumber(L, i);
        lua_pushnumber(L, sprites[i-1]);
        lua_settable(L, -3);
    }

    // free(sprites);

    return 1;
}

static int l_setsize(lua_State* L) {
    int x = luaL_checkinteger(L, 1);
    int error;
	FT_ERR_HANDLE(FT_Set_Char_Size (typeFace, x, 0, 100, 0), "FT_Set_Char_Size"); // 0 = copy last value
    return 0;
}

static int l_readrotary(lua_State* L) {
    rotary_encoder_event_t event;
    // rotary_encoder_get_state(infop, &state);
    if(xQueueReceive(infop->queue, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
        lua_newtable(L);
        lua_pushnumber(L, 1);
        lua_pushinteger(L, event.state.direction);
        lua_settable(L, -3);
        lua_pushnumber(L, 2);
        lua_pushinteger(L, event.state.position);
        lua_settable(L, -3);
        return 1;
    }
    lua_pushnil(L);
    return 1;
}

static int l_getgpio(lua_State* L) {
    button_event_t ev;
    // int x = luaL_checkinteger(L, 1);
    // int d = gpio_get_level(x);
    if(xQueueReceive(*button_events, &ev, 10/portTICK_PERIOD_MS) == pdTRUE && ev.event == BUTTON_DOWN) {
        lua_newtable(L);
        lua_pushnumber(L, 1);
        lua_pushinteger(L, ev.pin);
        lua_settable(L, -3);
        lua_pushnumber(L, 2);
        lua_pushinteger(L, ev.event);
        lua_settable(L, -3);
        return 1;
    }
    // lua_pushboolean(L, d);
    lua_pushnil(L);
    return 1;
}

static int l_wait(lua_State* L) {
    int x = luaL_checkinteger(L, 1);
    vTaskDelay(x);
    return 0;
}

static int l_center_sprite_group_x(lua_State* L) {
    luaL_checktype(L, 1, LUA_TTABLE);
    unsigned int k = lua_rawlen(L, 1);
    // int* sprites = malloc(k);
    int sprites[k];
    for(int i=1;i<=k;++i) {
        lua_pushinteger(L, i);
        lua_gettable(L, 1);
        sprites[i-1] = luaL_checkinteger(L, -1);
    }
    center_sprite_group_x(sprites, k);
    // free(sprites);
    return 0;
}

static int l_right_justify_sprite_group_x(lua_State* L) {
    luaL_checktype(L, 1, LUA_TTABLE);
    unsigned int k = lua_rawlen(L, 1);
    int pad = luaL_checkinteger(L, 2);
    // int* sprites = malloc(k);
    int sprites[k];
    for(int i=1;i<=k;++i) {
        lua_pushinteger(L, i);
        lua_gettable(L, 1);
        sprites[i-1] = luaL_checkinteger(L, -1);
    }
    right_justify_sprite_group_x(sprites, k, pad);
    // free(sprites);
    return 0;
}

static int l_draw_sprites(lua_State* L) {
    luaL_checktype(L, 1, LUA_TTABLE);
    unsigned int k = lua_rawlen(L, 1);
    // int* sprites = malloc(k);
    int sprites[k];
    for(int i=1;i<=k;++i) {
        lua_pushinteger(L, i);
        lua_gettable(L, 1);
        sprites[i-1] = luaL_checkinteger(L, -1);
    }
    draw_sprites(spi, sprites, k);
    // free(sprites);
    return 0;
}

static int l_delete_sprites(lua_State* L) {
    int k = luaL_checkinteger(L, 1);
    delete_sprite(k);
    return 0;
}

static int l_create_rectangle(lua_State* L) {
    uint24_RGB col;
    int x = luaL_checkinteger(L, 1);
    int y = luaL_checkinteger(L, 2);
    int width = luaL_checkinteger(L, 3);
    int height = luaL_checkinteger(L, 4);
    luaL_checktype(L, 5, LUA_TTABLE);
    lua_pushinteger(L, 1);
    lua_gettable(L, 5);
    col.pixelR = luaL_checkinteger(L, -1);

    lua_pushinteger(L, 2);
    lua_gettable(L, 5);
    col.pixelG = luaL_checkinteger(L, -1);
    
    lua_pushinteger(L, 3);
    lua_gettable(L, 5);
    col.pixelB = luaL_checkinteger(L, -1);
    lua_pop(L, 3);

    lua_pushinteger(L, sprite_rectangle(x, y, width, height, &col));
    return 1;
}

static int l_set_sprite_draw_flags(lua_State* L) {
    luaL_checktype(L, 1, LUA_TTABLE);
    int draw = lua_toboolean(L, 2);
    unsigned int k = lua_rawlen(L, 1);
    // int* sprites = malloc(k);
    for(int i=1;i<=k;++i) {
        lua_pushinteger(L, i);
        lua_gettable(L, 1);
        OAM_SPRITE_TABLE[luaL_checkinteger(L, -1)]->draw = draw;
    }
    return 0;
}

static int l_move_sprite_x(lua_State* L) {
    luaL_checktype(L, 1, LUA_TTABLE);
    int newx = luaL_checkinteger(L, 2);
    unsigned int k = lua_rawlen(L, 1);
    // int* sprites = malloc(k);
    for(int i=1;i<=k;++i) {
        lua_pushinteger(L, i);
        lua_gettable(L, 1);
        OAM_SPRITE_TABLE[luaL_checkinteger(L, -1)]->posX = newx;
    }
    return 0;
}

static int l_move_sprite_y(lua_State* L) {
    luaL_checktype(L, 1, LUA_TTABLE);
    int newy = luaL_checkinteger(L, 2);
    unsigned int k = lua_rawlen(L, 1);
    // int* sprites = malloc(k);
    for(int i=1;i<=k;++i) {
        lua_pushinteger(L, i);
        lua_gettable(L, 1);
        OAM_SPRITE_TABLE[luaL_checkinteger(L, -1)]->posY = newy;
    }
    return 0;
}

// pwm indices: 0 1 2 3 R G B screen
//              0 1 2 3 4 5 6 7
// static int l_setup_fade(lua_State* L) {
//     int channel = luaL_checkinteger(L, 1);
//     int start = luaL_checkinteger(L, 2);
//     int stop = luaL_checkinteger(L, 3);
//     int step = luaL_checkinteger(L, 4);
//     pwm_setup_fade(&pfade_channels[channel], start, stop, step);
//     return 0;
// }

// step a table of channels' fades
// static int l_step_fade(lua_State* L) {
//     luaL_checktype(L, 1, LUA_TTABLE);
//     unsigned int k = lua_rawlen(L, 1);
//     for(int i=1;i<=k;++i) {
//         lua_pushinteger(L, i);
//         lua_gettable(L, 1);
//         pwm_step_fade(&pfade_channels[luaL_checkinteger(L, -1)]);
//     }
//     return 0;
// }

static int l_set_pwm(lua_State* L) {
    int channel = luaL_checkinteger(L, 1);
    int value = luaL_checkinteger(L, 2);
    output_set_value(channel-1, value);
    // ledc_set_duty(LEDC_LOW_SPEED_MODE, channel-1, value);
    // ledc_update_duty(LEDC_LOW_SPEED_MODE, channel-1);
    return 0;
}

static int l_toggle_pwm(lua_State* L) {
    int channel = luaL_checkinteger(L, 1);
    output_toggle(channel-1);
    // ledc_stop(LEDC_LOW_SPEED_MODE, channel-1, 0);
    return 0;
}

static int l_get_output_value(lua_State* L) {
    int channel = luaL_checkinteger(L, 1);
    int c = output_get_value(channel-1);
    lua_pushnumber(L, c);
    return 1;
}

static int l_output_add_value(lua_State* L) {
    int channel = luaL_checkinteger(L, 1);
    int increment = luaL_checkinteger(L, 2);
    output_add_value(channel-1, increment);
    return 0;
}

static int l_output_off(lua_State* L) {
    int k = luaL_checkinteger(L, 1);
    lua_pushboolean(L, (int) is_off(k-1));
    return 1;
}

static int l_output_was_updated(lua_State* L) {
    int k = luaL_checkinteger(L, 1);
    lua_pushboolean(L, (int) output_was_updated(k-1));
    return 1;
}

static int l_get_foreground(lua_State* L) {
    lua_newtable(L);
    lua_pushnumber(L, 1);
    lua_pushnumber(L, foreground_color->pixelR);
    lua_settable(L, -3);
    lua_pushnumber(L, 2);
    lua_pushnumber(L, foreground_color->pixelG);
    lua_settable(L, -3);
    lua_pushnumber(L, 3);
    lua_pushnumber(L, foreground_color->pixelB);
    lua_settable(L, -3);
    return 1;
}

static int l_get_background(lua_State* L) {
    lua_newtable(L);
    lua_pushnumber(L, 1);
    lua_pushnumber(L, background_color->pixelR);
    lua_settable(L, -3);
    lua_pushnumber(L, 2);
    lua_pushnumber(L, background_color->pixelG);
    lua_settable(L, -3);
    lua_pushnumber(L, 3);
    lua_pushnumber(L, background_color->pixelB);
    lua_settable(L, -3);
    return 1;
}

static int l_set_text_cache_auto_delete(lua_State* L) {
    char x = lua_toboolean(L, 1);
    set_text_cache_auto_delete(x);
    return 0;
}

static int l_flush_text_cache(lua_State* L) {
    flush_text_cache();
    return 0;
}

static int l_delete_all_sprites(lua_State* L) {
    delete_all_sprites();
    return 0;
}

static int l_wifi_is_connected(lua_State* L) {
    lua_pushboolean(L, system_flags & FLAG_WIFI_CONNECTED);
    return 1;
}

static int l_server_is_connected(lua_State* L) {
    lua_pushboolean(L, system_flags & FLAG_SERVER_CONNECTED);
    return 1;
}

static int l_server_send_message(lua_State* L) {
    size_t len;
    const char* str = luaL_checklstring(L, 3, &len);
    int k = send_message(str, len);
    lua_pushinteger(L, k);
    return 1;
}

static int l_server_get_message(lua_State* L) {
    char buf[1024];
    memset(buf, 0, 1024);
    get_message(buf, 1023);
    lua_pushstring(L, buf);
    return 1;
}

static const struct luaL_Reg lpb_funcs[] = {
    { "draw_text", l_draw_text },
    { "set_char_size", l_setsize },
    { "readrotary", l_readrotary },
    { "getgpio", l_getgpio },
    { "wait", l_wait },
    { "center_sprites_x", l_center_sprite_group_x },
    { "right_justify_sprites_x", l_right_justify_sprite_group_x },
    { "draw_sprites", l_draw_sprites },
    { "delete_sprite", l_delete_sprites },
    { "draw_rectangle", l_create_rectangle },
    { "sprite_set_draw", l_set_sprite_draw_flags },
    { "sprite_move_x", l_move_sprite_x },
    { "sprite_move_y", l_move_sprite_y },
    { "set_output", l_set_pwm },
    { "toggle_output", l_toggle_pwm },
    { "output_off", l_output_off },
    { "output_was_updated", l_output_was_updated },
    { "increment_output", l_output_add_value },
    { "get_output_value", l_get_output_value },
    { "foreground_color", l_get_foreground },
    { "background_color", l_get_background },
    { "flush_text_cache", l_flush_text_cache },
    { "enable_text_cache_auto_delete", l_set_text_cache_auto_delete },
    { "delete_all_sprites", l_delete_all_sprites },
    { "wifi_is_connected", l_wifi_is_connected },
    { "server_is_connected", l_server_is_connected },
    { "server_send_message", l_server_send_message },
    { "server_get_message", l_server_get_message },
    { NULL, NULL }
};

static int luaopen_lpb(lua_State *L) {
    luaL_newlib(L, lpb_funcs);
    return 1;
}

lua_State* lua_init() {
    lua_State* L = luaL_newstate();
    luaL_openlibs(L);
    luaL_requiref(L, "lpb", luaopen_lpb, 1);
    return L;
}
