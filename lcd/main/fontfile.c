#include "fontfile.h"
#include <stdint.h>
#include <stdlib.h>
#include <rom/ets_sys.h>
#include <string.h>

FILE* FONT_FILE;
char font_file_open = false;
FONT_METADATA fm;
CHAR_METADATA cm;

int set_font_size(int sz) {
    char* font_name;
    switch(sz){
        case 12: font_name = "/mainfs/NC_12.cbf"; break;
        case 14: font_name = "/mainfs/NC_14.cbf"; break;
        case 18: font_name = "/mainfs/NC_18.cbf"; break;
        case 24: font_name = "/mainfs/NC_24.cbf"; break;
        case 42: font_name = "/mainfs/NC_42.cbf"; break;
        default: return -1;
    }
    if(font_file_open) {
        fclose(FONT_FILE);
        font_file_open = false;
    }
    FONT_FILE = fopen(font_name, "rb");
    font_file_open = true;
    if(verify_font_file(FONT_FILE))
        return -1;
    read_header(FONT_FILE, &fm);
    if(fm.font_size != sz)
        return -1;
    return 0;
}

// returns length of decoded data
int decode(uint8_t* indata, uint8_t* outdata) {
    int i=0;
    int j=0;
    int k=0;
    while(indata[i] != 0xff)
        if(indata[i] & 0x80) {
            j = k + (indata[i] ^ 0x80);
            while(k <= j)
                outdata[k++] = indata[++i];
            ++i;
        } else {
            for(j=0;j<indata[i] + 2;++j)
                outdata[k++] = indata[i+1];
            i += 2;
        }
    return k;
}

int decode_VERT(uint8_t* indata, uint8_t* outdata, uint16_t width, uint16_t height) {
    int i=0;
    int j=0;
    int m=0;
    int x=0;
    int y=0;
    while(indata[i] != 0xff)
        if(indata[i] & 0x80) {
            j = (indata[i] ^ 0x80);
            m = 0;
            while(m <= j) {
                outdata[y*width+x] = indata[++i];
                y++;
                if(y >= height) {
                    y=0;
                    x++;
                }
                m++;
            }
            ++i;
        } else {
            for(j=0;j<indata[i] + 2;++j) {
                outdata[y*width+x] = indata[i+1];
                y++;
                if(y >= height) {
                    y=0;
                    x++;
                }
            }
            i += 2;
        }
    return y*width+x;
}

const char correct_mbytes[] = {0x63, 0x62, 0x66, 0xe5, 0x9c, 0xa7, 0xe5, 0xad, 0x97, 0x02};
int verify_font_file(FILE* font) {
    char mbytes_buffer[10];
    fread(mbytes_buffer, 1, 10, font);
    for(int i=0;i<10;++i)
        if(mbytes_buffer[i] != correct_mbytes[i])
            return -1;
    return 0;
}

void read_header(FILE* font, FONT_METADATA* fm) {
    fseek(font, 10, SEEK_SET);
    fread(&fm->font_size, 2, 1, font);
    fread(&fm->num_glyphs, 4, 1, font);
}

uint32_t binary_search(FILE* f, uint16_t k) {
    long int offset = fm.num_glyphs >> 1;
    fseek(f, offset * 6 + 16, SEEK_SET);
    uint16_t prev_entry, curr_entry;
    fread(&curr_entry, 2, 1, f);
    prev_entry = curr_entry;
    while(curr_entry != k) {
        if(prev_entry < k && curr_entry > k && offset == 1) return 0;
        offset >>= 1;
        if(offset < 1) offset = 1;
        if(curr_entry > k)
            fseek(f, -offset * 6 - 2, SEEK_CUR);
        else
            fseek(f, offset * 6 - 2, SEEK_CUR);
        prev_entry = curr_entry;
        fread(&curr_entry, 2, 1, f);
    }
    fread(&offset, 4, 1, f);
    return offset;
}

int load_char(uint24_RGB** buf, CHAR_METADATA* cm, int curchar) {
    uint32_t offset = binary_search(FONT_FILE, curchar);
    if(offset == 0) {
        ets_printf("char not found: %x\n", curchar);
        return -1;
    }
    fseek(FONT_FILE, offset, SEEK_SET);
    fread(&cm->advance, 2, 1, FONT_FILE);
    fread(&cm->x, 2, 1, FONT_FILE);
    fread(&cm->y, 2, 1, FONT_FILE);
    fread(&cm->width, 2, 1, FONT_FILE);
    fread(&cm->height, 2, 1, FONT_FILE);
    if(curchar == 0x20) {
        *buf = NULL;
        return 0;
    }
    unsigned char* data = malloc(cm->width * cm->height + 2);
    uint24_RGB* decompressed = malloc(cm->width * cm->height);
    fread(data, cm->width * cm->height + 2, 1, FONT_FILE);
    if(cm->width == 0 || cm->height == 0) {
        ets_printf("BAD CHAR: %x has width %d height %d\n", cm->width, cm->height);
    }
    cm->vertical = (cm->advance & 0x1000) >> 12;
    cm->advance &= 0x4fff;
    if(cm->vertical)
        decode_VERT(data, (unsigned char*) decompressed, cm->height, cm->width);
    else
        decode(data, (unsigned char*) decompressed);
    free(data);
    *buf = decompressed;
    return 0;
}

int load_bgimg(uint24_RGB* buf, char* name) {
    FILE* f = fopen(name, "rb");
    char header[3];
    fread(header, 3, 1, f);
    if(strncmp(header, "cbi", 3)) {
        fclose(f);
        return -1;
    }
    uint16_t width, height;
    fread(&width, 2, 1, f);
    fread(&height, 2, 1, f);
    if(width != 240 || height != 320) {
        fclose(f);
        ets_printf("width: %d, height: %d\n", width, height);
        return -2;
    }
    uint8_t* compressed = malloc(320*240*3);
    fread(compressed, 320*240, 3, f);
    fclose(f);
    decode(compressed, (uint8_t*) buf);
    return 0;
}
