#ifndef FONTFILE_H
#define FONTFILE_H
#include "ILIDriver.h"

typedef struct {
    uint16_t font_size;
    uint32_t num_glyphs;
} FONT_METADATA;

typedef struct {
    uint16_t advance;
    int16_t x;
    int16_t y;
    uint16_t width;
    uint16_t height;
    char vertical;
} CHAR_METADATA;

extern FILE* FONT_FILE;
extern FONT_METADATA fm;
extern CHAR_METADATA cm;

int set_font_size(int sz);
int verify_font_file(FILE* font);
void read_header(FILE* font, FONT_METADATA* fm);
int load_char(uint24_RGB** buf, CHAR_METADATA* cm, int curchar);
int load_bgimg(uint24_RGB* buf, char* name, char forceload);

#endif
