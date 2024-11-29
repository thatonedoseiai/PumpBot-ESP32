#include "fontfile.h"
#include <stdint.h>
#include <stdlib.h>
#include <rom/ets_sys.h>
#include <string.h>

FILE* FONT_FILE;
FILE* IMAGE_COLLECTION_FILE;
char font_file_open = false;
FONT_METADATA fm;
CHAR_METADATA cm;
char* bgimg_filename = NULL;
extern uint24_RGB* foreground_color;
extern uint24_RGB* background_color;

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

int load_bgimg(uint24_RGB* buf, char* name, char force_load, int index) {
    int err = 0;
    char header[3];
    if(!bgimg_filename || force_load || strcmp(name, bgimg_filename)) {
        IMAGE_COLLECTION_FILE = fopen(name, "rb");
        fread(header, 3, 1, IMAGE_COLLECTION_FILE);
        if(strncmp(header, "cbi", 3)) {
            err = -1;
            goto ret_err;
        }
        fread(header, 1, 1, IMAGE_COLLECTION_FILE);
        if(header[0] != 2) {
            err = -3;
            goto ret_err;
        }
    }

    fread(header, 1, 1, IMAGE_COLLECTION_FILE);
    if(index >= header[0]) {
        err = -4;
        goto ret_err;
    } // index oob

    fseek(IMAGE_COLLECTION_FILE, 5+4*index, SEEK_SET);
    uint32_t imagedata_offset;
    fread(&imagedata_offset, 4, 1, IMAGE_COLLECTION_FILE);
    fseek(IMAGE_COLLECTION_FILE, imagedata_offset, SEEK_SET);

    uint8_t flags;
    uint16_t width, height;
    fread(&flags, 1, 1, IMAGE_COLLECTION_FILE);
    fread(&width, 2, 1, IMAGE_COLLECTION_FILE);
    fread(&height, 2, 1, IMAGE_COLLECTION_FILE);
    if(width != 240 || height != 320) {
        ets_printf("width: %d, height: %d\n", width, height);
        err = -2;
        goto ret_err;
    }
    char BITDEPTH = 3;
    if(flags & 0x1)
        BITDEPTH = 1;
    uint8_t* compressed = malloc(320*240*BITDEPTH);
    fread(compressed, 320*240, BITDEPTH, IMAGE_COLLECTION_FILE);
    if(flags & 0x1) {
        uint8_t* decompressed = malloc(320*240);
        decode(compressed, decompressed);
        free(compressed);
        for(int i=0;i<320*240;++i) {
            buf[i].pixelR = (decompressed[i] * (foreground_color->pixelR) + (255-decompressed[i]) * (background_color->pixelR)) / 255;
            buf[i].pixelG = (decompressed[i] * (foreground_color->pixelG) + (255-decompressed[i]) * (background_color->pixelG)) / 255;
            buf[i].pixelB = (decompressed[i] * (foreground_color->pixelB) + (255-decompressed[i]) * (background_color->pixelB)) / 255;
        }
        free(decompressed);
    } else {
        decode(compressed, (uint8_t*) buf);
        free(compressed);
    }
    
    if(bgimg_filename)
        free(bgimg_filename);
    bgimg_filename = malloc(strlen(name)+2);
    strcpy(bgimg_filename, name);
    return 0;

ret_err:
    if(bgimg_filename)
        free(bgimg_filename);
    bgimg_filename = NULL;
    fclose(IMAGE_COLLECTION_FILE);
    IMAGE_COLLECTION_FILE = NULL;
    return err;
}
