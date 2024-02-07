#include "rotenc.h"
#include "oam.h"
#include <rom/ets_sys.h>
#include "board_config.h"
#include "lua_exports.h"
#include "utf8.h"

#define FT_ERR_HANDLE(code, loc) error = code; if(error) ets_printf("Error occured at %s! Error: %d\n", loc, (int) error);

extern SPRITE_BITMAP* bitmap_cache[SPRITE_LIMIT];
extern uint32_t text_cache[SPRITE_LIMIT];
extern int text_size_cache[SPRITE_LIMIT];
extern uint8_t text_cache_size;
extern uint24_RGB* background_color;
extern uint64_t advance_x_cache[SPRITE_LIMIT];
extern uint16_t y_loc_cache[SPRITE_LIMIT];
extern uint16_t width_cache[SPRITE_LIMIT];
extern uint16_t height_cache[SPRITE_LIMIT];

extern FT_Face typeFace;
extern rotary_encoder_info_t* infop;
extern spi_device_handle_t spi;

int draw_text(int startX, int startY, char* string, FT_Face typeFace, int* sprites, uint24_RGB* color, uint24_RGB* bgcol) {
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
    uint16_t width;
    uint16_t height;
    FT_Int bmp_top;
    if (bgcol == NULL)
        bg = background_color;
    else
        bg = bgcol;
    SPRITE_BITMAP* bmp;
    while (*reader_head != 0) {
        curchar = decode_code_point(&reader_head);

        for(int i=0;i<text_cache_size;++i) {
            if(text_cache[i] == curchar && text_size_cache[i] == typeFace->size->metrics.height) {
                bmp = bitmap_cache[i];
                bmp_top = 240 - y_loc_cache[i] - startY;
                advance_x = advance_x_cache[i];
                width = width_cache[i];
                height = height_cache[i];
                goto skip_bitmap_assignment;
            }
        }

		FT_Set_Transform(typeFace, NULL, &offset);
		err = FT_Load_Char(typeFace, curchar, FT_LOAD_RENDER | FT_LOAD_TARGET_LCD_V);
        if(err)
            return err;

		uint24_RGB* spriteBuf = (uint24_RGB*) malloc(slot->bitmap.rows * slot->bitmap.width);
        bmp = (SPRITE_BITMAP*) malloc(sizeof(SPRITE_BITMAP));
        bmp->refcount = 1;
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
        if(text_cache_size < SPRITE_LIMIT) {
            text_cache[text_cache_size] = curchar;
            bitmap_cache[text_cache_size] = bmp;
            text_size_cache[text_cache_size] = typeFace->size->metrics.height;
            advance_x_cache[text_cache_size] = advance_x;
            y_loc_cache[text_cache_size] = 240 - startY - bmp_top;
            width_cache[text_cache_size] = width;
            height_cache[text_cache_size] = height;
            text_cache_size++;
        }

skip_bitmap_assignment:
		// int inx = init_sprite(bmp, slot->bitmap_left, bmp_top, slot->bitmap.width, slot->bitmap.rows/3, false, false, true);
		int inx = init_sprite(bmp, offset.x >> 6, bmp_top, width, height, false, false, true);

        if (sprites && curchar != ' ')
            sprites[i++] = inx;

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
    const char* str = luaL_checklstring(L, 3, &len);
    luaL_checktype(L, 4, LUA_TTABLE);
    luaL_checktype(L, 5, LUA_TTABLE);
    uint24_RGB fgcol, bgcol;
    int* sprites = malloc(len);

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

    draw_text(x, y, str, typeFace, sprites, &fgcol, &bgcol);
    lua_newtable(L);
    for(int i=1;i<=len;++i) {
        lua_pushnumber(L, i);
        lua_pushnumber(L, sprites[i-1]);
        lua_settable(L, -3);
    }

    free(sprites);

    return 1;
}

static int l_setsize(lua_State* L) {
    int x = luaL_checkinteger(L, 1);
    int error;
	FT_ERR_HANDLE(FT_Set_Char_Size (typeFace, x, 0, 100, 0), "FT_Set_Char_Size"); // 0 = copy last value
    return 0;
}

static int l_readrotary(lua_State* L) {
    rotary_encoder_state_t state = { 0 };
    rotary_encoder_get_state(infop, &state);
    lua_newtable(L);
    lua_pushnumber(L, 1);
    lua_pushinteger(L, state.direction);
    lua_settable(L, -3);
    lua_pushnumber(L, 2);
    lua_pushinteger(L, state.position);
    lua_settable(L, -3);
    return 1;
}

static int l_getgpio(lua_State* L) {
    int x = luaL_checkinteger(L, 1);
    int d = gpio_get_level(x);
    lua_pushboolean(L, d);
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
    int* sprites = malloc(k);
    for(int i=1;i<=k;++i) {
        lua_pushinteger(L, i);
        lua_gettable(L, 1);
        sprites[i-1] = luaL_checkinteger(L, -1);
    }
    center_sprite_group_x(sprites, k);
    free(sprites);
    return 0;
}

static int l_draw_sprites(lua_State* L) {
    luaL_checktype(L, 1, LUA_TTABLE);
    unsigned int k = lua_rawlen(L, 1);
    int* sprites = malloc(k);
    for(int i=1;i<=k;++i) {
        lua_pushinteger(L, i);
        lua_gettable(L, 1);
        sprites[i-1] = luaL_checkinteger(L, -1);
    }
    draw_sprites(spi, sprites, k);
    free(sprites);
    return 0;
}

static int l_delete_sprites(lua_State* L) {
    int k = luaL_checkinteger(L, 1);
    delete_sprite(k);
    return 0;
}

static const struct luaL_Reg lpb_funcs[] = {
    { "draw_text", l_draw_text },
    { "set_char_size", l_setsize },
    { "readrotary", l_readrotary },
    { "getgpio", l_getgpio },
    { "wait", l_wait },
    { "center_sprites_x", l_center_sprite_group_x },
    { "draw_sprites", l_draw_sprites },
    { "delete_sprite", l_delete_sprites },
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
