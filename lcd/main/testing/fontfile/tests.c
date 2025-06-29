#include "fontfile.h"
#include "stdlib.h"
#define EVALSUCCESS(code, f) if(code) printf("ERROR IN %s: code: %d.\n", f, code); else printf("%s SUCCESS\n", f);


uint24_RGB background_color;
uint24_RGB foreground_color;
extern FILE* FONT_FILE;
extern char font_file_open;

void cleanup(void) {
	if(FONT_FILE)
		fclose(FONT_FILE);
	if(font_file_open)
		font_file_open = false;
}

int TEST_SIZE_CHANGE(void) {
	if(font_file_open) return -3;
	int x = set_font_size(12);
	if(x) return x;
	if(FONT_FILE == NULL) return -2;
	if(!font_file_open) return -3;
	x = set_font_size(14);
	if(x) return x;
	if(FONT_FILE == NULL) return -2;
	if(!font_file_open) return -3;
	x = set_font_size(18);
	if(x) return x;
	if(FONT_FILE == NULL) return -2;
	if(!font_file_open) return -3;
	x = set_font_size(24);
	if(x) return x;
	if(FONT_FILE == NULL) return -2;
	if(!font_file_open) return -3;
	x = set_font_size(42);
	if(x) return x;
	if(FONT_FILE == NULL) return -2;
	if(!font_file_open) return -3;
	cleanup();
	return 0;
}

int TEST_DECODE(void) {
	set_font_size(12);
	uint24_RGB* buf;
	CHAR_METADATA cm;
	load_char(&buf, &cm, 0x65);
	cleanup();
	free(buf);
	return 0;
}

int main(void) {
	background_color.pixelR = 0;
	background_color.pixelG = 0;
	background_color.pixelB = 0;
	foreground_color.pixelR = 0xff;
	foreground_color.pixelG = 0xff;
	foreground_color.pixelB = 0xff;

	int k = TEST_SIZE_CHANGE();
	EVALSUCCESS(k, "TEST_SIZE_CHANGE");

	k = TEST_DECODE();
	EVALSUCCESS(k, "TEST_DECODE");
}